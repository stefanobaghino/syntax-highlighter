//! Indentation-structure tests for the Starlark subset: that `deeper` /
//! `same` actually enforce block shape — nested suites parse, dedents
//! close blocks, aligned siblings are accepted, and misaligned or
//! unindented lines make the parse fail. This is the behavioral payoff of
//! the parameterized indentation rules, distinct from the token-coverage
//! smoke tests in `tests/parse_starlark_tests.rs`.

#[path = "common/mod.rs"]
mod common;

use common::{is_complete, kind_at, matched_len};

const GRAMMAR: &str = include_str!("../../../grammars/starlark.peg");

fn complete(input: &str) -> bool {
    is_complete(GRAMMAR, input)
}

#[test]
fn nested_suite_parses_and_keeps_kinds() {
    let input = "def f():\n    x = 1\n    return x\n";
    assert!(complete(input), "matched {}", matched_len(GRAMMAR, input));
    // `def` keyword, the function name, and the nested `return` all keep
    // their kinds across the indentation boundary.
    assert_eq!(kind_at(GRAMMAR, input, 0).as_deref(), Some("keyword"));
    assert_eq!(kind_at(GRAMMAR, input, 4).as_deref(), Some("function")); // f
    let ret = input.find("return").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, ret).as_deref(), Some("keyword"));
}

#[test]
fn dedent_closes_block_and_resumes_outer() {
    // `g` is a column-0 sibling of `f`; it parses only if the dedent after
    // `f`'s body correctly closes `f`'s suite.
    let input = "def f():\n    return 1\n\ndef g():\n    return 2\n";
    assert!(complete(input), "matched {}", matched_len(GRAMMAR, input));
    let g = input.find("def g").unwrap() + 4;
    assert_eq!(kind_at(GRAMMAR, input, g).as_deref(), Some("function"));
}

#[test]
fn three_levels_of_nesting() {
    let input = "def f():\n    if a:\n        for x in y:\n            return x\n";
    assert!(complete(input), "matched {}", matched_len(GRAMMAR, input));
}

#[test]
fn aligned_siblings_in_a_suite_parse() {
    let input = "def f():\n    a = 1\n    b = 2\n    c = 3\n";
    assert!(complete(input), "matched {}", matched_len(GRAMMAR, input));
}

#[test]
fn misaligned_sibling_makes_parse_incomplete() {
    // Second body line is indented one column shallower than the first;
    // it aligns with no open block, so the parse cannot consume it.
    let input = "if a:\n    x = 1\n   y = 2\n";
    assert!(
        !complete(input),
        "a misaligned sibling must not parse: matched {} of {}",
        matched_len(GRAMMAR, input),
        input.len()
    );
}

#[test]
fn unindented_body_makes_parse_incomplete() {
    // The body of `if` is not indented past the header, so the suite's
    // `deeper` assertion fails and the block has no body.
    let input = "if a:\nx = 1\n";
    assert!(
        !complete(input),
        "an unindented suite body must not parse: matched {} of {}",
        matched_len(GRAMMAR, input),
        input.len()
    );
}

#[test]
fn over_indented_continuation_is_rejected() {
    // The third line is *deeper* than its siblings but is not a child of
    // the second (which is a simple statement, not a compound header), so
    // it cannot attach anywhere.
    let input = "def f():\n    a = 1\n        b = 2\n";
    assert!(
        !complete(input),
        "an over-indented orphan line must not parse: matched {} of {}",
        matched_len(GRAMMAR, input),
        input.len()
    );
}

#[test]
fn elif_else_chain_at_header_column_parses() {
    let input = "if a:\n    p = 1\nelif b:\n    q = 2\nelse:\n    r = 3\n";
    assert!(complete(input), "matched {}", matched_len(GRAMMAR, input));
    let elif = input.find("elif").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, elif).as_deref(), Some("keyword"));
    let els = input.find("else").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, els).as_deref(), Some("keyword"));
}
