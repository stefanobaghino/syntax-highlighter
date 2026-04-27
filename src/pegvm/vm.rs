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
    /// Frame for an in-flight left-recursive rule invocation. Pushed by
    /// `LRBody` on the first entry at a given `sp`; popped by `LRTail` when
    /// the seed-and-grow loop converges, or by `fail()` (which uses the
    /// `seed` to decide between rescuing the rule with the prior seed or
    /// continuing to unwind). On convergence `LRTail` writes the seed to
    /// `self.memo` (subject to the threshold filter); failure entries
    /// for LR rules are not cached yet (#48 scoped to converged seeds).
    LFrame {
        memo_id: MemoId,
        start_sp: usize,
        capture_start_len: usize,
        return_addr: usize,
        seed: Option<LSeed>,
    },
}

/// Successful seed of a left-recursive rule's prior iteration: the `sp` the
/// body matched up to, and the closed captures it produced past the
/// `LFrame`'s `capture_start_len`. Replayed verbatim on recursive entries
/// (where the recursive call must appear to have returned this seed) and on
/// the convergence step (where the rule itself exits with this match).
#[derive(Debug, Clone)]
struct LSeed {
    end_sp: usize,
    captures: Vec<Capture>,
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
    /// Length of `captures` at the moment `sp` reached `max_sp`. The
    /// captures themselves live in `captures[..max_captures_len]` until a
    /// `fail()` or `BackCommit` truncates the vector below that watermark,
    /// at which point `protect_max_captures` materializes a copy into
    /// `max_captures_saved`. Deferring the clone makes `maybe_snapshot`
    /// O(1) instead of O(n) per sp-advance — critical on large inputs
    /// where the eager-clone variant went quadratic in captures count.
    max_captures_len: usize,
    /// Populated lazily the first time a truncate would drop captures
    /// below `max_captures_len`. `None` means the canonical captures
    /// still reside in `captures[..max_captures_len]`. Cleared whenever
    /// `max_sp` advances (the prior snapshot becomes stale).
    max_captures_saved: Option<Vec<OpenCapture>>,
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
    /// Per-rule-invocation watermark of the farthest input position examined
    /// since the enclosing `StackEntry::Memo` frame was pushed. One entry
    /// per live `Memo` frame, pushed/popped in lockstep with them. The
    /// topmost value is bumped by every read site via `track_read`; at
    /// `MemoClose` or the `Memo` arm of `fail()` the popped value becomes
    /// the outgoing entry's `examined_max` and is then merged into the new
    /// top (the parent rule's watermark). Needed for incremental parsing:
    /// an edit at position `p` invalidates a cached entry iff
    /// `p < entry.examined_max`, so the bound must reflect lookahead reads
    /// past `end_sp` as well as the consumed span.
    memo_examined: Vec<usize>,
}

/// A memo-table entry for a rule invocation at a specific input position.
///
/// `end_sp.is_some()` encodes success and carries the sp at which the rule
/// finished matching, plus the captures it produced (already closed).
/// `end_sp.is_none()` encodes a cached failure — future hits at the same
/// `(memo_id, start_sp)` enter `fail()` directly without re-running the
/// rule body.
///
/// `examined_max` is the farthest input position the rule's execution ever
/// looked at, including lookahead past `end_sp` (`&expr` / `!expr`) and
/// failed reads. It is always `>= start_sp` (the key's second component),
/// and for successful entries `>= end_sp`. Incremental parsing uses it as
/// the invalidation bound: an edit touching any byte before
/// `examined_max` may change the rule's outcome, so the entry is stale.
///
/// Fields are `pub(crate)` so the `incremental` module (which holds the
/// `MemoCache` that reuses entries across edits) can read and shift them.
/// Outside the crate the type is opaque.
#[derive(Debug, Clone)]
pub(crate) struct MemoEntry {
    pub(crate) end_sp: Option<usize>,
    pub(crate) examined_max: usize,
    pub(crate) captures: Vec<Capture>,
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
            max_captures_len: 0,
            max_captures_saved: None,
            memo: HashMap::new(),
            memo_hits: 0,
            memo_misses: 0,
            memo_threshold: Self::DEFAULT_MEMO_THRESHOLD,
            memo_examined: Vec::new(),
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

