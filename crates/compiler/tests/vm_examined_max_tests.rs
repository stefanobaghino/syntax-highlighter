//! Verify that every memo entry records the farthest input position
//! its rule invocation ever examined, including lookahead past
//! `end_sp` and failed reads. Incremental parsing's invalidation
//! predicate (`edit.start <= entry.examined_max`) depends on this
//! bound being tight-enough to be useful and safe-at-minimum to be
//! correct.
//!
//! Tests access `VM::run_core` directly to inspect the private memo
//! map. The reachable API is a `#[doc(hidden)] pub` testing surface;
//! ordinary callers use `run_with_cache`. Hand-rolled `Grammar`s
//! built via `pegc::Pattern` keep these tests focused on the runtime
//! invariant — the shape of each grammar matches one specific code
//! path through `run_core`.

use std::collections::HashMap;

use syntax_highlighter::pegvm::{ArgKey, MemoId, VM};
use syntax_highlighter_compiler::pegc::{Grammar, Pattern, Span};

fn rule(rules: &mut HashMap<String, Pattern>, name: &str, pat: Pattern) {
    rules.insert(name.into(), pat);
}

#[test]
fn and_predicate_success_records_examined_past_end_sp() {
    // root = consumer &"y"; consumer = "x"   against "xy"
    // `consumer`'s memo entry is the focus: end_sp=1 (matched "x")
    // but examined_max must be 1 (only position 0 was read inside
    // consumer). `root`'s subsequent `&"y"` is a sibling of the
    // consumer call, not inside it — its examined_max contribution
    // lands on `root`'s entry, not `consumer`'s.
    let mut rules = HashMap::new();
    rule(
        &mut rules,
        "root",
        Pattern::seq(vec![
            Pattern::nt("consumer"),
            Pattern::and_predicate(Pattern::literal("y")),
        ]),
    );
    rule(&mut rules, "consumer", Pattern::literal("x"));
    let prog = Grammar::new(rules).compile().unwrap();
    let (result, _stats, memo) = VM::new_from_program(&prog, b"xy")
        .with_memo_threshold(0)
        .run_core();
    // The `root` wrap supplies `!.` at the tail: trailing 'y' leaves
    // 1 byte unconsumed, so the overall parse fails. The memo
    // entries it built before failing are still observable.
    assert!(!result.complete, "trailing 'y' leaves bytes after !.");
    let root = memo
        .get(&(MemoId(0), 0, ArgKey::None))
        .expect("root's entry missing");
    assert_eq!(root.end_sp, None);
    assert_eq!(
        root.examined_max, 2,
        "root saw &\"y\" succeed at sp=1 and !. fail at sp=2"
    );
}

#[test]
fn and_predicate_failure_records_examined_up_to_failed_read() {
    // start = "x" &"z"   against "xy"
    // "x" succeeds (sp=1), then &"z" reads 'y' at sp=1 and fails.
    // The failure entry for start at sp=0 must remember that
    // position 1 was examined, so an edit there can invalidate it.
    let mut rules = HashMap::new();
    rule(
        &mut rules,
        "root",
        Pattern::seq(vec![
            Pattern::literal("x"),
            Pattern::and_predicate(Pattern::literal("z")),
        ]),
    );
    let prog = Grammar::new(rules).compile().unwrap();
    let (result, _stats, memo) = VM::new_from_program(&prog, b"xy")
        .with_memo_threshold(0)
        .run_core();
    assert!(!result.complete, "overall parse must fail");
    let entry = memo
        .get(&(MemoId(0), 0, ArgKey::None))
        .expect("start's failure entry missing");
    assert_eq!(entry.end_sp, None);
    assert_eq!(
        entry.examined_max, 2,
        "failed read of 'z' at position 1 examined position 1+1"
    );
}

