use std::collections::HashMap;

use super::instruction::{CaptureKind, Instruction, MemoId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    pub kind: CaptureKind,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
enum StackEntry {
    Backtrack {
        ip: usize,
        sp: usize,
        capture_len: usize,
    },
    Return {
        ip: usize,
    },
    /// Frame for an in-flight memoized rule call. Pushed by `MemoOpen` on a
    /// cache miss, popped by `MemoClose` on success (which records the entry)
    /// or by `fail()` when the rule escapes via failure (which records a
    /// failure entry). Holds enough state to locate the cache slot
    /// (`memo_id`, `start_sp`) and to slice the captures produced inside the
    /// rule (`capture_start_len`).
    Memo {
        memo_id: MemoId,
        start_sp: usize,
        capture_start_len: usize,
    },
}

/// Outcome of running a compiled [`Program`](super::Program) against an input.
///
/// - `complete == true`: the VM reached `End`. `matched` is the `sp` at that
///   point, which may be less than `input.len()` if the grammar is designed
///   to stop early (no trailing `!.`).
/// - `complete == false`: the VM exhausted its backtrack stack without
///   reaching `End`. `matched` is the farthest input position the VM ever
///   reached before retreating (Ford 2004's "farthest failure position"),
///   and `captures` are the captures that were valid at that point — any
///   captures left open at `matched` are closed there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    pub matched: usize,
    pub captures: Vec<Capture>,
    pub complete: bool,
}

/// Diagnostic counters for the memoization cache, exposed via
/// [`VM::run_with_memo_stats`]. Primarily used by tests and benchmarks to
/// prove the cache is doing its job.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoStats {
    /// Number of distinct `(memo_id, start_sp)` pairs in the table at the
    /// end of the run. Each entry represents one cached rule outcome
    /// (success or failure).
    pub entries: usize,
    /// Number of times `MemoOpen` resolved via a cached entry instead of
    /// re-executing the rule body.
    pub hits: usize,
    /// Number of `MemoOpen` invocations that did *not* find a cached entry
    /// and had to execute the rule body. With the memo threshold at 0,
    /// every miss produces an entry; with a non-zero threshold, successful
    /// miss bodies shorter than the threshold are not written back, so
    /// `entries` and `misses` diverge. `hits + misses` is the total
    /// lookup count.
    pub misses: usize,
}

pub struct VM<'p, 'i> {
    program: &'p [Instruction],
    input: &'i [u8],
    ip: usize,
    sp: usize,
    stack: Vec<StackEntry>,
    captures: Vec<OpenCapture>,
    max_sp: usize,
    max_captures: Vec<OpenCapture>,
    /// Packrat memo table, keyed by `(memo_id, start_sp)`. Populated by
    /// `MemoClose` on success and by `fail()` on failure escape (Commit 5).
    memo: HashMap<(MemoId, usize), MemoEntry>,
    /// Running count of resolved cache hits, exposed via `MemoStats`.
    memo_hits: usize,
    /// Running count of `MemoOpen` cache misses, exposed via `MemoStats`.
    memo_misses: usize,
    /// Minimum successful-span length (in bytes) for which `MemoClose` will
    /// write the outcome back to the cache. Default is
    /// [`Self::DEFAULT_MEMO_THRESHOLD`]; `0` disables the filter and
    /// restores pure packrat behavior. Non-zero values skip tiny leaf-rule
    /// entries that pay lookup cost without a meaningful storage win — see
    /// GPeg (default 512) and Yedidia §5.2.4 (knee near 4096). Failure
    /// entries in `fail()` are not filtered; their value is short-circuiting.
    memo_threshold: usize,
}

/// A memo-table entry for a rule invocation at a specific input position.
///
/// `end_sp.is_some()` encodes success and carries the sp at which the rule
/// finished matching, plus the captures it produced (already closed).
/// `end_sp.is_none()` encodes a cached failure — future hits at the same
/// `(memo_id, start_sp)` enter `fail()` directly without re-running the
/// rule body.
#[derive(Debug, Clone)]
struct MemoEntry {
    end_sp: Option<usize>,
    captures: Vec<Capture>,
}

