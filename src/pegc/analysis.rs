//! Static analyses over the grammar's [`Pattern`] AST.
//!
//! The compiler consumes [`analyze_left_recursion`] to decide which
//! rules close with `LRTail` instead of `MemoClose`. Tooling consumes
//! the rest (FIRST/FOLLOW) for grammar inspection — see the
//! `pegc follow-set` subcommand.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use super::compiler::CompileError;
use super::parser::Grammar;
use super::pattern::{Pattern, Span};
use crate::pegvm::CharSet;

/// Returns the set of rule names that are left-recursive — both direct
/// (`A = A α / β`) and indirect (`A = B …; B = A …`). Each such rule
/// is emitted with `RuleEnter(_, RuleKind::Lr, _)` and closed with
/// `LRTail` instead of `MemoClose`, so its packrat slot isn't written
/// with a value that depends on an in-progress LR seed of a sibling in
/// the same cycle.
///
/// Algorithm:
/// 1. Compute may-match-empty (nullability) per rule via a fixpoint.
/// 2. Build the "first-call" graph: edge `A → B` iff `A`'s body can call
///    `B` before consuming any input.
/// 3. Find strongly connected components in that graph (Tarjan's). Any
///    SCC of size > 1 is an indirect-LR cycle — every member is wrapped.
///    A size-1 SCC is direct LR iff the rule has a self-edge in the
///    first-call graph.
pub(crate) fn analyze_left_recursion(
    rules: &HashMap<String, Pattern>,
) -> Result<HashSet<String>, CompileError> {
    let nullable = compute_nullable(rules);
    let first_calls = compute_first_calls(rules, &nullable);

    let sccs = tarjan_sccs(rules, &first_calls);
    let mut lr_rules = HashSet::new();
    for scc in &sccs {
        if scc.len() > 1 {
            for r in scc {
                lr_rules.insert(r.clone());
            }
        } else {
            let only = &scc[0];
            if first_calls
                .get(only)
                .map(|s| s.contains(only))
                .unwrap_or(false)
            {
                lr_rules.insert(only.clone());
            }
        }
    }
    Ok(lr_rules)
}

