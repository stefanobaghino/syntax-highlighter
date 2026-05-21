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
