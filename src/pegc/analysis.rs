//! Static analyses over the grammar's [`Pattern`] AST.

use std::collections::{HashMap, HashSet};

use super::compiler::CompileError;
use super::pattern::Pattern;

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