/// Per-rule nullability via fixpoint over the Pattern AST. A rule is
/// nullable iff its body can match the empty string.
pub(crate) fn compute_nullable(rules: &HashMap<String, Pattern>) -> HashSet<String> {
    let mut nullable: HashSet<String> = HashSet::new();
    loop {
        let mut changed = false;
        for (name, body) in rules {
            if !nullable.contains(name) && pattern_nullable(body, &nullable) {
                nullable.insert(name.clone());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    nullable
}

pub(crate) fn pattern_nullable(pat: &Pattern, nullable: &HashSet<String>) -> bool {
    match pat {
        Pattern::Literal { bytes, .. } => bytes.is_empty(),
        Pattern::CharClass { .. } | Pattern::AnyChar { .. } => false,
        Pattern::Sequence { items, .. } => items.iter().all(|p| pattern_nullable(p, nullable)),
        Pattern::OrderedChoice { alts, .. } => alts.iter().any(|p| pattern_nullable(p, nullable)),
        Pattern::Repeat { .. } | Pattern::Optional { .. } => true,
        Pattern::RepeatOne { inner, .. } => pattern_nullable(inner, nullable),
        // Predicates consume no input, so they always succeed-or-fail
        // without advancing — treated as nullable for sequence
        // propagation. They can still left-recurse: !A α matches only
        // if A would fail at sp, and A's evaluation can left-recurse
        // through itself just like a direct call.
        Pattern::NotPredicate { .. } | Pattern::AndPredicate { .. } => true,
        Pattern::Capture { inner, .. } => pattern_nullable(inner, nullable),
        // Catch succeeds-as-inner when inner succeeds, or
        // succeeds-as-recovery when inner fails. The recovery branch
        // only runs on inner's failure, so it never contributes a
        // "matches empty after success" path — nullability follows
        // inner.
        Pattern::Catch { inner, .. } => pattern_nullable(inner, nullable),
        // `~` is runtime-transparent.
        Pattern::Lenient { inner, .. } => pattern_nullable(inner, nullable),
        // Pre-resolution placeholder for `^^lbl`. Nullability follows
        // inner just like `Catch` — the eventual recovery body
        // `(!B .)*` is always nullable but only contributes on
        // failure, same shape as `Catch`'s analysis.
        Pattern::InferBoundaryCatch { inner, .. } => pattern_nullable(inner, nullable),
        Pattern::NonTerminal { name, .. } => nullable.contains(name),
    }
}

/// First-call graph: `first_calls[A]` is the set of rules `A`'s body can
/// invoke before consuming any input. Used to find left-recursive cycles.
fn compute_first_calls(
    rules: &HashMap<String, Pattern>,
    nullable: &HashSet<String>,
) -> HashMap<String, HashSet<String>> {
    let mut out = HashMap::new();
    for (name, body) in rules {
        let mut s = HashSet::new();
        collect_first_calls(body, nullable, &mut s);
        out.insert(name.clone(), s);
    }
    out
}

fn collect_first_calls(pat: &Pattern, nullable: &HashSet<String>, out: &mut HashSet<String>) {
    match pat {
        Pattern::Literal { .. } | Pattern::CharClass { .. } | Pattern::AnyChar { .. } => {}
        Pattern::Sequence { items, .. } => {
            for it in items {
                collect_first_calls(it, nullable, out);
                if !pattern_nullable(it, nullable) {
                    break;
                }
            }
        }
        Pattern::OrderedChoice { alts, .. } => {
            for it in alts {
                collect_first_calls(it, nullable, out);
            }
        }
        Pattern::Repeat { inner, .. }
        | Pattern::RepeatOne { inner, .. }
        | Pattern::Optional { inner, .. }
        | Pattern::NotPredicate { inner, .. }
        | Pattern::AndPredicate { inner, .. }
        | Pattern::Capture { inner, .. } => collect_first_calls(inner, nullable, out),
        Pattern::Catch {
            inner, recovery, ..
        } => {
            // Both branches are reachable at baseline sp: inner
            // unconditionally; recovery whenever inner fails at
            // baseline (i.e. without consuming) — possible regardless
            // of inner's nullability. Recurse into both, like
            // OrderedChoice, so the LR analysis doesn't miss cycles
            // routed through a catch's recovery body.
            collect_first_calls(inner, nullable, out);
            collect_first_calls(recovery, nullable, out);
        }
        Pattern::Lenient { inner, .. } => collect_first_calls(inner, nullable, out),
        // Pre-resolution placeholder for `^^lbl`. Only the inner can
        // route a first-call edge — the synthesized recovery
        // `(!B .)*` has no non-terminals other than B, and the
        // resolver hasn't picked B yet.
        Pattern::InferBoundaryCatch { inner, .. } => collect_first_calls(inner, nullable, out),
        Pattern::NonTerminal { name, .. } => {
            out.insert(name.clone());
        }
    }
}

/// Tarjan's strongly-connected-components on the first-call graph.
/// Returned SCCs are non-empty; their internal order is not specified.
fn tarjan_sccs(
    rules: &HashMap<String, Pattern>,
    edges: &HashMap<String, HashSet<String>>,
) -> Vec<Vec<String>> {
    struct State<'a> {
        edges: &'a HashMap<String, HashSet<String>>,
        index: usize,
        indices: HashMap<String, usize>,
        lowlink: HashMap<String, usize>,
        on_stack: HashSet<String>,
        stack: Vec<String>,
        sccs: Vec<Vec<String>>,
    }
    let mut st = State {
        edges,
        index: 0,
        indices: HashMap::new(),
        lowlink: HashMap::new(),
        on_stack: HashSet::new(),
        stack: Vec::new(),
        sccs: Vec::new(),
    };
    fn strongconnect(st: &mut State<'_>, v: &str) {
        st.indices.insert(v.to_string(), st.index);
        st.lowlink.insert(v.to_string(), st.index);
        st.index += 1;
        st.stack.push(v.to_string());
        st.on_stack.insert(v.to_string());
        let succs: Vec<String> = st
            .edges
            .get(v)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default();
        for w in succs {
            if !st.indices.contains_key(&w) {
                strongconnect(st, &w);
                let wl = st.lowlink[&w];
                let vl = st.lowlink[v];
                st.lowlink.insert(v.to_string(), vl.min(wl));
            } else if st.on_stack.contains(&w) {
                let wi = st.indices[&w];
                let vl = st.lowlink[v];
                st.lowlink.insert(v.to_string(), vl.min(wi));
            }
        }
        if st.lowlink[v] == st.indices[v] {
            let mut scc = Vec::new();
            loop {
                let w = st.stack.pop().expect("tarjan: stack underflow");
                st.on_stack.remove(&w);
                let done = w == v;
                scc.push(w);
                if done {
                    break;
                }
            }
            st.sccs.push(scc);
        }
    }
    let mut keys: Vec<&String> = rules.keys().collect();
    keys.sort();
    for k in keys {
        if !st.indices.contains_key(k) {
            strongconnect(&mut st, k);
        }
    }
    st.sccs
}

// -- FIRST / FOLLOW analysis ----------------------------------------------

/// A single element of a FIRST or FOLLOW set.
///
/// References to rules stay opaque (`Rule(name)`) rather than being
/// transitively expanded to leaves. The analysis is therefore one rule
/// reference deep: FOLLOW(`where_clause`) surfaces `Rule("returning_clause")`,
/// not the keyword inside it. Callers who want a deeper view recurse into
/// FIRST(`returning_clause`) themselves.
///
/// Captures are preserved (not stripped to their inner): `@punctuation ','`
/// and `@operator ','` are distinct elements.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum FollowElement {
    Literal(Vec<u8>),
    CharClass(CharSet),
    Rule(String),
    Capture {
        kind: String,
        inner: Box<FollowElement>,
    },
    /// End of input — contributed only to FOLLOW of the start rule (via
    /// the implicit `!.` at the top of every parse).
    Eof,
}

/// Per-rule FIRST or FOLLOW set. `BTreeSet` over `HashSet` for deterministic
/// iteration in JSON output.
pub type FollowSet = BTreeSet<FollowElement>;

/// FIRST set per rule: the elements that can appear as the first thing
/// matched when the rule starts.
///
/// One level deep — rule references stay opaque (`Rule(name)`). See
/// [`FollowElement`] for the data model.
pub fn compute_first(grammar: &Grammar) -> HashMap<String, FollowSet> {
    let nullable = compute_nullable(&grammar.rules);
    let mut first: HashMap<String, FollowSet> = grammar
        .rules
        .keys()
        .map(|n| (n.clone(), FollowSet::new()))
        .collect();
    loop {
        let mut changed = false;
        for (name, body) in &grammar.rules {
            let new_first = pattern_first(body, &nullable);
            let entry = first.get_mut(name).expect("rule entry present");
            for elem in new_first {
                if entry.insert(elem) {
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    first
}

/// FIRST of a single pattern. Rule references stay opaque; captures wrap
/// the inner's FIRST. Predicates contribute zero-width to their parent
/// sequence's continuation but FIRST captures their lookahead constraint:
/// `&p` surfaces FIRST(p); `!p` surfaces nothing (no positive constraint).
fn pattern_first(pat: &Pattern, nullable: &HashSet<String>) -> FollowSet {
    let mut out = FollowSet::new();
    match pat {
        Pattern::Literal { bytes, .. } => {
            out.insert(FollowElement::Literal(bytes.clone()));
        }
        Pattern::CharClass { set, .. } => {
            out.insert(FollowElement::CharClass(set.clone()));
        }
        Pattern::AnyChar { .. } => {
            out.insert(FollowElement::CharClass(CharSet::any()));
        }
        Pattern::NonTerminal { name, .. } => {
            out.insert(FollowElement::Rule(name.clone()));
        }
        Pattern::Sequence { items, .. } => {
            for it in items {
                out.extend(pattern_first(it, nullable));
                if !pattern_nullable(it, nullable) {
                    break;
                }
            }
        }
        Pattern::OrderedChoice { alts, .. } => {
            for alt in alts {
                out.extend(pattern_first(alt, nullable));
            }
        }
        Pattern::Repeat { inner, .. }
        | Pattern::RepeatOne { inner, .. }
        | Pattern::Optional { inner, .. } => {
            out.extend(pattern_first(inner, nullable));
        }
        Pattern::NotPredicate { .. } => {}
        Pattern::AndPredicate { inner, .. } => {
            out.extend(pattern_first(inner, nullable));
        }
        Pattern::Capture { kind, inner, .. } => {
            for elem in pattern_first(inner, nullable) {
                out.insert(FollowElement::Capture {
                    kind: kind.clone(),
                    inner: Box::new(elem),
                });
            }
        }
        Pattern::Catch { inner, .. } => {
            // Recovery body fires only on failure; doesn't participate
            // in FIRST.
            out.extend(pattern_first(inner, nullable));
        }
        Pattern::Lenient { inner, .. } => {
            out.extend(pattern_first(inner, nullable));
        }
        // Pre-resolution placeholder for `^^lbl`. The eventual recovery
        // body `(!B .)*` fires only on failure (same as Catch) so it
        // doesn't appear in FIRST; only the inner does.
        Pattern::InferBoundaryCatch { inner, .. } => {
            out.extend(pattern_first(inner, nullable));
        }
    }
    out
}

/// FOLLOW set per rule: the elements that can appear immediately after
/// any call to the rule at any of its call sites.
///
/// The start rule's FOLLOW seeds with [`FollowElement::Eof`] to model the
/// `!.` at the top of every parse. Other rules' FOLLOW is derived by
/// scanning every rule body for `NonTerminal(R)` references and adding the
/// FIRST of the remainder (plus FOLLOW of the enclosing rule when the
/// remainder is nullable). Iterated to fixed point.
pub fn compute_follow(grammar: &Grammar) -> HashMap<String, FollowSet> {
    let nullable = compute_nullable(&grammar.rules);
    let mut follow: HashMap<String, FollowSet> = grammar
        .rules
        .keys()
        .map(|n| (n.clone(), FollowSet::new()))
        .collect();
    if let Some(entry) = follow.get_mut(super::parser::ROOT_RULE) {
        entry.insert(FollowElement::Eof);
    }
    loop {
        let mut changed = false;
        for (rule_name, body) in &grammar.rules {
            let trailing = follow.get(rule_name).cloned().unwrap_or_default();
            collect_follow(body, &nullable, &trailing, &mut follow, &mut changed);
        }
        if !changed {
            break;
        }
    }
    follow
}

/// Walk `pat` looking for `NonTerminal` references; for each one,
/// add FIRST of the remainder (within the enclosing pattern) to its
/// FOLLOW, and FOLLOW of the enclosing rule when the remainder is nullable.
///
/// `trailing` is what would follow `pat` if `pat` were embedded directly
/// in the enclosing rule's body — i.e. the FIRST-of-remainder propagated
/// from outer scopes. Initially this is FOLLOW of the rule whose body
/// is being walked; recursive calls trim or extend it as appropriate.
fn collect_follow(
    pat: &Pattern,
    nullable: &HashSet<String>,
    trailing: &FollowSet,
    follow: &mut HashMap<String, FollowSet>,
    changed: &mut bool,
) {
    match pat {
        Pattern::Literal { .. } | Pattern::CharClass { .. } | Pattern::AnyChar { .. } => {}
        Pattern::NonTerminal { name, .. } => {
            extend_follow(follow, name, trailing, changed);
        }
        Pattern::Sequence { items, .. } => {
            // For each item at index i, "trailing-for-item-i" is FIRST of
            // items[i+1..] (sequenced through nullable items) union the
            // outer trailing when all of items[i+1..] is nullable.
            for i in 0..items.len() {
                let mut sub_trailing = FollowSet::new();
                let mut all_nullable = true;
                for it in items.iter().skip(i + 1) {
                    sub_trailing.extend(pattern_first(it, nullable));
                    if !pattern_nullable(it, nullable) {
                        all_nullable = false;
                        break;
                    }
                }
                if all_nullable {
                    sub_trailing.extend(trailing.iter().cloned());
                }
                collect_follow(&items[i], nullable, &sub_trailing, follow, changed);
            }
        }
        Pattern::OrderedChoice { alts, .. } => {
            for alt in alts {
                collect_follow(alt, nullable, trailing, follow, changed);
            }
        }
        Pattern::Repeat { inner, .. } | Pattern::RepeatOne { inner, .. } => {
            // The body can iterate, so its tail is followed by another
            // copy of FIRST(body). Plus whatever trails the repeat.
            let body_first = pattern_first(inner, nullable);
            let mut sub_trailing = trailing.clone();
            sub_trailing.extend(body_first);
            collect_follow(inner, nullable, &sub_trailing, follow, changed);
        }
        Pattern::Optional { inner, .. } => {
            collect_follow(inner, nullable, trailing, follow, changed);
        }
        Pattern::NotPredicate { .. } => {
            // Predicate consumes nothing; its body's FOLLOW relations
            // don't affect the outer sequence's flow. References inside
            // a `!p` predicate still need their FIRST recorded though,
            // since the predicate's match position is where they'd
            // appear — but FOLLOW is "what's after when matched," and
            // a `!p` predicate doesn't match anything. Skip.
        }
        Pattern::AndPredicate { inner, .. } => {
            // Similar zero-width logic. Inner FOLLOW relations are
            // structurally meaningful (a NonTerminal inside `&p` does
            // have its own callers expecting it to FIRST-match certain
            // things), so recurse with an empty trailing — the predicate
            // doesn't continue into a consumed tail.
            collect_follow(inner, nullable, &FollowSet::new(), follow, changed);
        }
        Pattern::Capture { inner, .. } => {
            collect_follow(inner, nullable, trailing, follow, changed);
        }
        Pattern::Catch {
            inner, recovery, ..
        } => {
            // Success path: inner takes the catch's place, so it inherits
            // the catch's trailing. Recovery path: recovery body runs
            // independently; its internal FOLLOW relations are computed
            // with an empty trailing because the recovery's own success
            // exits the catch with the same outer trailing — which we
            // could pass, but downstream tooling treats recovery bodies
            // as separate analytical contexts. Use the same trailing for
            // both branches to stay conservative (over-approximate is OK
            // for cross-check).
            collect_follow(inner, nullable, trailing, follow, changed);
            collect_follow(recovery, nullable, trailing, follow, changed);
        }
        Pattern::Lenient { inner, .. } => {
            collect_follow(inner, nullable, trailing, follow, changed);
        }
        // Pre-resolution placeholder for `^^lbl`. Treat like `Catch`'s
        // success arm: inner inherits the catch's trailing. The
        // synthesized recovery body has no NonTerminal references at
        // this stage (the resolver hasn't picked B yet), so there's
        // nothing to propagate into.
        Pattern::InferBoundaryCatch { inner, .. } => {
            collect_follow(inner, nullable, trailing, follow, changed);
        }
    }
}

fn extend_follow(
    follow: &mut HashMap<String, FollowSet>,
    name: &str,
    elements: &FollowSet,
    changed: &mut bool,
) {
    let Some(entry) = follow.get_mut(name) else {
        return;
    };
    for elem in elements {
        if entry.insert(elem.clone()) {
            *changed = true;
        }
    }
}

// -- Partial-match-leniency lint -----------------------------------------

/// The kind of lint that produced a [`LintFinding`]. Named even though
/// v1 has a single variant, so future lints can be added as discriminated
/// kinds without restructuring the API.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum LintKind {
    PartialMatchLeniency,
}

/// One static-lint finding: a rule that can succeed on a prefix of its
/// expected match, called from a site that does not anchor the boundary.
///
/// The fix shape (per PR #101) is a `&(ws <boundary_rule>)` lookahead at
/// the caller's site; once #103 lands, that lookahead is rewritten into
/// the boundary-anchored-catch operator.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct LintFinding {
    pub kind: LintKind,
    /// The rule whose body has a trailing-nullable position; the lenient one.
    pub rule: String,
    /// The rule whose body contains an unanchored call site to [`Self::rule`].
    pub caller: String,
    /// Source position of the unanchored `NonTerminal` call within
    /// `caller`'s body. Populated from [`Pattern::NonTerminal`]'s span;
    /// rendered as `{line}:{col}:` in `CompileError::PartialMatchLeniency`'s
    /// Display impl so each finding is directly navigable.
    pub call_site: Span,
}

/// Static lint for the partial-match-leniency antipattern from PR #101.
///
/// A rule `R` is flagged at a call site in some rule `R'` when:
///
/// 1. R has a non-empty `trailing_first` — there's a tail-position
///    `Optional` in R's body whose FIRST is non-empty.
/// 2. Walking outward from the call site through R's call chain, no
///    "validator" rejects bytes that R's trailing could have consumed.
///    A validator is either an `AndPredicate` (explicit lookahead anchor)
///    or a non-nullable consumer whose FIRST is disjoint from
///    `trailing_first(R)` (would fail if R left wrong bytes).
///
/// The reachability analysis differs from a local FIRST/FOLLOW overlap
/// check: it propagates outward through call sites of R's enclosing
/// rule, looking for the first hard validator. Only sites where no
/// validator exists anywhere along the outward chain are flagged.
///
/// Findings are returned sorted by `(rule, caller)`.
pub fn lint_partial_match(grammar: &Grammar) -> Vec<LintFinding> {
    let nullable = compute_nullable(&grammar.rules);

    let trailing: HashMap<String, FollowSet> = grammar
        .rules
        .iter()
        .map(|(name, body)| (name.clone(), trailing_first(body, &nullable)))
        .collect();

    let mut findings: Vec<LintFinding> = Vec::new();
    let mut rule_names: Vec<&String> = grammar.rules.keys().collect();
    rule_names.sort();
    let ctx = LintCtx {
        grammar,
        nullable: &nullable,
        trailing: &trailing,
    };
    for caller in rule_names {
        let body = &grammar.rules[caller];
        let mut cont_stack: Vec<&[Pattern]> = Vec::new();
        walk_for_call_sites(body, caller, &mut cont_stack, false, &ctx, &mut findings);
    }

    findings.sort();
    findings.dedup();
    findings
}

/// Immutable shared context for the lint's reachability walk.
struct LintCtx<'a> {
    grammar: &'a Grammar,
    nullable: &'a HashSet<String>,
    trailing: &'a HashMap<String, FollowSet>,
}

/// FIRST set of any trailing-Optional position in `pat`.
///
/// Restricted to `Optional` (not `Repeat` / `RepeatOne`): operator-chain
/// Repeats (`expr (binop expr)*`) are intentionally greedy and don't
/// exhibit the silent-leniency shape from PR #101, where a trailing
/// `(modifier)?` could fail to match and leave bytes the caller didn't
/// validate.
///
/// - Pattern is `Optional(inner)`: returns FIRST(inner).
/// - Pattern is a `Sequence`: walks items from the end; for each
///   trailing nullable item, includes FIRST only if the item is
///   directly an `Optional` (transparent through `Capture`/`Catch`).
///   Stops at the first non-nullable item.
/// - `Capture(_, inner)` / `Catch { inner, .. }`: recurse into inner.
/// - Otherwise: empty.
fn trailing_first(pat: &Pattern, nullable: &HashSet<String>) -> FollowSet {
    let mut out = FollowSet::new();
    match pat {
        Pattern::Optional { inner, .. } => {
            out.extend(pattern_first(inner, nullable));
        }
        Pattern::Sequence { items, .. } => {
            for item in items.iter().rev() {
                if pattern_nullable(item, nullable) {
                    if is_trailing_optional_like(item) {
                        out.extend(pattern_first(item, nullable));
                    }
                } else {
                    break;
                }
            }
        }
        Pattern::Capture { inner, .. } => {
            out.extend(trailing_first(inner, nullable));
        }
        Pattern::Catch { inner, .. } => {
            out.extend(trailing_first(inner, nullable));
        }
        // Definition-level `name: partial` opts the rule out of being a flag
        // target: its `trailing_first` is empty regardless of body shape.
        Pattern::Lenient { .. } => {}
        _ => {}
    }
    out
}

/// Is `pat` an `Optional` (possibly wrapped in transparent `Capture` /
/// `Catch`)? Used to restrict trailing-nullable detection to the
/// `(modifier)?` shape from PR #101, excluding `Repeat` operator
/// chains.
fn is_trailing_optional_like(pat: &Pattern) -> bool {
    match pat {
        Pattern::Optional { .. } => true,
        Pattern::Capture { inner, .. } => is_trailing_optional_like(inner),
        Pattern::Catch { inner, .. } => is_trailing_optional_like(inner),
        _ => false,
    }
}

/// Walk `pat` looking for `NonTerminal` call sites to rules with
/// non-empty `trailing_first`. Maintains a stack of `continuations` —
/// each frame is the slice of patterns sibling-after the current
/// position in some enclosing `Sequence`. At each call site, decide
/// whether the leniency is contained (validated outward) by examining
/// the continuation stack and recursively descending into call sites
/// of the enclosing rule when the stack is exhausted.
fn walk_for_call_sites<'a>(
    pat: &'a Pattern,
    caller: &str,
    continuations: &mut Vec<&'a [Pattern]>,
    inside_catch: bool,
    ctx: &LintCtx<'a>,
    findings: &mut Vec<LintFinding>,
) {
    match pat {
        Pattern::Literal { .. } | Pattern::CharClass { .. } | Pattern::AnyChar { .. } => {}
        Pattern::NonTerminal { name, span } => {
            let Some(t) = ctx.trailing.get(name) else {
                return;
            };
            if t.is_empty() {
                return;
            }
            if !leniency_reaches_unguarded(
                t,
                caller,
                continuations,
                inside_catch,
                ctx,
                &mut HashSet::new(),
            ) {
                return;
            }
            findings.push(LintFinding {
                kind: LintKind::PartialMatchLeniency,
                rule: name.clone(),
                caller: caller.to_string(),
                call_site: *span,
            });
        }
        Pattern::Sequence { items, .. } => {
            for i in 0..items.len() {
                continuations.push(&items[i + 1..]);
                walk_for_call_sites(
                    &items[i],
                    caller,
                    continuations,
                    inside_catch,
                    ctx,
                    findings,
                );
                continuations.pop();
            }
        }
        Pattern::OrderedChoice { alts, .. } => {
            for alt in alts {
                walk_for_call_sites(alt, caller, continuations, inside_catch, ctx, findings);
            }
        }
        Pattern::Optional { inner, .. }
        | Pattern::Capture { inner, .. }
        | Pattern::Repeat { inner, .. }
        | Pattern::RepeatOne { inner, .. } => {
            walk_for_call_sites(inner, caller, continuations, inside_catch, ctx, findings);
        }
        Pattern::NotPredicate { inner, .. } | Pattern::AndPredicate { inner, .. } => {
            let mut isolated: Vec<&[Pattern]> = Vec::new();
            walk_for_call_sites(inner, caller, &mut isolated, inside_catch, ctx, findings);
        }
        Pattern::Catch {
            inner, recovery, ..
        } => {
            // Inside the Catch's `inner`: leniency that leaks past `inner`
            // is absorbed by `recovery` (the recovery body fires whenever
            // the next outer-context matching step fails on the leftover).
            walk_for_call_sites(inner, caller, continuations, true, ctx, findings);
            let mut isolated: Vec<&[Pattern]> = Vec::new();
            walk_for_call_sites(recovery, caller, &mut isolated, false, ctx, findings);
        }
        // `Lenient` at a rule's body root makes the rule's `trailing_first`
        // empty — so the rule never appears as a flag target. The walker
        // still descends transparently to find unrelated unguarded calls
        // the lenient rule's body may itself contain.
        Pattern::Lenient { inner, .. } => {
            walk_for_call_sites(inner, caller, continuations, inside_catch, ctx, findings);
        }
        // The resolver runs before the lint, so the lint should
        // never see an unresolved boundary-anchored catch.
        Pattern::InferBoundaryCatch { .. } => {
            unreachable!(
                "Pattern::InferBoundaryCatch must be resolved by \
                 analysis::resolve_inferred_boundaries before lint_partial_match"
            )
        }
    }
}

/// Returns `true` iff bytes that `target_trailing` could match can reach
/// an unguarded position from the current continuation stack outward
/// through call sites of `caller`. "Unguarded" means no `AndPredicate`
/// anchor and no non-nullable consumer with a FIRST disjoint from
/// `target_trailing` is encountered along the way.
fn leniency_reaches_unguarded(
    target_trailing: &FollowSet,
    caller: &str,
    continuations: &[&[Pattern]],
    inside_catch: bool,
    ctx: &LintCtx<'_>,
    visited_callers: &mut HashSet<String>,
) -> bool {
    for frame in continuations.iter().rev() {
        match scan_frame_for_validator(frame, target_trailing, ctx.nullable) {
            FrameOutcome::Validated => {
                // A non-Catch validator only saves the call site if we
                // haven't already passed through a Catch absorber on the
                // path — a Catch's recovery body would consume the
                // leniency bytes before a downstream validator fires.
                if inside_catch {
                    continue;
                }
                return false;
            }
            FrameOutcome::Anchored => return false,
            FrameOutcome::PassedThrough => continue,
        }
    }

    if !visited_callers.insert(caller.to_string()) {
        // Caller cycle: returning `true` here would treat a recursive
        // self-call as a confirmed leak, but the same target_trailing
        // is already being evaluated at the outer frame. Defer the
        // verdict to that frame by returning `inside_catch` (same
        // sentinel the root-rule short-circuit below uses). If the
        // outer frame eventually concludes the leniency is contained,
        // the cycle is too; if it concludes the leniency leaks via
        // some non-cyclic caller, the iteration over other rules in
        // the outer frame will catch that.
        return inside_catch;
    }

    let mut rule_names: Vec<&String> = ctx.grammar.rules.keys().collect();
    rule_names.sort();

    if caller == super::parser::ROOT_RULE && !target_trailing.contains(&FollowElement::Eof) {
        visited_callers.remove(caller);
        return inside_catch;
    }

    let mut found_call_site = false;
    for other_rule in rule_names {
        let body = &ctx.grammar.rules[other_rule];
        let mut local_findings = false;
        let mut new_continuations: Vec<&[Pattern]> = Vec::new();
        let probe = CallsiteProbe {
            target_caller: caller,
            target_trailing,
            in_rule: other_rule,
        };
        if find_callsite_unguarded(
            body,
            &probe,
            &mut new_continuations,
            inside_catch,
            ctx,
            visited_callers,
            &mut local_findings,
        ) {
            visited_callers.remove(caller);
            return true;
        }
        found_call_site |= local_findings;
    }
    visited_callers.remove(caller);

    if !found_call_site {
        return inside_catch;
    }

    false
}

/// Per-target parameters that don't change as we descend through the
/// AST in `find_callsite_unguarded`. Bundled to keep the function
/// signature tight.
struct CallsiteProbe<'a> {
    target_caller: &'a str,
    target_trailing: &'a FollowSet,
    in_rule: &'a str,
}

