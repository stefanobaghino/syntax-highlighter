//! Memoization behavior tests: the cache populates, hits replay correctly,
//! and captures survive replay — even when the memoized rule is nested
//! inside an outer capture (the capture-replay bug most likely to bite a
//! naive first implementation).
//!
//! These tests pin `with_memo_threshold(0)` explicitly. They exercise the
//! mechanism on tiny inputs (1–3 bytes) whose rule bodies fall below the
//! production default; opting out makes the assertions independent of the
//! default value.

use std::collections::HashMap;

use syntax_highlighter::pegc::{Grammar, Pattern};
use syntax_highlighter::pegvm::{Capture, CaptureKind, CharSet, MatchResult, VM};

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
        "root".into(),
        Pattern::choice(vec![
            Pattern::seq(vec![Pattern::nt("X"), Pattern::literal("aa")]),
            Pattern::seq(vec![Pattern::nt("X"), Pattern::literal("bb")]),
        ]),
    );
    rules.insert(
        "X".into(),
        Pattern::char_class(CharSet::from_ranges(&[(b'a', b'z')])),
    );
    let prog = Grammar::new(rules).compile().unwrap();
    let (result, stats) = VM::new(&prog.code, b"abb")
        .with_memo_threshold(0)
        .run_with_memo_stats();
    assert_eq!(
        result,
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
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
        "root".into(),
        Pattern::seq(vec![Pattern::nt("X"), Pattern::nt("X")]),
    );
    rules.insert(
        "X".into(),
        Pattern::char_class(CharSet::from_ranges(&[(b'a', b'z')])),
    );
    let prog = Grammar::new(rules).compile().unwrap();
    let (_, stats) = VM::new(&prog.code, b"ab")
        .with_memo_threshold(0)
        .run_with_memo_stats();
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
        "root".into(),
        Pattern::capture(
            "outer",
            Pattern::choice(vec![
                Pattern::seq(vec![Pattern::nt("Inner"), Pattern::literal("x")]),
                Pattern::seq(vec![Pattern::nt("Inner"), Pattern::literal("y")]),
            ]),
        ),
    );
    rules.insert(
        "Inner".into(),
        Pattern::capture(
            "inner",
            Pattern::char_class(CharSet::from_ranges(&[(b'a', b'z')])),
        ),
    );
    let prog = Grammar::new(rules).compile().unwrap();
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

    let (result, stats) = VM::new(&prog.code, b"ay")
        .with_memo_threshold(0)
        .run_with_memo_stats();
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

/// `start <- (!A "b") / (!A "c")`, `A <- "a"`.
///
/// On input "cx" the first alternative's `!A` succeeds because A fails at
/// sp=0, but "b" fails at sp=0, forcing backtrack. The second alternative's
/// `!A` at sp=0 must hit the cached A-failure rather than re-executing A's
/// body.
#[test]
fn cached_failure_short_circuits_not_predicate() {
    let mut rules = HashMap::new();
    rules.insert(
        "root".into(),
        Pattern::choice(vec![
            Pattern::seq(vec![
                Pattern::not_predicate(Pattern::nt("A")),
                Pattern::literal("b"),
            ]),
            Pattern::seq(vec![
                Pattern::not_predicate(Pattern::nt("A")),
                Pattern::literal("c"),
            ]),
        ]),
    );
    rules.insert("A".into(), Pattern::literal("a"));
    let prog = Grammar::new(rules).compile().unwrap();
    let (result, stats) = VM::new(&prog.code, b"c")
        .with_memo_threshold(0)
        .run_with_memo_stats();
    assert!(result.complete);
    assert_eq!(result.matched, 1);
    assert!(
        stats.hits >= 1,
        "expected a cached-failure hit, got {}",
        stats.hits
    );
}

/// Nested memoized rules both failing must both land failure entries in the
/// table — not just the innermost one. `outer <- inner "b"`, `inner <- "a"`.
/// On input "x": inner fails, outer has no backtrack machinery so it also
/// fails. Both failures must be cached.
#[test]
fn nested_memoized_rule_failures_both_cached() {
    let mut rules = HashMap::new();
    rules.insert(
        "root".into(),
        Pattern::choice(vec![Pattern::nt("outer"), Pattern::nt("fallback")]),
    );
    rules.insert(
        "outer".into(),
        Pattern::seq(vec![Pattern::nt("inner"), Pattern::literal("b")]),
    );
    rules.insert("inner".into(), Pattern::literal("a"));
    rules.insert(
        "fallback".into(),
        Pattern::char_class(CharSet::from_ranges(&[(b'a', b'z')])),
    );
    let prog = Grammar::new(rules).compile().unwrap();
    let (result, stats) = VM::new(&prog.code, b"x")
        .with_memo_threshold(0)
        .run_with_memo_stats();
    assert!(result.complete);
    assert_eq!(result.matched, 1);
    // Entries expected: start(success at 0), outer(failure at 0),
    // inner(failure at 0), fallback(success at 0). Four distinct slots.
    assert_eq!(stats.entries, 4, "got entries: {}", stats.entries);
}

