use syntax_highlighter::highlight::{theme, Highlighter};

#[path = "common/mod.rs"]
mod common;
use common::strip_ansi;

const GO_GRAMMAR: &str = include_str!("../grammars/go.peg");

fn hl(input: &str) -> Highlighter {
    let mut h = Highlighter::new(GO_GRAMMAR).expect("Go grammar should compile");
    h.set_input(input.to_string());
    h
}

#[test]
fn round_trip_strips_to_input() {
    let cases: &[&str] = &[
        "package main\n",
        "package main\n\nimport \"fmt\"\n",
        "package p\n\nfunc f(x int) int { return x + 1 }\n",
        "package p\n\ntype T struct { X int }\n",
        "package p\n\nfunc f() { xs := []int{1, 2, 3}; _ = xs }\n",
        "package p\n\nfunc f() {\n\tfor i, v := range xs {\n\t\t_ = i\n\t\t_ = v\n\t}\n}\n",
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
    let out = hl("package main\n").highlight();
    assert!(
        out.contains(theme::color_for("keyword")),
        "expected keyword color in: {:?}",
        out
    );
}

#[test]
fn string_uses_string_color() {
    let out = hl("package p\n\nvar s = \"hello\"\n").highlight();
    assert!(
        out.contains(theme::color_for("string")),
        "expected string color in: {:?}",
        out
    );
}

#[test]
fn raw_string_uses_string_color() {
    let out = hl("package p\n\nvar s = `raw`\n").highlight();
    assert!(
        out.contains(theme::color_for("string")),
        "expected string color for raw string in: {:?}",
        out
    );
}

#[test]
fn number_uses_number_color() {
    let out = hl("package p\n\nvar n = 42\n").highlight();
    assert!(
        out.contains(theme::color_for("number")),
        "expected number color in: {:?}",
        out
    );
}

#[test]
fn line_comment_uses_comment_color() {
    let out = hl("package p\n// a line\nfunc f() {}\n").highlight();
    assert!(
        out.contains(theme::color_for("comment")),
        "expected comment color in: {:?}",
        out
    );
}

#[test]
fn block_comment_uses_comment_color() {
    let out = hl("package p\n/* a block */\nfunc f() {}\n").highlight();
    assert!(
        out.contains(theme::color_for("comment")),
        "expected comment color in: {:?}",
        out
    );
}

#[test]
fn fn_name_uses_function_color() {
    let out = hl("package p\n\nfunc compute() {}\n").highlight();
    let idx = out.find("compute").expect("`compute` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("function")),
        "expected function color before `compute`, got tail {:?}",
        preceding
    );
}

#[test]
fn type_name_uses_type_color() {
    let out = hl("package p\n\ntype Point struct {}\n").highlight();
    let idx = out.find("Point").expect("`Point` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("type")),
        "expected type color before `Point`, got tail {:?}",
        preceding
    );
}

#[test]
fn predeclared_type_uses_type_color() {
    let out = hl("package p\n\nvar x int = 1\n").highlight();
    let idx = out.find("int").expect("`int` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("type")),
        "expected type color before `int`, got tail {:?}",
        preceding
    );
}

#[test]
fn nil_uses_constant_color() {
    let out = hl("package p\n\nvar x = nil\n").highlight();
    assert!(
        out.contains(theme::color_for("constant")),
        "expected constant color for nil in: {:?}",
        out
    );
}

#[test]
fn assign_operator_present() {
    let out = hl("package p\n\nfunc f() { x := 1; _ = x }\n").highlight();
    assert!(
        out.contains(theme::color_for("operator")),
        "expected operator color (:=) in: {:?}",
        out
    );
}

#[test]
fn recovery_renders_malformed_region_plain() {
    let input = "package p\n\nfunc a() {}\n@@@ garbage @@@\nfunc b() {}\n";
    let out = hl(input).highlight();
    assert_eq!(strip_ansi(&out), input);
    assert!(out.contains(theme::color_for("keyword")));
}

#[test]
fn partial_match_on_truncated_input() {
    let input = "package p\n\nfunc incomplete() { x := 1\n";
    let out = hl(input).highlight();
    assert_eq!(strip_ansi(&out), input);
}
