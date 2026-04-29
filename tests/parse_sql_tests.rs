//! Per-grammar parse tests for SQLite.

#[path = "common/mod.rs"]
mod common;

use common::{is_complete, kind_at};

const GRAMMAR: &str = include_str!("../grammars/sqlite.peg");

#[test]
fn keyword_is_keyword() {
    let input = "SELECT 1";
    let pos = input.find("SELECT").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("keyword"));
}

#[test]
fn string_is_string() {
    let input = "SELECT 'hi' FROM t";
    let pos = input.find('\'').unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("string"));
}

#[test]
fn number_is_number() {
    let input = "SELECT 42 FROM t";
    let pos = input.find("42").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("number"));
}

#[test]
fn null_is_constant() {
    let input = "SELECT NULL";
    let pos = input.find("NULL").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("constant"));
}

#[test]
fn comment_is_comment() {
    let input = "-- a note\nSELECT 1";
    let pos = input.find("--").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("comment"));
}

#[test]
fn function_call_is_function() {
    let input = "SELECT COUNT(*) FROM t";
    let pos = input.find("COUNT").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("function"));
}

#[test]
fn table_name_in_from_is_type() {
    let input = "SELECT c FROM users";
    let pos = input.find("users").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("type"));
}

#[test]
fn cast_target_is_type() {
    let input = "SELECT CAST(x AS INTEGER) FROM t";
    let pos = input.find("INTEGER").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("type"));
}

#[test]
fn bind_param_is_variable() {
    let input = "SELECT :name FROM t";
    let pos = input.find(":name").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("variable"));
}

#[test]
fn large_fixture_parses_to_completion() {
    // End-to-end guard: catches false-reject regressions (e.g. reserving
    // an identifier that real SQL uses as a column name).
    let input = include_str!("../benches/fixtures/large.sql");
    assert!(
        is_complete(GRAMMAR, input),
        "expected full parse of benches/fixtures/large.sql"
    );
}
