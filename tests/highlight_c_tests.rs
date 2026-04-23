use syntax_highlighter::highlight::{theme, Highlighter};

#[path = "common/mod.rs"]
mod common;
use common::strip_ansi;

const C_GRAMMAR: &str = include_str!("../grammars/c.peg");

fn hl(input: &str) -> Highlighter {
    let mut h = Highlighter::new(C_GRAMMAR).expect("C grammar should compile");
    h.set_input(input.to_string());
    h
}

#[test]
fn round_trip_strips_to_input() {
    let cases: &[&str] = &[
        "int main(void) { return 0; }\n",
        "#include <stdio.h>\nint main(void) { return 0; }\n",
        "struct Point { int x; int y; };\n",
        "enum C { A, B, C };\n",
        "int add(int a, int b) { return a + b; }\n",
        "// line\n/* block */\nint x = 1;\n",
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
    let out = hl("int main(void) { return 0; }\n").highlight();
    assert!(
        out.contains(theme::color_for("keyword")),
        "expected keyword color in: {:?}",
        out
    );
}

#[test]
fn predef_type_uses_type_color() {
    let out = hl("int x;\n").highlight();
    let idx = out.find("int").expect("`int` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("type")),
        "expected type color before `int`, got tail {:?}",
        preceding
    );
}

#[test]
fn string_uses_string_color() {
    let out = hl("const char *s = \"hello\";\n").highlight();
    assert!(
        out.contains(theme::color_for("string")),
        "expected string color in: {:?}",
        out
    );
}

#[test]
fn number_uses_number_color() {
    let out = hl("int x = 42;\n").highlight();
    assert!(
        out.contains(theme::color_for("number")),
        "expected number color in: {:?}",
        out
    );
}

#[test]
fn preprocessor_uses_comment_color() {
    let out = hl("#include <stdio.h>\nint x;\n").highlight();
    let idx = out.find("#include").expect("pp must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("comment")),
        "expected comment color before #include, got tail {:?}",
        preceding
    );
}

#[test]
fn line_comment_uses_comment_color() {
    let out = hl("// a line\nint x;\n").highlight();
    assert!(
        out.contains(theme::color_for("comment")),
        "expected comment color in: {:?}",
        out
    );
}

#[test]
fn block_comment_uses_comment_color() {
    let out = hl("/* block */\nint x;\n").highlight();
    assert!(
        out.contains(theme::color_for("comment")),
        "expected comment color in: {:?}",
        out
    );
}

#[test]
fn function_name_uses_function_color() {
    let out = hl("int compute(int n) { return n; }\n").highlight();
    let idx = out.find("compute").expect("`compute` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("function")),
        "expected function color before `compute`, got tail {:?}",
        preceding
    );
}

#[test]
fn struct_name_uses_type_color() {
    let out = hl("struct Point { int x; };\n").highlight();
    let idx = out.find("Point").expect("`Point` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("type")),
        "expected type color before `Point`, got tail {:?}",
        preceding
    );
}

#[test]
fn enum_items_use_constant_color() {
    let out = hl("enum C { A, B };\n").highlight();
    assert!(
        out.contains(theme::color_for("constant")),
        "expected constant color for enum items: {:?}",
        out
    );
}

#[test]
fn recovery_renders_malformed_region_plain() {
    let input = "int a(void) { return 1; }\n@@@ garbage @@@\nint b(void) { return 2; }\n";
    let out = hl(input).highlight();
    assert_eq!(strip_ansi(&out), input);
    assert!(out.contains(theme::color_for("keyword")));
}

#[test]
fn partial_match_on_truncated_input() {
    let input = "int incomplete(void) { int x = 1\n";
    let out = hl(input).highlight();
    assert_eq!(strip_ansi(&out), input);
}
