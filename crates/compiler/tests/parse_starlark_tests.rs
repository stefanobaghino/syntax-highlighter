//! Smoke tests for the Starlark subset grammar: representative snippets
//! parse to completion and a few bytes carry the expected capture kind.
//! Deeper indentation-structure assertions live in
//! `tests/grammar_starlark_tests.rs`.

#[path = "common/mod.rs"]
mod common;

use common::{is_complete, kind_at, matched_len};

const GRAMMAR: &str = include_str!("../../../grammars/starlark.peg");

fn complete(input: &str) {
    assert!(
        is_complete(GRAMMAR, input),
        "expected a full parse; matched {} of {} bytes:\n{input}",
        matched_len(GRAMMAR, input),
        input.len()
    );
}

#[test]
fn def_with_nested_if_else_parses() {
    complete(
        "def greet(name, greeting = \"hi\"):\n\
        \x20   if name:\n\
        \x20       return greeting + name\n\
        \x20   else:\n\
        \x20       return \"nobody\"\n",
    );
}

#[test]
fn module_level_assignments_and_loop() {
    complete(
        "x = [1, 2, 3]\n\
        y = {\"a\": 1, \"b\": 2}\n\
        for i in x:\n\
        \x20   print(i)\n",
    );
}

#[test]
fn multiline_call_spans_lines_inside_brackets() {
    complete(
        "result = func(\n\
        \x20   first,\n\
        \x20   second = 2,\n\
        \x20   *rest,\n\
        )\n",
    );
}

#[test]
fn comments_and_blank_lines_between_statements() {
    complete(
        "# leading comment\n\
        \n\
        a = 1  # trailing\n\
        \n\
        # gap\n\
        b = 2\n",
    );
}

#[test]
fn keyword_and_string_and_number_capture_kinds() {
    let input = "def f():\n    return 42\n";
    assert_eq!(kind_at(GRAMMAR, input, 0).as_deref(), Some("keyword")); // `def`
    let n = input.find("42").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, n).as_deref(), Some("number"));
    let r = input.find("return").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, r).as_deref(), Some("keyword"));
}

#[test]
fn triple_quoted_string_is_one_string_span() {
    let input = "doc = \"\"\"line one\nline two\"\"\"\n";
    complete(input);
    let q = input.find("\"\"\"").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, q + 4).as_deref(), Some("string"));
}
