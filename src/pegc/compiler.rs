use std::collections::HashMap;

use super::pattern::Pattern;
use crate::pegvm::{CaptureKind, Instruction, Label, MemoId, Program};

#[derive(Debug)]
pub enum CompileError {
    UndefinedRule(String),
    UnknownStartRule(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::UndefinedRule(name) => write!(f, "undefined rule: {}", name),
            CompileError::UnknownStartRule(name) => write!(f, "unknown start rule: {}", name),
        }
    }
}

impl std::error::Error for CompileError {}

struct Compiler {
    code: Vec<Instruction>,
    pending_calls: Vec<(usize, String)>,
    capture_kinds: HashMap<String, CaptureKind>,
    capture_names: Vec<String>,
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            code: Vec::new(),
            pending_calls: Vec::new(),
            capture_kinds: HashMap::new(),
            capture_names: Vec::new(),
        }
    }

    fn pos(&self) -> usize {
        self.code.len()
    }

    fn emit(&mut self, i: Instruction) -> usize {
        let idx = self.code.len();
        self.code.push(i);
        idx
    }

    fn patch_jump(&mut self, idx: usize, target: usize) {
        debug_assert!(
            idx < self.code.len(),
            "patch_jump: idx {} out of bounds (code has {} instructions)",
            idx,
            self.code.len()
        );
        debug_assert!(
            target <= self.code.len(),
            "patch_jump: target {} out of bounds (code has {} instructions)",
            target,
            self.code.len()
        );
        let target = Label(target);
        let new = match &self.code[idx] {
            Instruction::Jump(_) => Instruction::Jump(target),
            Instruction::Choice(_) => Instruction::Choice(target),
            Instruction::Commit(_) => Instruction::Commit(target),
            Instruction::PartialCommit(_) => Instruction::PartialCommit(target),
            Instruction::BackCommit(_) => Instruction::BackCommit(target),
            Instruction::TestChar(b, _) => Instruction::TestChar(*b, target),
            Instruction::TestSet(s, _) => Instruction::TestSet(*s, target),
            Instruction::Call(_) => Instruction::Call(target),
            Instruction::MemoOpen(id, _) => Instruction::MemoOpen(*id, target),
            Instruction::LRBody(id, _) => Instruction::LRBody(*id, target),
            Instruction::LRTail(id, _) => Instruction::LRTail(*id, target),
            other => panic!("patch_jump: not a jump instruction: {:?}", other),
        };
        self.code[idx] = new;
    }

    fn intern_capture(&mut self, name: &str) -> CaptureKind {
        if let Some(&k) = self.capture_kinds.get(name) {
            return k;
        }
        let k = CaptureKind(self.capture_names.len() as u16);
        self.capture_kinds.insert(name.to_string(), k);
        self.capture_names.push(name.to_string());
        k
    }

    fn compile_pat(&mut self, p: &Pattern) {
        match p {
            Pattern::Literal(bytes) => {
                for &b in bytes {
                    self.emit(Instruction::Char(b));
                }
            }
            Pattern::CharClass(set) => {
                self.emit(Instruction::Set(*set));
            }
            Pattern::AnyChar => {
                self.emit(Instruction::Any(1));
            }
            Pattern::Sequence(items) => {
                for it in items {
                    self.compile_pat(it);
                }
            }
            Pattern::OrderedChoice(items) => {
                if items.is_empty() {
                    return;
                }
                let mut commit_indices = Vec::new();
                let last_idx = items.len() - 1;
                for (i, item) in items.iter().enumerate() {
                    if i < last_idx {
                        let choice = self.emit(Instruction::Choice(Label(0)));
                        self.compile_pat(item);
                        let commit = self.emit(Instruction::Commit(Label(0)));
                        commit_indices.push(commit);
                        let after = self.pos();
                        self.patch_jump(choice, after);
                    } else {
                        self.compile_pat(item);
                    }
                }
                let end = self.pos();
                for idx in commit_indices {
                    self.patch_jump(idx, end);
                }
            }
            Pattern::Repeat(inner) => {
                // Choice L2 ; L_body: <p> ; PartialCommit L_body ; L2:
                // PartialCommit re-uses the existing Backtrack — we must NOT re-execute Choice.
                let choice = self.emit(Instruction::Choice(Label(0)));
                let body = self.pos();
                self.compile_pat(inner);
                self.emit(Instruction::PartialCommit(Label(body)));
                let l2 = self.pos();
                self.patch_jump(choice, l2);
            }
            Pattern::RepeatOne(inner) => {
                // <p> ; (Repeat p)
                self.compile_pat(inner);
                self.compile_pat(&Pattern::Repeat(inner.clone()));
            }
            Pattern::Optional(inner) => {
                // Choice L1 ; <p> ; Commit L1 ; L1:
                let choice = self.emit(Instruction::Choice(Label(0)));
                self.compile_pat(inner);
                let commit = self.emit(Instruction::Commit(Label(0)));
                let l1 = self.pos();
                self.patch_jump(choice, l1);
                self.patch_jump(commit, l1);
            }
            Pattern::NotPredicate(inner) => {
                // Choice L1 ; <p> ; FailTwice ; L1:
                let choice = self.emit(Instruction::Choice(Label(0)));
                self.compile_pat(inner);
                self.emit(Instruction::FailTwice);
                let l1 = self.pos();
                self.patch_jump(choice, l1);
            }
            Pattern::AndPredicate(inner) => {
                // Choice L1 ; <p> ; BackCommit L2 ; L1: Fail ; L2:
                let choice = self.emit(Instruction::Choice(Label(0)));
                self.compile_pat(inner);
                let back = self.emit(Instruction::BackCommit(Label(0)));
                let l1 = self.pos();
                self.emit(Instruction::Fail);
                let l2 = self.pos();
                self.patch_jump(choice, l1);
                self.patch_jump(back, l2);
            }
            Pattern::NonTerminal(name) => {
                let idx = self.emit(Instruction::Call(Label(0)));
                self.pending_calls.push((idx, name.clone()));
            }
            Pattern::Capture(name, inner) => {
                let kind = self.intern_capture(name);
                self.emit(Instruction::CaptureBegin(kind));
                self.compile_pat(inner);
                self.emit(Instruction::CaptureEnd);
            }
            Pattern::RecoverRepeat {
                inner,
                recovery_kind,
            } => {
                // loop_top: Choice rec
                //           <inner>
                //           Commit loop_top
                // rec:      Choice exit       ; Any(1) at EOF exits cleanly
                //           CaptureBegin <recovery_kind>
                //           Any(1)
                //           CaptureEnd
                //           Commit loop_top
                // exit:
                //
                // Uses fresh Choice/Commit per iteration (not PartialCommit) so
                // each retry gets a backtrack baseline at the advanced sp; an
                // in-place PartialCommit on a stale frame would violate the
                // hazard documented in src/pegvm/README.md invariant 1.
                let kind = self.intern_capture(recovery_kind);
                let loop_top = self.pos();
                let outer_choice = self.emit(Instruction::Choice(Label(0))); // → rec
                self.compile_pat(inner);
                self.emit(Instruction::Commit(Label(loop_top)));

                let rec = self.pos();
                self.patch_jump(outer_choice, rec);

                let inner_choice = self.emit(Instruction::Choice(Label(0))); // → exit
                self.emit(Instruction::CaptureBegin(kind));
                self.emit(Instruction::Any(1));
                self.emit(Instruction::CaptureEnd);
                self.emit(Instruction::Commit(Label(loop_top)));

                let exit = self.pos();
                self.patch_jump(inner_choice, exit);
            }
        }
    }
}

