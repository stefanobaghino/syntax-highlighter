use syntax_highlighter::highlight::{theme, Highlighter};

#[path = "common/mod.rs"]
mod common;
use common::strip_ansi;

const CSS_GRAMMAR: &str = include_str!("../grammars/css.peg");

fn hl(input: &str) -> Highlighter {
    let mut h = Highlighter::new(CSS_GRAMMAR).expect("CSS grammar should compile");
    h.set_input(input.to_string());
    h
}

#[test]
fn round_trip_strips_to_input() {
    let cases: &[&str] = &[
        "body { color: red; }\n",
        ".foo { margin: 10px; }\n",
        "#bar { background: #ff0000; }\n",
        "a:hover { text-decoration: underline; }\n",
        "@media (min-width: 600px) { .c { color: red; } }\n",
        ".parent { color: red; .child { color: blue; } }\n",
        "/* comment */\n.c { color: red; }\n",
    ];
    for &input in cases {
        let out = hl(input).highlight();
        assert_eq!(
            strip_ansi(&out),
            input,
            "round-trip mismatch for: {:?}",
            input
        );
    }
}

#[test]
fn property_uses_property_color() {
    let out = hl("a { color: red; }\n").highlight();
    let idx = out.find("color").expect("`color` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("property")),
        "expected property color before `color`, got tail {:?}",
        preceding
    );
}

#[test]
fn tag_selector_uses_type_color() {
    let out = hl("body { color: red; }\n").highlight();
    let idx = out.find("body").expect("`body` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("type")),
        "expected type color before `body`, got tail {:?}",
        preceding
    );
}

#[test]
fn class_selector_uses_property_color() {
    let out = hl(".foo { color: red; }\n").highlight();
    assert!(
        out.contains(theme::color_for("property")),
        "expected property color for class selector in: {:?}",
        out
    );
}

#[test]
fn id_selector_uses_constant_color() {
    let out = hl("#foo { color: red; }\n").highlight();
    assert!(
        out.contains(theme::color_for("constant")),
        "expected constant color for id selector in: {:?}",
        out
    );
}

#[test]
fn hex_color_uses_constant_color() {
    let out = hl("a { color: #ff0000; }\n").highlight();
    assert!(
        out.contains(theme::color_for("constant")),
        "expected constant color for hex in: {:?}",
        out
    );
}

#[test]
fn number_with_unit_uses_number_color() {
    let out = hl("a { margin: 10px; }\n").highlight();
    assert!(
        out.contains(theme::color_for("number")),
        "expected number color in: {:?}",
        out
    );
}

#[test]
fn at_rule_uses_keyword_color() {
    let out = hl("@media (min-width: 600px) { .c { color: red; } }\n").highlight();
    let idx = out.find("@media").expect("`@media` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("keyword")),
        "expected keyword color before `@media`, got tail {:?}",
        preceding
    );
}

#[test]
fn function_call_uses_function_color() {
    let out = hl("a { color: rgb(255, 0, 0); }\n").highlight();
    let idx = out.find("rgb").expect("`rgb` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("function")),
        "expected function color before `rgb`, got tail {:?}",
        preceding
    );
}

#[test]
fn pseudo_class_uses_function_color() {
    let out = hl("a:hover { color: red; }\n").highlight();
    assert!(
        out.contains(theme::color_for("function")),
        "expected function color for pseudo-class in: {:?}",
        out
    );
}

#[test]
fn string_uses_string_color() {
    let out = hl("a::before { content: \"x\"; }\n").highlight();
    assert!(
        out.contains(theme::color_for("string")),
        "expected string color in: {:?}",
        out
    );
}

#[test]
fn comment_uses_comment_color() {
    let out = hl("/* block */\na { color: red; }\n").highlight();
    assert!(
        out.contains(theme::color_for("comment")),
        "expected comment color in: {:?}",
        out
    );
}

#[test]
fn important_uses_keyword_color() {
    let out = hl("a { color: red !important; }\n").highlight();
    let idx = out.find("!important").expect("`!important` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("keyword")),
        "expected keyword color before `!important`, got tail {:?}",
        preceding
    );
}

#[test]
fn recovery_renders_malformed_region_plain() {
    let input = ".a { color: red; }\n@@@ garbage @@@\n.b { color: blue; }\n";
    let out = hl(input).highlight();
    assert_eq!(strip_ansi(&out), input);
    assert!(out.contains(theme::color_for("property")));
}

#[test]
fn partial_match_on_truncated_input() {
    let input = ".incomplete { color: red\n";
    let out = hl(input).highlight();
    assert_eq!(strip_ansi(&out), input);
}
