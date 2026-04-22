use syntax_highlighter::highlight::{theme, Highlighter};

#[path = "common/mod.rs"]
mod common;
use common::strip_ansi;

const JS_GRAMMAR: &str = include_str!("../grammars/javascript.peg");

fn hl(input: &str) -> Highlighter {
    let mut h = Highlighter::new(JS_GRAMMAR).expect("JavaScript grammar should compile");
    h.set_input(input.to_string());
    h
}

#[test]
fn round_trip_strips_to_input() {
    let cases: &[&str] = &[
        "function main() {}\n",
        "const x = 42;\n",
        "import { a, b as c } from 'mod';\n",
        "const add = (a, b) => a + b;\n",
        "class Foo extends Bar { constructor(x) { super(); this.x = x; } }\n",
        "const s = `hello ${name}`;\n",
        "async function f() { const x = await g(); return x; }\n",
        "for (const x of arr) { log(x); }\n",
        "// comment\nconst x = 1;\n",
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
fn keyword_uses_keyword_color() {
    let out = hl("function main() {}\n").highlight();
    assert!(
        out.contains(theme::color_for("keyword")),
        "expected keyword color in: {:?}",
        out
    );
}

#[test]
fn string_uses_string_color() {
    let out = hl("const s = \"hello\";\n").highlight();
    assert!(
        out.contains(theme::color_for("string")),
        "expected string color in: {:?}",
        out
    );
    assert!(out.contains("hello"));
}

#[test]
fn template_literal_uses_string_color() {
    let out = hl("const s = `hello ${name}`;\n").highlight();
    assert!(
        out.contains(theme::color_for("string")),
        "expected string color for template: {:?}",
        out
    );
}

#[test]
fn number_uses_number_color() {
    let out = hl("const x = 42;\n").highlight();
    assert!(
        out.contains(theme::color_for("number")),
        "expected number color in: {:?}",
        out
    );
}

#[test]
fn line_comment_uses_comment_color() {
    let out = hl("// a line\nconst x = 1;\n").highlight();
    assert!(
        out.contains(theme::color_for("comment")),
        "expected comment color in: {:?}",
        out
    );
}

#[test]
fn block_comment_uses_comment_color() {
    let out = hl("/* a block */ const x = 1;\n").highlight();
    assert!(
        out.contains(theme::color_for("comment")),
        "expected comment color in: {:?}",
        out
    );
}

#[test]
fn fn_name_uses_function_color() {
    let out = hl("function compute() {}\n").highlight();
    let idx = out.find("compute").expect("`compute` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("function")),
        "expected function color before `compute`, got tail {:?}",
        preceding
    );
}

#[test]
fn class_name_uses_type_color() {
    let out = hl("class Foo {}\n").highlight();
    let idx = out.find("Foo").expect("`Foo` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("type")),
        "expected type color before `Foo`, got tail {:?}",
        preceding
    );
}

#[test]
fn new_target_uses_type_color() {
    let out = hl("const x = new Foo(1);\n").highlight();
    let idx = out.find("Foo").expect("`Foo` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("type")),
        "expected type color before `Foo` after new, got tail {:?}",
        preceding
    );
}

#[test]
fn bool_and_null_use_constant_color() {
    let out = hl("const x = true; const y = null;\n").highlight();
    assert!(
        out.contains(theme::color_for("constant")),
        "expected constant color in: {:?}",
        out
    );
}

#[test]
fn arrow_operator_punctuation_present() {
    let out = hl("const f = x => x;\n").highlight();
    assert!(
        out.contains(theme::color_for("operator")),
        "expected operator color (=>) in: {:?}",
        out
    );
}

#[test]
fn call_uses_function_color() {
    let out = hl("compute(1, 2);\n").highlight();
    let idx = out.find("compute").expect("`compute` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("function")),
        "expected function color before call target, got tail {:?}",
        preceding
    );
}

#[test]
fn recovery_renders_malformed_region_plain() {
    let input = "function a() {}\n@@@ garbage @@@\nfunction b() {}\n";
    let out = hl(input).highlight();
    assert_eq!(strip_ansi(&out), input);
    assert!(out.contains(theme::color_for("keyword")));
}

#[test]
fn partial_match_on_truncated_input() {
    // Missing closing brace — matched prefix should still render with
    // styling and the remainder plain.
    let input = "function incomplete() { const x = 1\n";
    let out = hl(input).highlight();
    assert_eq!(strip_ansi(&out), input);
}
