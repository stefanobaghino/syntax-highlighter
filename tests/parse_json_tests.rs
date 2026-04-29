//! Per-grammar parse tests for JSON.
//!
//! Asserts kind-at-byte via [`common::kind_at`] over `walk`'s segment
//! stream. Renderer correctness (ANSI emission) is tested separately
//! in the demo binary.

#[path = "common/mod.rs"]
mod common;

use common::{is_complete, kind_at, matched_len, segments};

const GRAMMAR: &str = include_str!("../grammars/json.peg");

#[test]
fn property_key_is_property() {
    let input = r#"{"name": "alice"}"#;
    let key = input.find("\"name\"").unwrap();
    assert_eq!(
        kind_at(GRAMMAR, input, key).as_deref(),
        Some("property"),
        "property key should be kinded `property`"
    );
}

#[test]
fn string_value_is_string() {
    let input = r#"{"name": "alice"}"#;
    let val = input.rfind("\"alice\"").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, val).as_deref(), Some("string"));
}

#[test]
fn number_is_number() {
    let input = r#"{"x": 42}"#;
    let n = input.find("42").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, n).as_deref(), Some("number"));
}

#[test]
fn constants_are_constant() {
    for lit in ["true", "false", "null"] {
        let input = format!("{{\"x\": {lit}}}");
        let pos = input.find(lit).unwrap();
        assert_eq!(
            kind_at(GRAMMAR, &input, pos).as_deref(),
            Some("constant"),
            "{lit} should be `constant`"
        );
    }
}

#[test]
fn punctuation_is_punctuation() {
    let input = r#"{"x": 42}"#;
    let brace = input.find('{').unwrap();
    assert_eq!(
        kind_at(GRAMMAR, input, brace).as_deref(),
        Some("punctuation")
    );
}

#[test]
fn full_match_is_complete() {
    let input = r#"{"a": 1, "b": [true, null]}"#;
    assert!(is_complete(GRAMMAR, input));
    assert_eq!(matched_len(GRAMMAR, input), input.len());
}

#[test]
fn truncated_input_yields_partial_match() {
    let input = r#"{"a": 1"#;
    assert!(!is_complete(GRAMMAR, input));
}

#[test]
fn segments_tile_the_input() {
    let input = r#"{"a": 1}"#;
    let segs = segments(GRAMMAR, input);
    let mut cursor = 0;
    for (r, _) in &segs {
        assert_eq!(r.start, cursor);
        cursor = r.end;
    }
    assert_eq!(cursor, input.len());
}
