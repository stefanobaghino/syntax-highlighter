use syntax_highlighter::highlight::{theme, Highlighter};

#[path = "common/mod.rs"]
mod common;
use common::strip_ansi;

const SQLITE_GRAMMAR: &str = include_str!("../grammars/sqlite.peg");

fn hl(input: &str) -> Highlighter {
    let mut h = Highlighter::new(SQLITE_GRAMMAR).expect("SQLite grammar should compile");
    h.set_input(input.to_string());
    h
}

#[test]
fn parses_minimal_select() {
    let input = "SELECT 1;";
    let h = hl(input);
    let (matched, caps) = h.captures();
    assert_eq!(matched, input.len());
    assert!(!caps.is_empty());
}

#[test]
fn highlight_keyword_uses_keyword_color() {
    let out = hl("SELECT 1").highlight();
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
    let out = hl("SELECT 'hi' FROM t").highlight();
    assert!(
        out.contains(theme::color_for("string")),
        "expected string color, got {:?}",
        out
    );
}

#[test]
fn highlight_number_uses_number_color() {
    let out = hl("SELECT 42 FROM t").highlight();
    assert!(
        out.contains(theme::color_for("number")),
        "expected number color, got {:?}",
        out
    );
}

#[test]
fn highlight_null_uses_constant_color() {
    let out = hl("SELECT NULL").highlight();
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
    let out = hl("-- note\nSELECT 1").highlight();
    assert!(
        out.contains(theme::color_for("comment")),
        "expected comment color, got {:?}",
        out
    );
}

#[test]
fn highlight_function_uses_function_color() {
    let out = hl("SELECT COUNT(*) FROM t").highlight();
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
    let out = hl("SELECT c FROM users").highlight();
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
    let out = hl("SELECT CAST(x AS INTEGER) FROM t").highlight();
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
    let out = hl("SELECT a FROM t WHERE a = 1").highlight();
    assert!(
        out.contains(theme::color_for("operator")),
        "expected operator color, got {:?}",
        out
    );
}

#[test]
fn highlight_bind_param_uses_variable_color() {
    let out = hl("SELECT :name FROM t").highlight();
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
    let out = hl(input).highlight();
    let stripped = strip_ansi(&out);
    assert_eq!(stripped, input);
}

#[test]
fn partial_match_unterminated_string_still_round_trips() {
    let input = "SELECT 'oops\nFROM t";
    let out = hl(input).highlight();
    assert_eq!(strip_ansi(&out), input);
}

#[test]
fn partial_match_unclosed_paren_still_round_trips() {
    let input = "SELECT COUNT( FROM t";
    let out = hl(input).highlight();
    assert_eq!(strip_ansi(&out), input);
}

#[test]
fn partial_match_renders_prefix_styled_and_tail_plain() {
    // The valid SELECT prefix should carry styling; trailing garbage renders
    // verbatim. This mirrors the TOML partial-match test.
    let input = "SELECT 1; !!garbage";
    let out = hl(input).highlight();
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

#[test]
fn large_fixture_parses_to_completion() {
    // Round-trip on the memo-bench fixture: every byte must parse.
    // This is the end-to-end guard that catches false-reject regressions
    // (e.g. reserving an identifier that real SQL uses as a column name).
    let input = include_str!("../benches/fixtures/large.sql");
    let mut h = Highlighter::new(SQLITE_GRAMMAR).expect("grammar compiles");
    h.set_input(input.to_string());
    let (matched, _caps) = h.captures();
    assert_eq!(
        matched,
        input.len(),
        "expected full parse of benches/fixtures/large.sql; stopped at byte {} before {:?}",
        matched,
        &input[matched..matched.saturating_add(80).min(input.len())]
    );
    assert_eq!(strip_ansi(&h.highlight()), input, "round-trip must hold");
}

#[test]
fn multi_statement_error_recovery() {
    // The full SQLite grammar exercises farthest-failure tracking across
    // a `;`-separated script: DDL and DML preceding a malformed chunk
    // should stay styled; the tail from `!!!` onward renders plain.
    let input = "CREATE TABLE t (a INTEGER);\n\
                 INSERT INTO t VALUES (1);\n\
                 !!! oops\n\
                 SELECT * FROM t;";
    let out = hl(input).highlight();
    assert_eq!(strip_ansi(&out), input, "round-trip must hold");
    assert!(
        out.contains(theme::color_for("keyword")),
        "expected the CREATE/INSERT prefix to carry keyword styling"
    );
    assert!(
        out.contains("!!! oops"),
        "invalid middle chunk must render verbatim"
    );
    // The trailing SELECT is past the farthest-failure point, so it
    // renders plain rather than styled.
    let tail_idx = out
        .find("!!! oops")
        .expect("garbage marker must appear in output");
    let tail = &out[tail_idx..];
    assert!(
        !tail.contains(theme::color_for("keyword")),
        "tail past the error should render without styling, got {tail:?}"
    );
}
