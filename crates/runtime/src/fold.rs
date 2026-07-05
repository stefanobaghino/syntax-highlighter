//! Capture-forest fold: reduce the capture stream bottom-up into
//! caller-typed values.
//!
//! [`fold_captures`] is the typed sibling of [`walk`](crate::walk):
//! where `walk` flattens the capture stream into a segment tiling for
//! renderers, the fold hands the *nesting* to the caller — one closure
//! call per capture, children before parents, producing an arbitrary
//! caller-defined value per node (an AST, a CST, a count, a dump).
//! The library never sees the value type; extension is entirely at
//! the call site.
//!
//! Relies on the producer contract documented on
//! [`Capture`](crate::pegvm::Capture): the stream is a properly-nested
//! forest emitted in `CaptureBegin` order — `start`-ascending, parent
//! before child — i.e. a pre-order forest traversal. That holds for
//! full, partial, and recovery-materialized parses alike, so the fold
//! works unchanged on all of them.
//!
//! Semantics:
//!
//! - The closure runs in post-order: a capture's children fold before
//!   it does, and `children` holds their values in source order
//!   (ascending start position).
//! - The return value is the forest's root values, in source order.
//! - Zero-length captures close before any same-position successor
//!   opens (the same `end <= start` tie-break as `walk`), so they are
//!   always leaves; two zero-length captures at one position are
//!   siblings in emission order.
//! - Captures sharing an `end` fold innermost-first, so an inner
//!   value lands in its outer's `children`.
//! - Uncaptured gaps are not represented: the closure sees only
//!   captured children, and a parent's range covers any gap between
//!   them. Callers that need gap or leaf text slice their own input
//!   by the provided range.
//!
//! Pure-structural, like `walk`: no grammar, theme, or rendering
//! concepts, and no dependence on the input bytes.

use crate::pegvm::Capture;
use std::ops::Range;

/// One still-open capture during the sweep: the capture's index plus
/// the already-folded values of its direct children.
struct Frame<T> {
    cap_idx: usize,
    children: Vec<T>,
}

/// Fold `captures` bottom-up into caller-typed values.
///
/// Calls `f(kind, range, children)` once per capture — children
/// before parents, `children` in source order — and returns the
/// folded values of the forest's roots, in source order. See the
/// module docs for the exact tie-break semantics.
///
/// `capture_kinds` resolves each capture's kind index to its name;
/// every `Capture::kind` must index into it (holds for any stream a
/// [`Program`](crate::pegvm::Program) produced alongside its own
/// `capture_kinds` table).
///
/// Runs in `O(N)` with an explicit stack — deep nesting cannot
/// overflow the call stack. The properly-nested-forest precondition
/// is checked with `debug_assert!`; on a malformed hand-built stream
/// release builds produce an unspecified (but safe) tree shape.
pub fn fold_captures<T, F>(captures: &[Capture], capture_kinds: &[String], mut f: F) -> Vec<T>
where
    F: FnMut(&str, Range<usize>, Vec<T>) -> T,
{
    let mut stack: Vec<Frame<T>> = Vec::new();
    let mut roots: Vec<T> = Vec::new();

    for (i, c) in captures.iter().enumerate() {
        debug_assert!(c.start <= c.end, "capture with start > end: {c:?}");
        debug_assert!(
            i == 0 || captures[i - 1].start <= c.start,
            "captures not start-ascending: pre-order contract violated at index {i}"
        );

        // Close every open frame that ended at or before `c.start` —
        // the same tie-break as `walk`, so a zero-length capture
        // closes before a same-position successor opens.
        while stack
            .last()
            .is_some_and(|fr| captures[fr.cap_idx].end <= c.start)
        {
            close_top(&mut stack, &mut roots, captures, capture_kinds, &mut f);
        }

        // Whatever remains open must properly contain `c`.
        debug_assert!(
            stack.last().map_or(true, |fr| {
                let p = &captures[fr.cap_idx];
                p.start <= c.start && c.end <= p.end
            }),
            "capture stream is not a properly nested forest at index {i}"
        );

        stack.push(Frame {
            cap_idx: i,
            children: Vec::new(),
        });
    }

    // Final drain: innermost first, so shared-end inners fold into
    // their outers.
    while !stack.is_empty() {
        close_top(&mut stack, &mut roots, captures, capture_kinds, &mut f);
    }

    roots
}

/// Pop the top frame, fold it, and hand the value to its parent's
/// `children` (or to `roots` if the stack emptied).
fn close_top<T, F>(
    stack: &mut Vec<Frame<T>>,
    roots: &mut Vec<T>,
    captures: &[Capture],
    capture_kinds: &[String],
    f: &mut F,
) where
    F: FnMut(&str, Range<usize>, Vec<T>) -> T,
{
    let frame = stack.pop().expect("close_top called with empty stack");
    let c = &captures[frame.cap_idx];
    let kind = capture_kinds[c.kind.0 as usize].as_str();
    let value = f(kind, c.start..c.end, frame.children);
    match stack.last_mut() {
        Some(parent) => parent.children.push(value),
        None => roots.push(value),
    }
}

#[cfg(test)]
mod tests {
    //! Hand-constructed `Capture`-sequence tests for [`fold_captures`],
    //! in the same synthetic style as `walk`'s: slices built by hand,
    //! no parser, no grammar. The property under test is structural —
    //! nesting, ordering, and invocation discipline.
    use super::fold_captures;
    use crate::pegvm::{Capture, CaptureKind};

