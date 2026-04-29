//! Per-grammar parse tests for Go.

#[path = "common/mod.rs"]
mod common;

use common::kind_at;

const GRAMMAR: &str = include_str!("../grammars/go.peg");

#[test]
fn keyword_is_keyword() {
    let input = "package main\n";
    let pos = input.find("package").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("keyword"));
}

#[test]
fn string_is_string() {
    let input = "package p\n\nvar s = \"hello\"\n";
    let pos = input.find('"').unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("string"));
}

#[test]
fn raw_string_is_string() {
    let input = "package p\n\nvar s = `raw`\n";
    let pos = input.find('`').unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("string"));
}

#[test]
fn number_is_number() {
    let input = "package p\n\nvar n = 42\n";
    let pos = input.find("42").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("number"));
}

#[test]
fn line_comment_is_comment() {
    let input = "package p\n// a line\nfunc f() {}\n";
    let pos = input.find("//").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("comment"));
}

#[test]
fn fn_name_is_function() {
    let input = "package p\n\nfunc compute() {}\n";
    let pos = input.find("compute").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("function"));
}

#[test]
fn type_name_is_type() {
    let input = "package p\n\ntype Point struct {}\n";
    let pos = input.find("Point").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("type"));
}

#[test]
fn predeclared_type_is_type() {
    let input = "package p\n\nvar x int = 1\n";
    let pos = input.find("int").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("type"));
}

#[test]
fn nil_is_constant() {
    let input = "package p\n\nvar x = nil\n";
    let pos = input.find("nil").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("constant"));
}

#[test]
fn qualified_ident_dot_is_nested_punctuation() {
    // Go's `qualified_ident <- ident (@punctuation{'.'} ident)?` wrapped
    // by `@type{...}` produces a punctuation capture nested inside a
    // type capture. The walker resolves to the innermost kind, so the
    // dot's byte should report `punctuation`, not `type`.
    let input = "package p\nvar x pkg.Foo\n";
    let pos = input.find('.').unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("punctuation"));
    // The surrounding `pkg`/`Foo` is the outer `type` kind.
    let pkg_pos = input.find("pkg").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pkg_pos).as_deref(), Some("type"));
}