#[derive(Debug, Clone)]
struct OpenCapture {
    kind: CaptureKind,
    start: usize,
    end: Option<usize>,
}

impl<'p, 'i> VM<'p, 'i> {
    /// Default memo-threshold applied by [`VM::new`]. Picked from the sweep
    /// at `benches/memo.rs`: the time-vs-entries curve is flat from ~32
    /// bytes upward on every shipped grammar, so any value in that range
    /// is defensible. 128 matches GPeg's benchmark reference point and
    /// stays conservative against hardware and corpus variation.
    pub const DEFAULT_MEMO_THRESHOLD: usize = 128;

    pub fn new(program: &'p [Instruction], input: &'i [u8]) -> Self {
        VM {
            program,
            input,
            ip: 0,
            sp: 0,
            stack: Vec::new(),
            captures: Vec::new(),
            max_sp: 0,
            max_captures: Vec::new(),
            memo: HashMap::new(),
            memo_hits: 0,
            memo_misses: 0,
            memo_threshold: Self::DEFAULT_MEMO_THRESHOLD,
        }
    }

    /// Override the default memo threshold. `bytes = 0` disables the
    /// filter and restores pure packrat behavior (useful for tests that
    /// exercise the caching mechanism on small inputs). See the
    /// `memo_threshold` field doc for the rationale.
    pub fn with_memo_threshold(mut self, bytes: usize) -> Self {
        self.memo_threshold = bytes;
        self
    }

    pub fn run(self) -> MatchResult {
        self.run_with_memo_stats().0
    }

