//! Static analyses over the grammar's [`Pattern`] AST.
//!
//! The compiler consumes [`analyze_left_recursion`] to decide which
//! rules close with `LRTail` instead of `MemoClose`. Tooling consumes
//! the rest (FIRST/FOLLOW) for grammar inspection — see the
//! `pegc follow-set` subcommand.

use std::collections::{BTreeSet, HashMap, HashSet};

use super::compiler::CompileError;
use super::parser::Grammar;
use super::pattern::Pattern;
use crate::pegvm::CharSet;

/// Returns the set of rule names that are left-recursive — both direct
/// (`A <- A α / β`) and indirect (`A <- B …; B <- A …`). Each such rule
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
        Pattern::Literal(bytes) => bytes.is_empty(),
        Pattern::CharClass(_) | Pattern::AnyChar => false,
        Pattern::Sequence(items) => items.iter().all(|p| pattern_nullable(p, nullable)),
        Pattern::OrderedChoice(items) => items.iter().any(|p| pattern_nullable(p, nullable)),
        Pattern::Repeat(_) | Pattern::Optional(_) => true,
        Pattern::RepeatOne(inner) => pattern_nullable(inner, nullable),
        // Predicates consume no input, so they always succeed-or-fail
        // without advancing — treated as nullable for sequence
        // propagation. They can still left-recurse: !A α matches only
        // if A would fail at sp, and A's evaluation can left-recurse
        // through itself just like a direct call.
        Pattern::NotPredicate(_) | Pattern::AndPredicate(_) => true,
        Pattern::Capture(_, inner) => pattern_nullable(inner, nullable),
        // Catch succeeds-as-inner when inner succeeds, or
        // succeeds-as-recovery when inner fails. The recovery branch
        // only runs on inner's failure, so it never contributes a
        // "matches empty after success" path — nullability follows
        // inner.
        Pattern::Catch { inner, .. } => pattern_nullable(inner, nullable),
        Pattern::NonTerminal(name) => nullable.contains(name),
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
        Pattern::Literal(_) | Pattern::CharClass(_) | Pattern::AnyChar => {}
        Pattern::Sequence(items) => {
            for it in items {
                collect_first_calls(it, nullable, out);
                if !pattern_nullable(it, nullable) {
                    break;
                }
            }
        }
        Pattern::OrderedChoice(items) => {
            for it in items {
                collect_first_calls(it, nullable, out);
            }
        }
        Pattern::Repeat(inner)
        | Pattern::RepeatOne(inner)
        | Pattern::Optional(inner)
        | Pattern::NotPredicate(inner)
        | Pattern::AndPredicate(inner)
        | Pattern::Capture(_, inner) => collect_first_calls(inner, nullable, out),
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
        Pattern::NonTerminal(name) => {
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
/// Captures are preserved (not stripped to their inner): `@punctuation{','}`
/// and `@operator{','}` are distinct elements.
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
        Pattern::Literal(bytes) => {
            out.insert(FollowElement::Literal(bytes.clone()));
        }
        Pattern::CharClass(cs) => {
            out.insert(FollowElement::CharClass(*cs));
        }
        Pattern::AnyChar => {
            out.insert(FollowElement::CharClass(CharSet::full()));
        }
        Pattern::NonTerminal(name) => {
            out.insert(FollowElement::Rule(name.clone()));
        }
        Pattern::Sequence(items) => {
            for it in items {
                out.extend(pattern_first(it, nullable));
                if !pattern_nullable(it, nullable) {
                    break;
                }
            }
        }
        Pattern::OrderedChoice(alts) => {
            for alt in alts {
                out.extend(pattern_first(alt, nullable));
            }
        }
        Pattern::Repeat(inner) | Pattern::RepeatOne(inner) | Pattern::Optional(inner) => {
            out.extend(pattern_first(inner, nullable));
        }
        Pattern::NotPredicate(_) => {}
        Pattern::AndPredicate(inner) => {
            out.extend(pattern_first(inner, nullable));
        }
        Pattern::Capture(kind, inner) => {
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
    if let Some(entry) = follow.get_mut(&grammar.start) {
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
        Pattern::Literal(_) | Pattern::CharClass(_) | Pattern::AnyChar => {}
        Pattern::NonTerminal(name) => {
            extend_follow(follow, name, trailing, changed);
        }
        Pattern::Sequence(items) => {
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
        Pattern::OrderedChoice(alts) => {
            for alt in alts {
                collect_follow(alt, nullable, trailing, follow, changed);
            }
        }
        Pattern::Repeat(inner) | Pattern::RepeatOne(inner) => {
            // The body can iterate, so its tail is followed by another
            // copy of FIRST(body). Plus whatever trails the repeat.
            let body_first = pattern_first(inner, nullable);
            let mut sub_trailing = trailing.clone();
            sub_trailing.extend(body_first);
            collect_follow(inner, nullable, &sub_trailing, follow, changed);
        }
        Pattern::Optional(inner) => {
            collect_follow(inner, nullable, trailing, follow, changed);
        }
        Pattern::NotPredicate(_) => {
            // Predicate consumes nothing; its body's FOLLOW relations
            // don't affect the outer sequence's flow. References inside
            // a `!p` predicate still need their FIRST recorded though,
            // since the predicate's match position is where they'd
            // appear — but FOLLOW is "what's after when matched," and
            // a `!p` predicate doesn't match anything. Skip.
        }
        Pattern::AndPredicate(inner) => {
            // Similar zero-width logic. Inner FOLLOW relations are
            // structurally meaningful (a NonTerminal inside `&p` does
            // have its own callers expecting it to FIRST-match certain
            // things), so recurse with an empty trailing — the predicate
            // doesn't continue into a consumed tail.
            collect_follow(inner, nullable, &FollowSet::new(), follow, changed);
        }
        Pattern::Capture(_, inner) => {
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
        Pattern::Optional(inner) => {
            out.extend(pattern_first(inner, nullable));
        }
        Pattern::Sequence(items) => {
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
        Pattern::Capture(_, inner) => {
            out.extend(trailing_first(inner, nullable));
        }
        Pattern::Catch { inner, .. } => {
            out.extend(trailing_first(inner, nullable));
        }
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
        Pattern::Optional(_) => true,
        Pattern::Capture(_, inner) => is_trailing_optional_like(inner),
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
        Pattern::Literal(_) | Pattern::CharClass(_) | Pattern::AnyChar => {}
        Pattern::NonTerminal(name) => {
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
            });
        }
        Pattern::Sequence(items) => {
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
        Pattern::OrderedChoice(alts) => {
            for alt in alts {
                walk_for_call_sites(alt, caller, continuations, inside_catch, ctx, findings);
            }
        }
        Pattern::Optional(inner)
        | Pattern::Capture(_, inner)
        | Pattern::Repeat(inner)
        | Pattern::RepeatOne(inner) => {
            walk_for_call_sites(inner, caller, continuations, inside_catch, ctx, findings);
        }
        Pattern::NotPredicate(inner) | Pattern::AndPredicate(inner) => {
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
        return true;
    }

    let mut rule_names: Vec<&String> = ctx.grammar.rules.keys().collect();
    rule_names.sort();

    if caller == ctx.grammar.start && !target_trailing.contains(&FollowElement::Eof) {
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
        Pattern::Literal(_) | Pattern::CharClass(_) | Pattern::AnyChar => false,
        Pattern::NonTerminal(name) => {
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
        Pattern::Sequence(items) => {
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
        Pattern::OrderedChoice(alts) => {
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
        Pattern::Optional(inner)
        | Pattern::Capture(_, inner)
        | Pattern::Repeat(inner)
        | Pattern::RepeatOne(inner) => find_callsite_unguarded(
            inner,
            probe,
            continuations,
            inside_catch,
            ctx,
            visited_callers,
            found_callsite,
        ),
        Pattern::NotPredicate(inner) | Pattern::AndPredicate(inner) => {
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
        if matches!(item, Pattern::AndPredicate(_)) {
            return FrameOutcome::Anchored;
        }
        if let Pattern::NotPredicate(inner) = item {
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
    let any_char = CharSet::full();
    if set
        .iter()
        .any(|s| matches!(s, FollowElement::CharClass(cs) if *cs == any_char))
    {
        return true;
    }
    set.contains(elem)
}
