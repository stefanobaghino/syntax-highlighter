use std::collections::HashMap;

use super::analysis::{analyze_left_recursion, LintFinding};
use super::pattern::{IndentArg, IndentOp, Pattern, Span};
use syntax_highlighter::pegvm::{
    ArgSrc, CaptureKind, CmpOp, Instruction, Label, LabelId, MemoId, Program, RuleKind,
};

#[derive(Debug)]
pub enum CompileError {
    UndefinedRule(String),
    /// Grammar's entry rule is absent from the rule map. For parsed
    /// grammars the parser guarantees the single top-level declaration
    /// is present; this mainly guards hand-built grammars (built via
    /// `Grammar::new`, which bypass the parser) whose map lacks a
    /// `"root"` entry.
    MissingRootRule,
    /// One or more `^^lbl` catches have no following context to infer
    /// their boundary from. Emitted by `resolve_inferred_boundaries`.
    /// `span` is the source position of the `^^lbl` placeholder in the
    /// grammar; rendered as `{line}:{col}:` in [`Display`].
    CannotInferBoundary {
        rule: String,
        label: String,
        span: Span,
    },
    /// `lint_partial_match` returned a non-empty result. Each finding
    /// names a `(rule, caller)` pair where a trailing-nullable rule's
    /// partial-match leniency reaches an unguarded scope. Anchor the rule
    /// to its FOLLOW (`^&lbl`, or `^^lbl B` / `^^lbl` when recovery is also
    /// wanted) or restructure it so the trailing optional can't win on a
    /// prefix. Each [`LintFinding`] carries the call-site's span so the
    /// rendered message points directly at the unanchored call.
    PartialMatchLeniency(Vec<LintFinding>),
    /// One or more literals are reachable from both a `reserved` rule and
    /// a `preferred` rule, so the synthesized `reserved` and `preferred`
    /// sets would overlap — a word can't be both barred from and allowed
    /// in identifier position. Emitted by `synthesize_reserved_preferred`.
    /// The payload is the offending literals (UTF-8 lossy), sorted.
    ReservedPreferredConflict(Vec<String>),
    /// A parameterized call passes the wrong number of indentation
    /// arguments for the callee's declared parameter list — e.g.
    /// `suite()` or `suite(a, b)` for a `suite(outer)`. Detected in
    /// `check_refs` against [`Grammar::rule_params`](super::Grammar::rule_params).
    ArgCountMismatch {
        callee: String,
        expected: usize,
        found: usize,
    },
    /// An indent combinator or call argument references a local name
    /// (`same(i)`, `block(n)`) that is not in scope — no enclosing rule
    /// parameter or earlier `as`-binding introduced it. Carries the
    /// offending name and the rule it appeared in.
    UnboundIndentLocal {
        rule: String,
        name: String,
    },
    /// A `%align` / `%indent` operator, or a call to an
    /// indentation-anchored rule, is reachable from the grammar root with
    /// no enclosing `%root` / `%indent` to establish the anchor column, so
    /// the column would have no provider at runtime. Emitted by
    /// `desugar_indent`. `rule` is where the unanchored demand surfaced
    /// (the grammar root); `demand` describes the construct that needed
    /// the anchor.
    IndentAnchorOutOfScope {
        rule: String,
        demand: String,
    },
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::UndefinedRule(name) => write!(f, "undefined rule: {}", name),
            CompileError::MissingRootRule => write!(
                f,
                "grammar must define an entry rule (the single top-level declaration)"
            ),
            CompileError::CannotInferBoundary { rule, label, span } => write!(
                f,
                "{span}: cannot infer boundary for `^^{label}` in rule `{rule}`: no following context. \
                 Either delete the unreachable catch or specify an explicit boundary `^^{label} B`."
            ),
            CompileError::PartialMatchLeniency(findings) => {
                writeln!(f, "partial-match leniency detected:")?;
                for finding in findings {
                    writeln!(
                        f,
                        "  - {}: rule `{}` is called unanchored by `{}`; its trailing optional \
                         can succeed on a prefix, leaving bytes the caller does not validate",
                        finding.call_site, finding.rule, finding.caller
                    )?;
                }
                Ok(())
            }
            CompileError::ReservedPreferredConflict(words) => write!(
                f,
                "the following word(s) are marked both `reserved` and `preferred`, \
                 so the synthesized `reserved` / `preferred` sets would overlap: {}",
                words.join(", ")
            ),
            CompileError::ArgCountMismatch {
                callee,
                expected,
                found,
            } => write!(
                f,
                "rule `{callee}` takes {expected} indentation argument(s) but is called with {found}"
            ),
            CompileError::UnboundIndentLocal { rule, name } => write!(
                f,
                "indentation local `{name}` referenced in rule `{rule}` is not in scope \
                 (no enclosing parameter or `as` binding introduces it)"
            ),
            CompileError::IndentAnchorOutOfScope { rule, demand } => write!(
                f,
                "{demand} in rule `{rule}` has no indentation anchor in scope: it is reachable \
                 from the grammar root with no enclosing `%root` or `%indent` to establish the \
                 column"
            ),
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
    char_sets: Vec<syntax_highlighter::pegvm::CharSet>,
    char_set_dedup: HashMap<syntax_highlighter::pegvm::CharSet, syntax_highlighter::pegvm::SetId>,
    /// Activation-slot environment for the rule currently being
    /// compiled, in slot order: `Some(name)` for a parameter or an
    /// `as`-bound indent width, `None` for an anonymous measure (e.g.
    /// the throwaway width a `same(i)` measures). The slot index of a
    /// name is its position here, so `IndentArg::Local` resolves by
    /// reverse lookup. Reset per rule by [`begin_rule_locals`].
    ///
    /// [`begin_rule_locals`]: Compiler::begin_rule_locals
    locals: Vec<Option<String>>,
    /// The rule name owning `locals`, for error messages.
    current_rule: String,
    /// First unresolved-local error seen while lowering, surfaced by
    /// `compile_rules` after the whole grammar is walked. Kept as
    /// accumulated state (rather than threading `Result` through the
    /// infallible `compile_pat`) so the bare-pattern path stays simple;
    /// the validation in `check_refs` catches the same class of error
    /// for the grammar path up front.
    local_error: Option<CompileError>,
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
            char_sets: Vec::new(),
            char_set_dedup: HashMap::new(),
            locals: Vec::new(),
            current_rule: String::new(),
            local_error: None,
        }
    }

    /// Reset the activation-slot environment for a new rule: parameters
    /// occupy the first slots in declaration order, anonymous /
    /// `as`-bound slots accrue after them as the body is lowered.
    fn begin_rule_locals(&mut self, rule: &str, params: &[String]) {
        self.current_rule = rule.to_string();
        self.locals = params.iter().map(|p| Some(p.clone())).collect();
    }

    /// Resolve an [`IndentArg`] to a runtime [`ArgSrc`]: a literal
    /// verbatim, a local by reverse-lookup of its slot. An unresolved
    /// local records a [`CompileError::UnboundIndentLocal`] (first one
    /// wins) and falls back to slot 0 so lowering can continue to a
    /// clean error return rather than panicking.
    fn indent_arg_src(&mut self, arg: &IndentArg) -> ArgSrc {
        match arg {
            IndentArg::Lit(v) => ArgSrc::Lit(*v),
            IndentArg::Local(name) => ArgSrc::Local(self.resolve_local(name)),
        }
    }

    /// Slot index of local `name` in the current rule's environment
    /// (most-recent binding wins). Records an error and returns 0 if
    /// absent.
    fn resolve_local(&mut self, name: &str) -> u8 {
        if let Some(pos) = self
            .locals
            .iter()
            .rposition(|slot| slot.as_deref() == Some(name))
        {
            u8::try_from(pos).unwrap_or(u8::MAX)
        } else {
            if self.local_error.is_none() {
                self.local_error = Some(CompileError::UnboundIndentLocal {
                    rule: self.current_rule.clone(),
                    name: name.to_string(),
                });
            }
            0
        }
    }

    /// Intern a character set. Identical sets share one entry in
    /// `char_sets` and one `SetId`; this matters for grammars that
    /// inline the same character class at many sites (e.g. the
    /// ASCII-letter `{lower, upper}` pairs that show up in every
    /// `i"…"` literal across the SQL keywords).
    fn intern_char_set(
        &mut self,
        set: syntax_highlighter::pegvm::CharSet,
    ) -> syntax_highlighter::pegvm::SetId {
        if let Some(&id) = self.char_set_dedup.get(&set) {
            return id;
        }
        let id = syntax_highlighter::pegvm::SetId(
            u16::try_from(self.char_sets.len()).expect("char_sets count exceeds u16::MAX"),
        );
        self.char_sets.push(set.clone());
        self.char_set_dedup.insert(set, id);
        id
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
            Instruction::Call(_) => Instruction::Call(target),
            Instruction::RuleEnter(id, kind, _, argc) => {
                Instruction::RuleEnter(*id, *kind, target, *argc)
            }
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
            Pattern::Literal { bytes, .. } => {
                for &b in bytes {
                    self.emit(Instruction::Byte(b));
                }
            }
            Pattern::CharClass { set, .. } => {
                // Intern "recovery" so the VM has a capture kind to tag
                // UTF-8 recovery spans with at dispatch time. Without
                // this the runtime falls back to silent-skip on bad
                // bytes — the compiler is the right layer to guarantee
                // the invariant.
                self.intern_capture("recovery");
                let set_id = self.intern_char_set(set.clone());
                self.emit(Instruction::CharSet(set_id));
            }
            Pattern::AnyChar { .. } => {
                self.emit(Instruction::Any);
            }
            Pattern::Sequence { items, .. } => {
                for it in items {
                    self.compile_pat(it);
                }
            }
            Pattern::OrderedChoice { alts, .. } => {
                if alts.is_empty() {
                    return;
                }
                let mut commit_indices = Vec::new();
                let last_idx = alts.len() - 1;
                for (i, item) in alts.iter().enumerate() {
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
            Pattern::Repeat { inner, .. } => {
                // Choice L2 ; L_body: <p> ; PartialCommit L_body ; L2:
                // PartialCommit re-uses the existing Backtrack — we must NOT re-execute Choice.
                let choice = self.emit(Instruction::Choice(Label(0)));
                let body = self.pos();
                self.compile_pat(inner);
                self.emit(Instruction::PartialCommit(pos_to_label(body)));
                let l2 = self.pos();
                self.patch_jump(choice, l2);
            }
            Pattern::RepeatOne { inner, span } => {
                // <p> ; (Repeat p)
                self.compile_pat(inner);
                self.compile_pat(&Pattern::Repeat {
                    inner: inner.clone(),
                    span: *span,
                });
            }
            Pattern::Optional { inner, .. } => {
                // Choice L1 ; <p> ; Commit L1 ; L1:
                let choice = self.emit(Instruction::Choice(Label(0)));
                self.compile_pat(inner);
                let commit = self.emit(Instruction::Commit(Label(0)));
                let l1 = self.pos();
                self.patch_jump(choice, l1);
                self.patch_jump(commit, l1);
            }
            Pattern::NotPredicate { inner, .. } => {
                // Choice L1 ; <p> ; FailTwice ; L1:
                let choice = self.emit(Instruction::Choice(Label(0)));
                self.compile_pat(inner);
                self.emit(Instruction::FailTwice);
                let l1 = self.pos();
                self.patch_jump(choice, l1);
            }
            Pattern::AndPredicate { inner, .. } => {
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
            Pattern::NonTerminal { name, args, .. } => {
                // Push the indentation arguments (if any) left-to-right so
                // the callee's `RuleEnter` seeds parameter slot 0 with the
                // first argument. A plain reference (empty `args`) emits
                // just the `Call`, exactly as before.
                for arg in args {
                    let src = self.indent_arg_src(arg);
                    self.emit(Instruction::ArgPush(src));
                }
                let idx = self.emit(Instruction::Call(Label(0)));
                self.pending_calls.push((idx, name.clone()));
            }
            Pattern::IndentCombinator { op, arg, bind, .. } => {
                // Resolve the reference column against the environment
                // *before* introducing this combinator's own slot, so
                // `same(i)` binds to the earlier `i` and a degenerate
                // `deeper(i) as i` resolves `i` as unbound rather than to
                // its own not-yet-measured slot.
                let src = self.indent_arg_src(arg);
                let measure_slot = u8::try_from(self.locals.len()).unwrap_or(u8::MAX);
                // The measured width takes the next slot — named by the
                // `as` binder, or anonymous for a bare assertion.
                self.locals.push(bind.clone());
                self.emit(Instruction::IndentMeasure(measure_slot));
                let cmp = match op {
                    IndentOp::Deeper => CmpOp::Gt,
                    IndentOp::Same => CmpOp::Eq,
                    IndentOp::AtLeast => CmpOp::Ge,
                };
                self.emit(Instruction::IndentCmp(cmp, measure_slot, src));
            }
            Pattern::Capture { kind, inner, .. } => {
                let kind_id = self.intern_capture(kind);
                self.emit(Instruction::CaptureBegin(kind_id));
                self.compile_pat(inner);
                self.emit(Instruction::CaptureEnd);
            }
            Pattern::Catch {
                inner,
                label,
                recovery,
                ..
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
            // The FOLLOW-inferred resolver must run before bytecode
            // emission; reaching this arm is a bug.
            Pattern::InferBoundaryCatch { .. } => {
                unreachable!(
                    "Pattern::InferBoundaryCatch must be resolved by \
                     analysis::resolve_inferred_boundaries before compilation"
                )
            }
            // `desugar_indent` runs first in `Grammar::compile`, rewriting
            // every `%`-operator into the combinator IR; none survives to
            // lowering.
            Pattern::IndentOp { .. } => {
                unreachable!(
                    "Pattern::IndentOp must be desugared by desugar_indent before compilation"
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
    if let Some(err) = &c.local_error {
        panic!("compile_pattern: {err} — use Grammar::compile");
    }
    Program {
        code: c.code,
        capture_kinds: c.capture_names,
        rule_count: 0,
        rule_names: Vec::new(),
        label_kinds: c.label_names,
        rule_is_ignore: Vec::new(),
        char_sets: c.char_sets,
    }
}

/// Compile a full grammar with named rules. Crate-internal; callers
/// reach this via [`Grammar::compile`](super::Grammar::compile) or
/// the one-step [`super::compile`].
///
/// `root` is the grammar's entry rule (the single top-level
/// declaration's name, or [`ROOT_RULE`](super::parser::ROOT_RULE) for
/// hand-built grammars); `Grammar::compile` checks for its presence up
/// front and wraps its body via [`super::analysis::wrap_root`] before
/// this is called.
///
/// Code layout:
///   0: Call(<root address>)
///   1: End
///   2..: rule bodies, each ending with Return
pub(crate) fn compile_rules(
    rules: &HashMap<String, Pattern>,
    root: &str,
    rule_params: &HashMap<String, Vec<String>>,
) -> Result<Program, CompileError> {
    let start = root;
    debug_assert!(
        rules.contains_key(start),
        "compile_rules: entry rule must be present (Grammar::compile enforces this)"
    );

    // Detect undefined NonTerminal references and argument-arity
    // mismatches up front so the LR analysis doesn't have to defend
    // against missing rules in the call graph.
    for (rule_name, body) in rules {
        check_refs(rule_name, body, rules, rule_params)?;
    }

    // Identify left-recursive rules (direct and indirect).
    let lr_rules = analyze_left_recursion(rules)?;

    // Compute the per-rule ignore bit by cascading from any `ignore = …`
    // reserved-name root the grammar defines. Catch-bearing rules are
    // pinned out of the cascade so their frames stay visible in
    // `pegdb recoveries explain`.
    let ignore_bits = super::analysis::compute_ignore_rules(rules);

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
    let mut rule_is_ignore: Vec<bool> = Vec::new();
    let mut rule_count: u32 = 0;
    for name in ordered {
        rule_addrs.insert(name.clone(), c.pos());
        let memo_id = MemoId(rule_count);
        rule_names.push(name.clone());
        rule_is_ignore.push(ignore_bits.get(name).copied().unwrap_or(false));
        rule_count += 1;
        let kind = if lr_rules.contains(name.as_str()) {
            RuleKind::Lr
        } else {
            RuleKind::Memo
        };
        // RuleEnter's Label is patched to the Return address below so a cache
        // hit can skip straight past the body. LR rules close the body with
        // LRTail (the seed-and-grow controller); non-LR rules close with
        // MemoClose (the success-entry committer). `argc` is the rule's
        // declared parameter count, looked up from the grammar's
        // `rule_params`; 0 for every non-parameterized rule.
        let argc =
            u8::try_from(rule_params.get(name.as_str()).map_or(0, Vec::len)).unwrap_or(u8::MAX);
        let enter = c.emit(Instruction::RuleEnter(memo_id, kind, Label(0), argc));
        let body_start = c.pos();
        // Seed the activation-slot environment with this rule's params
        // before lowering its body, so `IndentArg::Local` references and
        // `as`-bindings resolve to stable slot indices.
        let params: &[String] = rule_params.get(name).map_or(&[], Vec::as_slice);
        c.begin_rule_locals(name, params);
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

    // Surface any unresolved indentation-local reference encountered
    // during lowering (the env-tracked counterpart to `check_refs`'s
    // arity check).
    if let Some(err) = c.local_error.take() {
        return Err(err);
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
        rule_is_ignore,
        char_sets: c.char_sets,
    })
}

/// Walk a pattern reporting any `NonTerminal(name)` whose name isn't in
/// `rules`. Splits the undefined-rule check out of the bytecode-emit pass
/// so the LR analysis can assume every reference resolves.
fn check_refs(
    _rule: &str,
    pat: &Pattern,
    rules: &HashMap<String, Pattern>,
    rule_params: &HashMap<String, Vec<String>>,
) -> Result<(), CompileError> {
    match pat {
        Pattern::Literal { .. }
        | Pattern::CharClass { .. }
        | Pattern::AnyChar { .. }
        // An indent combinator references no rule and carries no inner
        // pattern; its only name is an `IndentArg::Local`, validated
        // during lowering against the activation environment.
        | Pattern::IndentCombinator { .. } => Ok(()),
        Pattern::Sequence { items, .. } => {
            for it in items {
                check_refs(_rule, it, rules, rule_params)?;
            }
            Ok(())
        }
        Pattern::OrderedChoice { alts, .. } => {
            for it in alts {
                check_refs(_rule, it, rules, rule_params)?;
            }
            Ok(())
        }
        Pattern::Repeat { inner, .. }
        | Pattern::RepeatOne { inner, .. }
        | Pattern::Optional { inner, .. }
        | Pattern::NotPredicate { inner, .. }
        | Pattern::AndPredicate { inner, .. }
        | Pattern::Capture { inner, .. } => check_refs(_rule, inner, rules, rule_params),
        Pattern::Catch {
            inner, recovery, ..
        } => {
            check_refs(_rule, inner, rules, rule_params)?;
            check_refs(_rule, recovery, rules, rule_params)
        }
        Pattern::InferBoundaryCatch { inner, .. } => check_refs(_rule, inner, rules, rule_params),
        // Desugared away before `compile_rules` runs.
        Pattern::IndentOp { .. } => {
            unreachable!("Pattern::IndentOp must be desugared by desugar_indent before compilation")
        }
        Pattern::NonTerminal { name, args, .. } => {
            if !rules.contains_key(name) {
                return Err(CompileError::UndefinedRule(name.clone()));
            }
            // A parameterized call must pass exactly the callee's
            // declared parameter count; a plain reference (no args) to a
            // parameterized rule is itself a mismatch (0 vs N).
            let expected = rule_params.get(name).map_or(0, Vec::len);
            if args.len() != expected {
                return Err(CompileError::ArgCountMismatch {
                    callee: name.clone(),
                    expected,
                    found: args.len(),
                });
            }
            Ok(())
        }
    }
}