    pub fn run_with_memo_stats(mut self) -> (MatchResult, MemoStats) {
        loop {
            let instr = match self.program.get(self.ip) {
                Some(i) => i,
                None => return self.finalize_partial(),
            };
            match instr {
                Instruction::Char(b) => {
                    if self.input.get(self.sp) == Some(b) {
                        self.sp += 1;
                        self.ip += 1;
                    } else if !self.fail() {
                        return self.finalize_partial();
                    }
                }
                Instruction::Set(set) => match self.input.get(self.sp) {
                    Some(b) if set.contains(*b) => {
                        self.sp += 1;
                        self.ip += 1;
                    }
                    _ => {
                        if !self.fail() {
                            return self.finalize_partial();
                        }
                    }
                },
                Instruction::Any(n) => {
                    let n = *n as usize;
                    if self.sp + n <= self.input.len() {
                        self.sp += n;
                        self.ip += 1;
                    } else if !self.fail() {
                        return self.finalize_partial();
                    }
                }
                Instruction::TestChar(b, label) => {
                    if self.input.get(self.sp) == Some(b) {
                        self.sp += 1;
                        self.ip += 1;
                    } else {
                        self.ip = label.0;
                    }
                }
                Instruction::TestSet(set, label) => match self.input.get(self.sp) {
                    Some(b) if set.contains(*b) => {
                        self.sp += 1;
                        self.ip += 1;
                    }
                    _ => {
                        self.ip = label.0;
                    }
                },
                Instruction::Jump(label) => {
                    self.ip = label.0;
                }
                Instruction::Choice(label) => {
                    self.stack.push(StackEntry::Backtrack {
                        ip: label.0,
                        sp: self.sp,
                        capture_len: self.captures.len(),
                    });
                    self.ip += 1;
                }
                Instruction::Commit(label) => {
                    self.pop_backtrack();
                    self.ip = label.0;
                }
                Instruction::PartialCommit(label) => {
                    let top = self.stack.last_mut().expect("PartialCommit on empty stack");
                    match top {
                        StackEntry::Backtrack {
                            sp, capture_len, ..
                        } => {
                            *sp = self.sp;
                            *capture_len = self.captures.len();
                        }
                        _ => panic!("PartialCommit expected Backtrack on stack top"),
                    }
                    self.ip = label.0;
                }
                Instruction::BackCommit(label) => {
                    self.maybe_snapshot();
                    let entry = self.pop_backtrack();
                    if let StackEntry::Backtrack {
                        sp, capture_len, ..
                    } = entry
                    {
                        self.sp = sp;
                        self.captures.truncate(capture_len);
                    }
                    self.ip = label.0;
                }
                Instruction::FailTwice => {
                    self.pop_backtrack();
                    if !self.fail() {
                        return self.finalize_partial();
                    }
                }
                Instruction::Fail => {
                    if !self.fail() {
                        return self.finalize_partial();
                    }
                }
                Instruction::Call(label) => {
                    self.stack.push(StackEntry::Return { ip: self.ip + 1 });
                    self.ip = label.0;
                }
                Instruction::Return => {
                    let ret_ip = match self.stack.pop() {
                        Some(StackEntry::Return { ip }) => ip,
                        Some(other) => {
                            panic!("Return expected Return on stack top, got {:?}", other)
                        }
                        None => panic!("Return on empty stack"),
                    };
                    self.ip = ret_ip;
                }
                Instruction::MemoOpen(memo_id, return_label) => {
                    if let Some(entry) = self.memo.get(&(*memo_id, self.sp)) {
                        self.memo_hits += 1;
                        match entry.end_sp {
                            Some(end_sp) => {
                                // Success hit: replay captures into the live
                                // buffer. They are keyed by absolute offsets
                                // and valid iff this hit fires at the same
                                // start_sp — which is exactly the key we just
                                // matched on. Insert as already-closed so the
                                // enclosing `CaptureEnd`'s `rposition` search
                                // (which binds to the innermost still-open
                                // capture) skips over them.
                                for c in &entry.captures {
                                    self.captures.push(OpenCapture {
                                        kind: c.kind,
                                        start: c.start,
                                        end: Some(c.end),
                                    });
                                }
                                self.sp = end_sp;
                                self.ip = return_label.0;
                                // Applying a hit advances `sp` past code that
                                // did not execute; the farthest-failure
                                // bookkeeping must see the advance.
                                self.maybe_snapshot();
                            }
                            None => {
                                // Cached failure: enter fail() immediately
                                // without re-executing the rule body. Do not
                                // push a Memo frame.
                                if !self.fail() {
                                    return self.finalize_partial();
                                }
                            }
                        }
                    } else {
                        self.memo_misses += 1;
                        self.stack.push(StackEntry::Memo {
                            memo_id: *memo_id,
                            start_sp: self.sp,
                            capture_start_len: self.captures.len(),
                        });
                        self.ip += 1;
                    }
                }
                Instruction::MemoClose(memo_id) => {
                    let (top_id, start_sp, capture_start_len) = match self.stack.pop() {
                        Some(StackEntry::Memo {
                            memo_id,
                            start_sp,
                            capture_start_len,
                        }) => (memo_id, start_sp, capture_start_len),
                        other => {
                            panic!("MemoClose expected Memo on stack top, got {:?}", other)
                        }
                    };
                    debug_assert_eq!(
                        top_id, *memo_id,
                        "MemoClose id mismatch: expected {:?}, found {:?}",
                        memo_id, top_id,
                    );
                    debug_assert!(
                        start_sp <= self.sp,
                        "MemoClose: rule body retreated past start_sp ({} > {})",
                        start_sp,
                        self.sp,
                    );
                    debug_assert!(
                        capture_start_len <= self.captures.len(),
                        "MemoClose: capture buffer shrank below entry baseline ({} > {})",
                        capture_start_len,
                        self.captures.len(),
                    );
                    // Threshold filter: skip caching if the matched span is
                    // smaller than `memo_threshold`. Tiny leaf rules pay the
                    // lookup cost without a meaningful storage win — see
                    // GPeg's 512-byte default and Yedidia §5.2.4. Captures
                    // produced inside the rule remain in the live buffer
                    // (they are the caller's captures now); only the memo
                    // entry is dropped.
                    if self.sp - start_sp >= self.memo_threshold {
                        // Snapshot the captures produced inside the rule.
                        // All of them must be closed at this point — PEG
                        // captures are lexically scoped to `@name{...}` and
                        // cannot straddle a rule boundary, so any
                        // `OpenCapture` with `end.is_none()` would be a
                        // compiler bug.
                        let rule_captures: Vec<Capture> = self.captures[capture_start_len..]
                            .iter()
                            .map(|c| {
                                debug_assert!(
                                    c.end.is_some(),
                                    "MemoClose: open capture straddles rule boundary"
                                );
                                Capture {
                                    kind: c.kind,
                                    start: c.start,
                                    end: c.end.unwrap_or(self.sp),
                                }
                            })
                            .collect();
                        self.memo.insert(
                            (*memo_id, start_sp),
                            MemoEntry {
                                end_sp: Some(self.sp),
                                captures: rule_captures,
                            },
                        );
                    }
                    self.ip += 1;
                }
                Instruction::CaptureBegin(kind) => {
                    self.captures.push(OpenCapture {
                        kind: *kind,
                        start: self.sp,
                        end: None,
                    });
                    self.ip += 1;
                }
                Instruction::CaptureEnd => {
                    let idx = self
                        .captures
                        .iter()
                        .rposition(|c| c.end.is_none())
                        .expect("CaptureEnd without matching CaptureBegin");
                    self.captures[idx].end = Some(self.sp);
                    self.ip += 1;
                }
                Instruction::End => {
                    let stats = MemoStats {
                        entries: self.memo.len(),
                        hits: self.memo_hits,
                        misses: self.memo_misses,
                    };
                    return (
                        MatchResult {
                            matched: self.sp,
                            captures: close_captures(self.captures, self.sp),
                            complete: true,
                        },
                        stats,
                    );
                }
            }
        }
    }

