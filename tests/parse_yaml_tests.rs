//! Smoke tests for the pruned YAML grammar: representative documents
//! parse to completion and key bytes carry the expected capture kind.
//! Indentation-structure assertions live in `tests/grammar_yaml_tests.rs`.

#[path = "common/mod.rs"]
mod common;

use common::{is_complete, kind_at, matched_len};

const GRAMMAR: &str = include_str!("../grammars/yaml.peg");

fn complete(input: &str) {
    assert!(
        is_complete(GRAMMAR, input),
        "expected a full parse; matched {} of {} bytes:\n{input}",
        matched_len(GRAMMAR, input),
        input.len()
    );
}

#[test]
fn flat_mapping_parses() {
    complete("name: example\nversion: 1.0\nenabled: true\n");
}

#[test]
fn nested_mapping_and_sequence_parse() {
    complete(
        "metadata:\n\
        \x20 name: app\n\
        \x20 labels:\n\
        \x20   tier: backend\n\
        items:\n\
        \x20 - first\n\
        \x20 - second\n",
    );
}

#[test]
fn flow_collections_parse() {
    complete("nums: [1, 2, 3]\npoint: {x: 1, y: 2}\nnested: [[a, b], [c, d]]\n");
}

#[test]
fn document_markers_and_comments_parse() {
    complete(
        "--- # a document\n\
        # leading comment\n\
        key: value  # trailing\n\
        ...\n",
    );
}

#[test]
fn quoted_scalars_parse() {
    complete("single: 'a quoted value'\ndouble: \"with \\\"escape\\\"\"\n");
}

#[test]
fn capture_kinds_on_a_simple_entry() {
    let input = "name: example\n";
    assert_eq!(kind_at(GRAMMAR, input, 0).as_deref(), Some("property")); // name
    assert_eq!(kind_at(GRAMMAR, input, 4).as_deref(), Some("punctuation")); // :
    let v = input.find("example").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, v).as_deref(), Some("string"));
}

#[test]
fn multiline_flow_sequence_parses() {
    complete("items: [\n  alpha,\n  beta,\n  gamma,\n]\n");
}
