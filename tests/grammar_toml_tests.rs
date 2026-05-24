use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use syntax_highlighter::pegc;
use syntax_highlighter::pegvm::{Capture, Program, VM};

const TOML_GRAMMAR: &str = include_str!("../grammars/toml.peg");

// Compile the grammar exactly once per test process. The init lock's
// poisoning is what concentrates a broken-grammar failure into one
// informative panic plus N-1 trivial "previously failed" panics across
// the rest of the file's tests.
static TOML_PROGRAM: OnceLock<Program> = OnceLock::new();
static TOML_INIT: Mutex<()> = Mutex::new(());

fn toml_program() -> &'static Program {
    if let Some(p) = TOML_PROGRAM.get() {
        return p;
    }
    let _guard = TOML_INIT
        .lock()
        .expect("TOML grammar compile previously failed; see the first failure");
    TOML_PROGRAM.get_or_init(|| pegc::compile(TOML_GRAMMAR).expect("TOML grammar should compile"))
}

#[test]
fn grammar_compiles() {
    let _ = toml_program();
}

fn run(input: &str) -> (usize, Vec<Capture>, Vec<String>, bool) {
    let prog = toml_program();
    let r = VM::new_from_program(prog, input.as_bytes()).run();
    (
        r.matched,
        r.captures,
        prog.capture_kinds.clone(),
        r.complete,
    )
}

fn kinds_for<'a>(captures: &[Capture], kinds: &'a [String]) -> Vec<&'a str> {
    captures
        .iter()
        .map(|c| kinds[c.kind.0 as usize].as_str())
        .collect()
}

fn kind_spans(captures: &[Capture], kinds: &[String], kind: &str) -> Vec<String> {
    captures
        .iter()
        .filter(|c| kinds[c.kind.0 as usize] == kind)
        .map(|c| format!("{}..{}", c.start, c.end))
        .collect()
}

fn assert_complete_full(input: &str) -> (Vec<Capture>, Vec<String>) {
    let (matched, caps, kinds, complete) = run(input);
    assert!(
        complete,
        "expected complete match, got matched={} kinds={:?}",
        matched, caps
    );
    assert_eq!(matched, input.len(), "expected full-input match");
    (caps, kinds)
}

#[test]
fn grammar_parses_and_compiles() {
    let prog = toml_program();
    assert!(!prog.code.is_empty());
    assert!(!prog.capture_kinds.is_empty());
}

#[test]
fn capture_kinds_are_only_theme_kinds() {
    let prog = toml_program();
    let expected: HashSet<&str> = [
        "comment",
        "property",
        "type",
        "string",
        "number",
        "constant",
        "punctuation",
        "recovery",
    ]
    .into_iter()
    .collect();
    let actual: HashSet<&str> = prog.capture_kinds.iter().map(String::as_str).collect();
    assert_eq!(
        actual, expected,
        "TOML grammar must stay within the hardcoded-theme vocabulary"
    );
}

#[test]
fn empty_document_matches() {
    let (caps, _) = assert_complete_full("");
    assert!(caps.is_empty());
}

#[test]
fn whitespace_only_document_matches() {
    assert_complete_full("\n\n   \n");
}

#[test]
fn single_line_comment_is_captured() {
    let (caps, kinds) = assert_complete_full("# hi\n");
    assert_eq!(kinds_for(&caps, &kinds), vec!["comment"]);
}

#[test]
fn simple_pair_captures_property_punctuation_value() {
    let (caps, kinds) = assert_complete_full("key = 1\n");
    assert_eq!(
        kinds_for(&caps, &kinds),
        vec!["property", "punctuation", "number"]
    );
}

#[test]
fn dotted_key_emits_property_segments_and_dot_punctuation() {
    let (caps, kinds) = assert_complete_full("a.b.c = true\n");
    assert_eq!(
        kinds_for(&caps, &kinds),
        vec![
            "property",
            "punctuation", // .
            "property",
            "punctuation", // .
            "property",
            "punctuation", // =
            "constant"
        ]
    );
}

#[test]
fn quoted_keys_are_still_property() {
    let (caps, kinds) = assert_complete_full("\"a.b\".c = 1\n");
    let ks = kinds_for(&caps, &kinds);
    assert_eq!(
        ks,
        vec![
            "property",
            "punctuation",
            "property",
            "punctuation",
            "number"
        ]
    );
}

#[test]
fn table_header_path_is_type() {
    let (caps, kinds) = assert_complete_full("[foo.bar]\n");
    assert_eq!(
        kinds_for(&caps, &kinds),
        vec!["punctuation", "type", "punctuation", "type", "punctuation"]
    );
}

#[test]
fn array_of_tables_header_is_type() {
    let (caps, kinds) = assert_complete_full("[[servers]]\n");
    assert_eq!(
        kinds_for(&caps, &kinds),
        vec!["punctuation", "type", "punctuation"]
    );
}

#[test]
fn string_value_basic() {
    let (caps, kinds) = assert_complete_full("s = \"hi\"\n");
    assert_eq!(
        kinds_for(&caps, &kinds),
        vec!["property", "punctuation", "string"]
    );
}

#[test]
fn string_value_literal() {
    let (caps, kinds) = assert_complete_full("s = 'hi'\n");
    assert_eq!(
        kinds_for(&caps, &kinds),
        vec!["property", "punctuation", "string"]
    );
}

#[test]
fn string_multiline_basic() {
    let input = "s = \"\"\"line1\nline2\"\"\"\n";
    let (caps, kinds) = assert_complete_full(input);
    assert_eq!(
        kinds_for(&caps, &kinds),
        vec!["property", "punctuation", "string"]
    );
}

