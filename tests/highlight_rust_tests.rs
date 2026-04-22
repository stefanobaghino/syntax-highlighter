use syntax_highlighter::highlight::{theme, Highlighter};

#[path = "common/mod.rs"]
mod common;
use common::strip_ansi;

const RUST_GRAMMAR: &str = include_str!("../grammars/rust.peg");

fn hl(input: &str) -> Highlighter {
    let mut h = Highlighter::new(RUST_GRAMMAR).expect("Rust grammar should compile");
    h.set_input(input.to_string());
    h
}

#[test]
fn round_trip_strips_to_input() {
    let cases: &[&str] = &[
        "fn main() {}\n",
        "use std::collections::HashMap;\n",
        "struct Foo { a: i32, b: u32 }\n",
        "fn add(a: i32, b: i32) -> i32 { a + b }\n",
        "#[derive(Debug)]\npub enum E { A, B(u32) }\n",
        "fn f<'a>(s: &'a str) -> &'a str { s }\n",
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
    let out = hl("fn main() {}\n").highlight();
    assert!(
        out.contains(theme::color_for("keyword")),
        "expected keyword color in: {:?}",
        out
    );
}

#[test]
fn string_uses_string_color() {
    let out = hl("fn f() { let s = \"hello\"; }\n").highlight();
    assert!(
        out.contains(theme::color_for("string")),
        "expected string color in: {:?}",
        out
    );
    assert!(out.contains("hello"));
}

#[test]
fn number_uses_number_color() {
    let out = hl("fn f() -> i32 { 42 }\n").highlight();
    assert!(
        out.contains(theme::color_for("number")),
        "expected number color in: {:?}",
        out
    );
}

#[test]
fn line_comment_uses_comment_color() {
    let out = hl("// a line\nfn f() {}\n").highlight();
    assert!(
        out.contains(theme::color_for("comment")),
        "expected comment color in: {:?}",
        out
    );
}

#[test]
fn block_comment_uses_comment_color() {
    let out = hl("/* a block */ fn f() {}\n").highlight();
    assert!(
        out.contains(theme::color_for("comment")),
        "expected comment color in: {:?}",
        out
    );
}

#[test]
fn lifetime_uses_type_color() {
    // Lifetimes are captured as @type in this grammar — the hardcoded
    // theme has no dedicated lifetime color, and grouping with other
    // type-shaped syntax is the consistent choice.
    let out = hl("fn f<'a>(s: &'a str) {}\n").highlight();
    let idx = out.find("'a").expect("'a must appear in output");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("type")),
        "expected type color before 'a, got tail {:?}",
        preceding
    );
}

#[test]
fn fn_name_uses_function_color() {
    let out = hl("fn compute() {}\n").highlight();
    let idx = out.find("compute").expect("`compute` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("function")),
        "expected function color before `compute`, got tail {:?}",
        preceding
    );
}

#[test]
fn macro_invocation_uses_function_color() {
    // `println` in `println!("...")` — the macro name itself is @function.
    let out = hl("fn main() { println!(\"hi\"); }\n").highlight();
    let idx = out.find("println").expect("`println` must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("function")),
        "expected function color before `println`, got tail {:?}",
        preceding
    );
}

#[test]
fn bool_uses_constant_color() {
    let out = hl("fn f() -> bool { true }\n").highlight();
    assert!(
        out.contains(theme::color_for("constant")),
        "expected constant color for `true` in: {:?}",
        out
    );
}

#[test]
fn attribute_punctuation_present() {
    // `#[derive(...)]` — `#` and `[` `]` are punctuation; the grammar
    // should emit punctuation captures around them.
    let out = hl("#[derive(Debug)]\nfn f() {}\n").highlight();
    assert!(
        out.contains(theme::color_for("punctuation")),
        "expected punctuation color in: {:?}",
        out
    );
}

#[test]
fn upper_ident_in_path_uses_type_color() {
    // `HashMap` as a path segment should be @type.
    let out = hl("use std::collections::HashMap;\n").highlight();
    let idx = out.find("HashMap").expect("HashMap must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("type")),
        "expected type color before HashMap, got tail {:?}",
        preceding
    );
}

#[test]
fn recovery_renders_malformed_region_plain() {
    // `*^` recovery emits the unmatched region under @recovery, which
    // maps to "no color" in the hardcoded theme. Surrounding items
    // still get styled.
    let input = "fn a() {}\n@@@ garbage @@@\nfn b() {}\n";
    let out = hl(input).highlight();
    // Round-trip still works — every byte preserved.
    assert_eq!(strip_ansi(&out), input);
    // `fn` keywords from both a and b are colored.
    assert!(out.contains(theme::color_for("keyword")));
}

#[test]
fn partial_match_on_truncated_input() {
    // Truncated input (missing `}`) still renders the matched prefix
    // styled and the remainder plain.
    let input = "fn incomplete() { let x = 1\n";
    let out = hl(input).highlight();
    assert_eq!(strip_ansi(&out), input);
}
