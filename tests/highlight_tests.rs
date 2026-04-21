use syntax_highlighter::highlight::{theme, Highlighter};

#[path = "common/mod.rs"]
mod common;
use common::strip_ansi;

const JSON_GRAMMAR: &str = include_str!("../grammars/json.peg");

fn hl(input: &str) -> Highlighter {
    let mut h = Highlighter::new(JSON_GRAMMAR).expect("JSON grammar should compile");
    h.set_input(input.to_string());
    h
}

#[test]
fn parses_minimal_object() {
    let h = hl("{}");
    let (matched, caps) = h.captures();
    assert_eq!(matched, 2);
    assert!(
        !caps.is_empty(),
        "expected at least the punctuation captures"
    );
}

#[test]
fn parses_minimal_array() {
    let h = hl("[]");
    let (matched, _) = h.captures();
    assert_eq!(matched, 2);
}

#[test]
fn parses_string_value() {
    let h = hl(r#""hello""#);
    let (matched, _) = h.captures();
    assert_eq!(matched, 7);
}

#[test]
fn parses_number_value() {
    assert_eq!(hl("42").captures().0, 2);
    assert_eq!(hl("-3.14").captures().0, 5);
    assert_eq!(hl("1e10").captures().0, 4);
    assert_eq!(hl("-0.5e-3").captures().0, 7);
}

#[test]
fn parses_constants() {
    assert_eq!(hl("true").captures().0, 4);
    assert_eq!(hl("false").captures().0, 5);
    assert_eq!(hl("null").captures().0, 4);
}

#[test]
fn parses_nested_object() {
    let input = r#"{"a": {"b": [1, 2, true]}}"#;
    let h = hl(input);
    let (matched, caps) = h.captures();
    assert_eq!(matched, input.len());
    assert!(caps.len() > 5);
}

#[test]
fn parses_string_with_escapes() {
    let input = r#""he said \"hi\"""#;
    let h = hl(input);
    let (matched, _) = h.captures();
    assert_eq!(matched, input.len());
}

#[test]
fn parses_string_with_unicode_escape() {
    let input = r#""\u00e9""#;
    assert_eq!(hl(input).captures().0, input.len());
}

#[test]
fn partial_unterminated_string_advances_past_quote() {
    // On malformed input the VM no longer gives up at sp=0; it surfaces the
    // farthest position reached. The quote itself is consumed before the
    // string body starts failing.
    let input = r#""oops"#;
    let h = hl(input);
    let (matched, _) = h.captures();
    assert!(matched > 0, "expected some progress, got {}", matched);
    assert!(matched <= input.len());
}

#[test]
fn partial_trailing_garbage_reports_max_sp() {
    // The json rule ends with `!.`; trailing garbage fails the top-level
    // parse. The VM now reports the farthest position reached (the end of
    // the valid JSON prefix) rather than zero.
    let input = r#"{"a": 1} extra"#;
    let h = hl(input);
    let (matched, _) = h.captures();
    assert!(matched > 0, "expected progress past the valid prefix");
    assert!(matched <= input.len());
}

#[test]
fn highlight_string_uses_green() {
    let out = hl(r#""hi""#).highlight();
    assert!(
        out.contains(theme::color_for("string")),
        "expected string color in {:?}",
        out
    );
    assert!(out.contains("hi"));
    assert!(out.contains(theme::RESET));
}

#[test]
fn highlight_number_uses_yellow() {
    let out = hl("42").highlight();
    assert!(out.contains(theme::color_for("number")), "got {:?}", out);
    assert!(out.contains("42"));
}

#[test]
fn highlight_constant_uses_magenta() {
    let out = hl("true").highlight();
    assert!(out.contains(theme::color_for("constant")), "got {:?}", out);
    assert!(out.contains("true"));
}

#[test]
fn highlight_property_uses_cyan_not_string_green() {
    // In `pair`, the key is wrapped in @property{string} — it should render as a
    // property (cyan), not as a string (green).
    let out = hl(r#"{"name": "value"}"#).highlight();
    let prop = theme::color_for("property");
    let str_color = theme::color_for("string");
    // The key "name" should appear under property color
    let idx_name = out.find("\"name\"").expect("key text not present");
    let preceding = &out[..idx_name];
    assert!(
        preceding.ends_with(prop),
        "expected property color before key, got tail {:?}",
        preceding
    );
    // The value "value" should appear under string color
    let idx_val = out.find("\"value\"").expect("value text not present");
    let preceding_val = &out[..idx_val];
    assert!(
        preceding_val.ends_with(str_color),
        "expected string color before value, got tail {:?}",
        preceding_val
    );
}

#[test]
fn highlighting_preserves_input_text() {
    // Stripping ANSI codes should yield the original input.
    let input = r#"{"key": [1, true, "hello"], "nested": {"a": null}}"#;
    let out = hl(input).highlight();
    let stripped = strip_ansi(&out);
    assert_eq!(stripped, input);
}

#[test]
fn partial_match_renders_prefix_styled_and_tail_plain() {
    // The valid JSON prefix must be styled; the trailing garbage must appear
    // verbatim in the output (no ANSI codes wrapping it). The round-trip
    // invariant must hold for the whole output.
    let input = r#"{"a": 1} extra"#;
    let out = hl(input).highlight();
    assert_eq!(strip_ansi(&out), input, "round-trip must be preserved");
    assert!(
        out.contains(theme::color_for("number")) || out.contains(theme::color_for("string")),
        "expected some styling on the valid prefix, got {:?}",
        out
    );
    // The literal " extra" substring must appear without any ANSI codes
    // interleaved between its bytes.
    assert!(
        out.contains(" extra"),
        "trailing garbage must render verbatim, got {:?}",
        out
    );
}

#[test]
fn unterminated_string_still_round_trips() {
    // The load-bearing strip_ansi invariant must hold even when the input
    // is malformed — partial-match rendering must not reorder, drop, or
    // substitute input bytes.
    let input = r#"{"a": "oops"#;
    let out = hl(input).highlight();
    assert_eq!(strip_ansi(&out), input);
}