    fn pop_backtrack(&mut self) -> StackEntry {
        match self.stack.pop() {
            Some(e @ StackEntry::Backtrack { .. }) => e,
            other => panic!("expected Backtrack on stack top, got {:?}", other),
        }
    }

    fn fail(&mut self) -> bool {
        self.maybe_snapshot();
        while let Some(entry) = self.stack.pop() {
            // Explicit match on all three variants — silently dropping a
            // `Memo` frame would skip caching its failure and leak
            // re-executions on future hits at the same sp.
            match entry {
                StackEntry::Backtrack {
                    ip,
                    sp,
                    capture_len,
                } => {
                    self.ip = ip;
                    self.sp = sp;
                    self.captures.truncate(capture_len);
                    return true;
                }
                StackEntry::Memo {
                    memo_id,
                    start_sp,
                    capture_start_len: _,
                } => {
                    // Cache the failure so a future call at the same sp
                    // short-circuits into `fail()` without re-executing the
                    // body. Captures produced inside the rule will be
                    // truncated by whichever `Backtrack` ultimately catches
                    // this unwind (its `capture_len` was snapshotted *before*
                    // MemoOpen pushed this frame).
                    self.memo.insert(
                        (memo_id, start_sp),
                        MemoEntry {
                            end_sp: None,
                            captures: Vec::new(),
                        },
                    );
                }
                StackEntry::Return { .. } => {
                    // Rule-call frame unwinding past its caller. The caller's
                    // Backtrack (if any) is deeper on the stack and will be
                    // found by continued popping.
                }
            }
        }
        false
    }

    /// Update the farthest-failure snapshot if `sp` has advanced past the
    /// previous maximum. Called from the only two sites where `sp` retreats —
    /// [`fail`](Self::fail) and the `BackCommit` handler — plus defensively
    /// at finalize time. Between retreats `sp` is monotone non-decreasing,
    /// so these hooks capture the true deepest point.
    fn maybe_snapshot(&mut self) {
        if self.sp > self.max_sp {
            self.max_sp = self.sp;
            self.max_captures = self.captures.clone();
        }
    }

    fn finalize_partial(mut self) -> (MatchResult, MemoStats) {
        self.maybe_snapshot();
        let stats = MemoStats {
            entries: self.memo.len(),
            hits: self.memo_hits,
            misses: self.memo_misses,
        };
        let result = MatchResult {
            matched: self.max_sp,
            captures: close_captures(self.max_captures, self.max_sp),
            complete: false,
        };
        (result, stats)
    }
}

fn close_captures(open: Vec<OpenCapture>, close_at: usize) -> Vec<Capture> {
    open.into_iter()
        .map(|c| Capture {
            kind: c.kind,
            start: c.start,
            end: c.end.unwrap_or(close_at),
        })
        .collect()
}
