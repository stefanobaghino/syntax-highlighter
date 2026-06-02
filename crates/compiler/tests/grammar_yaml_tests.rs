//! Indentation-structure tests for the pruned YAML grammar: that
//! `deeper` (nested value past its key) and `same` (aligned mapping
//! entries / sequence items) actually enforce block shape, and that
//! misaligned or wrongly-indented lines make the parse fail. The
//! behavioral payoff of the parameterized indentation rules for the
//! relative-indent (YAML) class.

#[path = "common/mod.rs"]
mod common;

use common::{is_complete, kind_at, matched_len};

const GRAMMAR: &str = include_str!("../../../grammars/yaml.peg");

fn complete(input: &str) -> bool {
    is_complete(GRAMMAR, input)
}

#[test]
fn nested_value_must_be_deeper_than_its_key() {
    let input = "outer:\n  inner: 1\n";
    assert!(complete(input), "matched {}", matched_len(GRAMMAR, input));
    let inner = input.find("inner").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, inner).as_deref(), Some("property"));
}

#[test]
fn sibling_entries_must_align() {
    let input = "a: 1\nb: 2\nc: 3\n";
    assert!(complete(input), "matched {}", matched_len(GRAMMAR, input));
}

#[test]
fn deeply_nested_mapping_parses() {
    let input = "l1:\n  l2:\n    l3:\n      leaf: x\n";
    assert!(complete(input), "matched {}", matched_len(GRAMMAR, input));
}

#[test]
fn dedent_returns_to_outer_mapping() {
    // `sibling` at column 0 parses only if the nested block under `parent`
    // is correctly closed by the dedent.
    let input = "parent:\n  child: 1\nsibling: 2\n";
    assert!(complete(input), "matched {}", matched_len(GRAMMAR, input));
    let sib = input.find("sibling").unwrap();
    assert_eq!(kind_at(GRAMMAR, input, sib).as_deref(), Some("property"));
}

#[test]
fn sequence_items_align_under_a_key() {
    let input = "list:\n  - a\n  - b\n  - c\n";
    assert!(complete(input), "matched {}", matched_len(GRAMMAR, input));
}

#[test]
fn misaligned_mapping_entry_makes_parse_incomplete() {
    // Three nested entries at three *different* columns: the second and
    // third align with no open mapping level.
    let input = "outer:\n  a: 1\n   b: 2\n";
    assert!(
        !complete(input),
        "misaligned nested entries must not parse: matched {} of {}",
        matched_len(GRAMMAR, input),
        input.len()
    );
}

#[test]
fn nested_value_not_deeper_makes_parse_incomplete() {
    // `child` is at the same column as `parent`, so it cannot be
    // `parent`'s nested value; `parent:` has a null value and `child`
    // becomes a sibling — fine — but a value that is *less* indented than
    // its key has nowhere to attach.
    let input = "top:\n  a: 1\n b: 2\n";
    assert!(
        !complete(input),
        "a half-dedented entry must not parse: matched {} of {}",
        matched_len(GRAMMAR, input),
        input.len()
    );
}

#[test]
fn flow_sequence_may_span_lines() {
    let input = "items: [\n  1,\n  2,\n]\nnext: done\n";
    assert!(complete(input), "matched {}", matched_len(GRAMMAR, input));
}