    /// Construct a VM whose memo table is pre-populated with `cache`.
    /// Entries the seeded cache agrees with the program on (same rule +
    /// position) will be served as hits without re-executing the rule
    /// body — that is the entire point of incremental parsing. The
    /// caller is responsible for ensuring the cache was built against
    /// either this same input or a prior input plus a correctly applied
    /// [`MemoCache::apply_edit`](super::incremental::MemoCache::apply_edit).
    pub fn new_with_cache(
        program: &'p [Instruction],
        input: &'i [u8],
        mut cache: super::incremental::MemoCache,
    ) -> Self {
        let mut vm = Self::new(program, input);
        vm.memo = cache.take();
        vm
    }

    pub fn run(self) -> MatchResult {
        self.run_with_memo_stats().0
    }

    pub fn run_with_memo_stats(self) -> (MatchResult, MemoStats) {
        let (result, stats, _memo) = self.run_core();
        (result, stats)
    }

    /// Run the VM to completion and return the populated memo as a
    /// [`MemoCache`](super::incremental::MemoCache) so the caller can
    /// carry it across an edit. Equivalent to `run_with_memo_stats`
    /// plus a rewrap; the cache retains every entry recorded during
    /// this run, including entries seeded in via [`new_with_cache`]
    /// that survived the parse.
    pub fn run_with_cache(self) -> (MatchResult, MemoStats, super::incremental::MemoCache) {
        let (result, stats, memo) = self.run_core();
        let mut cache = super::incremental::MemoCache::new();
        cache.install(memo);
        (result, stats, cache)
    }