/// Walks the body of `in_rule` searching for occurrences of
/// `target_caller` (a NonTerminal whose name matches). For each
/// occurrence, sets the current continuations stack to reflect the
/// position in `in_rule`'s body and recursively checks whether
/// `target_trailing` reaches unguarded from there.
///
/// Returns `true` if any occurrence is unguarded. Sets
/// `found_callsite` to true if at least one occurrence exists (so the
/// caller can distinguish "no call sites at all" from "all call sites
/// guarded").
fn find_callsite_unguarded<'a>(
    pat: &'a Pattern,
    probe: &CallsiteProbe<'a>,
    continuations: &mut Vec<&'a [Pattern]>,
    inside_catch: bool,
    ctx: &LintCtx<'a>,
    visited_callers: &mut HashSet<String>,
    found_callsite: &mut bool,
) -> bool {
    match pat {
        Pattern::Literal { .. } | Pattern::CharClass { .. } | Pattern::AnyChar { .. } => false,
        Pattern::NonTerminal { name, .. } => {
            if name != probe.target_caller {
                return false;
            }
            *found_callsite = true;
            leniency_reaches_unguarded(
                probe.target_trailing,
                probe.in_rule,
                continuations,
                inside_catch,
                ctx,
                visited_callers,
            )
        }
        Pattern::Sequence { items, .. } => {
            for i in 0..items.len() {
                continuations.push(&items[i + 1..]);
                let leaked = find_callsite_unguarded(
                    &items[i],
                    probe,
                    continuations,
                    inside_catch,
                    ctx,
                    visited_callers,
                    found_callsite,
                );
                continuations.pop();
                if leaked {
                    return true;
                }
            }
            false
        }
        Pattern::OrderedChoice { alts, .. } => {
            for alt in alts {
                if find_callsite_unguarded(
                    alt,
                    probe,
                    continuations,
                    inside_catch,
                    ctx,
                    visited_callers,
                    found_callsite,
                ) {
                    return true;
                }
            }
            false
        }
        Pattern::Optional { inner, .. }
        | Pattern::Capture { inner, .. }
        | Pattern::Repeat { inner, .. }
        | Pattern::RepeatOne { inner, .. } => find_callsite_unguarded(
            inner,
            probe,
            continuations,
            inside_catch,
            ctx,
            visited_callers,
            found_callsite,
        ),
        Pattern::NotPredicate { inner, .. } | Pattern::AndPredicate { inner, .. } => {
            let mut isolated: Vec<&[Pattern]> = Vec::new();
            find_callsite_unguarded(
                inner,
                probe,
                &mut isolated,
                inside_catch,
                ctx,
                visited_callers,
                found_callsite,
            )
        }
        Pattern::Catch {
            inner, recovery, ..
        } => {
            if find_callsite_unguarded(
                inner,
                probe,
                continuations,
                true,
                ctx,
                visited_callers,
                found_callsite,
            ) {
                return true;
            }
            let mut isolated: Vec<&[Pattern]> = Vec::new();
            find_callsite_unguarded(
                recovery,
                probe,
                &mut isolated,
                false,
                ctx,
                visited_callers,
                found_callsite,
            )
        }
        // Propagation walks through the lenient marker transparently;
        // the marker only suppresses the IMMEDIATE call's flag (handled
        // in `walk_for_call_sites`), not deeper detection or outward
        // propagation from nested rules' leniency.
        Pattern::Lenient { inner, .. } => find_callsite_unguarded(
            inner,
            probe,
            continuations,
            inside_catch,
            ctx,
            visited_callers,
            found_callsite,
        ),
        Pattern::InferBoundaryCatch { .. } => {
            unreachable!(
                "Pattern::InferBoundaryCatch must be resolved by \
                 analysis::resolve_inferred_boundaries before lint_partial_match"
            )
        }
    }
}