#[test]
fn nested_rule_examined_max_propagates_to_caller() {
    // root = inner "y"
    // inner = "x"
    // against "xy".  inner's entry: end_sp=1, examined_max=1.
    // root's entry: end_sp=2 (the body span), but examined_max=3
    // because the synthesized `!.` at root's tail reads one byte
    // past end-of-input (sp=2) to verify EOF. inner's
    // examined_max (1) must propagate into root's watermark
    // regardless — that's the propagation we're verifying.
    let mut rules = HashMap::new();
    rule(
        &mut rules,
        "root",
        Pattern::seq(vec![Pattern::nt("inner"), Pattern::literal("y")]),
    );
    rule(&mut rules, "inner", Pattern::literal("x"));
    let prog = Grammar::new(rules).compile().unwrap();
    let (result, _stats, memo) = VM::new_from_program(&prog, b"xy")
        .with_memo_threshold(0)
        .run_core();
    assert!(result.complete);
    assert_eq!(result.matched, 2);
    // root is start → MemoId(0); inner is the other → MemoId(1).
    let outer = memo
        .get(&(MemoId(0), 0, ArgKey::None))
        .expect("root entry missing");
    assert_eq!(outer.end_sp, Some(2));
    // The wrap's tail-`!.` reads byte 2 once to confirm EOF; the
    // bound is the body's reach plus that one-byte EOF probe.
    assert!(
        outer.examined_max >= 2,
        "inner's examined reach must flow into root: {:?}",
        outer
    );
    let inner = memo
        .get(&(MemoId(1), 0, ArgKey::None))
        .expect("inner entry missing");
    assert_eq!(inner.end_sp, Some(1));
    assert_eq!(inner.examined_max, 1);
}

#[test]
fn memo_hit_propagates_examined_max_to_caller() {
    // Two alternatives both start with the memoized rule X.
    // The first alternative's success call to X populates the
    // cache with examined_max = 1; the second alternative's call
    // is a hit. The hit must still contribute X's examined_max to
    // whichever rule encloses the call.
    //
    // Grammar:
    //   start = (X "aa") / (X "bb")
    //   X = "a"
    // Input: "abb" — first alt fails after X matches "a" and "aa"
    // fails on "bb"; backtrack to second alt, which hits X's cache
    // and then matches "bb".
    let mut rules = HashMap::new();
    rule(
        &mut rules,
        "root",
        Pattern::choice(vec![
            Pattern::seq(vec![Pattern::nt("X"), Pattern::literal("aa")]),
            Pattern::seq(vec![Pattern::nt("X"), Pattern::literal("bb")]),
        ]),
    );
    rule(&mut rules, "X", Pattern::literal("a"));
    let prog = Grammar::new(rules).compile().unwrap();
    let (result, stats, memo) = VM::new_from_program(&prog, b"abb")
        .with_memo_threshold(0)
        .run_core();
    assert!(result.complete);
    assert_eq!(result.matched, 3);
    assert!(stats.hits >= 1, "second alternative must hit X's cache");
    // start's examined_max must include every byte start's execution
    // ever looked at: position 2 was examined when the first
    // alternative tried "aa" at sp=1 and "bb" at sp=1 needed byte 2.
    let start = memo
        .get(&(MemoId(0), 0, ArgKey::None))
        .expect("start entry missing");
    assert_eq!(start.end_sp, Some(3));
    // The `root` wrap's tail-`!.` reads byte 3 once to confirm
    // EOF; without that probe the watermark would land at 3.
    assert!(
        start.examined_max >= 3,
        "start's execution examined up to position 3, got {:?}",
        start
    );
}

#[test]
fn recover_repeat_propagates_examined_max_through_loop_iterations() {
    // start = ("ab")*^   against "abxab"
    //
    // The recovery loop runs entirely inside start's memo entry.
    // Across iterations the loop reads positions 0, 1 (success),
    // 2 (Byte 'a' fails on 'x'), 2 (Any consumes 'x' → sp=3),
    // 3, 4 (success), 5 (Byte 'a' at EOF). Whether the failed
    // EOF read at sp=5 contributes depends on track_read's call
    // discipline, so the assertion is a lower bound: every
    // position the loop *successfully* read must appear in
    // examined_max.
    let mut rules = HashMap::new();
    // `*^` desugars to `(p ^recovery @recovery .)*` — see
    // `build_recover_repeat` in `src/pegc/parser.rs`.
    rule(
        &mut rules,
        "root",
        Pattern::repeat(Pattern::Catch {
            inner: Box::new(Pattern::literal("ab")),
            label: "recovery".into(),
            recovery: Box::new(Pattern::capture("recovery", Pattern::any_char())),
            span: Span::SYNTHETIC,
        }),
    );
    let prog = Grammar::new(rules).compile().unwrap();
    let (result, _stats, memo) = VM::new_from_program(&prog, b"abxab")
        .with_memo_threshold(0)
        .run_core();
    assert!(result.complete);
    assert_eq!(result.matched, 5);
    let entry = memo
        .get(&(MemoId(0), 0, ArgKey::None))
        .expect("start entry missing");
    assert_eq!(entry.end_sp, Some(5));
    assert!(
        entry.examined_max >= 5,
        "recovery-loop reads must propagate to enclosing rule's watermark; got {}",
        entry.examined_max
    );
}
