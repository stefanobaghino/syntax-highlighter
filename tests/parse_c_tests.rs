//! Per-grammar parse tests for C.

#[path = "common/mod.rs"]
mod common;

use common::kind_at;

const GRAMMAR: &str = include_str!("../grammars/c.peg");

#[test]
fn keyword_is_keyword() {
    let input = "int main(void) { return 0; }\n";
    let pos = input.find("return").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("keyword"));
}

#[test]
fn predef_type_is_type() {
    let input = "int x;\n";
    let pos = input.find("int").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("type"));
}

#[test]
fn string_is_string() {
    let input = "const char *s = \"hello\";\n";
    let pos = input.find('"').unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("string"));
}

#[test]
fn number_is_number() {
    let input = "int x = 42;\n";
    let pos = input.find("42").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("number"));
}

#[test]
fn line_comment_is_comment() {
    let input = "// a line\nint x;\n";
    let pos = input.find("//").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("comment"));
}

#[test]
fn block_comment_is_comment() {
    let input = "/* block */\nint x;\n";
    let pos = input.find("/*").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("comment"));
}

#[test]
fn preprocessor_is_keyword() {
    let input = "#include <stdio.h>\nint x;\n";
    let pos = input.find("#include").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("keyword"));
}

#[test]
fn function_name_is_function() {
    let input = "int compute(int n) { return n; }\n";
    let pos = input.find("compute").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("function"));
}

#[test]
fn struct_name_is_type() {
    let input = "struct Point { int x; };\n";
    let pos = input.find("Point").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("type"));
}

#[test]
fn enum_items_are_constant() {
    let input = "enum C { A, B };\n";
    let pos = input.find('A').unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("constant"));
}
