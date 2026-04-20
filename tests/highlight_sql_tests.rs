use syntax_highlighter::highlight::{theme, Highlighter};

#[path = "common/mod.rs"]
mod common;
use common::strip_ansi;

const SQLITE_GRAMMAR: &str = include_str!("../grammars/sqlite.peg");

fn hl() -> Highlighter {
    Highlighter::new(SQLITE_GRAMMAR).expect("SQLite grammar should compile")
}

#[test]
fn parses_minimal_select() {
    let h = hl();
    let input = "SELECT 1;";
    let (matched, caps) = h.captures(input);
    assert_eq!(matched, input.len());
    assert!(!caps.is_empty());
}

#[test]
fn highlight_keyword_uses_keyword_color() {
    let h = hl();
    let out = h.highlight("SELECT 1");
    let kw = theme::color_for("keyword");
    let idx = out.find("SELECT").expect("SELECT must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(kw),
        "expected keyword color before SELECT, got tail {:?}",
        preceding
    );
}

#[test]
fn highlight_string_uses_string_color() {
    let h = hl();
    let out = h.highlight("SELECT 'hi' FROM t");
    assert!(
        out.contains(theme::color_for("string")),
        "expected string color, got {:?}",
        out
    );
}

#[test]
fn highlight_number_uses_number_color() {
    let h = hl();
    let out = h.highlight("SELECT 42 FROM t");
    assert!(
        out.contains(theme::color_for("number")),
        "expected number color, got {:?}",
        out
    );
}

#[test]
fn highlight_null_uses_constant_color() {
    let h = hl();
    let out = h.highlight("SELECT NULL");
    let c = theme::color_for("constant");
    let idx = out.find("NULL").expect("NULL must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(c),
        "expected constant color before NULL, got tail {:?}",
        preceding
    );
}

#[test]
fn highlight_comment_uses_comment_color() {
    let h = hl();
    let out = h.highlight("-- note\nSELECT 1");
    assert!(
        out.contains(theme::color_for("comment")),
        "expected comment color, got {:?}",
        out
    );
}

#[test]
fn highlight_function_uses_function_color() {
    let h = hl();
    let out = h.highlight("SELECT COUNT(*) FROM t");
    let f = theme::color_for("function");
    let idx = out.find("COUNT").expect("COUNT must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(f),
        "expected function color before COUNT, got tail {:?}",
        preceding
    );
}

#[test]
fn highlight_table_in_from_uses_type_color() {
    let h = hl();
    let out = h.highlight("SELECT c FROM users");
    let t = theme::color_for("type");
    let idx = out.find("users").expect("table name must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(t),
        "expected type color before table name, got tail {:?}",
        preceding
    );
}

#[test]
fn highlight_cast_target_uses_type_color() {
    let h = hl();
    let out = h.highlight("SELECT CAST(x AS INTEGER) FROM t");
    let t = theme::color_for("type");
    let idx = out.find("INTEGER").expect("INTEGER must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(t),
        "expected type color before CAST target, got tail {:?}",
        preceding
    );
}

#[test]
fn highlight_operator_uses_operator_color() {
    let h = hl();
    let out = h.highlight("SELECT a FROM t WHERE a = 1");
    assert!(
        out.contains(theme::color_for("operator")),
        "expected operator color, got {:?}",
        out
    );
}

#[test]
fn highlight_bind_param_uses_variable_color() {
    let h = hl();
    let out = h.highlight("SELECT :name FROM t");
    let v = theme::color_for("variable");
    let idx = out.find(":name").expect(":name must appear");
    let preceding = &out[..idx];
    assert!(
        preceding.ends_with(v),
        "expected variable color before :name, got tail {:?}",
        preceding
    );
}

#[test]
fn highlighting_preserves_input_text() {
    // Stripping ANSI codes must yield the original input byte-for-byte.
    let h = hl();
    let input = "\
-- A representative query.
WITH recent AS (SELECT id FROM orders)
SELECT u.id, COUNT(*) AS n
FROM users u
LEFT JOIN recent r ON u.id = r.id
WHERE u.active AND u.score > 0.5
GROUP BY u.id
HAVING n > 0
ORDER BY n DESC
LIMIT 10;

SELECT 1 UNION ALL SELECT 2;
";
    let out = h.highlight(input);
    let stripped = strip_ansi(&out);
    assert_eq!(stripped, input);
}

#[test]
fn partial_match_unterminated_string_still_round_trips() {
    let h = hl();
    let input = "SELECT 'oops\nFROM t";
    let out = h.highlight(input);
    assert_eq!(strip_ansi(&out), input);
}

#[test]
fn partial_match_unclosed_paren_still_round_trips() {
    let h = hl();
    let input = "SELECT COUNT( FROM t";
    let out = h.highlight(input);
    assert_eq!(strip_ansi(&out), input);
}

#[test]
fn partial_match_renders_prefix_styled_and_tail_plain() {
    // The valid SELECT prefix should carry styling; trailing garbage renders
    // verbatim. This mirrors the TOML partial-match test.
    let h = hl();
    let input = "SELECT 1; !!garbage";
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
