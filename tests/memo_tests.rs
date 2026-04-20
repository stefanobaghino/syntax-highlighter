//! Memoization behavior tests: the cache populates, hits replay correctly,
//! and captures survive replay — even when the memoized rule is nested
//! inside an outer capture (the capture-replay bug most likely to bite a
//! naive first implementation).

use std::collections::HashMap;

use syntax_highlighter::pegvm::{
    compile_grammar, Capture, CaptureKind, CharSet, MatchResult, Pattern, VM,
};

fn cap(kind: u16, start: usize, end: usize) -> Capture {
    Capture {
        kind: CaptureKind(kind),
        start,
        end,
    }
}

/// `start <- (X "aa") / (X "bb")`, `X <- [a-z]`.
///
/// On input "ab" the first alternative fails after X matches at sp=0, the VM
/// backtracks to sp=0, and the second alternative re-calls X at the same
/// position. The second call must hit the cache.
#[test]
fn second_alternative_reuses_memoized_x() {
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
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
    rules.insert(
        "X".into(),
        Pattern::CharClass(CharSet::from_ranges(&[(b'a', b'z')])),
    );
    let prog = compile_grammar(&rules, "start").unwrap();
    let (result, stats) = VM::new(&prog.code, b"abb").run_with_memo_stats();
    assert_eq!(
        result,
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
        }
    );
    assert!(stats.hits >= 1, "expected ≥1 memo hit, got {}", stats.hits);
}

/// Same shape as above but verifies the memo table holds entries (not just
/// hits). One entry per distinct `(rule, sp)` pair actually executed.
#[test]
fn memo_table_populates_on_success() {
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::seq(vec![
            Pattern::NonTerminal("X".into()),
            Pattern::NonTerminal("X".into()),
        ]),
    );
    rules.insert(
        "X".into(),
        Pattern::CharClass(CharSet::from_ranges(&[(b'a', b'z')])),
    );
    let prog = compile_grammar(&rules, "start").unwrap();
    let (_, stats) = VM::new(&prog.code, b"ab").run_with_memo_stats();
    // start at sp=0, X at sp=0, X at sp=1 — three distinct entries.
    assert_eq!(stats.entries, 3);
    // No alternatives are tried twice at the same sp, so no hits.
    assert_eq!(stats.hits, 0);
}

/// Bug A regression: a memoized rule is called from inside an enclosing
/// `@outer{ ... }` capture. On the second call (a memo hit) the replay
/// inserts cached captures as *closed* entries; the enclosing `CaptureEnd`
/// must still bind to the outer capture (the only one with `end.is_none()`)
/// rather than mis-closing a replayed entry.
///
/// Grammar:
///   start <- outer
///   outer <- @outer{ Inner Inner }   # two calls to Inner inside a capture
///   Inner <- @inner{ [a-z] }
///
/// On "aa": first Inner call at sp=0 produces `@inner{"a"}`. Second Inner
/// call at sp=1 is a distinct entry (different sp) — but the structure
/// exercises the replay machinery: both @inner captures must nest under
/// @outer correctly.
///
/// To force a true replay at the same sp, we instead use:
///   start <- outer
///   outer <- @outer{ (Inner "x") / (Inner "y") }
///   Inner <- @inner{ [a-z] }
/// On "ay": first alternative fails ("ay" doesn't have trailing "x"),
/// Inner's cached result is reused in the second alternative. The critical
/// assertion is that the outer capture closes at sp=2 (not at sp=1 — which
/// is where the inner capture closes), i.e. CaptureEnd bound to the right
/// slot during the replay.
#[test]
fn memo_hit_inside_outer_capture_preserves_nesting() {
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::Capture(
            "outer".into(),
            Box::new(Pattern::choice(vec![
                Pattern::seq(vec![
                    Pattern::NonTerminal("Inner".into()),
                    Pattern::literal("x"),
                ]),
                Pattern::seq(vec![
                    Pattern::NonTerminal("Inner".into()),
                    Pattern::literal("y"),
                ]),
            ])),
        ),
    );
    rules.insert(
        "Inner".into(),
        Pattern::Capture(
            "inner".into(),
            Box::new(Pattern::CharClass(CharSet::from_ranges(&[(b'a', b'z')]))),
        ),
    );
    let prog = compile_grammar(&rules, "start").unwrap();
    // Capture kind ids are assigned in first-seen order: "outer" = 0, "inner" = 1.
    let outer_kind = prog
        .capture_kinds
        .iter()
        .position(|n| n == "outer")
        .unwrap() as u16;
    let inner_kind = prog
        .capture_kinds
        .iter()
        .position(|n| n == "inner")
        .unwrap() as u16;

    let (result, stats) = VM::new(&prog.code, b"ay").run_with_memo_stats();
    assert!(result.complete);
    assert_eq!(result.matched, 2);
    // Outer capture spans the full input [0,2); inner captures the 'a' at [0,1).
    // The inner capture appears twice because the memoized Inner is first
    // executed (emitting @inner{[0,1)}) and then *replayed* from cache on the
    // second alternative (emitting another @inner{[0,1)}). Both must nest
    // under @outer, which must close at 2.
    assert!(
        result.captures.contains(&cap(outer_kind, 0, 2)),
        "outer capture missing or mis-closed: {:?}",
        result.captures
    );
    assert!(
        result.captures.contains(&cap(inner_kind, 0, 1)),
        "inner capture missing: {:?}",
        result.captures
    );
    assert!(stats.hits >= 1, "expected a cache hit, got {}", stats.hits);
}

/// A grammar that's memoized-by-default produces byte-identical results on
/// a "happy path" input with no backtracking — the cache has no effect on
/// correctness, only on work done.
#[test]
fn memoization_does_not_change_results_on_linear_parse() {
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::RepeatOne(Box::new(Pattern::NonTerminal("digit".into()))),
    );
    rules.insert(
        "digit".into(),
        Pattern::CharClass(CharSet::from_ranges(&[(b'0', b'9')])),
    );
    let prog = compile_grammar(&rules, "start").unwrap();
    let result = VM::new(&prog.code, b"12345").run();
    assert_eq!(
        result,
        MatchResult {
            matched: 5,
            captures: vec![],
            complete: true,
        }
    );
}
