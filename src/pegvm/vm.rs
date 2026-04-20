use super::instruction::{CaptureKind, Instruction};

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    pub matched: usize,
    pub captures: Vec<Capture>,
}

pub struct VM<'p, 'i> {
    program: &'p [Instruction],
    input: &'i [u8],
    ip: usize,
    sp: usize,
    stack: Vec<StackEntry>,
    captures: Vec<OpenCapture>,
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
        }
    }

    pub fn run(mut self) -> Option<MatchResult> {
        loop {
            let instr = self.program.get(self.ip)?;
            match instr {
                Instruction::Char(b) => {
                    if self.input.get(self.sp) == Some(b) {
                        self.sp += 1;
                        self.ip += 1;
                    } else if !self.fail() {
                        return None;
                    }
                }
                Instruction::Set(set) => match self.input.get(self.sp) {
                    Some(b) if set.contains(*b) => {
                        self.sp += 1;
                        self.ip += 1;
                    }
                    _ => {
                        if !self.fail() {
                            return None;
                        }
                    }
                },
                Instruction::Any(n) => {
                    let n = *n as usize;
                    if self.sp + n <= self.input.len() {
                        self.sp += n;
                        self.ip += 1;
                    } else if !self.fail() {
                        return None;
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
                        return None;
                    }
                }
                Instruction::Fail => {
                    if !self.fail() {
                        return None;
                    }
                }
                Instruction::Call(label) => {
                    self.stack.push(StackEntry::Return { ip: self.ip + 1 });
                    self.ip = label.0;
                }
                Instruction::Return => {
                    let ret_ip = match self.stack.pop() {
                        Some(StackEntry::Return { ip }) => ip,
                        Some(_) => panic!("Return found Backtrack on stack"),
                        None => panic!("Return on empty stack"),
                    };
                    self.ip = ret_ip;
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
                    return Some(MatchResult {
                        matched: self.sp,
                        captures: self
                            .captures
                            .into_iter()
                            .map(|c| Capture {
                                kind: c.kind,
                                start: c.start,
                                end: c.end.unwrap_or(c.start),
                            })
                            .collect(),
                    });
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
}