enum FrameOutcome {
    /// A validator was found in this frame: leniency is contained.
    Validated,
    /// An `AndPredicate` was found: explicit anchor.
    Anchored,
    /// The frame was entirely nullable / overlapping with target — walk
    /// out further.
    PassedThrough,
}

/// Scan one continuation frame (a slice of siblings after the current
/// position in some Sequence) looking for a hard validator.
fn scan_frame_for_validator(
    frame: &[Pattern],
    target_trailing: &FollowSet,
    nullable: &HashSet<String>,
) -> FrameOutcome {
    for item in frame {
        if matches!(item, Pattern::AndPredicate { .. }) {
            return FrameOutcome::Anchored;
        }
        if let Pattern::NotPredicate { inner, .. } = item {
            // `!P` succeeds iff P fails. It rejects bytes that match P.
            // If FIRST(P) contains every element of target_trailing
            // (target's possible leftover all matches P), then `!P`
            // unconditionally rejects target's leftover — a validator.
            // The common case is `!.` (end-of-input assertion), where
            // FIRST(.) is the full alphabet and any non-empty
            // target_trailing is subsumed.
            let inner_first = pattern_first(inner, nullable);
            if target_trailing
                .iter()
                .all(|t| element_subsumed_by(t, &inner_first))
            {
                return FrameOutcome::Validated;
            }
            // Otherwise it's a zero-width assertion that doesn't help;
            // keep walking.
            continue;
        }
        if !pattern_nullable(item, nullable) {
            let item_first = pattern_first(item, nullable);
            if item_first.is_disjoint(target_trailing) {
                return FrameOutcome::Validated;
            }
            // Non-nullable consumer whose FIRST overlaps with target's
            // trailing — the consumer doesn't reject what target could
            // have left. It still consumes, so further siblings are
            // beyond it; stop walking this frame.
            return FrameOutcome::PassedThrough;
        }
        // Nullable item: walk past it.
    }
    FrameOutcome::PassedThrough
}

/// Returns true iff `elem` is "subsumed" by `set` — every byte that
/// `elem` can match also can match some element of `set`. Used in
/// NotPredicate validator analysis.
///
/// Conservative: treats `CharClass(full)` as subsuming everything,
/// `AnyChar` similarly; otherwise checks for exact membership. Rule
/// references and captures aren't expanded.
fn element_subsumed_by(elem: &FollowElement, set: &FollowSet) -> bool {
    let any_char = CharSet::any();
    if set
        .iter()
        .any(|s| matches!(s, FollowElement::CharClass(cs) if *cs == any_char))
    {
        return true;
    }
    set.contains(elem)
}

// -- FOLLOW-inferred boundary catch resolver -----------------------------

/// Synthesize a boundary pattern from a FOLLOW set. Used by the
/// `^^lbl` resolver to fill in the boundary argument the author
/// omitted. The synthesized pattern is an `OrderedChoice` of:
/// - each literal byte sequence as `Pattern::Literal(bytes)`,
/// - the union of all character classes as one `Pattern::CharClass(set)`,
/// - each rule reference as `Pattern::NonTerminal(name)`,
/// - each capture as `Pattern::Capture(kind, inner)`,
/// - EOF (if present) as `Pattern::NotPredicate(AnyChar)` (i.e. `!.`).
///
/// Single-element synthesis returns the element directly rather than
/// wrapping in a one-arm `OrderedChoice`.
pub fn follow_set_to_boundary_pattern(fs: &FollowSet) -> Pattern {
    let mut alts: Vec<Pattern> = Vec::new();
    let mut char_union = CharSet::empty();
    let mut has_char_class = false;
    for elem in fs {
        match elem {
            FollowElement::CharClass(cs) => {
                char_union = char_union.union(cs);
                has_char_class = true;
            }
            FollowElement::Literal(bytes) => {
                alts.push(Pattern::literal_bytes(bytes.clone()));
            }
            FollowElement::Rule(name) => {
                alts.push(Pattern::nt(name.clone()));
            }
            FollowElement::Capture { kind, inner } => {
                alts.push(Pattern::capture(
                    kind.clone(),
                    follow_element_inner_to_pattern(inner),
                ));
            }
            FollowElement::Eof => {
                alts.push(Pattern::not_predicate(Pattern::any_char()));
            }
        }
    }
    if has_char_class {
        alts.insert(0, Pattern::char_class(char_union));
    }
    if alts.len() == 1 {
        alts.into_iter().next().unwrap()
    } else {
        // Synthesized from FOLLOW set — no source position to inherit.
        Pattern::OrderedChoice {
            alts,
            span: Span::SYNTHETIC,
        }
    }
}