    /// Core execution loop. Returns the result, stats, and the final memo
    /// table. Package-internal wrapper for tests that need to inspect
    /// per-entry details (notably `examined_max`) and for the public
    /// `run_with_cache` variant that rewraps the table into a
    /// [`MemoCache`](super::incremental::MemoCache).
    fn run_core(mut self) -> (MatchResult, MemoStats, HashMap<(MemoId, usize), MemoEntry>) {
        loop {
            let instr = match self.program.get(self.ip) {
                Some(i) => i,
                None => return self.finalize_partial(),
            };
            match instr {
                Instruction::Char(b) => {
                    self.track_read(1);
                    if self.input.get(self.sp) == Some(b) {
                        self.sp += 1;
                        self.ip += 1;
                    } else if !self.fail() {
                        return self.finalize_partial();
                    }
                }
                Instruction::Set(set) => {
                    self.track_read(1);
                    match self.input.get(self.sp) {
                        Some(b) if set.contains(*b) => {
                            self.sp += 1;
                            self.ip += 1;
                        }
                        _ => {
                            if !self.fail() {
                                return self.finalize_partial();
                            }
                        }
                    }
                }
                Instruction::Any(n) => {
                    let n = *n as usize;
                    self.track_read(n);
                    if self.sp + n <= self.input.len() {
                        self.sp += n;
                        self.ip += 1;
                    } else if !self.fail() {
                        return self.finalize_partial();
                    }
                }
                Instruction::TestChar(b, label) => {
                    self.track_read(1);
                    if self.input.get(self.sp) == Some(b) {
                        self.sp += 1;
                        self.ip += 1;
                    } else {
                        self.ip = label.0;
                    }
                }
                Instruction::TestSet(set, label) => {
                    self.track_read(1);
                    match self.input.get(self.sp) {
                        Some(b) if set.contains(*b) => {
                            self.sp += 1;
                            self.ip += 1;
                        }
                        _ => {
                            self.ip = label.0;
                        }
                    }
                }
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
                        self.protect_max_captures(capture_len);
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
                        // Cache hit substitutes for executing the rule body;
                        // propagate its examined watermark to the enclosing
                        // rule (if any) so an outer rule's `examined_max`
                        // subsumes everything this call would have looked at.
                        // The bump is deferred past the `entry` borrow
                        // below (`self.bump_top_memo_examined` is `&mut
                        // self` and would conflict with it).
                        let hit_examined = entry.examined_max;
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
                                // `entry` is no longer used past this point;
                                // the immutable borrow of `self.memo` ends
                                // by NLL, releasing `&mut self` for the
                                // bump and maybe_snapshot below.
                                self.bump_top_memo_examined(hit_examined);
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
                                self.bump_top_memo_examined(hit_examined);
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
                        // Parallel watermark for this frame. Starts at the
                        // rule's entry sp; read sites bump it as the body
                        // executes. Popped together with the `Memo` frame
                        // at `MemoClose` or in `fail()`'s `Memo` arm.
                        self.memo_examined.push(self.sp);
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
                    // Pop this frame's examined watermark. Every `Memo`
                    // frame push (in `MemoOpen` miss) is paired with a
                    // `memo_examined` push, and this is the only
                    // non-failure pop site, so the stacks must align.
                    let examined_max = self
                        .memo_examined
                        .pop()
                        .expect("MemoClose: memo_examined underflow");
                    debug_assert!(
                        examined_max >= self.sp,
                        "MemoClose: examined_max ({}) fell behind end_sp ({})",
                        examined_max,
                        self.sp,
                    );
                    // Propagate up so the parent rule's watermark covers
                    // everything this one examined.
                    self.bump_top_memo_examined(examined_max);
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
                                examined_max,
                                captures: rule_captures,
                            },
                        );
                    }
                    self.ip += 1;
                }
                Instruction::LRBody(memo_id, return_label) => {
                    // Packrat cache check first — mirrors MemoOpen's hit
                    // path. A cached entry at this (memo_id, sp) was
                    // committed by a previous outer invocation that already
                    // popped its LFrame; a current in-flight LFrame for the
                    // same key cannot coexist, so the live-stack walk
                    // below is only reached on a cache miss.
                    if let Some(entry) = self.memo.get(&(*memo_id, self.sp)) {
                        self.memo_hits += 1;
                        let hit_examined = entry.examined_max;
                        match entry.end_sp {
                            Some(end_sp) => {
                                for c in &entry.captures {
                                    self.captures.push(OpenCapture {
                                        kind: c.kind,
                                        start: c.start,
                                        end: Some(c.end),
                                    });
                                }
                                self.bump_top_memo_examined(hit_examined);
                                self.sp = end_sp;
                                self.ip = return_label.0;
                                self.maybe_snapshot();
                            }
                            None => {
                                self.bump_top_memo_examined(hit_examined);
                                if !self.fail() {
                                    return self.finalize_partial();
                                }
                            }
                        }
                        continue;
                    }
                    // Walk the stack top-down for an LFrame at the same
                    // (memo_id, sp) — that signals recursive re-entry. For
                    // direct LR the cycle has length 1, so any matching
                    // LFrame is this rule's own.
                    let lookup: Option<Option<LSeed>> = self.stack.iter().rev().find_map(|e| {
                        if let StackEntry::LFrame {
                            memo_id: id,
                            start_sp,
                            seed,
                            ..
                        } = e
                        {
                            if *id == *memo_id && *start_sp == self.sp {
                                return Some(seed.clone());
                            }
                        }
                        None
                    });
                    match lookup {
                        Some(Some(found)) => {
                            // Recursive entry with a seed: replay captures
                            // already-closed (mirrors `MemoOpen`'s hit
                            // path), set sp to seed end, and jump to the
                            // rule's Return so the caller's `Call`-pushed
                            // Return frame fires normally.
                            for c in &found.captures {
                                self.captures.push(OpenCapture {
                                    kind: c.kind,
                                    start: c.start,
                                    end: Some(c.end),
                                });
                            }
                            self.bump_top_memo_examined(found.end_sp);
                            self.sp = found.end_sp;
                            self.ip = return_label.0;
                            self.maybe_snapshot();
                        }
                        Some(None) => {
                            // Recursive entry with no seed yet — the rule
                            // has not succeeded once at this position, so
                            // the recursive call must fail (bound 0).
                            if !self.fail() {
                                return self.finalize_partial();
                            }
                        }
                        None => {
                            // First entry at this sp — push an LFrame and
                            // a paired memo_examined watermark, then fall
                            // through to the body.
                            self.stack.push(StackEntry::LFrame {
                                memo_id: *memo_id,
                                start_sp: self.sp,
                                capture_start_len: self.captures.len(),
                                return_addr: return_label.0,
                                seed: None,
                            });
                            self.memo_examined.push(self.sp);
                            self.ip += 1;
                        }
                    }
                }
                Instruction::LRTail(memo_id, body_start) => {
                    // Peek the topmost LFrame. The body just succeeded; we
                    // must decide between growing (re-iterate) and
                    // accepting (commit and fall through to Return).
                    let top_idx = self
                        .stack
                        .iter()
                        .rposition(|e| matches!(e, StackEntry::LFrame { .. }))
                        .expect("LRTail without an enclosing LFrame");
                    let StackEntry::LFrame {
                        memo_id: top_id,
                        start_sp,
                        capture_start_len,
                        return_addr: _,
                        seed,
                    } = &mut self.stack[top_idx]
                    else {
                        unreachable!("rposition matched LFrame")
                    };
                    debug_assert_eq!(
                        *top_id, *memo_id,
                        "LRTail id mismatch: expected {:?}, found {:?}",
                        memo_id, top_id,
                    );
                    let start_sp = *start_sp;
                    let capture_start_len = *capture_start_len;
                    let body_end_sp = self.sp;
                    let grew = match seed {
                        None => body_end_sp > start_sp,
                        Some(prev) => body_end_sp > prev.end_sp,
                    };
                    if grew {
                        // Snapshot the iteration's captures (closed at
                        // body_end_sp where any still-open ones land) and
                        // store as the new seed; rewind for re-iteration.
                        let new_caps: Vec<Capture> = self.captures[capture_start_len..]
                            .iter()
                            .map(|c| Capture {
                                kind: c.kind,
                                start: c.start,
                                end: c.end.unwrap_or(body_end_sp),
                            })
                            .collect();
                        *seed = Some(LSeed {
                            end_sp: body_end_sp,
                            captures: new_caps,
                        });
                        self.captures.truncate(capture_start_len);
                        self.sp = start_sp;
                        self.ip = body_start.0;
                    } else {
                        // No growth — accept the prior seed. If the seed
                        // is still None here (body matched empty on first
                        // try), the rule succeeds with an empty match.
                        let final_seed = seed.take();
                        // Pop the LFrame and the paired memo_examined.
                        let _frame = self.stack.remove(top_idx);
                        let examined = self
                            .memo_examined
                            .pop()
                            .expect("LRTail: memo_examined underflow");
                        self.bump_top_memo_examined(examined);
                        // Replay seed captures (if any), restore sp, and
                        // extract the captures vec for the cache write so
                        // we don't have to re-clone from the live buffer.
                        self.captures.truncate(capture_start_len);
                        let (final_sp, seed_captures) = match final_seed {
                            Some(s) => {
                                for c in &s.captures {
                                    self.captures.push(OpenCapture {
                                        kind: c.kind,
                                        start: c.start,
                                        end: Some(c.end),
                                    });
                                }
                                (s.end_sp, s.captures)
                            }
                            None => (start_sp, Vec::new()),
                        };
                        self.sp = final_sp;
                        // Cache the converged seed under the same threshold
                        // filter MemoClose uses. examined is the high-water
                        // mark across every iteration of the seed-and-grow
                        // loop (LRBody pushed one memo_examined slot at
                        // entry; growth iterations don't pop it, only this
                        // commit does), so it is the correct invalidation
                        // bound for MemoCache::apply_edit. Failure caching
                        // for LR rules is not yet implemented (#48 scoped
                        // to converged seeds only).
                        if final_sp - start_sp >= self.memo_threshold {
                            self.memo.insert(
                                (*memo_id, start_sp),
                                MemoEntry {
                                    end_sp: Some(final_sp),
                                    examined_max: examined,
                                    captures: seed_captures,
                                },
                            );
                        }
                        self.maybe_snapshot();
                        self.ip += 1;
                    }
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
                        self.memo,
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
                    self.protect_max_captures(capture_len);
                    self.captures.truncate(capture_len);
                    return true;
                }
                StackEntry::Memo {
                    memo_id,
                    start_sp,
                    capture_start_len: _,
                } => {
                    // Pop the paired examined watermark. See `MemoOpen`
                    // miss path — every `StackEntry::Memo` push is twinned
                    // with a `memo_examined` push.
                    let examined_max = self
                        .memo_examined
                        .pop()
                        .expect("fail(): memo_examined underflow on Memo frame");
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
                            examined_max,
                            captures: Vec::new(),
                        },
                    );
                    // Propagate to the parent rule: a failure here depended
                    // on input through `examined_max`, and any caller that
                    // ultimately retries a different path still saw those
                    // bytes.
                    self.bump_top_memo_examined(examined_max);
                }
                StackEntry::Return { .. } => {
                    // Rule-call frame unwinding past its caller. The caller's
                    // Backtrack (if any) is deeper on the stack and will be
                    // found by continued popping.
                }
                StackEntry::LFrame {
                    memo_id: _,
                    start_sp: _,
                    capture_start_len,
                    return_addr,
                    seed,
                } => {
                    // Pop the paired examined watermark. Successful LR
                    // converged seeds are cached at LRTail; LR-rule
                    // failures (this arm with seed=None) are not cached
                    // yet — symmetric with `fail()`'s `Memo` arm but
                    // deferred per #48's scoping. The watermark still
                    // flows up to the parent rule's frame.
                    let examined_max = self
                        .memo_examined
                        .pop()
                        .expect("fail(): memo_examined underflow on LFrame");
                    self.bump_top_memo_examined(examined_max);
                    if let Some(s) = seed {
                        // Body failed on a re-iteration after the seed
                        // already grew at least once. Bounded LR
                        // semantics: accept the prior seed as the rule's
                        // match. Restore captures to the LFrame baseline,
                        // replay the seed's closed captures, set sp to
                        // seed.end_sp, and jump to the rule's Return.
                        // Returning `true` from `fail()` resumes execution
                        // at `self.ip` — the Return frame the caller's
                        // `Call` pushed is still on the stack and will
                        // pop normally.
                        self.protect_max_captures(capture_start_len);
                        self.captures.truncate(capture_start_len);
                        for c in &s.captures {
                            self.captures.push(OpenCapture {
                                kind: c.kind,
                                start: c.start,
                                end: Some(c.end),
                            });
                        }
                        self.sp = s.end_sp;
                        self.ip = return_addr;
                        self.maybe_snapshot();
                        return true;
                    }
                    // No seed yet — the LR rule has failed without ever
                    // growing past bound 0. Continue unwinding past this
                    // frame; the captures-truncate happens at whichever
                    // Backtrack ultimately catches the unwind (its
                    // capture_len was snapshotted before LRBody pushed
                    // this frame).
                }
            }
        }
        false
    }

    /// Bump the in-flight rule's examined watermark to at least `pos`.
    /// Invoked from every read site (Char/Set/Any/TestChar/TestSet) and
    /// from memo-hit propagation, so a rule's `MemoEntry.examined_max`
    /// reflects lookahead and failed reads past `end_sp`. A no-op when no
    /// `Memo` frame is live (top-level program reads don't feed any
    /// cached entry).
    fn bump_top_memo_examined(&mut self, pos: usize) {
        if let Some(top) = self.memo_examined.last_mut() {
            if pos > *top {
                *top = pos;
            }
        }
    }

    /// Record that the current instruction examines `n` bytes starting at
    /// `self.sp`. The probe happens at the current `sp` whether the read
    /// succeeds (consume) or fails (EOF or mismatch) — in both cases the
    /// outcome depends on bytes up to `sp + n`, so the watermark must
    /// advance there. `n = 1` for Char/Set/TestChar/TestSet; `n` for Any.
    fn track_read(&mut self, n: usize) {
        self.bump_top_memo_examined(self.sp + n);
    }

    /// Update the farthest-failure snapshot if `sp` has advanced past the
    /// previous maximum. Called from the only two sites where `sp` retreats —
    /// [`fail`](Self::fail) and the `BackCommit` handler — plus defensively
    /// at finalize time. Between retreats `sp` is monotone non-decreasing,
    /// so these hooks capture the true deepest point.
    ///
    /// O(1) per call: stores a length only; the captures are materialized
    /// lazily via [`protect_max_captures`](Self::protect_max_captures) or
    /// [`finalize_partial`](Self::finalize_partial). The eager-clone
    /// variant went quadratic in captures count on large inputs and was
    /// pure overhead on the success path (which never reads the
    /// snapshot).
    fn maybe_snapshot(&mut self) {
        if self.sp > self.max_sp {
            self.max_sp = self.sp;
            self.max_captures_len = self.captures.len();
            self.max_captures_saved = None;
        }
    }

    /// Called before truncating `captures` to `new_len`. If the truncate
    /// would drop captures the farthest-failure snapshot still needs,
    /// copy them aside first. Runs at most once per `max_sp` epoch —
    /// subsequent truncates below the same watermark are no-ops.
    fn protect_max_captures(&mut self, new_len: usize) {
        if new_len < self.max_captures_len && self.max_captures_saved.is_none() {
            self.max_captures_saved = Some(self.captures[..self.max_captures_len].to_vec());
        }
    }

    fn finalize_partial(mut self) -> (MatchResult, MemoStats, HashMap<(MemoId, usize), MemoEntry>) {
        self.maybe_snapshot();
        let stats = MemoStats {
            entries: self.memo.len(),
            hits: self.memo_hits,
            misses: self.memo_misses,
        };
        // Materialize the farthest-failure captures exactly once. If a
        // prior truncate below the watermark forced an early save, take
        // that; otherwise the canonical captures are still sitting in
        // `self.captures[..max_captures_len]`.
        let max_captures = self
            .max_captures_saved
            .take()
            .unwrap_or_else(|| self.captures[..self.max_captures_len].to_vec());
        let result = MatchResult {
            matched: self.max_sp,
            captures: close_captures(max_captures, self.max_sp),
            complete: false,
        };
        (result, stats, self.memo)
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

#[cfg(test)]
mod examined_max_tests {
    //! Verify that every memo entry records the farthest input position
    //! its rule invocation ever examined, including lookahead past
    //! `end_sp` and failed reads. Incremental parsing's invalidation
    //! predicate (`edit.start <= entry.examined_max`) depends on this
    //! bound being tight-enough to be useful and safe-at-minimum to be
    //! correct.
    //!
    //! Tests access `VM::run_core` directly to inspect the private
    //! memo map — post-run the public `run_with_memo_stats` has already
    //! discarded it.
    use super::*;
    use crate::pegc::{Grammar, Pattern};
    use std::collections::HashMap;

    fn rule(rules: &mut HashMap<String, Pattern>, name: &str, pat: Pattern) {
        rules.insert(name.into(), pat);
    }

    #[test]
    fn and_predicate_success_records_examined_past_end_sp() {
        // start <- "x" &"y"   against "xy"
        // Consumed span is "x" (end_sp = 1) but the &"y" lookahead
        // read position 1, so examined_max must be 2.
        let mut rules = HashMap::new();
        rule(
            &mut rules,
            "start",
            Pattern::seq(vec![
                Pattern::literal("x"),
                Pattern::AndPredicate(Box::new(Pattern::literal("y"))),
            ]),
        );
        let prog = Grammar::new(rules, "start").compile().unwrap();
        let (result, _stats, memo) = VM::new(&prog.code, b"xy").with_memo_threshold(0).run_core();
        assert!(result.complete);
        assert_eq!(result.matched, 1, "only 'x' is consumed");
        let entry = memo
            .get(&(MemoId(0), 0))
            .expect("start's memo entry missing");
        assert_eq!(entry.end_sp, Some(1));
        assert_eq!(
            entry.examined_max, 2,
            "&\"y\" read past end_sp; examined_max must reflect it"
        );
    }

    #[test]
    fn and_predicate_failure_records_examined_up_to_failed_read() {
        // start <- "x" &"z"   against "xy"
        // "x" succeeds (sp=1), then &"z" reads 'y' at sp=1 and fails.
        // The failure entry for start at sp=0 must remember that
        // position 1 was examined, so an edit there can invalidate it.
        let mut rules = HashMap::new();
        rule(
            &mut rules,
            "start",
            Pattern::seq(vec![
                Pattern::literal("x"),
                Pattern::AndPredicate(Box::new(Pattern::literal("z"))),
            ]),
        );
        let prog = Grammar::new(rules, "start").compile().unwrap();
        let (result, _stats, memo) = VM::new(&prog.code, b"xy").with_memo_threshold(0).run_core();
        assert!(!result.complete, "overall parse must fail");
        let entry = memo
            .get(&(MemoId(0), 0))
            .expect("start's failure entry missing");
        assert_eq!(entry.end_sp, None);
        assert_eq!(
            entry.examined_max, 2,
            "failed read of 'z' at position 1 examined position 1+1"
        );
    }

    #[test]
    fn nested_rule_examined_max_propagates_to_caller() {
        // outer <- inner "y"
        // inner <- "x"
        // against "xy".  inner's entry: end_sp=1, examined_max=1.
        // outer's entry: end_sp=2, examined_max=2 (reads 'y' at sp=1).
        // The propagation test: inner's examined_max (1) must flow into
        // outer's watermark when inner's MemoClose pops.
        let mut rules = HashMap::new();
        rule(
            &mut rules,
            "outer",
            Pattern::seq(vec![
                Pattern::NonTerminal("inner".into()),
                Pattern::literal("y"),
            ]),
        );
        rule(&mut rules, "inner", Pattern::literal("x"));
        let prog = Grammar::new(rules, "outer").compile().unwrap();
        let (result, _stats, memo) = VM::new(&prog.code, b"xy").with_memo_threshold(0).run_core();
        assert!(result.complete);
        assert_eq!(result.matched, 2);
        // outer is start → MemoId(0); inner is the other → MemoId(1).
        let outer = memo.get(&(MemoId(0), 0)).expect("outer entry missing");
        assert_eq!(outer.end_sp, Some(2));
        assert_eq!(outer.examined_max, 2);
        let inner = memo.get(&(MemoId(1), 0)).expect("inner entry missing");
        assert_eq!(inner.end_sp, Some(1));
        assert_eq!(inner.examined_max, 1);
    }

    #[test]
    fn memo_hit_propagates_examined_max_to_caller() {
        // Two alternatives both start with the memoized rule X.
        // The first alternative's success call to X populates the
        // cache with examined_max = 1; the second alternative's call
        // is a hit. The hit must still contribute X's examined_max to
        // whichever rule encloses the call.
        //
        // Grammar:
        //   start <- (X "aa") / (X "bb")
        //   X <- "a"
        // Input: "abb" — first alt fails after X matches "a" and "aa"
        // fails on "bb"; backtrack to second alt, which hits X's cache
        // and then matches "bb".
        let mut rules = HashMap::new();
        rule(
            &mut rules,
            "start",
            Pattern::choice(vec![
                Pattern::seq(vec![
                    Pattern::NonTerminal("X".into()),
                    Pattern::literal("aa"),
                ]),
                Pattern::seq(vec![
                    Pattern::NonTerminal("X".into()),
                    Pattern::literal("bb"),
                ]),
            ]),
        );
        rule(&mut rules, "X", Pattern::literal("a"));
        let prog = Grammar::new(rules, "start").compile().unwrap();
        let (result, stats, memo) = VM::new(&prog.code, b"abb")
            .with_memo_threshold(0)
            .run_core();
        assert!(result.complete);
        assert_eq!(result.matched, 3);
        assert!(stats.hits >= 1, "second alternative must hit X's cache");
        // start's examined_max must include every byte start's execution
        // ever looked at: position 2 was examined when the first
        // alternative tried "aa" at sp=1 and "bb" at sp=1 needed byte 2.
        let start = memo.get(&(MemoId(0), 0)).expect("start entry missing");
        assert_eq!(start.end_sp, Some(3));
        assert_eq!(
            start.examined_max, 3,
            "start's execution examined up to position 3"
        );
    }

    #[test]
    fn recover_repeat_propagates_examined_max_through_loop_iterations() {
        // start <- ("ab")*^   against "abxab"
        //
        // The recovery loop runs entirely inside start's memo entry.
        // Across iterations the loop reads positions 0, 1 (success),
        // 2 (Char 'a' fails on 'x'), 2 (Any(1) consumes 'x' → sp=3),
        // 3, 4 (success), 5 (Char 'a' at EOF). Whether the failed
        // EOF read at sp=5 contributes depends on track_read's call
        // discipline, so the assertion is a lower bound: every
        // position the loop *successfully* read must appear in
        // examined_max.
        let mut rules = HashMap::new();
        rule(
            &mut rules,
            "start",
            Pattern::RecoverRepeat {
                inner: Box::new(Pattern::literal("ab")),
                recovery_kind: "recovery".into(),
            },
        );
        let prog = Grammar::new(rules, "start").compile().unwrap();
        let (result, _stats, memo) = VM::new(&prog.code, b"abxab")
            .with_memo_threshold(0)
            .run_core();
        assert!(result.complete);
        assert_eq!(result.matched, 5);
        let entry = memo.get(&(MemoId(0), 0)).expect("start entry missing");
        assert_eq!(entry.end_sp, Some(5));
        assert!(
            entry.examined_max >= 5,
            "recovery-loop reads must propagate to enclosing rule's watermark; got {}",
            entry.examined_max
        );
    }
}
