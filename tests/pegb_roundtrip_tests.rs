//! Cross-grammar round-trip checks for `pegb::encode` / `pegb::decode`.
//!
//! For each bundled grammar this exercises:
//! 1. Compile source → `Program`.
//! 2. Encode → bytes, decode → second `Program`, assert structural
//!    equality (`code`, `capture_kinds`, `rule_count`).
//! 3. Run the same canned input through (a) `VM` directly on both
//!    Programs and (b) `Parser::from_program` on the decoded one,
//!    assert all three observe the same captures + completion. Confirms
//!    the wire format preserves runtime behavior end-to-end through both
//!    the low-level VM and the deep-module `Parser` surface.
//! 4. Determinism: encoding twice yields byte-equal output.
//!
//! Sqlite is the largest bundled grammar (5 K+ instructions, 426
//! rules) and exercises the largest varint widths and the bigger end
//! of the instruction stream.

use syntax_highlighter::parser::Parser;
use syntax_highlighter::pegvm::VM;
use syntax_highlighter::{pegb, pegc};

fn assert_roundtrip(grammar: &str, sample: &[u8], tag: &str) {
    let p1 = pegc::compile(grammar).expect("grammar compiles");

    let bytes = pegb::encode(&p1);
    let p2 = pegb::decode(&bytes).expect("decode succeeds");

    assert_eq!(p1.code, p2.code, "{tag}: code differs after round-trip");
    assert_eq!(
        p1.capture_kinds, p2.capture_kinds,
        "{tag}: capture_kinds differ after round-trip"
    );
    assert_eq!(
        p1.label_kinds, p2.label_kinds,
        "{tag}: label_kinds differ after round-trip"
    );
    assert_eq!(
        p1.rule_count, p2.rule_count,
        "{tag}: rule_count differs after round-trip (encoder vs derived-on-decode)"
    );

    // Determinism: encoding twice produces the same bytes.
    assert_eq!(
        bytes,
        pegb::encode(&p1),
        "{tag}: encoding is not deterministic"
    );

    // End-to-end via VM: running both Programs against the same input
    // must produce identical match results.
    let r1 = VM::new(&p1.code, sample).run();
    let r2 = VM::new(&p2.code, sample).run();
    assert_eq!(r1.matched, r2.matched, "{tag}: matched differs");
    assert_eq!(r1.complete, r2.complete, "{tag}: complete flag differs");
    assert_eq!(r1.captures, r2.captures, "{tag}: captures differ");

    // End-to-end via Parser::from_program: a Parser built from the
    // decoded Program must agree with one built from grammar source on
    // captures and completion.
    let mut from_decoded = Parser::from_program(p2);
    let mut from_source = Parser::new(grammar).unwrap();
    from_decoded.set_input(sample.to_vec());
    from_source.set_input(sample.to_vec());
    assert_eq!(
        from_decoded.captures(),
        from_source.captures(),
        "{tag}: Parser::from_program captures diverge from Parser::new"
    );
    assert_eq!(
        from_decoded.is_complete(),
        from_source.is_complete(),
        "{tag}: Parser::from_program completion diverges from Parser::new"
    );
}

fn assert_grammar(path_label: &str, grammar: &str, sample: &[u8]) {
    assert_roundtrip(grammar, sample, path_label);
}

#[test]
fn json_roundtrips() {
    assert_grammar(
        "json",
        include_str!("../grammars/json.peg"),
        br#"{"name": "ada", "age": 36, "tags": ["pioneer", null]}"#,
    );
}

#[test]
fn toml_roundtrips() {
    assert_grammar(
        "toml",
        include_str!("../grammars/toml.peg"),
        b"[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    );
}

#[test]
fn css_roundtrips() {
    assert_grammar(
        "css",
        include_str!("../grammars/css.peg"),
        b".btn { color: #333; padding: 4px 8px; }\n",
    );
}

#[test]
fn c_roundtrips() {
    assert_grammar(
        "c",
        include_str!("../grammars/c.peg"),
        b"int main(void) { return 0; }\n",
    );
}

#[test]
fn go_roundtrips() {
    assert_grammar(
        "go",
        include_str!("../grammars/go.peg"),
        b"package main\nfunc main() { println(\"hi\") }\n",
    );
}

#[test]
fn javascript_roundtrips() {
    assert_grammar(
        "javascript",
        include_str!("../grammars/javascript.peg"),
        b"const x = 42; function f() { return x + 1; }\n",
    );
}

#[test]
fn rust_roundtrips() {
    assert_grammar(
        "rust",
        include_str!("../grammars/rust.peg"),
        b"fn main() { println!(\"hi\"); }\n",
    );
}

#[test]
fn sqlite_roundtrips() {
    assert_grammar(
        "sqlite",
        include_str!("../grammars/sqlite.peg"),
        b"SELECT id, name FROM users WHERE id = 42;\n",
    );
}

#[test]
fn labeled_catch_program_round_trips() {
    // Exercises `RecoverScopeBegin(LabelId)` plus the label-name
    // table. Input drives the catch through both success and recovery
    // branches in case any future encoder change depends on runtime
    // state.
    assert_grammar("labeled_catch", "root <- 'a' ^lbl 'b'", b"ab");
}
