use syntax_highlighter::highlight::{theme, Highlighter};

#[path = "common/mod.rs"]
mod common;
use common::strip_ansi;

const TOML_GRAMMAR: &str = include_str!("../grammars/toml.peg");

fn hl() -> Highlighter {
    Highlighter::new(TOML_GRAMMAR).expect("TOML grammar should compile")
}

#[test]
fn parses_simple_pair() {
    let h = hl();
    let input = "key = 1\n";
    let (matched, caps) = h.captures(input);
    assert_eq!(matched, input.len());
    assert!(!caps.is_empty());
}

#[test]
fn highlight_string_uses_green() {
    let h = hl();
    let out = h.highlight("s = \"hello\"\n");
    assert!(
        out.contains(theme::color_for("string")),
        "expected string color, got {:?}",
        out
    );
    assert!(out.contains("hello"));
}

#[test]
fn highlight_number_uses_yellow() {
    let h = hl();
    let out = h.highlight("n = 42\n");
    assert!(
        out.contains(theme::color_for("number")),
        "expected number color, got {:?}",
        out
    );
}

#[test]
fn highlight_boolean_uses_constant() {
    let h = hl();
    let out = h.highlight("b = true\n");
    assert!(
        out.contains(theme::color_for("constant")),
        "expected constant color, got {:?}",
        out
    );
}

#[test]
fn highlight_inf_uses_constant_not_number() {
    // inf / nan are @constant in this grammar, grouped with true/false
    // rather than with 42 / 3.14. Verify the color lands as constant.
    let h = hl();
    let out = h.highlight("f = inf\n");
    // Locate where "inf" appears and check the preceding ANSI escape.
    let idx = out.find("inf").expect("inf must appear in output");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(theme::color_for("constant")),
        "expected constant color before inf, got tail {:?}",
        preceding
    );
}

#[test]
fn highlight_comment_uses_comment_color() {
    let h = hl();
    let out = h.highlight("# a comment\n");
    assert!(
        out.contains(theme::color_for("comment")),
        "expected comment color, got {:?}",
        out
    );
}

#[test]
fn highlight_key_uses_property_not_string() {
    // A bare key on the LHS of = should render as @property (cyan), not
    // as @string (green). This is the JSON-equivalent of the "property vs
    // string" distinction, adapted to TOML bare keys.
    let h = hl();
    let out = h.highlight("name = \"alice\"\n");
    let prop = theme::color_for("property");
    let str_color = theme::color_for("string");

    let idx_name = out.find("name").expect("key text not present");
    let preceding_key = &out[..idx_name];
    assert!(
        preceding_key.ends_with(prop),
        "expected property color before key, got tail {:?}",
        preceding_key
    );

    let idx_val = out.find("\"alice\"").expect("value text not present");
    let preceding_val = &out[..idx_val];
    assert!(
        preceding_val.ends_with(str_color),
        "expected string color before value, got tail {:?}",
        preceding_val
    );
}

#[test]
fn highlight_section_header_uses_type_color() {
    let h = hl();
    let out = h.highlight("[package]\n");
    let ty = theme::color_for("type");
    let idx = out.find("package").expect("section path must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(ty),
        "expected type color before section path, got tail {:?}",
        preceding
    );
}

#[test]
fn highlight_array_of_tables_header_uses_type_color() {
    let h = hl();
    let out = h.highlight("[[bin]]\n");
    let ty = theme::color_for("type");
    let idx = out.find("bin").expect("section path must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(ty),
        "expected type color before section path, got tail {:?}",
        preceding
    );
}

#[test]
fn highlighting_preserves_input_text() {
    // Stripping ANSI codes must yield the original input byte-for-byte.
    let h = hl();
    let input = r#"# A realistic snippet.
[package]
name = "demo"
version = "0.1.0"
edition = 2021
authors = ["alice", "bob"]
publish = false

[dependencies]
serde = { version = "1.0", features = ["derive"] }

[[bin]]
name = "demo"
path = "src/main.rs"
"#;
    let out = h.highlight(input);
    let stripped = strip_ansi(&out);
    assert_eq!(stripped, input);
}

#[test]
fn blank_lines_and_trailing_comments_are_preserved() {
    // Blank lines between entries and trailing end-of-line comments must
    // survive the round trip verbatim.
    let h = hl();
    let input = "\n\n# leading\n\nkey = 1 # trailing\n\n\n";
    let out = h.highlight(input);
    assert_eq!(strip_ansi(&out), input);
}

#[test]
fn partial_match_unterminated_string_still_round_trips() {
    // The load-bearing strip_ansi invariant must hold on malformed input.
    let h = hl();
    let input = "key = \"oops\n";
    let out = h.highlight(input);
    assert_eq!(strip_ansi(&out), input);
}

#[test]
fn partial_match_unclosed_table_header_still_round_trips() {
    let h = hl();
    let input = "[package\nname = \"x\"";
    let out = h.highlight(input);
    assert_eq!(strip_ansi(&out), input);
}

#[test]
fn partial_match_renders_prefix_styled_and_tail_plain() {
    // The valid prefix should carry some ANSI styling; the trailing
    // garbage should render verbatim in the output.
    let h = hl();
    let input = "key = 1\n!!garbage";
    let out = h.highlight(input);
    assert_eq!(strip_ansi(&out), input, "round-trip must hold");
    assert!(
        out.contains(theme::color_for("number")),
        "expected styled prefix, got {:?}",
        out
    );
    assert!(
        out.contains("!!garbage"),
        "trailing garbage must render verbatim, got {:?}",
        out
    );
}
