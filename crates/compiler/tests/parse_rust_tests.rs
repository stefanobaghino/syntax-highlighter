//! Per-grammar parse tests for Rust.

#[path = "common/mod.rs"]
mod common;

use common::kind_at;

const GRAMMAR: &str = include_str!("../../../grammars/rust.peg");

#[test]
fn keyword_is_keyword() {
    let input = "fn main() {}\n";
    let pos = input.find("fn").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("keyword"));
}

#[test]
fn string_is_string() {
    let input = "fn f() { let s = \"hello\"; }\n";
    let pos = input.find('"').unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("string"));
}

#[test]
fn number_is_number() {
    let input = "fn f() -> i32 { 42 }\n";
    let pos = input.find("42").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("number"));
}

#[test]
fn line_comment_is_comment() {
    let input = "// a line\nfn f() {}\n";
    let pos = input.find("//").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("comment"));
}

#[test]
fn block_comment_is_comment() {
    let input = "/* a block */ fn f() {}\n";
    let pos = input.find("/*").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("comment"));
}

#[test]
fn fn_name_is_function() {
    let input = "fn compute() {}\n";
    let pos = input.find("compute").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("function"));
}

#[test]
fn macro_invocation_name_is_function() {
    let input = "fn main() { println!(\"hi\"); }\n";
    let pos = input.find("println").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("function"));
}

#[test]
fn upper_ident_in_path_is_type() {
    let input = "use std::collections::HashMap;\n";
    let pos = input.find("HashMap").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("type"));
}

#[test]
fn lifetime_is_type() {
    // The grammar groups lifetimes with type-shaped syntax — no
    // dedicated lifetime kind in the hardcoded theme.
    let input = "fn f<'a>(s: &'a str) {}\n";
    let pos = input.find("'a").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("type"));
}

#[test]
fn bool_literal_is_constant() {
    let input = "fn f() -> bool { true }\n";
    let pos = input.find("true").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("constant"));
}