fn follow_element_inner_to_pattern(elem: &FollowElement) -> Pattern {
    match elem {
        FollowElement::Literal(bytes) => Pattern::literal_bytes(bytes.clone()),
        FollowElement::CharClass(cs) => Pattern::char_class(cs.clone()),
        FollowElement::Rule(name) => Pattern::nt(name.clone()),
        FollowElement::Capture { kind, inner } => {
            Pattern::capture(kind.clone(), follow_element_inner_to_pattern(inner))
        }
        // EOF nested inside a capture would be malformed in practice;
        // synthesize the same `!.` shape as the top-level case.
        FollowElement::Eof => Pattern::not_predicate(Pattern::any_char()),
    }
}

/// Resolve every `Pattern::InferBoundaryCatch` placeholder in `grammar`
/// to the lowered explicit-form `Catch` shape. Boundary is synthesized
/// from the placeholder's per-site FOLLOW set; an empty FOLLOW set
/// returns `CompileError::CannotInferBoundary`.
///
/// Runs before `lint_partial_match` and bytecode emission. After this
/// pass, no `InferBoundaryCatch` remains in the rule bodies.
pub fn resolve_inferred_boundaries(grammar: &mut Grammar) -> Result<(), CompileError> {
    let nullable = compute_nullable(&grammar.rules);
    let follow = compute_follow(grammar);
    let mut new_rules: HashMap<String, Pattern> = HashMap::with_capacity(grammar.rules.len());
    for (name, body) in &grammar.rules {
        let rule_follow = follow.get(name).cloned().unwrap_or_default();
        let resolved = resolve_in_pattern(body, &rule_follow, &nullable, name)?;
        new_rules.insert(name.clone(), resolved);
    }
    grammar.rules = new_rules;
    Ok(())
}

fn resolve_in_pattern(
    pat: &Pattern,
    trailing: &FollowSet,
    nullable: &HashSet<String>,
    rule_name: &str,
) -> Result<Pattern, CompileError> {
    match pat {
        Pattern::Literal { .. }
        | Pattern::CharClass { .. }
        | Pattern::AnyChar { .. }
        | Pattern::NonTerminal { .. } => Ok(pat.clone()),
        Pattern::Sequence { items, span } => {
            let mut out = Vec::with_capacity(items.len());
            for i in 0..items.len() {
                let sub_trailing = sequence_suffix_trailing(&items[i + 1..], trailing, nullable);
                out.push(resolve_in_pattern(
                    &items[i],
                    &sub_trailing,
                    nullable,
                    rule_name,
                )?);
            }
            Ok(Pattern::Sequence {
                items: out,
                span: *span,
            })
        }
        Pattern::OrderedChoice { alts, span } => {
            let mut out = Vec::with_capacity(alts.len());
            for alt in alts {
                out.push(resolve_in_pattern(alt, trailing, nullable, rule_name)?);
            }
            Ok(Pattern::OrderedChoice {
                alts: out,
                span: *span,
            })
        }
        Pattern::Repeat { inner, span } => {
            // The inner can iterate; its successor includes FIRST(inner)
            // plus the outer trailing.
            let mut sub_trailing = trailing.clone();
            sub_trailing.extend(pattern_first(inner, nullable));
            Ok(Pattern::Repeat {
                inner: Box::new(resolve_in_pattern(
                    inner,
                    &sub_trailing,
                    nullable,
                    rule_name,
                )?),
                span: *span,
            })
        }
        Pattern::RepeatOne { inner, span } => {
            let mut sub_trailing = trailing.clone();
            sub_trailing.extend(pattern_first(inner, nullable));
            Ok(Pattern::RepeatOne {
                inner: Box::new(resolve_in_pattern(
                    inner,
                    &sub_trailing,
                    nullable,
                    rule_name,
                )?),
                span: *span,
            })
        }
        Pattern::Optional { inner, span } => Ok(Pattern::Optional {
            inner: Box::new(resolve_in_pattern(inner, trailing, nullable, rule_name)?),
            span: *span,
        }),
        Pattern::NotPredicate { inner, span } => Ok(Pattern::NotPredicate {
            inner: Box::new(resolve_in_pattern(
                inner,
                &FollowSet::new(),
                nullable,
                rule_name,
            )?),
            span: *span,
        }),
        Pattern::AndPredicate { inner, span } => Ok(Pattern::AndPredicate {
            inner: Box::new(resolve_in_pattern(
                inner,
                &FollowSet::new(),
                nullable,
                rule_name,
            )?),
            span: *span,
        }),
        Pattern::Capture { kind, inner, span } => Ok(Pattern::Capture {
            kind: kind.clone(),
            inner: Box::new(resolve_in_pattern(inner, trailing, nullable, rule_name)?),
            span: *span,
        }),
        Pattern::Lenient { inner, span } => Ok(Pattern::Lenient {
            inner: Box::new(resolve_in_pattern(inner, trailing, nullable, rule_name)?),
            span: *span,
        }),
        Pattern::Catch {
            inner,
            label,
            recovery,
            span,
        } => {
            // Success branch: inner inherits the catch's trailing.
            // Recovery branch: trailing is also the catch's trailing.
            Ok(Pattern::Catch {
                inner: Box::new(resolve_in_pattern(inner, trailing, nullable, rule_name)?),
                label: label.clone(),
                recovery: Box::new(resolve_in_pattern(recovery, trailing, nullable, rule_name)?),
                span: *span,
            })
        }
        Pattern::InferBoundaryCatch { inner, label, span } => {
            if trailing.is_empty() {
                return Err(CompileError::CannotInferBoundary {
                    rule: rule_name.into(),
                    label: label.clone(),
                    span: *span,
                });
            }
            let boundary = follow_set_to_boundary_pattern(trailing);
            // Resolve nested catches inside the inner first.
            let resolved_inner = resolve_in_pattern(inner, trailing, nullable, rule_name)?;
            Ok(lower_inferred_boundary_catch(
                resolved_inner,
                label.clone(),
                boundary,
                *span,
            ))
        }
    }
}

/// FIRST of `items[..]` with `outer_trailing` propagated through if
/// all items are nullable. Mirrors the Sequence arm of
/// `collect_follow` but produces a fresh FollowSet rather than
/// extending one in place.
fn sequence_suffix_trailing(
    items: &[Pattern],
    outer_trailing: &FollowSet,
    nullable: &HashSet<String>,
) -> FollowSet {
    let mut out = FollowSet::new();
    let mut all_nullable = true;
    for it in items {
        out.extend(pattern_first(it, nullable));
        if !pattern_nullable(it, nullable) {
            all_nullable = false;
            break;
        }
    }
    if all_nullable {
        out.extend(outer_trailing.iter().cloned());
    }
    out
}

/// Mirror of `parser::lower_boundary_catch` for the resolver path.
/// Builds the same `Catch { Sequence([inner, &B]), label, @recovery ((!B .)*) }`
/// shape from a synthesized boundary. The produced tree inherits the
/// original `^^lbl` placeholder's span; synthesized boundary subnodes
/// already carry [`Span::SYNTHETIC`] from `follow_set_to_boundary_pattern`.
fn lower_inferred_boundary_catch(
    inner: Pattern,
    label: String,
    boundary: Pattern,
    span: Span,
) -> Pattern {
    let lookahead = Pattern::AndPredicate {
        inner: Box::new(boundary.clone()),
        span,
    };
    let stop_loop = Pattern::Repeat {
        inner: Box::new(Pattern::Sequence {
            items: vec![
                Pattern::NotPredicate {
                    inner: Box::new(boundary),
                    span,
                },
                Pattern::AnyChar { span },
            ],
            span,
        }),
        span,
    };
    let recovery = Pattern::Capture {
        kind: "recovery".into(),
        inner: Box::new(stop_loop),
        span,
    };
    Pattern::Catch {
        inner: Box::new(Pattern::Sequence {
            items: vec![inner, lookahead],
            span,
        }),
        label,
        recovery: Box::new(recovery),
        span,
    }
}

/// The reserved rule name that marks the ignore subgraph's root.
/// If a grammar defines a rule with this name, the compiler cascades
/// the ignore bit through every rule transitively reachable from it
/// (subject to the catch-exclusion below).
pub(crate) const IGNORE_ROOT_RULE: &str = "ignore";

/// Compute the per-rule ignore bit. A rule is ignore iff it's
/// transitively reachable through the call graph from a rule named
/// [`IGNORE_ROOT_RULE`], excluding rules whose body contains a recovery
/// catch (`Pattern::Catch` or `Pattern::InferBoundaryCatch`). The
/// catch-exclusion preserves diagnostic visibility for catch-bearing
/// frames in `pegdb recoveries explain`: the catch is the author's
/// signal that the rule's frame is load-bearing, and silently trimming
/// it would hide exactly the rule the catch lives in.
///
/// Returns an entry for every rule in `rules`; rules in a grammar with
/// no `ignore` root all get `false`.
pub(crate) fn compute_ignore_rules(rules: &HashMap<String, Pattern>) -> HashMap<String, bool> {
    let mut ignore: HashMap<String, bool> = rules.keys().map(|n| (n.clone(), false)).collect();
    if !rules.contains_key(IGNORE_ROOT_RULE) {
        return ignore;
    }
    let pinned: HashSet<String> = rules
        .iter()
        .filter(|(_, body)| body_contains_catch(body))
        .map(|(name, _)| name.clone())
        .collect();
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: Vec<String> = vec![IGNORE_ROOT_RULE.to_string()];
    while let Some(rule) = queue.pop() {
        if !visited.insert(rule.clone()) {
            continue;
        }
        if !pinned.contains(&rule) {
            ignore.insert(rule.clone(), true);
        }
        if let Some(body) = rules.get(&rule) {
            let mut callees = HashSet::new();
            collect_non_terminal_refs(body, &mut callees);
            for callee in callees {
                if rules.contains_key(&callee) && !visited.contains(&callee) {
                    queue.push(callee);
                }
            }
        }
    }
    ignore
}

