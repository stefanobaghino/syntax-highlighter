//! End-to-end pegb round-trip + truncation cases that need a real
//! compiled `Program` for their fixture. The pegb in-source unit
//! tests cover the varint helpers, error-path tags, and hand-built
//! Programs; this file complements them with a real-grammar surface
//! exercised through `pegc::compile`.

use syntax_highlighter::pegb;
use syntax_highlighter::pegvm::Program;
use syntax_highlighter_compiler::pegc;

fn assert_roundtrip(p: &Program) {
    let bytes = pegb::encode(p);
    let p2 = pegb::decode(&bytes).expect("decode succeeds on freshly-encoded bytes");
    assert_eq!(p.code, p2.code, "code mismatch after round-trip");
    assert_eq!(
        p.capture_kinds, p2.capture_kinds,
        "capture_kinds mismatch after round-trip"
    );
    assert_eq!(
        p.rule_names, p2.rule_names,
        "rule_names mismatch after round-trip"
    );
    assert_eq!(
        p.label_kinds, p2.label_kinds,
        "label_kinds mismatch after round-trip"
    );
    assert_eq!(
        p.rule_count, p2.rule_count,
        "rule_count mismatch after round-trip (encoder vs derived-on-decode)"
    );
    assert_eq!(bytes, pegb::encode(p), "encoding is deterministic");
}

fn json_program_bytes() -> Vec<u8> {
    let p = pegc::compile(include_str!("../../../grammars/json.peg")).unwrap();
    pegb::encode(&p)
}

#[test]
fn smallest_program_round_trips() {
    // Trivial one-rule grammar: `start = "x"`. Compiles to a
    // bootstrap `Call` + `End` plus the rule body wrapped in
    // `RuleEnter`/`MemoClose`/`Return`.
    let p = pegc::compile("root = \"x\"").unwrap();
    assert_roundtrip(&p);
}

#[test]
fn truncated_in_instruction_payload_errors() {
    let mut bytes = json_program_bytes();
    bytes.pop(); // drop the last byte of the last instruction
    let err = pegb::decode(&bytes).unwrap_err();
    assert!(matches!(
        err,
        pegb::Error::TruncatedInput { .. } | pegb::Error::MalformedVarint { .. }
    ));
}

#[test]
fn trailing_bytes_errors() {
    let mut bytes = json_program_bytes();
    bytes.push(0x42);
    let err = pegb::decode(&bytes).unwrap_err();
    assert!(matches!(err, pegb::Error::TrailingBytes { remaining: 1 }));
}
