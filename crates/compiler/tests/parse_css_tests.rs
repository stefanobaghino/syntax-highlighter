//! Per-grammar parse tests for CSS.

#[path = "common/mod.rs"]
mod common;

use common::kind_at;

const GRAMMAR: &str = include_str!("../../../grammars/css.peg");

#[test]
fn property_name_is_property() {
    let input = "a { color: red; }\n";
    let pos = input.find("color").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("property"));
}

#[test]
fn tag_selector_is_type() {
    let input = "body { color: red; }\n";
    let pos = input.find("body").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("type"));
}

#[test]
fn class_selector_is_property() {
    let input = ".foo { color: red; }\n";
    let pos = input.find(".foo").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("property"));
}

#[test]
fn id_selector_is_constant() {
    let input = "#foo { color: red; }\n";
    let pos = input.find("#foo").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("constant"));
}

#[test]
fn hex_color_is_constant() {
    let input = "a { color: #ff0000; }\n";
    let pos = input.find("#ff0000").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("constant"));
}

#[test]
fn number_with_unit_is_number() {
    let input = "a { margin: 10px; }\n";
    let pos = input.find("10").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("number"));
}

#[test]
fn at_rule_is_keyword() {
    let input = "@media (min-width: 600px) { .c { color: red; } }\n";
    let pos = input.find("@media").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("keyword"));
}

#[test]
fn function_call_name_is_function() {
    let input = "a { color: rgb(255, 0, 0); }\n";
    let pos = input.find("rgb").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("function"));
}

#[test]
fn comment_is_comment() {
    let input = "/* block */\na { color: red; }\n";
    let pos = input.find("/*").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("comment"));
}

#[test]
fn important_is_keyword() {
    let input = "a { color: red !important; }\n";
    let pos = input.find("!important").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, pos).as_deref(), Some("keyword"));
}