/// Walk `pat` and bump `out[name]` for every `Pattern::NonTerminal { name }`
/// occurrence. Unlike [`collect_non_terminal_refs`], duplicates count —
/// `pegc stats` uses this to surface per-rule reference totals across all
/// rule bodies.
pub fn tally_non_terminal_refs(pat: &Pattern, out: &mut HashMap<String, usize>) {
    match pat {
        Pattern::Literal { .. } | Pattern::CharClass { .. } | Pattern::AnyChar { .. } => {}
        Pattern::Sequence { items, .. } => {
            for it in items {
                tally_non_terminal_refs(it, out);
            }
        }
        Pattern::OrderedChoice { alts, .. } => {
            for it in alts {
                tally_non_terminal_refs(it, out);
            }
        }
        Pattern::Repeat { inner, .. }
        | Pattern::RepeatOne { inner, .. }
        | Pattern::Optional { inner, .. }
        | Pattern::NotPredicate { inner, .. }
        | Pattern::AndPredicate { inner, .. }
        | Pattern::Capture { inner, .. }
        | Pattern::Lenient { inner, .. } => tally_non_terminal_refs(inner, out),
        Pattern::Catch {
            inner, recovery, ..
        } => {
            tally_non_terminal_refs(inner, out);
            tally_non_terminal_refs(recovery, out);
        }
        Pattern::InferBoundaryCatch { inner, .. } => tally_non_terminal_refs(inner, out),
        Pattern::NonTerminal { name, .. } => {
            *out.entry(name.clone()).or_insert(0) += 1;
        }
    }
}

fn collect_non_terminal_refs(pat: &Pattern, out: &mut HashSet<String>) {
    match pat {
        Pattern::Literal { .. } | Pattern::CharClass { .. } | Pattern::AnyChar { .. } => {}
        Pattern::Sequence { items, .. } => {
            for it in items {
                collect_non_terminal_refs(it, out);
            }
        }
        Pattern::OrderedChoice { alts, .. } => {
            for it in alts {
                collect_non_terminal_refs(it, out);
            }
        }
        Pattern::Repeat { inner, .. }
        | Pattern::RepeatOne { inner, .. }
        | Pattern::Optional { inner, .. }
        | Pattern::NotPredicate { inner, .. }
        | Pattern::AndPredicate { inner, .. }
        | Pattern::Capture { inner, .. }
        | Pattern::Lenient { inner, .. } => collect_non_terminal_refs(inner, out),
        Pattern::Catch {
            inner, recovery, ..
        } => {
            collect_non_terminal_refs(inner, out);
            collect_non_terminal_refs(recovery, out);
        }
        Pattern::InferBoundaryCatch { inner, .. } => collect_non_terminal_refs(inner, out),
        Pattern::NonTerminal { name, .. } => {
            out.insert(name.clone());
        }
    }
}

fn body_contains_catch(pat: &Pattern) -> bool {
    match pat {
        Pattern::Catch { .. } | Pattern::InferBoundaryCatch { .. } => true,
        Pattern::Literal { .. }
        | Pattern::CharClass { .. }
        | Pattern::AnyChar { .. }
        | Pattern::NonTerminal { .. } => false,
        Pattern::Sequence { items, .. } => items.iter().any(body_contains_catch),
        Pattern::OrderedChoice { alts, .. } => alts.iter().any(body_contains_catch),
        Pattern::Repeat { inner, .. }
        | Pattern::RepeatOne { inner, .. }
        | Pattern::Optional { inner, .. }
        | Pattern::NotPredicate { inner, .. }
        | Pattern::AndPredicate { inner, .. }
        | Pattern::Capture { inner, .. }
        | Pattern::Lenient { inner, .. } => body_contains_catch(inner),
    }
}

/// True iff the grammar has an active auto-insertion target: a rule
/// named `ignore`. Both [`inject_auto_ignore`] and [`wrap_root`]
/// consult this; on `false` they no-op (no inter-Sequence splicing, no
/// `ignore?` siblings in the root wrap). The `ignore` rule carries no
/// qualifiers, so omitting it is the only way to disable auto-insertion.
fn auto_insertion_active(grammar: &Grammar) -> bool {
    grammar.rules.contains_key(IGNORE_ROOT_RULE)
}

/// AST-level rewrite: in every non-atomic rule's body, splice a
/// `NonTerminal("ignore")` call between consecutive items of every
/// `Sequence`, and prepend one to every `Repeat` / `RepeatOne`
/// iteration so inter-iteration whitespace gets consumed. Runs after
/// `lint_partial_match` and the FOLLOW-driven `^^lbl` resolver so both
/// still see the author's original body.
///
/// No-op when the grammar has no `ignore` rule (the grammar-wide
/// opt-out — `ignore` carries no qualifiers). The `ignore` rule itself
/// is exempt from injection: splicing ignore inside its own body would
/// recurse the runtime indefinitely.
///
/// The walker descends transparently through `Choice`, `Optional`,
/// `Capture`, `Catch`, `Lenient`, `AndPredicate`, `NotPredicate`, and
/// stops at `NonTerminal` and the atom leaves (`Literal`, `CharClass`,
/// `AnyChar`). Crossing a `NonTerminal` would pull ignore into the
/// callee's body — which the callee handles by its own auto-insertion
/// if non-atomic, or explicitly if atomic.
pub fn inject_auto_ignore(grammar: &mut Grammar) {
    if !auto_insertion_active(grammar) {
        return;
    }
    let atomic = grammar.atomic_rules.clone();
    let rule_names: Vec<String> = grammar.rules.keys().cloned().collect();
    for name in rule_names {
        // The ignore rule itself is exempt: every `ignore_call` from
        // the rewriter calls back into it, so injecting ignore inside
        // its body would recurse the runtime indefinitely. Atomic
        // rules opt out explicitly via the `name: atomic` ascription.
        if atomic.contains(&name) || name == IGNORE_ROOT_RULE {
            continue;
        }
        if let Some(body) = grammar.rules.get_mut(&name) {
            inject_ignore_in(body);
        }
    }
}

/// Walk one rule body, splicing `NonTerminal("ignore")` between every
/// pair of consecutive `Sequence` items at every depth and prepending
/// one to every `Repeat` / `RepeatOne` iteration.
fn inject_ignore_in(pat: &mut Pattern) {
    match pat {
        Pattern::Sequence { items, .. } => {
            for it in items.iter_mut() {
                inject_ignore_in(it);
            }
            if items.len() >= 2 {
                let mut spliced = Vec::with_capacity(items.len() * 2 - 1);
                let drained: Vec<Pattern> = std::mem::take(items);
                let last = drained.len() - 1;
                for (i, it) in drained.into_iter().enumerate() {
                    spliced.push(it);
                    if i != last {
                        spliced.push(ignore_call());
                    }
                }
                *items = spliced;
            }
        }
        Pattern::OrderedChoice { alts, .. } => {
            for it in alts {
                inject_ignore_in(it);
            }
        }
        Pattern::Repeat { inner, .. } | Pattern::RepeatOne { inner, .. } => {
            inject_ignore_in(inner);
            // Prepend a ignore call so each iteration consumes
            // ambient whitespace before its body runs. Without this,
            // `(',' pair)*` would fail on the second iteration's
            // leading whitespace — the inter-iteration boundary has
            // no parent `Sequence` to splice ignore into. Flatten
            // when `inner` is already a `Sequence` so the result
            // stays at one level of nesting.
            let original = std::mem::replace(inner.as_mut(), Pattern::any_char());
            let wrapped = match original {
                Pattern::Sequence { mut items, span } => {
                    items.insert(0, ignore_call());
                    Pattern::Sequence { items, span }
                }
                other => Pattern::seq(vec![ignore_call(), other]),
            };
            **inner = wrapped;
        }
        Pattern::Optional { inner, .. }
        | Pattern::NotPredicate { inner, .. }
        | Pattern::AndPredicate { inner, .. }
        | Pattern::Capture { inner, .. }
        | Pattern::Lenient { inner, .. } => inject_ignore_in(inner),
        Pattern::Catch {
            inner, recovery, ..
        } => {
            inject_ignore_in(inner);
            inject_ignore_in(recovery);
        }
        Pattern::InferBoundaryCatch { inner, .. } => inject_ignore_in(inner),
        Pattern::Literal { .. }
        | Pattern::CharClass { .. }
        | Pattern::AnyChar { .. }
        | Pattern::NonTerminal { .. } => {}
    }
}

/// Wrap the start rule's body so it reads `ignore? body ignore? !.` —
/// implicit leading/trailing ignore and end-of-input assertion. Built
/// manually as a `Sequence` *after* [`inject_auto_ignore`] has run on
/// `root`'s body, so the synthesized siblings here don't themselves
/// pick up ignore injection.
///
/// `ignore?` is included iff auto-insertion is active. With no `ignore`
/// rule the wrap collapses to `body !.` — still supplying the EOF
/// assertion uniformly.
pub fn wrap_root(grammar: &mut Grammar) {
    let active = auto_insertion_active(grammar);
    let Some(body) = grammar.rules.remove(super::parser::ROOT_RULE) else {
        return;
    };
    let mut items: Vec<Pattern> = Vec::with_capacity(4);
    if active {
        items.push(Pattern::optional(ignore_call()));
    }
    items.push(body);
    if active {
        items.push(Pattern::optional(ignore_call()));
    }
    items.push(Pattern::not_predicate(Pattern::any_char()));
    grammar
        .rules
        .insert(super::parser::ROOT_RULE.to_string(), Pattern::seq(items));
}

