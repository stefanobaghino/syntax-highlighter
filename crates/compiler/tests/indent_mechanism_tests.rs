//! Low-level end-to-end tests for the implicit indentation operators
//! (`%root` / `%align` / `%indent`) and the column-threading IR they
//! desugar into — the `deeper` / `same` / `at_least` combinators,
//! parameterized rule calls, and the arg-keyed memo — on a deliberately
//! tiny hand-written grammar, so a failure points at the desugar / VM /
//! compiler lowering rather than at the much larger Starlark / YAML
//! grammars that also exercise this path.
//!
//! The toy language is "indented words": each line is a lowercase word,
//! and a more-indented run of lines forms that word's child block.

#[path = "common/mod.rs"]
mod common;

use common::{is_complete, kind_at, matched_len};

/// `%root` aligns the top block to its first line's column; `%indent`
/// opens a strictly-deeper child block and `%align` holds siblings at the
/// ambient column. The block rules are `atomic` so the horizontal-only
/// auto-`ignore` can't eat a line's indentation before an operator
/// measures it.
const GRAMMAR: &str = "\
root = nl_opt rootblock nl_opt {\n\
    rootblock: atomic =\n\
        %root (\n\
            @word word child?\n\
            ( hardnl %align @word word child? )*\n\
        )\n\
    childblock: atomic =\n\
        %indent (\n\
            @word word child?\n\
            ( hardnl %align @word word child? )*\n\
        )\n\
    child = hardnl childblock\n\
    word: atomic = [a-z]+\n\
    hardnl: atomic = '\\n'\n\
    nl_opt: atomic = '\\n'*\n\
}\n";

#[test]
fn flat_block_of_aligned_siblings_parses_fully() {
    let input = "a\nb\nc";
    assert!(
        is_complete(GRAMMAR, input),
        "three column-0 siblings should all parse"
    );
    assert_eq!(kind_at(GRAMMAR, input, 0).as_deref(), Some("word"));
    assert_eq!(kind_at(GRAMMAR, input, 2).as_deref(), Some("word"));
    assert_eq!(kind_at(GRAMMAR, input, 4).as_deref(), Some("word"));
}

#[test]
fn nested_deeper_block_parses_fully() {
    // `a` owns a child block [b, c] at indent 2; `d` is a column-0 sibling
    // of `a`.
    let input = "a\n  b\n  c\nd";
    assert!(
        is_complete(GRAMMAR, input),
        "nested deeper block plus a dedented sibling should fully parse: matched {}",
        matched_len(GRAMMAR, input)
    );
}

#[test]
fn three_level_nesting_parses_fully() {
    let input = "a\n  b\n    c\n  d\ne";
    assert!(
        is_complete(GRAMMAR, input),
        "matched only {} of {}",
        matched_len(GRAMMAR, input),
        input.len()
    );
}

#[test]
fn misaligned_sibling_is_not_consumed() {
    // `b` sits one column to the right of `a` but is not deep enough to be
    // a child (it would need its own `deeper` line first) — actually it is
    // deeper, so it becomes a child. The genuinely broken case is a line
    // *less* indented than the block but more than its parent with no
    // matching level: trailing bytes remain unconsumed.
    let input = "a\n   b\n  c"; // b at 3, c at 2: c aligns with neither a(0) nor b(3)
    assert!(
        !is_complete(GRAMMAR, input),
        "the dangling `c` at an unmatched indent must leave the parse incomplete"
    );
}

#[test]
fn same_indent_is_required_for_siblings() {
    // Second line is indented one space deeper than the first top-level
    // line; since the top block aligns on the first line's column (0),
    // the `  b` is a child of `a`, then `c` at column 0 is a sibling.
    let input = "a\n b\nc";
    assert!(is_complete(GRAMMAR, input), "child then dedented sibling");
}