#[test]
fn string_multiline_literal() {
    let input = "s = '''line1\nline2'''\n";
    let (caps, kinds) = assert_complete_full(input);
    assert_eq!(
        kinds_for(&caps, &kinds),
        vec!["property", "punctuation", "string"]
    );
}

#[test]
fn integer_variants_all_parse() {
    for input in [
        "n = 0\n",
        "n = 42\n",
        "n = -17\n",
        "n = +99\n",
        "n = 1_000_000\n",
        "n = 0xDEAD_beef\n",
        "n = 0o755\n",
        "n = 0b1010_1010\n",
    ] {
        let (caps, kinds) = assert_complete_full(input);
        assert_eq!(
            kinds_for(&caps, &kinds),
            vec!["property", "punctuation", "number"],
            "input: {:?}",
            input
        );
    }
}

#[test]
fn float_variants_all_parse() {
    for input in [
        "f = 3.14\n",
        "f = -0.5\n",
        "f = 1e10\n",
        "f = 6.626e-34\n",
        "f = 9_224_617.445_991_228\n",
    ] {
        let (caps, kinds) = assert_complete_full(input);
        assert_eq!(
            kinds_for(&caps, &kinds),
            vec!["property", "punctuation", "number"],
            "input: {:?}",
            input
        );
    }
}

#[test]
fn inf_nan_are_constants() {
    for input in ["f = inf\n", "f = -inf\n", "f = +inf\n", "f = nan\n"] {
        let (caps, kinds) = assert_complete_full(input);
        assert_eq!(
            kinds_for(&caps, &kinds),
            vec!["property", "punctuation", "constant"],
            "input: {:?}",
            input
        );
    }
}

#[test]
fn booleans_are_constants() {
    for input in ["b = true\n", "b = false\n"] {
        let (caps, kinds) = assert_complete_full(input);
        assert_eq!(
            kinds_for(&caps, &kinds),
            vec!["property", "punctuation", "constant"],
            "input: {:?}",
            input
        );
    }
}

#[test]
fn datetime_variants_are_numbers() {
    for input in [
        "d = 1979-05-27T07:32:00Z\n",
        "d = 1979-05-27T07:32:00-07:00\n",
        "d = 1979-05-27T07:32:00.999999\n",
        "d = 1979-05-27T07:32:00\n",
        "d = 1979-05-27 07:32:00\n",
        "d = 1979-05-27\n",
        "d = 07:32:00\n",
        "d = 07:32:00.999999\n",
    ] {
        let (caps, kinds) = assert_complete_full(input);
        assert_eq!(
            kinds_for(&caps, &kinds),
            vec!["property", "punctuation", "number"],
            "input: {:?}",
            input
        );
    }
}

#[test]
fn empty_array_matches() {
    let (caps, kinds) = assert_complete_full("a = []\n");
    assert_eq!(
        kinds_for(&caps, &kinds),
        vec!["property", "punctuation", "punctuation", "punctuation"]
    );
}

#[test]
fn array_with_mixed_primitive_values() {
    let (caps, kinds) = assert_complete_full("a = [1, \"two\", true]\n");
    assert_eq!(
        kinds_for(&caps, &kinds),
        vec![
            "property",
            "punctuation", // =
            "punctuation", // [
            "number",
            "punctuation", // ,
            "string",
            "punctuation", // ,
            "constant",
            "punctuation", // ]
        ]
    );
}

#[test]
fn multiline_array_with_trailing_comma_and_interior_comment() {
    let input = "a = [\n  1,\n  # inside\n  2,\n]\n";
    let (caps, kinds) = assert_complete_full(input);
    // Order of captures follows start position. Before-`[` we have key and =
    // punctuation; after that the capture interleaving matters less than the
    // fact that each expected kind appears.
    let ks = kinds_for(&caps, &kinds);
    assert!(
        ks.contains(&"comment"),
        "expected interior comment, got {:?}",
        ks
    );
    assert_eq!(ks.iter().filter(|k| **k == "number").count(), 2);
}

#[test]
fn inline_table_value() {
    let input = "t = { a = 1, b = \"x\" }\n";
    let (caps, kinds) = assert_complete_full(input);
    let ks = kinds_for(&caps, &kinds);
    assert!(ks.contains(&"string"));
    assert!(ks.contains(&"number"));
    // The outer key "t" and the inner keys "a", "b" are all @property.
    assert_eq!(ks.iter().filter(|k| **k == "property").count(), 3);
}

#[test]
fn trailing_comment_after_value() {
    let (caps, kinds) = assert_complete_full("k = 1 # trailing\n");
    let ks = kinds_for(&caps, &kinds);
    assert!(ks.contains(&"comment"));
}

#[test]
fn realistic_cargo_toml_excerpt() {
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
    let (caps, kinds) = assert_complete_full(input);
    let ks: HashSet<&str> = kinds_for(&caps, &kinds).into_iter().collect();
    for expected in [
        "property",
        "type",
        "string",
        "number",
        "constant",
        "punctuation",
        "comment",
    ] {
        assert!(ks.contains(expected), "missing kind {}", expected);
    }
    // Section headers for [package], [dependencies], [[bin]] produce `type`
    // captures; verify we have at least 3.
    let type_spans = kind_spans(&caps, &kinds, "type");
    assert!(
        type_spans.len() >= 3,
        "expected >= 3 type captures for section headers, got {:?}",
        type_spans
    );
}