    fn cap(kind: u16, start: usize, end: usize) -> Capture {
        Capture {
            kind: CaptureKind(kind),
            start,
            end,
        }
    }

    fn kinds(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    /// Fold to a compact s-expression per root: leaves render as
    /// `kind[start..end]`, interior nodes as `kind(child child ...)`.
    fn sexpr(captures: &[Capture], kinds: &[String]) -> Vec<String> {
        fold_captures(captures, kinds, |kind, range, children: Vec<String>| {
            if children.is_empty() {
                format!("{kind}[{}..{}]", range.start, range.end)
            } else {
                format!("{kind}({})", children.join(" "))
            }
        })
    }

    #[test]
    fn empty_stream_returns_empty_forest_without_calling_closure() {
        let mut calls = 0;
        let roots: Vec<()> = fold_captures(&[], &[], |_, _, _| {
            calls += 1;
        });
        assert!(roots.is_empty());
        assert_eq!(calls, 0);
    }

    #[test]
    fn single_capture_is_one_leaf_root() {
        let ks = kinds(&["k"]);
        let roots = sexpr(&[cap(0, 2, 5)], &ks);
        assert_eq!(roots, vec!["k[2..5]"]);
    }

    #[test]
    fn flat_siblings_become_roots_in_source_order() {
        let ks = kinds(&["a", "b", "c"]);
        let roots = sexpr(&[cap(0, 0, 2), cap(1, 3, 5), cap(2, 5, 8)], &ks);
        assert_eq!(roots, vec!["a[0..2]", "b[3..5]", "c[5..8]"]);
    }

    #[test]
    fn nested_capture_folds_into_parent_children() {
        // Same shape as walk's nesting test: outer 0..10, inner 3..7.
        let ks = kinds(&["outer", "inner"]);
        let roots = sexpr(&[cap(0, 0, 10), cap(1, 3, 7)], &ks);
        assert_eq!(roots, vec!["outer(inner[3..7])"]);
    }

    #[test]
    fn shared_end_inner_is_child_not_sibling() {
        // Walk's shared-end fixture: outer 0..10, inner 5..10. The
        // final drain pops innermost first, so inner folds into
        // outer's children.
        let ks = kinds(&["outer", "inner"]);
        let roots = sexpr(&[cap(0, 0, 10), cap(1, 5, 10)], &ks);
        assert_eq!(roots, vec!["outer(inner[5..10])"]);
    }

    #[test]
    fn children_arrive_in_source_order() {
        let ks = kinds(&["p", "a", "b", "c"]);
        let roots = sexpr(
            &[cap(0, 0, 12), cap(1, 1, 3), cap(2, 4, 6), cap(3, 8, 11)],
            &ks,
        );
        assert_eq!(roots, vec!["p(a[1..3] b[4..6] c[8..11])"]);
    }

    #[test]
    fn zero_length_capture_is_leaf_inside_parent() {
        let ks = kinds(&["p", "z"]);
        let roots = sexpr(&[cap(0, 0, 5), cap(1, 2, 2)], &ks);
        assert_eq!(roots, vec!["p(z[2..2])"]);
    }

    #[test]
    fn zero_length_then_same_position_capture_are_siblings() {
        // The `end <= start` tie-break (same as walk's) closes the
        // zero-length capture before a successor starting at the same
        // position opens — siblings, not parent/child.
        let ks = kinds(&["p", "z", "k"]);
        let roots = sexpr(&[cap(0, 0, 10), cap(1, 3, 3), cap(2, 3, 6)], &ks);
        assert_eq!(roots, vec!["p(z[3..3] k[3..6])"]);
    }

    #[test]
    fn two_zero_length_at_same_position_are_siblings() {
        let ks = kinds(&["a", "b"]);
        let roots = sexpr(&[cap(0, 3, 3), cap(1, 3, 3)], &ks);
        assert_eq!(roots, vec!["a[3..3]", "b[3..3]"]);
    }

    #[test]
    fn closure_runs_in_post_order() {
        // parent 0..10 with children 1..3 and 5..7: children fold
        // before the parent, in source order.
        let ks = kinds(&["p", "a", "b"]);
        let mut seen = Vec::new();
        fold_captures(
            &[cap(0, 0, 10), cap(1, 1, 3), cap(2, 5, 7)],
            &ks,
            |kind, _, _: Vec<()>| {
                seen.push(kind.to_string());
            },
        );
        assert_eq!(seen, vec!["a", "b", "p"]);
    }

    #[test]
    fn deep_nesting_10k_does_not_recurse() {
        // 10k strictly-nested captures: cap i spans i..(20_000 - i).
        // A recursive fold would overflow the thread stack long
        // before this depth; the explicit stack must not.
        let ks = kinds(&["k"]);
        let captures: Vec<Capture> = (0..10_000).map(|i| cap(0, i, 20_000 - i)).collect();
        let roots = fold_captures(&captures, &ks, |_, _, children: Vec<usize>| {
            children.first().copied().unwrap_or(0) + 1
        });
        assert_eq!(roots, vec![10_000]);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "properly nested")]
    fn overlapping_captures_trip_the_forest_assert() {
        // 0..6 and 4..10 partially overlap — impossible from the VM,
        // and the debug_assert must say so rather than folding a
        // silently wrong tree.
        let ks = kinds(&["a", "b"]);
        let _ = sexpr(&[cap(0, 0, 6), cap(1, 4, 10)], &ks);
    }
}
