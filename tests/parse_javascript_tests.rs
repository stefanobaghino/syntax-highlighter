//! Per-grammar parse tests for JavaScript.

#[path = "common/mod.rs"]
mod common;

use common::kind_at;

const GRAMMAR: &str = include_str!("../grammars/javascript.peg");

#[test]
fn keyword_is_keyword() {
    let input = "function main() {}\n";
    let pos = input.find("function").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("keyword"));
}

#[test]
fn string_is_string() {
    let input = "const s = \"hello\";\n";
    let pos = input.find('"').unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("string"));
}

#[test]
fn template_literal_is_string() {
    let input = "const s = `hello ${name}`;\n";
    let pos = input.find('`').unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("string"));
}

#[test]
fn number_is_number() {
    let input = "const x = 42;\n";
    let pos = input.find("42").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("number"));
}

#[test]
fn line_comment_is_comment() {
    let input = "// a line\nconst x = 1;\n";
    let pos = input.find("//").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("comment"));
}

#[test]
fn fn_name_is_function() {
    let input = "function compute() {}\n";
    let pos = input.find("compute").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("function"));
}

#[test]
fn class_name_is_type() {
    let input = "class Foo {}\n";
    let pos = input.find("Foo").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("type"));
}

#[test]
fn new_target_is_type() {
    let input = "const x = new Foo(1);\n";
    let pos = input.find("Foo").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("type"));
}

#[test]
fn null_is_constant() {
    let input = "const x = null;\n";
    let pos = input.find("null").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("constant"));
}
