//! Per-grammar parse tests for TOML.

#[path = "common/mod.rs"]
mod common;

use common::{is_complete, kind_at};

const GRAMMAR: &str = include_str!("../grammars/toml.peg");

#[test]
fn key_is_property() {
    let input = "name = \"alice\"\n";
    let pos = input.find("name").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("property"));
}

#[test]
fn string_value_is_string() {
    let input = "name = \"alice\"\n";
    let pos = input.find('"').unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("string"));
}

#[test]
fn number_is_number() {
    let input = "n = 42\n";
    let pos = input.find("42").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("number"));
}

#[test]
fn boolean_is_constant() {
    let input = "b = true\n";
    let pos = input.find("true").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("constant"));
}

#[test]
fn inf_is_constant_not_number() {
    // inf and nan are constants in this grammar; group with true/false,
    // not with numeric literals.
    let input = "f = inf\n";
    let pos = input.find("inf").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("constant"));
}

#[test]
fn comment_is_comment() {
    let input = "# leading\nkey = 1\n";
    let pos = input.find('#').unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("comment"));
}

#[test]
fn section_header_is_type() {
    let input = "[package]\n";
    let pos = input.find("package").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("type"));
}

#[test]
fn array_of_tables_header_is_type() {
    let input = "[[bin]]\n";
    let pos = input.find("bin").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("type"));
}

#[test]
fn full_realistic_snippet_parses_to_completion() {
    let input = r#"# A realistic snippet.
[package]
name = "demo"
version = "0.1.0"
edition = 2021

[[bin]]
name = "demo"
"#;
    assert!(is_complete(GRAMMAR, input));
}
