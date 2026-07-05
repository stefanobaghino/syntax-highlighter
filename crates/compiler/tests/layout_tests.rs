//! Tests for the layout-preserving emission substrate: the
//! [`RuleLayout`] tree that [`parse`] retains and the splicing
//! emitter in `pegc::emit`.
//!
//! The load-bearing property is the fidelity gate: re-emitting any
//! parsed grammar with no edits must reproduce the source
//! byte-for-byte — comments, blank lines, and spelling included.

use std::collections::HashMap;

use syntax_highlighter_compiler::pegc::{
    emit, emit_with_edits, parse, EmitError, Grammar, LayoutEdit,
};

fn parse_ok(src: &str) -> Grammar {
    parse(src).expect("parse failed")
}

/// Emit `src` unedited and assert byte identity.
fn assert_round_trip(src: &str) {
    let g = parse_ok(src);
    let layout = g.layout.as_ref().expect("parsed grammar has a layout");
    assert_eq!(emit(src, layout), src, "identity round-trip failed");
}

#[test]
fn round_trip_identity_for_all_shipped_grammars() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../grammars");
    let mut count = 0usize;
    for entry in std::fs::read_dir(dir).expect("grammars/ directory") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("peg") {
            continue;
        }
        let src = std::fs::read_to_string(&path).expect("readable grammar");
        let g = parse(&src).unwrap_or_else(|e| panic!("{} fails to parse: {e}", path.display()));
        let layout = g.layout.as_ref().expect("parsed grammar has a layout");
        assert_eq!(
            emit(&src, layout),
            src,
            "identity round-trip failed for {}",
            path.display()
        );
        count += 1;
    }
    assert!(
        count >= 10,
        "expected a real corpus, found {count} grammars"
    );
}

#[test]
fn round_trip_nested_blocks_with_comments_and_blank_lines() {
    assert_round_trip(
        "# leading file comment\n\
         root = a b {\n\
             \n\
             # group: terminals\n\
             a = 'x' {\n\
                 inner = 'i'   # odd spacing kept\n\
             }\n\
             \n\
             b = a / inner\n\
             # comment before the closing brace\n\
         }\n\
         # trailing file comment\n",
    );
}

#[test]
fn round_trip_ascriptions_with_odd_spacing() {
    assert_round_trip(
        "root = kw {\n\
             kw :  reserved  = 'if' / 'else'\n\
             op:atomic= '+'\n\
         }\n",
    );
}

#[test]
fn round_trip_same_line_trailing_comment() {
    assert_round_trip("root = 'x' # trailing note\n");
}

#[test]
fn round_trip_empty_block() {
    assert_round_trip("root = 'x' { }\n");
}

#[test]
fn round_trip_no_trailing_newline() {
    assert_round_trip("root = 'x'");
}

#[test]
fn round_trip_comment_between_body_and_block() {
    assert_round_trip("root = a # which a?\n{\n    a = 'x'\n}\n");
}

#[test]
fn replace_body_touches_only_the_body_range() {
    let src = "root = a b {\n    a = 'x'  # keep me\n    b = 'y'\n}\n";
    let g = parse_ok(src);
    let layout = g.layout.as_ref().unwrap();
    let out = emit_with_edits(
        src,
        layout,
        &[LayoutEdit::ReplaceBody {
            rule: "a".to_string(),
            text: "'x' / 'z'".to_string(),
        }],
    )
    .expect("edit applies");
    assert_eq!(
        out,
        "root = a b {\n    a = 'x' / 'z'  # keep me\n    b = 'y'\n}\n"
    );

    // The result reparses, the edited rule has the new structure, and
    // every untouched rule is structurally unchanged.
    let new_g = parse_ok(&out);
    let strip = |g: &Grammar| -> HashMap<String, _> {
        g.rules
            .iter()
            .map(|(k, v)| (k.clone(), v.clone().strip_spans()))
            .collect()
    };
    let old_rules = strip(&g);
    let new_rules = strip(&new_g);
    assert_ne!(old_rules["a"], new_rules["a"], "edited rule must change");
    assert_eq!(old_rules["root"], new_rules["root"]);
    assert_eq!(old_rules["b"], new_rules["b"]);
}

#[test]
fn replace_body_of_nested_rule_by_flat_name() {
    let src = "root = a {\n    a = inner {\n        inner = 'i'\n    }\n}\n";
    let g = parse_ok(src);
    let out = emit_with_edits(
        src,
        g.layout.as_ref().unwrap(),
        &[LayoutEdit::ReplaceBody {
            rule: "a::inner".to_string(),
            text: "'j'".to_string(),
        }],
    )
    .expect("edit applies");
    assert_eq!(
        out,
        "root = a {\n    a = inner {\n        inner = 'j'\n    }\n}\n"
    );
}

/// The interplay with the body-range watermark: a comment trailing the
/// edited body belongs to the gap, not the body, so a body edit must
/// leave it standing.
#[test]
fn replace_body_preserves_trailing_comment() {
    let src = "root = a {\n    a = 'x'\n    # section divider\n}\n";
    let g = parse_ok(src);
    let out = emit_with_edits(
        src,
        g.layout.as_ref().unwrap(),
        &[LayoutEdit::ReplaceBody {
            rule: "a".to_string(),
            text: "'y'".to_string(),
        }],
    )
    .expect("edit applies");
    assert_eq!(out, "root = a {\n    a = 'y'\n    # section divider\n}\n");
}

#[test]
fn replace_body_unknown_rule_errors() {
    let src = "root = 'x'\n";
    let g = parse_ok(src);
    let err = emit_with_edits(
        src,
        g.layout.as_ref().unwrap(),
        &[LayoutEdit::ReplaceBody {
            rule: "nope".to_string(),
            text: "'y'".to_string(),
        }],
    )
    .expect_err("unknown rule must error");
    match err {
        EmitError::UnknownRule(name) => assert_eq!(name, "nope"),
    }
}

#[test]
fn duplicate_edits_last_wins() {
    let src = "root = 'x'\n";
    let g = parse_ok(src);
    let out = emit_with_edits(
        src,
        g.layout.as_ref().unwrap(),
        &[
            LayoutEdit::ReplaceBody {
                rule: "root".to_string(),
                text: "'first'".to_string(),
            },
            LayoutEdit::ReplaceBody {
                rule: "root".to_string(),
                text: "'second'".to_string(),
            },
        ],
    )
    .expect("edits apply");
    assert_eq!(out, "root = 'second'\n");
}