/// Compile a single pattern (no NonTerminal references) into a runnable program.
pub fn compile_pattern(pat: &Pattern) -> Program {
    let mut c = Compiler::new();
    c.compile_pat(pat);
    c.emit(Instruction::End);
    if let Some((_, name)) = c.pending_calls.first() {
        panic!(
            "compile_pattern: unresolved NonTerminal({}) — use Grammar::compile",
            name
        );
    }
    Program {
        code: c.code,
        capture_kinds: c.capture_names,
        memo_count: 0,
    }
}

/// Compile a full grammar with named rules. Crate-internal; callers
/// reach this via [`Grammar::compile`](super::Grammar::compile) or
/// the one-step [`super::compile`].
///
/// Code layout:
///   0: Call(<start address>)
///   1: End
///   2..: rule bodies, each ending with Return
pub(crate) fn compile_rules(
    rules: &HashMap<String, Pattern>,
    start: &str,
) -> Result<Program, CompileError> {
    if !rules.contains_key(start) {
        return Err(CompileError::UnknownStartRule(start.to_string()));
    }

    let mut c = Compiler::new();
    c.emit(Instruction::Call(Label(0))); // patched below
    c.emit(Instruction::End);

    // Compile rules in a stable order: start first, then alphabetical (deterministic for tests).
    let mut names: Vec<&String> = rules.keys().collect();
    names.sort();
    let mut ordered: Vec<&String> = vec![&rules.keys().find(|k| k.as_str() == start).unwrap()];
    for n in names {
        if n != start {
            ordered.push(n);
        }
    }

    let mut rule_addrs: HashMap<String, usize> = HashMap::new();
    let mut memo_count: u32 = 0;
    for name in ordered {
        rule_addrs.insert(name.clone(), c.pos());
        let memo_id = MemoId(memo_count);
        memo_count += 1;
        // MemoOpen's Label is patched to the Return address below so a cache
        // hit can skip straight past the body to the rule's Return.
        let memo_open = c.emit(Instruction::MemoOpen(memo_id, Label(0)));
        c.compile_pat(&rules[name]);
        c.emit(Instruction::MemoClose(memo_id));
        let return_addr = c.pos();
        c.emit(Instruction::Return);
        c.patch_jump(memo_open, return_addr);
    }

    // Patch the bootstrap Call.
    let start_addr = rule_addrs[start];
    c.patch_jump(0, start_addr);

    // Patch all NonTerminal Calls.
    for (idx, name) in std::mem::take(&mut c.pending_calls) {
        let target = rule_addrs
            .get(&name)
            .copied()
            .ok_or_else(|| CompileError::UndefinedRule(name.clone()))?;
        c.patch_jump(idx, target);
    }

    Ok(Program {
        code: c.code,
        capture_kinds: c.capture_names,
        memo_count: memo_count as usize,
    })
}
