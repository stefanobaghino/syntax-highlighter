//! End-to-end tests for direct left recursion: grammar source → compile
//! → run, asserting both parse outcomes and capture forests reflect
//! left-associative matching. Pairs with `compiler_tests.rs` (which
//! pins the bytecode skeleton) and `vm_tests.rs` (which pins the
//! VM-level bytecode semantics on hand-built programs).

use syntax_highlighter::pegc::{self, CompileError};
use syntax_highlighter::pegvm::{Capture, CaptureKind, MatchResult, VM};

fn run(src: &str, input: &[u8]) -> MatchResult {
    let prog = pegc::compile(src).expect("compile failed");
    VM::new(&prog.code, input).run()
}

fn cap(kinds: &[String], name: &str, start: usize, end: usize) -> Capture {
    let id = kinds
        .iter()
        .position(|k| k == name)
        .unwrap_or_else(|| panic!("capture kind '{}' not in {:?}", name, kinds));
    Capture {
        kind: CaptureKind(id as u16),
        start,
        end,
    }
}

#[test]
fn left_associative_addition() {
    // Standard left-associative arithmetic. Each '+' operand under the
    // 'op' tag, each leaf number under 'num'.
    let src = r#"
        expr <- expr @op{'+'} @num{[0-9]+} / @num{[0-9]+}
    "#;
    let prog = pegc::compile(src).expect("compile");
    let r = VM::new(&prog.code, b"1+2+3").run();
    assert!(r.complete);
    assert_eq!(r.matched, 5);
    let kinds = prog.capture_kinds.clone();
    // Captures are emitted in input order: leaf '1' (the seed),
    // then '+' '2' (first growth), then '+' '3' (second growth).
    assert_eq!(
        r.captures,
        vec![
            cap(&kinds, "num", 0, 1),
            cap(&kinds, "op", 1, 2),
            cap(&kinds, "num", 2, 3),
            cap(&kinds, "op", 3, 4),
            cap(&kinds, "num", 4, 5),
        ]
    );
}

#[test]
fn left_associative_with_distinct_operators() {
    // 'expr - x - y' must parse left-associatively, otherwise subtraction
    // semantics are wrong. The capture forest itself doesn't encode
    // associativity, but the seed-and-grow loop produces this ordering.
    let src = r#"
        expr <- expr @minus{'-'} @num{[0-9]+} / @num{[0-9]+}
    "#;
    let r = run(src, b"9-1-2");
    assert!(r.complete);
    assert_eq!(r.matched, 5);
}

#[test]
fn precedence_via_two_lr_layers() {
    // expr <- expr '+' term / term
    // term <- term '*' factor / factor
    // factor <- [0-9]+ / '(' expr ')'
    // Both expr and term are directly LR. The compiler must accept
    // both; the parse must reflect '*' binding tighter than '+'.
    let src = r#"
        expr   <- expr @plus{'+'} term / term
        term   <- term @star{'*'} factor / factor
        factor <- @num{[0-9]+} / '(' expr ')'
    "#;
    let r = run(src, b"1+2*3");
    assert!(r.complete);
    assert_eq!(r.matched, 5);
    // 1, +, 2, *, 3 — captures emitted in input order. The grouping
    // structure is implicit in how the seed-and-grow ran but the
    // capture stream must include the '*' between 2 and 3.
    let prog = pegc::compile(src).unwrap();
    let kinds = prog.capture_kinds.clone();
    assert_eq!(
        r.captures,
        vec![
            cap(&kinds, "num", 0, 1),
            cap(&kinds, "plus", 1, 2),
            cap(&kinds, "num", 2, 3),
            cap(&kinds, "star", 3, 4),
            cap(&kinds, "num", 4, 5),
        ]
    );
}

#[test]
fn lr_with_parenthesised_subexpression() {
    let src = r#"
        expr   <- expr @plus{'+'} term / term
        term   <- term @star{'*'} factor / factor
        factor <- @num{[0-9]+} / '(' expr ')'
    "#;
    let r = run(src, b"(1+2)*3");
    assert!(r.complete);
    assert_eq!(r.matched, 7);
}

#[test]
fn lr_failure_returns_partial() {
    // Input doesn't even start with a valid leaf.
    let src = r#"
        expr <- expr @op{'+'} @num{[0-9]+} / @num{[0-9]+}
    "#;
    let r = run(src, b"x");
    assert!(!r.complete);
    assert_eq!(r.matched, 0);
    assert!(r.captures.is_empty());
}