fn ignore_call() -> Pattern {
    Pattern::nt(IGNORE_ROOT_RULE)
}

/// Append a `boundary` word-boundary call inside the terminal captures of
/// every `reserved` rule (see [`Grammar::percent_rules`]).
///
/// The boundary is pushed to the tail of the matched region, *inside*
/// the capture that ends the match, so the capture commits only when
/// the boundary holds. Placing `boundary` after the capture instead would let
/// the keyword capture commit before the boundary check fails, leaking a
/// stray keyword token through top-level `*^` recovery on inputs like
/// `typedefx`. The rule is already atomic (the `reserved` ascription inserts it into
/// `atomic_rules`), so no `ignore` is spliced between the literal and
/// the appended call.
///
/// A `boundary` call into a rule with no `boundary` definition surfaces downstream
/// as `CompileError::UndefinedRule("boundary")` from `compile_rules`.
pub fn append_boundary_check(grammar: &mut Grammar) {
    let names: Vec<String> = grammar.percent_rules.iter().cloned().collect();
    for name in names {
        if let Some(mut body) = grammar.rules.remove(&name) {
            append_boundary_in(&mut body);
            grammar.rules.insert(name, body);
        }
    }
}

/// Recursively append a `boundary` call at the tail of `pat`'s match, pushing
/// it inside the final capture and distributing across choice branches
/// so each alternative ends in its own boundary check.
fn append_boundary_in(pat: &mut Pattern) {
    // A choice whose branches hold no capture can't leak a committed
    // capture through recovery, so one trailing `boundary` after the whole
    // choice is equivalent to — and leaner than — one per branch. Only
    // distribute when there is a capture to keep the boundary inside of.
    if matches!(pat, Pattern::OrderedChoice { .. }) && !contains_capture(pat) {
        // When every branch is a plain literal, prefix-factor the set
        // into a longest-match trie (with `boundary` at each accept) instead of
        // a flat alternation under one trailing `boundary`. The trie makes
        // source ordering irrelevant — a shorter keyword can no longer
        // shadow a longer one that shares its prefix (#13: `do`/`double`,
        // `go`/`goto`, `int`/`int8`) — and turns the hot
        // identifier-exclusion scan into a prefix walk. Non-literal
        // capture-less choices (e.g. the case-insensitive `i"…"` keyword
        // sets, which lower to char-class sequences) keep the flat
        // single-trailing-`boundary` form; `synthesize_reserved_preferred`
        // emits those longest-first so the flat form is still correct.
        if let Pattern::OrderedChoice { alts, .. } = pat {
            if let Some(literals) = all_plain_literals(alts) {
                *pat = factor_literal_trie(literals);
                return;
            }
        }
        let taken = std::mem::replace(pat, Pattern::any_char());
        *pat = Pattern::seq(vec![taken, Pattern::nt(super::parser::BOUNDARY_RULE)]);
        return;
    }
    match pat {
        Pattern::OrderedChoice { alts, .. } => {
            for alt in alts.iter_mut() {
                append_boundary_in(alt);
            }
        }
        Pattern::Sequence { items, .. } => {
            if let Some(last) = items.last_mut() {
                append_boundary_in(last);
            }
        }
        Pattern::Capture { inner, .. } => append_boundary_in(inner),
        other => {
            let taken = std::mem::replace(other, Pattern::any_char());
            *other = Pattern::seq(vec![taken, Pattern::nt(super::parser::BOUNDARY_RULE)]);
        }
    }
}

/// Whether `pat` contains a [`Pattern::Capture`] anywhere in its tree.
/// `NonTerminal`s are opaque (a referenced rule's captures aren't
/// visible here) — consistent with the rest of `append_boundary_in`, which
/// appends `boundary` after a `NonTerminal` call rather than inside it.
fn contains_capture(pat: &Pattern) -> bool {
    match pat {
        Pattern::Capture { .. } => true,
        Pattern::Sequence { items, .. } => items.iter().any(contains_capture),
        Pattern::OrderedChoice { alts, .. } => alts.iter().any(contains_capture),
        Pattern::Repeat { inner, .. }
        | Pattern::RepeatOne { inner, .. }
        | Pattern::Optional { inner, .. }
        | Pattern::NotPredicate { inner, .. }
        | Pattern::AndPredicate { inner, .. }
        | Pattern::Lenient { inner, .. } => contains_capture(inner),
        Pattern::Catch {
            inner, recovery, ..
        } => contains_capture(inner) || contains_capture(recovery),
        Pattern::InferBoundaryCatch { inner, .. } => contains_capture(inner),
        Pattern::Literal { .. }
        | Pattern::CharClass { .. }
        | Pattern::AnyChar { .. }
        | Pattern::NonTerminal { .. } => false,
    }
}

/// If every alternative of a choice is a plain [`Pattern::Literal`],
/// return their raw byte-strings; otherwise `None`. The all-or-nothing
/// gate keeps [`append_boundary_in`] on the flat single-`boundary` path for choices
/// that mix literals with char classes / sub-rules (e.g. the
/// case-insensitive `i"…"` keyword sets).
fn all_plain_literals(alts: &[Pattern]) -> Option<Vec<Vec<u8>>> {
    let mut out = Vec::with_capacity(alts.len());
    for a in alts {
        match a {
            Pattern::Literal { bytes, .. } => out.push(bytes.clone()),
            _ => return None,
        }
    }
    Some(out)
}

/// Prefix-factor a set of literal byte-strings into a longest-match trie
/// with a [`BOUNDARY_RULE`](super::parser::BOUNDARY_RULE) call at every accepting
/// node. Replaces a flat capture-less literal alternation: the trie is
/// order-independent (a shorter keyword can't shadow a longer one that
/// extends it) and walks the input once instead of re-scanning every
/// alternative.
fn factor_literal_trie(mut literals: Vec<Vec<u8>>) -> Pattern {
    literals.sort();
    literals.dedup();
    build_trie(literals)
}

/// Build one trie node from the suffixes that reach it. An empty suffix
/// means "a keyword ends here" → emit `boundary`. Distinct continuation bytes
/// become sibling branches; a single-child, no-accept chain is coalesced
/// into one multi-byte literal. The accept (`boundary`) branch is emitted
/// *last* so maximal munch wins: with `long` and `long long` in the set,
/// trying ` long` before accepting `long` is what stops `long long` from
/// matching only its `long` prefix (the space is a valid boundary, so a
/// `boundary`-first order would accept the short form).
fn build_trie(entries: Vec<Vec<u8>>) -> Pattern {
    let mut accept = false;
    let mut groups: BTreeMap<u8, Vec<Vec<u8>>> = BTreeMap::new();
    for e in entries {
        match e.split_first() {
            None => accept = true,
            Some((&b, rest)) => groups.entry(b).or_default().push(rest.to_vec()),
        }
    }
    let mut branches: Vec<Pattern> = Vec::with_capacity(groups.len() + 1);
    for (b, subs) in groups {
        let mut prefix = vec![b];
        let mut cur = subs;
        loop {
            if cur.iter().any(|e| e.is_empty()) {
                break;
            }
            let firsts: BTreeSet<u8> = cur.iter().filter_map(|e| e.first().copied()).collect();
            if firsts.len() != 1 {
                break;
            }
            let only = *firsts.iter().next().unwrap();
            prefix.push(only);
            cur = cur.into_iter().map(|e| e[1..].to_vec()).collect();
        }
        let sub = build_trie(cur);
        branches.push(Pattern::seq(vec![Pattern::literal_bytes(prefix), sub]));
    }
    if accept {
        branches.push(Pattern::nt(super::parser::BOUNDARY_RULE));
    }
    Pattern::choice(branches)
}

/// Synthesize the compiler-generated `reserved` and `preferred` rules
/// from the literals reachable through the grammar's `reserved` /
/// `preferred` rules.
///
/// `reserved` collects every literal of a `reserved` rule (excluding
/// `preferred`); `preferred` collects every literal of a `preferred`
/// rule. A rule whose body isn't a fixed keyword shape (it has a
/// quantifier, wildcard, predicate, or an unresolvable reference — e.g.
/// a number body like `'0x' [\da-fA-F]+`) contributes nothing. Both
/// rules are emitted as a flat alternation ordered longest-first and
/// registered as `reserved` rules, so the following [`append_boundary_check`]
/// gives them the same trie + `boundary` treatment as any author-written
/// keyword set.
///
/// A literal reachable from both a `reserved` and a `preferred` rule is
/// a contradiction (it can't be both barred from and allowed in
/// identifier position) and raises
/// [`CompileError::ReservedPreferredConflict`].
pub fn synthesize_reserved_preferred(grammar: &mut Grammar) -> Result<(), CompileError> {
    let mut reserved: BTreeSet<Vec<CharSet>> = BTreeSet::new();
    let mut preferred: BTreeSet<Vec<CharSet>> = BTreeSet::new();

    let mut percent: Vec<String> = grammar.percent_rules.iter().cloned().collect();
    percent.sort();
    for name in &percent {
        let Some(body) = grammar.rules.get(name) else {
            continue;
        };
        let mut visiting = vec![name.clone()];
        let Some(entries) = keyword_entries(body, &grammar.rules, &mut visiting) else {
            continue;
        };
        let target = if grammar.preferred_rules.contains(name) {
            &mut preferred
        } else {
            &mut reserved
        };
        target.extend(entries);
    }

    let overlap: Vec<&Vec<CharSet>> = reserved.intersection(&preferred).collect();
    if !overlap.is_empty() {
        let mut words: Vec<String> = overlap.iter().map(|e| entry_display(e)).collect();
        words.sort();
        return Err(CompileError::ReservedPreferredConflict(words));
    }

    if !reserved.is_empty() {
        let rule = build_word_set_rule(reserved);
        grammar
            .rules
            .insert(super::parser::RESERVED_RULE.to_string(), rule);
        grammar
            .percent_rules
            .insert(super::parser::RESERVED_RULE.to_string());
        grammar
            .atomic_rules
            .insert(super::parser::RESERVED_RULE.to_string());
    }
    if !preferred.is_empty() {
        let rule = build_word_set_rule(preferred);
        grammar
            .rules
            .insert(super::parser::PREFERRED_RULE.to_string(), rule);
        grammar
            .percent_rules
            .insert(super::parser::PREFERRED_RULE.to_string());
        grammar
            .atomic_rules
            .insert(super::parser::PREFERRED_RULE.to_string());
    }
    Ok(())
}