/// `&A` succeeds in both alternatives; the second alternative's `&A` call
/// must hit the cache AND BackCommit must still find a `Backtrack` on top
/// (not the `Memo` frame from the successful rule call, which `MemoClose`
/// has already popped).
///
/// `start <- (&A "z") / (&A "y")`, `A <- [a-z]`. On input "y": &A peeks and
/// succeeds at sp=0, "z" fails at sp=0, backtrack to the second alternative,
/// &A peeks again at sp=0 (memo hit), "y" matches.
#[test]
fn and_predicate_with_memoized_rule() {
    let mut rules = HashMap::new();
    rules.insert(
        "root".into(),
        Pattern::choice(vec![
            Pattern::seq(vec![
                Pattern::and_predicate(Pattern::nt("A")),
                Pattern::literal("z"),
            ]),
            Pattern::seq(vec![
                Pattern::and_predicate(Pattern::nt("A")),
                Pattern::literal("y"),
            ]),
        ]),
    );
    rules.insert(
        "A".into(),
        Pattern::char_class(CharSet::from_ranges(&[(b'a', b'z')])),
    );
    let prog = Grammar::new(rules).compile().unwrap();
    let (result, stats) = VM::new(&prog.code, b"y")
        .with_memo_threshold(0)
        .run_with_memo_stats();
    assert!(result.complete);
    assert_eq!(result.matched, 1);
    assert!(stats.hits >= 1, "expected a hit, got {}", stats.hits);
}

/// LR-specific cache write (#48): a converged seed must land in the memo
/// so a re-parse of the same input at the same sp short-circuits via a
/// cache hit instead of re-running the seed-and-grow loop.
#[test]
fn lr_converged_seed_persists_across_parses() {
    use syntax_highlighter::pegc;
    use syntax_highlighter::pegvm::MemoCache;
    let src = r#"
        root <- expr
        expr <- expr '+' [0-9]+ / [0-9]+
    "#;
    let prog = pegc::compile(src).expect("compile");
    let input = b"1+2+3";

    // Cold parse populates the memo with the LR rule's converged seed.
    let (cold, cold_stats, memo) = VM::new(&prog.code, input)
        .with_memo_threshold(0)
        .run_with_cache();
    assert!(cold.complete);
    assert_eq!(cold.matched, 5);
    assert_eq!(cold_stats.hits, 0, "cold run starts empty");
    let cold_entries = memo.len();
    assert!(
        cold_entries >= 1,
        "expected ≥1 memo entry after cold LR parse, got {cold_entries}"
    );

    // Warm parse with the same input: RuleEnter at sp=0 must hit the cache.
    let (warm, warm_stats, _) = VM::new_with_cache(&prog.code, input, memo)
        .with_memo_threshold(0)
        .run_with_cache();
    assert_eq!(warm, cold);
    assert!(
        warm_stats.hits >= 1,
        "expected ≥1 cache hit on warm LR parse, got {}",
        warm_stats.hits
    );

    // Sanity check: an empty cache reproduces the cold-run misses,
    // confirming the warm hit came from the seeded cache.
    let (_, fresh_stats, _) = VM::new_with_cache(&prog.code, input, MemoCache::new())
        .with_memo_threshold(0)
        .run_with_cache();
    assert_eq!(fresh_stats.hits, 0);
}

/// A grammar that's memoized-by-default produces byte-identical results on
/// a "happy path" input with no backtracking — the cache has no effect on
/// correctness, only on work done.
#[test]
fn memoization_does_not_change_results_on_linear_parse() {
    let mut rules = HashMap::new();
    rules.insert("root".into(), Pattern::repeat_one(Pattern::nt("digit")));
    rules.insert(
        "digit".into(),
        Pattern::char_class(CharSet::from_ranges(&[(b'0', b'9')])),
    );
    let prog = Grammar::new(rules).compile().unwrap();
    let result = VM::new(&prog.code, b"12345").run();
    assert_eq!(
        result,
        MatchResult {
            matched: 5,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
}
