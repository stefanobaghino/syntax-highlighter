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

pub struct VM<'p, 'i> {
    program: &'p [Instruction],
    input: &'i [u8],
    ip: usize,
    sp: usize,
    stack: Vec<StackEntry>,
    captures: Vec<OpenCapture>,
    max_sp: usize,
    max_captures: Vec<OpenCapture>,
}

#[derive(Debug, Clone)]
struct OpenCapture {
    kind: CaptureKind,
    start: usize,
    end: Option<usize>,
}

impl<'p, 'i> VM<'p, 'i> {
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
        }
    }

    pub fn run(mut self) -> MatchResult {
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
                Instruction::MemoOpen(memo_id, _return_label) => {
                    self.stack.push(StackEntry::Memo {
                        memo_id: *memo_id,
                        start_sp: self.sp,
                        capture_start_len: self.captures.len(),
                    });
                    self.ip += 1;
                }
                Instruction::MemoClose(memo_id) => {
                    match self.stack.pop() {
                        Some(StackEntry::Memo {
                            memo_id: top_id,
                            start_sp,
                            capture_start_len,
                        }) => {
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
                        }
                        other => {
                            panic!("MemoClose expected Memo on stack top, got {:?}", other)
                        }
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
                    return MatchResult {
                        matched: self.sp,
                        captures: close_captures(self.captures, self.sp),
                        complete: true,
                    };
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
            if let StackEntry::Backtrack {
                ip,
                sp,
                capture_len,
            } = entry
            {
                self.ip = ip;
                self.sp = sp;
                self.captures.truncate(capture_len);
                return true;
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

    fn finalize_partial(mut self) -> MatchResult {
        self.maybe_snapshot();
        MatchResult {
            matched: self.max_sp,
            captures: close_captures(self.max_captures, self.max_sp),
            complete: false,
        }
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
