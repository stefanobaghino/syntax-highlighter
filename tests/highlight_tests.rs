use syntax_highlighter::highlight::{theme, Highlighter};

const JSON_GRAMMAR: &str = include_str!("../grammars/json.peg");

fn hl() -> Highlighter {
    Highlighter::new(JSON_GRAMMAR).expect("JSON grammar should compile")
}

#[test]
fn parses_minimal_object() {
    let h = hl();
    let (matched, caps) = h.captures("{}");
    assert_eq!(matched, 2);
    assert!(
        !caps.is_empty(),
        "expected at least the punctuation captures"
    );
}

#[test]
fn parses_minimal_array() {
    let h = hl();
    let (matched, _) = h.captures("[]");
    assert_eq!(matched, 2);
}

#[test]
fn parses_string_value() {
    let h = hl();
    let (matched, _) = h.captures(r#""hello""#);
    assert_eq!(matched, 7);
}

#[test]
fn parses_number_value() {
    let h = hl();
    assert_eq!(h.captures("42").0, 2);
    assert_eq!(h.captures("-3.14").0, 5);
    assert_eq!(h.captures("1e10").0, 4);
    assert_eq!(h.captures("-0.5e-3").0, 7);
}

#[test]
fn parses_constants() {
    let h = hl();
    assert_eq!(h.captures("true").0, 4);
    assert_eq!(h.captures("false").0, 5);
    assert_eq!(h.captures("null").0, 4);
}

#[test]
fn parses_nested_object() {
    let h = hl();
    let input = r#"{"a": {"b": [1, 2, true]}}"#;
    let (matched, caps) = h.captures(input);
    assert_eq!(matched, input.len());
    assert!(caps.len() > 5);
}

#[test]
fn parses_string_with_escapes() {
    let h = hl();
    let input = r#""he said \"hi\"""#;
    let (matched, _) = h.captures(input);
    assert_eq!(matched, input.len());
}

#[test]
fn parses_string_with_unicode_escape() {
    let h = hl();
    let input = r#""\u00e9""#;
    assert_eq!(h.captures(input).0, input.len());
}

#[test]
fn partial_unterminated_string_advances_past_quote() {
    // On malformed input the VM no longer gives up at sp=0; it surfaces the
    // farthest position reached. The quote itself is consumed before the
    // string body starts failing.
    let h = hl();
    let input = r#""oops"#;
    let (matched, _) = h.captures(input);
    assert!(matched > 0, "expected some progress, got {}", matched);
    assert!(matched <= input.len());
}

#[test]
fn partial_trailing_garbage_reports_max_sp() {
    // The json rule ends with `!.`; trailing garbage fails the top-level
    // parse. The VM now reports the farthest position reached (the end of
    // the valid JSON prefix) rather than zero.
    let h = hl();
    let input = r#"{"a": 1} extra"#;
    let (matched, _) = h.captures(input);
    assert!(matched > 0, "expected progress past the valid prefix");
    assert!(matched <= input.len());
}

#[test]
fn highlight_string_uses_green() {
    let h = hl();
    let out = h.highlight(r#""hi""#);
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
    let h = hl();
    let out = h.highlight("42");
    assert!(out.contains(theme::color_for("number")), "got {:?}", out);
    assert!(out.contains("42"));
}

#[test]
fn highlight_constant_uses_magenta() {
    let h = hl();
    let out = h.highlight("true");
    assert!(out.contains(theme::color_for("constant")), "got {:?}", out);
    assert!(out.contains("true"));
}

#[test]
fn highlight_property_uses_cyan_not_string_green() {
    // In `pair`, the key is wrapped in @property{string} — it should render as a
    // property (cyan), not as a string (green).
    let h = hl();
    let out = h.highlight(r#"{"name": "value"}"#);
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
    let h = hl();
    let input = r#"{"key": [1, true, "hello"], "nested": {"a": null}}"#;
    let out = h.highlight(input);
    let stripped = strip_ansi(&out);
    assert_eq!(stripped, input);
}

fn strip_ansi(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() && !bytes[i].is_ascii_alphabetic() {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).unwrap()
}