#[test]
fn lr_partial_chain_returns_longest_prefix() {
    // The wrapper rule (start, by source order) requires a trailing
    // '!' after the LR expression. For input "1+2" the LR rule
    // succeeds with the seed at sp=3, but '!' then fails — the
    // surrounding parse must report the deepest sp reached, which is
    // past the LR seed's growth.
    let src = r#"
        wrapper <- expr '!'
        expr    <- expr @op{'+'} @num{[0-9]+} / @num{[0-9]+}
    "#;
    let r = run(src, b"1+2");
    assert!(!r.complete);
    assert_eq!(r.matched, 3);
}

#[test]
fn nullable_lr_terminates() {
    // Pathological nullable LR: A <- A / "x". Without LR support this
    // would be an infinite Call → Call loop. Bounded LR semantics
    // accept the empty match (seed=None, body succeeds with sp
    // unchanged ⇒ no growth ⇒ commit empty seed). For input "x" the
    // first iteration runs the recursive `A` (fails, seed None), then
    // the second alternative 'x' wins; LRTail sees growth (0→1),
    // updates seed, re-iterates; second iteration recursive `A` hits
    // seed and replays sp=1, then `Or "x"` retries from sp=1 (no
    // growth past seed); LRTail commits at sp=1. Just assert it
    // terminates and matched is 1.
    let src = r#"
        a <- a / 'x'
    "#;
    let r = run(src, b"x");
    assert!(r.complete);
    assert_eq!(r.matched, 1);
}

#[test]
fn nullable_lr_on_empty_input_terminates() {
    // Same grammar, input "". The first iteration's recursive call
    // fails (seed None); the second alternative 'x' also fails. The
    // body fails on first iteration with seed None, so the LR rule
    // fails — partial parse at sp=0.
    let src = r#"
        a <- a / 'x'
    "#;
    let r = run(src, b"");
    assert!(!r.complete);
    assert_eq!(r.matched, 0);
}

#[test]
fn right_recursive_grammar_is_not_marked_lr() {
    // Sanity: a right-recursive grammar must still compile and run
    // (with the standard MemoOpen/MemoClose path). The compiler-level
    // assertion is in compiler_tests; here we just confirm runtime
    // behavior is unchanged.
    let src = r#"
        list <- @num{[0-9]+} (',' list)?
    "#;
    let r = run(src, b"1,2,3");
    assert!(r.complete);
    assert_eq!(r.matched, 5);
}

#[test]
fn indirect_lr_in_source_returns_compile_error() {
    let src = r#"
        a <- b 'x' / 'y'
        b <- a 'z' / 'w'
    "#;
    match pegc::compile(src) {
        Err(pegc::Error::Compile(CompileError::IndirectLeftRecursion(cycle))) => {
            assert!(cycle.contains(&"a".to_string()));
            assert!(cycle.contains(&"b".to_string()));
        }
        other => panic!("expected IndirectLeftRecursion, got {:?}", other),
    }
}

#[test]
fn lr_inside_recover_repeat_resyncs_cleanly() {
    // top <- (expr ';')*^ !.
    // expr <- expr @op{'+'} @num{[0-9]+} / @num{[0-9]+}
    //
    // Input "1+2;BAD;3+4;" — the middle "BAD" can't parse as an expr,
    // so the *^ recovery branch consumes one byte at a time. The
    // surrounding LR rule's failure must be rescued to its prior
    // seed (or its first-iteration failure must propagate cleanly so
    // *^'s outer Choice/Commit catches it).
    let src = r#"
        top  <- (expr ';')*^ !.
        expr <- expr @op{'+'} @num{[0-9]+} / @num{[0-9]+}
    "#;
    let r = run(src, b"1+2;BAD;3+4;");
    assert!(
        r.complete,
        "*^ must terminate cleanly even with mid-stream failure"
    );
    assert_eq!(r.matched, 12);
    // Captures should include the well-formed expressions plus
    // recovery captures for the BAD region.
    let prog = pegc::compile(src).unwrap();
    let kinds = prog.capture_kinds.clone();
    let nums: Vec<&Capture> = r
        .captures
        .iter()
        .filter(|c| kinds[c.kind.0 as usize] == "num")
        .collect();
    // Four numbers across the two valid expressions.
    assert_eq!(nums.len(), 4, "got: {:?}", r.captures);
    let recoveries: Vec<&Capture> = r
        .captures
        .iter()
        .filter(|c| kinds[c.kind.0 as usize] == "recovery")
        .collect();
    assert!(
        !recoveries.is_empty(),
        "the BAD region must produce recovery captures"
    );
}