/// Extract the fixed keyword strings a pattern can match, as one
/// `Vec<CharSet>` per string (each `CharSet` matching one code point — a
/// singleton for a case-sensitive byte, a two-element set for an `i"…"`
/// case-folded letter). Returns `None` for anything that isn't a fixed
/// shape: a quantifier, wildcard, predicate, catch, or a reference that
/// cycles or escapes the keyword shape. `NonTerminal`s are resolved
/// through `rules` (with a `visiting` cycle guard) so wrappers like
/// `bool_lit: reserved = @constant bool_body` reach their literals.
fn keyword_entries(
    pat: &Pattern,
    rules: &HashMap<String, Pattern>,
    visiting: &mut Vec<String>,
) -> Option<Vec<Vec<CharSet>>> {
    match pat {
        Pattern::Literal { bytes, .. } => {
            let s = std::str::from_utf8(bytes).ok()?;
            Some(vec![s.chars().map(CharSet::singleton).collect()])
        }
        Pattern::CharClass { set, .. } => Some(vec![vec![set.clone()]]),
        Pattern::Capture { inner, .. } | Pattern::Lenient { inner, .. } => {
            keyword_entries(inner, rules, visiting)
        }
        Pattern::OrderedChoice { alts, .. } => {
            let mut out = Vec::new();
            for a in alts {
                out.extend(keyword_entries(a, rules, visiting)?);
            }
            Some(out)
        }
        Pattern::Sequence { items, .. } => {
            // A sequence is one keyword: concatenate the pieces. Each
            // piece must contribute exactly one fixed string; a choice
            // inside the sequence would be a product we don't expand, so
            // reject the whole rule.
            let mut acc: Vec<CharSet> = Vec::new();
            for it in items {
                let sub = keyword_entries(it, rules, visiting)?;
                let [one] = &sub[..] else {
                    return None;
                };
                acc.extend(one.iter().cloned());
            }
            Some(vec![acc])
        }
        Pattern::NonTerminal { name, .. } => {
            if visiting.iter().any(|n| n == name) {
                return None;
            }
            let body = rules.get(name)?;
            visiting.push(name.clone());
            let r = keyword_entries(body, rules, visiting);
            visiting.pop();
            r
        }
        _ => None,
    }
}

/// Build a `reserved` / `preferred` rule body from its keyword set: a
/// flat alternation ordered longest-first. All-singleton-ASCII entries
/// become byte literals (so [`append_boundary_in`] folds them into a trie);
/// case-folded entries stay sequences of char classes.
fn build_word_set_rule(set: BTreeSet<Vec<CharSet>>) -> Pattern {
    let mut entries: Vec<Vec<CharSet>> = set.into_iter().collect();
    entries.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
    let alts: Vec<Pattern> = entries.iter().map(|e| entry_to_pattern(e)).collect();
    Pattern::choice(alts)
}

/// Lower one keyword entry to a pattern: a byte literal when every code
/// point is a singleton ASCII set, otherwise a sequence of char classes.
fn entry_to_pattern(entry: &[CharSet]) -> Pattern {
    if let Some(bytes) = entry
        .iter()
        .map(charset_singleton_ascii)
        .collect::<Option<Vec<u8>>>()
    {
        return Pattern::literal_bytes(bytes);
    }
    let items: Vec<Pattern> = entry
        .iter()
        .map(|s| Pattern::char_class(s.clone()))
        .collect();
    Pattern::seq(items)
}

/// The single ASCII byte a char set matches, or `None` if it isn't a
/// singleton ASCII set.
fn charset_singleton_ascii(set: &CharSet) -> Option<u8> {
    let ranges = set.ranges();
    if ranges.len() == 1 && ranges[0].0 == ranges[0].1 && ranges[0].0.is_ascii() {
        Some(ranges[0].0 as u8)
    } else {
        None
    }
}

/// Best-effort rendering of a keyword entry for diagnostics: one
/// representative code point per position (the highest, so a case-folded
/// `{a, A}` shows as `a`).
fn entry_display(entry: &[CharSet]) -> String {
    entry
        .iter()
        .map(|s| s.ranges().last().map(|r| r.0).unwrap_or('\u{FFFD}'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ---- ignore cascade (compute_ignore_rules) -----------------------
    //
    // Exercised directly on the parsed (pre-injection) rule map: the
    // cascade is forward reachability from `ignore`, independent of
    // auto-insertion, so a plain `ignore` rule is the faithful fixture.

    fn ignore_bits(src: &str) -> HashMap<String, bool> {
        compute_ignore_rules(&super::super::parser::parse(src).expect("parse").rules)
    }

    #[test]
    fn no_ignore_rule_leaves_all_bits_false() {
        let bits = ignore_bits("root = 'x'");
        assert!(bits.values().all(|&b| !b));
    }

    #[test]
    fn ignore_rule_marks_itself_not_root() {
        let bits = ignore_bits("root = ignore 'x'\nignore = ' '*");
        assert!(bits["ignore"]);
        assert!(!bits["root"]);
    }

    #[test]
    fn ignore_cascade_reaches_transitive_callees() {
        let bits = ignore_bits(
            "root = ignore 'x'\n\
             ignore = ws\n\
             ws = (comment / ' ')*\n\
             comment = '#' (!'\\n' .)*",
        );
        assert!(bits["ignore"] && bits["ws"] && bits["comment"]);
        assert!(!bits["root"]);
    }

    #[test]
    fn ignore_cascade_skips_catch_bearing_rules() {
        // `victim` is reachable from ignore but carries a catch, so it
        // is pinned out of the cascade — keeping its frame visible in
        // `pegdb recoveries explain`.
        let bits = ignore_bits(
            "root = ignore 'x'\n\
             ignore = (victim / ' ')*\n\
             victim = 'a' ^bad 'b'",
        );
        assert!(bits["ignore"]);
        assert!(!bits["victim"], "catch-bearing rule must not cascade");
    }

    #[test]
    fn ignore_cascade_terminates_on_recursion() {
        let bits = ignore_bits(
            "root = ignore 'x'\n\
             ignore = ws\n\
             ws = (ws / ' ')*",
        );
        assert!(bits["ignore"] && bits["ws"]);
    }

    fn count_boundary(pat: &Pattern) -> usize {
        match pat {
            Pattern::NonTerminal { name, .. }
                if name.as_str() == super::super::parser::BOUNDARY_RULE =>
            {
                1
            }
            Pattern::Sequence { items, .. } => items.iter().map(count_boundary).sum(),
            Pattern::OrderedChoice { alts, .. } => alts.iter().map(count_boundary).sum(),
            Pattern::Repeat { inner, .. }
            | Pattern::RepeatOne { inner, .. }
            | Pattern::Optional { inner, .. }
            | Pattern::NotPredicate { inner, .. }
            | Pattern::AndPredicate { inner, .. }
            | Pattern::Capture { inner, .. }
            | Pattern::Lenient { inner, .. } => count_boundary(inner),
            Pattern::Catch {
                inner, recovery, ..
            } => count_boundary(inner) + count_boundary(recovery),
            Pattern::InferBoundaryCatch { inner, .. } => count_boundary(inner),
            _ => 0,
        }
    }

    /// Mark a single rule `t` as a `reserved` rule, run `append_boundary_check`, and
    /// return the rewritten body.
    fn append_to(body: Pattern) -> Pattern {
        let mut rules = HashMap::new();
        rules.insert("t".to_string(), body);
        let mut g = Grammar::new(rules);
        g.percent_rules.insert("t".to_string());
        append_boundary_check(&mut g);
        g.rules.remove("t").unwrap()
    }

    #[test]
    fn capture_less_charclass_choice_gets_one_trailing_boundary() {
        // `t: reserved = [ab] / [cd]` — capture-less but not all-literal, so the
        // trie path is skipped and a single trailing `boundary` follows the
        // whole choice, not one per branch.
        let body = Pattern::choice(vec![
            Pattern::char_class(CharSet::from_chars(&['a', 'b'])),
            Pattern::char_class(CharSet::from_chars(&['c', 'd'])),
        ]);
        assert_eq!(count_boundary(&append_to(body)), 1);
    }

    #[test]
    fn capture_less_literal_choice_factors_into_trie() {
        // `t: reserved = 'do' / 'double'` — all-literal, so it folds into a
        // longest-match trie: `do` shared, then `uble` accept vs the bare
        // `do` accept. Two accepts → two `boundary`s.
        let body = Pattern::choice(vec![Pattern::literal("do"), Pattern::literal("double")]);
        let trie = append_to(body);
        assert_eq!(count_boundary(&trie), 2);
        // Source order must not matter: the scrambled set factors to the
        // identical trie (this is the structural #13 fix).
        let scrambled = append_to(Pattern::choice(vec![
            Pattern::literal("double"),
            Pattern::literal("do"),
        ]));
        assert_eq!(trie.strip_spans(), scrambled.strip_spans());
    }

    #[test]
    fn capture_choice_distributes_boundary_per_branch() {
        // `t: reserved = @k 'a' / @k 'b'` — each capture must keep `boundary` inside
        // so a committed keyword can't leak through recovery.
        let body = Pattern::choice(vec![
            Pattern::capture("k", Pattern::literal("a")),
            Pattern::capture("k", Pattern::literal("b")),
        ]);
        assert_eq!(count_boundary(&append_to(body)), 2);
    }
}
