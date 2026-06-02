//! Incremental-reparse soundness for the indentation grammars.
//!
//! The arg-keyed memo only stays correct if `IndentMeasure` consumes the
//! indentation run *forward* — so the enclosing rule's `examined_max`
//! covers it and an edit inside the indentation invalidates the cached
//! entry. These tests edit whitespace at and around measured line starts
//! and assert the warm (incremental) capture stream equals a cold parse
//! of the same post-edit input. A backward indentation scan would leave a
//! stale entry and diverge here.

use syntax_highlighter_compiler::parser::Parser;

const STARLARK: &str = include_str!("../../../grammars/starlark.peg");
const YAML: &str = include_str!("../../../grammars/yaml.peg");

/// Apply `edits` (each `(start, old_end, replacement)`) through one
/// incremental `Parser`, then assert the result equals a fresh cold parse
/// of the final input.
fn assert_warm_matches_cold(grammar: &str, initial: &str, edits: &[(usize, usize, &[u8])]) {
    let mut inc = Parser::new(grammar).expect("grammar compiles");
    inc.set_input(initial.as_bytes().to_vec());
    let _ = inc.captures(); // prime the warm cache
    for (start, old_end, replacement) in edits {
        inc.edit(*start, *old_end, replacement);
    }
    let (warm_matched, warm_caps) = inc.captures();

    let mut fresh = Parser::new(grammar).expect("grammar compiles");
    fresh.set_input(inc.input().to_vec());
    let (cold_matched, cold_caps) = fresh.captures();

    assert_eq!(
        (warm_matched, warm_caps),
        (cold_matched, cold_caps),
        "warm reparse diverged from a cold parse of {:?}",
        std::str::from_utf8(inc.input()).unwrap()
    );
}

#[test]
fn starlark_deepen_body_line_indent() {
    // Add four spaces to the body line's indentation. The suite's
    // `%indent` (a strictly-deeper level) still holds, but the entry that
    // measured the old indent must be invalidated and recomputed.
    let src = "def f():\n    return 1\n";
    let at = src.find("    return").unwrap();
    assert_warm_matches_cold(STARLARK, src, &[(at, at, b"    ")]);
}

#[test]
fn starlark_break_sibling_alignment() {
    // Push the second statement one column right; it no longer aligns
    // with the first, so the parse must become incomplete — and warm must
    // agree with cold about exactly where it stops.
    let src = "def f():\n    a = 1\n    b = 2\n";
    let at = src.find("    b").unwrap();
    assert_warm_matches_cold(STARLARK, src, &[(at, at, b" ")]);
}

#[test]
fn starlark_dedent_a_line_into_alignment() {
    // Start misaligned (incomplete), then delete the offending extra
    // space so the block becomes well-formed. Exercises invalidation in
    // the repair direction too.
    let src = "def f():\n    a = 1\n     b = 2\n";
    let extra = src.find("     b").unwrap(); // points at the run of 5 spaces
    assert_warm_matches_cold(STARLARK, src, &[(extra, extra + 1, b"")]);
}

#[test]
fn starlark_insert_blank_line_before_measured_line() {
    let src = "def f():\n    a = 1\n    b = 2\n";
    let at = src.find("    b").unwrap();
    assert_warm_matches_cold(STARLARK, src, &[(at, at, b"\n")]);
}

#[test]
fn yaml_deepen_nested_entry_indent() {
    let src = "outer:\n  inner: 1\n";
    let at = src.find("  inner").unwrap();
    assert_warm_matches_cold(YAML, src, &[(at, at, b"  ")]);
}

#[test]
fn yaml_break_mapping_alignment() {
    let src = "a: 1\nb: 2\nc: 3\n";
    let at = src.find("b: 2").unwrap();
    assert_warm_matches_cold(YAML, src, &[(at, at, b"  ")]);
}

#[test]
fn yaml_edit_indent_of_sequence_item() {
    let src = "list:\n  - a\n  - b\n";
    let at = src.find("  - b").unwrap();
    assert_warm_matches_cold(YAML, src, &[(at, at, b" ")]);
}
