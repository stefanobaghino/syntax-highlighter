use std::collections::HashMap;

use super::analysis::{analyze_left_recursion, LintFinding};
use super::pattern::Pattern;
use crate::pegvm::{CaptureKind, Instruction, Label, LabelId, MemoId, Program, RuleKind};

#[derive(Debug)]
pub enum CompileError {
    UndefinedRule(String),
    UnknownStartRule(String),
    /// One or more `^^lbl` catches have no following context to infer
    /// their boundary from. Emitted by `resolve_inferred_boundaries`.
    CannotInferBoundary {
        rule: String,
        label: String,
    },
    /// `lint_partial_match` returned a non-empty result. Each finding
    /// names a `(rule, caller)` pair where a trailing-nullable rule's
    /// partial-match leniency reaches an unguarded scope. Anchor with
    /// `^^lbl B` (or `^^lbl`) when the leniency is a bug, or wrap the
    /// call site with `~p` when the leniency is intentional.
    PartialMatchLeniency(Vec<LintFinding>),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::UndefinedRule(name) => write!(f, "undefined rule: {}", name),
            CompileError::UnknownStartRule(name) => write!(f, "unknown start rule: {}", name),
            CompileError::CannotInferBoundary { rule, label } => write!(
                f,
                "cannot infer boundary for `^^{label}` in rule `{rule}`: no following context. \
                 Either delete the unreachable catch or specify an explicit boundary `^^{label} B`."
            ),
            CompileError::PartialMatchLeniency(findings) => {
                writeln!(f, "partial-match leniency detected:")?;
                for finding in findings {
                    writeln!(
                        f,
                        "  - rule `{}` is called unanchored by `{}`; its trailing optional \
                         can succeed on a prefix, leaving bytes the caller does not validate",
                        finding.rule, finding.caller
                    )?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for CompileError {}

/// Construct a `Label` from a `usize` instruction position. The expect()
/// is unreachable for any program `pegc` actually produces — sqlite, our
/// largest grammar, compiles to ~6 K instructions, six orders of
/// magnitude below `u32::MAX`.
fn pos_to_label(pos: usize) -> Label {
    Label(u32::try_from(pos).expect("instruction position exceeds u32::MAX"))
}

struct Compiler {
    code: Vec<Instruction>,
    pending_calls: Vec<(usize, String)>,
    capture_kinds: HashMap<String, CaptureKind>,
    capture_names: Vec<String>,
    label_kinds: HashMap<String, LabelId>,
    label_names: Vec<String>,
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            code: Vec::new(),
            pending_calls: Vec::new(),
            capture_kinds: HashMap::new(),
            capture_names: Vec::new(),
            label_kinds: HashMap::new(),
            label_names: Vec::new(),
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
        let target = pos_to_label(target);
        let new = match &self.code[idx] {
            Instruction::Jump(_) => Instruction::Jump(target),
            Instruction::Choice(_) => Instruction::Choice(target),
            Instruction::Commit(_) => Instruction::Commit(target),
            Instruction::PartialCommit(_) => Instruction::PartialCommit(target),
            Instruction::BackCommit(_) => Instruction::BackCommit(target),
            Instruction::TestChar(b, _) => Instruction::TestChar(*b, target),
            Instruction::TestSet(s, _) => Instruction::TestSet(*s, target),
            Instruction::Call(_) => Instruction::Call(target),
            Instruction::RuleEnter(id, kind, _) => Instruction::RuleEnter(*id, *kind, target),
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

    fn intern_label(&mut self, name: &str) -> LabelId {
        if let Some(&k) = self.label_kinds.get(name) {
            return k;
        }
        let k = LabelId(self.label_names.len() as u16);
        self.label_kinds.insert(name.to_string(), k);
        self.label_names.push(name.to_string());
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
                self.emit(Instruction::PartialCommit(pos_to_label(body)));
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
            Pattern::Catch {
                inner,
                label,
                recovery,
            } => {
                //              RecoverScopeBegin(label)
                //              Choice rec
                //              <inner>
                //              Commit done            ; pops outer Backtrack
                // rec:         RecoverToScopedMax     ; splice failed inner's
                //                                     ;   deepest-reach captures
                //              <recovery>
                // done:        RecoverScopeEnd
                //
                // Catches every anonymous failure of `<inner>`. The
                // `label` is a diagnostic tag threaded into the
                // `RecoverScope` frame so `RecoverToScopedMax`'s
                // emitted `RecoveryDiagnostic` carries it; `pegdb
                // recoveries explain` clusters firings by it.
                //
                // `RecoverScope` preserves the failed attempt's
                // deepest-reach captures via `RecoverToScopedMax`
                // before `<recovery>` runs — the capture-preservation
                // story from #16.
                //
                // If `<recovery>` also fails the fail propagates past
                // the `RecoverScope` frame (cleaned up by the
                // `RecoverScope` arm of `fail()` in src/pegvm/vm.rs)
                // and the whole catch fails to its enclosing
                // backtrack.
                let scope_label = self.intern_label(label);
                self.emit(Instruction::RecoverScopeBegin(scope_label));
                let outer_choice = self.emit(Instruction::Choice(Label(0))); // → rec
                self.compile_pat(inner);
                let success_commit = self.emit(Instruction::Commit(Label(0))); // → done

                let rec = self.pos();
                self.patch_jump(outer_choice, rec);
                self.emit(Instruction::RecoverToScopedMax);
                self.compile_pat(recovery);

                let done = self.pos();
                self.patch_jump(success_commit, done);
                self.emit(Instruction::RecoverScopeEnd);
            }
            // Transparent at runtime — the `~` marker affects only
            // the lint walker. See `Pattern::Lenient` documentation.
            Pattern::Lenient(inner) => self.compile_pat(inner),
            // The FOLLOW-inferred resolver must run before bytecode
            // emission; reaching this arm is a bug.
            Pattern::InferBoundaryCatch { .. } => {
                unreachable!(
                    "Pattern::InferBoundaryCatch must be resolved by \
                     analysis::resolve_inferred_boundaries before compilation"
                )
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
        rule_count: 0,
        rule_names: Vec::new(),
        label_kinds: c.label_names,
        rule_is_trivia: Vec::new(),
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

    // Detect undefined NonTerminal references up front so the LR analysis
    // doesn't have to defend against missing rules in the call graph.
    for (rule_name, body) in rules {
        check_refs(rule_name, body, rules)?;
    }

    // Identify left-recursive rules (direct and indirect).
    let lr_rules = analyze_left_recursion(rules)?;

    // Compute the per-rule trivia bit by cascading from any `trivia <- …`
    // reserved-name root the grammar defines. Catch-bearing rules are
    // pinned out of the cascade so their frames stay visible in
    // `pegdb recoveries explain`.
    let trivia_bits = super::analysis::compute_trivia_rules(rules);

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
    let mut rule_names: Vec<String> = Vec::new();
    let mut rule_is_trivia: Vec<bool> = Vec::new();
    let mut rule_count: u32 = 0;
    for name in ordered {
        rule_addrs.insert(name.clone(), c.pos());
        let memo_id = MemoId(rule_count);
        rule_names.push(name.clone());
        rule_is_trivia.push(trivia_bits.get(name).copied().unwrap_or(false));
        rule_count += 1;
        let kind = if lr_rules.contains(name.as_str()) {
            RuleKind::Lr
        } else {
            RuleKind::Memo
        };
        // RuleEnter's Label is patched to the Return address below so a cache
        // hit can skip straight past the body. LR rules close the body with
        // LRTail (the seed-and-grow controller); non-LR rules close with
        // MemoClose (the success-entry committer).
        let enter = c.emit(Instruction::RuleEnter(memo_id, kind, Label(0)));
        let body_start = c.pos();
        c.compile_pat(&rules[name]);
        match kind {
            RuleKind::Lr => {
                c.emit(Instruction::LRTail(memo_id, pos_to_label(body_start)));
            }
            RuleKind::Memo => {
                c.emit(Instruction::MemoClose(memo_id));
            }
        }
        let return_addr = c.pos();
        c.emit(Instruction::Return);
        c.patch_jump(enter, return_addr);
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
        rule_count: rule_count as usize,
        rule_names,
        label_kinds: c.label_names,
        rule_is_trivia,
    })
}

/// Walk a pattern reporting any `NonTerminal(name)` whose name isn't in
/// `rules`. Splits the undefined-rule check out of the bytecode-emit pass
/// so the LR analysis can assume every reference resolves.
fn check_refs(
    _rule: &str,
    pat: &Pattern,
    rules: &HashMap<String, Pattern>,
) -> Result<(), CompileError> {
    match pat {
        Pattern::Literal(_) | Pattern::CharClass(_) | Pattern::AnyChar => Ok(()),
        Pattern::Sequence(items) | Pattern::OrderedChoice(items) => {
            for it in items {
                check_refs(_rule, it, rules)?;
            }
            Ok(())
        }
        Pattern::Repeat(inner)
        | Pattern::RepeatOne(inner)
        | Pattern::Optional(inner)
        | Pattern::NotPredicate(inner)
        | Pattern::AndPredicate(inner)
        | Pattern::Capture(_, inner)
        | Pattern::Lenient(inner) => check_refs(_rule, inner, rules),
        Pattern::Catch {
            inner, recovery, ..
        } => {
            check_refs(_rule, inner, rules)?;
            check_refs(_rule, recovery, rules)
        }
        Pattern::InferBoundaryCatch { inner, .. } => check_refs(_rule, inner, rules),
        Pattern::NonTerminal(name) => {
            if rules.contains_key(name) {
                Ok(())
            } else {
                Err(CompileError::UndefinedRule(name.clone()))
            }
        }
    }
}
