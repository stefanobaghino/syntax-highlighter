//! Capture-stream walker: tile input bytes with kinded segments.
//!
//! [`walk`] takes a `Parser`'s capture output and emits one [`Segment`]
//! per active-kind run between event boundaries. The full sequence
//! tiles `0..input.len()` exactly — no gaps, no overlaps — which is
//! the load-bearing structural property of the renderer pipeline.
//!
//! Pure-structural traversal: knows nothing about ANSI, themes, or
//! rendering. Consumers wrap this layer with whatever presentation
//! they need (the demo CLI in `src/bin/demo/highlight.rs` wraps
//! kinded segments in ANSI escapes).

use crate::pegvm::Capture;

/// One byte-range slice of the walker's flattened view of the input.
/// `kind` is `Some(_)` for bytes covered by a capture (the innermost
/// still-open one) and `None` for unkinded gaps — bytes between
/// captures, or the trailing run past the parser's farthest matched
/// position. The full sequence emitted by [`walk`] tiles
/// `0..input.len()` exactly.
#[derive(Clone, Debug)]
pub struct Segment<'a> {
    pub range: std::ops::Range<usize>,
    pub kind: Option<&'a str>,
}

#[derive(Clone, Copy, Debug)]
enum EventKind {
    Begin,
    End,
}

#[derive(Clone, Copy, Debug)]
struct Event {
    pos: usize,
    kind: EventKind,
    cap_idx: usize,
}

/// Walk `captures` over `input`, invoking `on_segment` for each
/// active-kind run between event boundaries. Together the segments
/// form a contiguous, non-overlapping cover of `0..input.len()` —
/// the renderer's load-bearing structural property.
///
/// Implementation: build Begin/End events, sort with the tie-breaking
/// rule that keeps the active-capture stack consistent under PEG
/// nesting (forest invariant — captures are nested or disjoint, never
/// overlapping), and emit one segment per non-empty `cursor..ev.pos`
/// slice. The active kind is the innermost open capture's kind.
pub fn walk<'a, F>(
    input: &'a str,
    captures: &'a [Capture],
    capture_kinds: &'a [String],
    mut on_segment: F,
) where
    F: FnMut(Segment<'a>),
{
    let len = input.len();
    if captures.is_empty() {
        if len > 0 {
            on_segment(Segment {
                range: 0..len,
                kind: None,
            });
        }
        return;
    }

    let mut events: Vec<Event> = Vec::with_capacity(captures.len() * 2);
    for (i, c) in captures.iter().enumerate() {
        events.push(Event {
            pos: c.start,
            kind: EventKind::Begin,
            cap_idx: i,
        });
        events.push(Event {
            pos: c.end,
            kind: EventKind::End,
            cap_idx: i,
        });
    }

    // Order events at the same position so the stack stays consistent with
    // properly nested captures:
    //   - END events come before BEGIN events (a capture that ends here closes
    //     before a sibling that starts here).
    //   - For ENDs at the same position, inner captures (higher cap_idx) end first.
    //   - For BEGINs at the same position, outer captures (lower cap_idx) start first.
    events.sort_by(|a, b| {
        a.pos.cmp(&b.pos).then_with(|| match (a.kind, b.kind) {
            (EventKind::End, EventKind::Begin) => std::cmp::Ordering::Less,
            (EventKind::Begin, EventKind::End) => std::cmp::Ordering::Greater,
            (EventKind::End, EventKind::End) => b.cap_idx.cmp(&a.cap_idx),
            (EventKind::Begin, EventKind::Begin) => a.cap_idx.cmp(&b.cap_idx),
        })
    });

    let mut stack: Vec<usize> = Vec::new(); // capture indices, innermost on top
    let mut cursor = 0usize;

    for ev in events {
        if ev.pos > cursor {
            let kind = stack
                .last()
                .map(|&i| capture_kinds[captures[i].kind.0 as usize].as_str());
            on_segment(Segment {
                range: cursor..ev.pos,
                kind,
            });
            cursor = ev.pos;
        }
        match ev.kind {
            EventKind::Begin => stack.push(ev.cap_idx),
            EventKind::End => {
                // With proper PEG nesting, the matching capture is on top of the stack.
                if let Some(top) = stack.last() {
                    if *top == ev.cap_idx {
                        stack.pop();
                    } else if let Some(pos) = stack.iter().rposition(|&i| i == ev.cap_idx) {
                        // Defensive: search and remove (shouldn't happen for valid grammars).
                        stack.remove(pos);
                    }
                }
            }
        }
    }

    // Trailing input past the last event — unkinded by definition (no
    // open capture covers it). For partial parses this is the
    // unmatched tail; for full parses it is empty.
    if cursor < len {
        on_segment(Segment {
            range: cursor..len,
            kind: None,
        });
    }
}

#[cfg(test)]
mod tests {
    //! Hand-constructed `Capture`-sequence tests for [`walk`]. The
    //! property under test is structural: emitted segment ranges form
    //! a contiguous, non-overlapping cover of `0..input.len()`. Tests
    //! pass synthetic `Capture` slices directly to `walk`, bypassing
    //! the parser entirely — this is the kind of structural assertion
    //! that doesn't depend on any specific grammar.
    use super::{walk, Segment};
    use crate::pegvm::{Capture, CaptureKind};

    fn cap(kind: u16, start: usize, end: usize) -> Capture {
        Capture {
            kind: CaptureKind(kind),
            start,
            end,
        }
    }

    fn collect(
        input: &str,
        captures: &[Capture],
        kinds: &[String],
    ) -> Vec<(usize, usize, Option<String>)> {
        let mut out = Vec::new();
        walk(input, captures, kinds, |seg: Segment<'_>| {
            out.push((
                seg.range.start,
                seg.range.end,
                seg.kind.map(|k| k.to_string()),
            ));
        });
        out
    }

    fn assert_tiles(input: &str, segs: &[(usize, usize, Option<String>)]) {
        if input.is_empty() {
            assert!(
                segs.is_empty(),
                "empty input must emit no segments, got {segs:?}"
            );
            return;
        }
        assert!(
            !segs.is_empty(),
            "non-empty input must emit at least one segment"
        );
        assert_eq!(segs[0].0, 0, "first segment must start at 0");
        for pair in segs.windows(2) {
            assert_eq!(
                pair[0].1, pair[1].0,
                "segments must be contiguous (gap between {} and {})",
                pair[0].1, pair[1].0
            );
            assert!(
                pair[0].0 < pair[0].1,
                "segment must be non-empty: {:?}",
                pair[0]
            );
        }
        assert_eq!(
            segs.last().unwrap().1,
            input.len(),
            "last segment must end at input.len()"
        );
    }

    #[test]
    fn empty_input_emits_no_segments() {
        let segs = collect("", &[], &[]);
        assert!(segs.is_empty());
    }

    #[test]
    fn no_captures_emits_one_unkinded_segment_covering_input() {
        let input = "hello";
        let segs = collect(input, &[], &[]);
        assert_tiles(input, &segs);
        assert_eq!(segs, vec![(0, 5, None)]);
    }

    #[test]
    fn single_capture_in_middle_emits_three_segments() {
        let input = "abcdef";
        let kinds = vec!["k".to_string()];
        let captures = vec![cap(0, 2, 4)];
        let segs = collect(input, &captures, &kinds);
        assert_tiles(input, &segs);
        assert_eq!(
            segs,
            vec![(0, 2, None), (2, 4, Some("k".to_string())), (4, 6, None),]
        );
    }

    #[test]
    fn capture_covering_whole_input_emits_one_kinded_segment() {
        let input = "abc";
        let kinds = vec!["k".to_string()];
        let captures = vec![cap(0, 0, 3)];
        let segs = collect(input, &captures, &kinds);
        assert_tiles(input, &segs);
        assert_eq!(segs, vec![(0, 3, Some("k".to_string()))]);
    }

    #[test]
    fn nested_captures_emit_inner_kind_inside_outer() {
        // outer 0..10 of kind "outer"; inner 3..7 of kind "inner".
        // Expected segments: 0..3 outer, 3..7 inner, 7..10 outer.
        let input = "abcdefghij";
        let kinds = vec!["outer".to_string(), "inner".to_string()];
        let captures = vec![cap(0, 0, 10), cap(1, 3, 7)];
        let segs = collect(input, &captures, &kinds);
        assert_tiles(input, &segs);
        assert_eq!(
            segs,
            vec![
                (0, 3, Some("outer".to_string())),
                (3, 7, Some("inner".to_string())),
                (7, 10, Some("outer".to_string())),
            ]
        );
    }

    #[test]
    fn nested_captures_with_shared_end_close_inner_first() {
        // Outer 0..10, inner 5..10 — both End events at byte 10. The
        // tie-break must close the innermost first so the segment
        // 5..10 is tagged "inner", not "outer". Locks walk's
        // descending-cap_idx End ordering against the producer
        // contract documented on `pegvm::Capture` (CaptureBegin order:
        // parent first, so cap_idx ascends with depth).
        let input = "abcdefghij";
        let kinds = vec!["outer".to_string(), "inner".to_string()];
        let captures = vec![cap(0, 0, 10), cap(1, 5, 10)];
        let segs = collect(input, &captures, &kinds);
        assert_tiles(input, &segs);
        assert_eq!(
            segs,
            vec![
                (0, 5, Some("outer".to_string())),
                (5, 10, Some("inner".to_string())),
            ]
        );
    }

    #[test]
    fn overlapping_captures_still_tile_input() {
        // Pathological input the walker shouldn't see in practice
        // (PEG nesting guarantees a forest), but the defensive
        // `rposition` path must keep coverage honest.
        let input = "abcdefghij";
        let kinds = vec!["a".to_string(), "b".to_string()];
        let captures = vec![cap(0, 0, 6), cap(1, 4, 10)];
        let segs = collect(input, &captures, &kinds);
        assert_tiles(input, &segs);
    }

    #[test]
    fn partial_parse_trailing_tail_is_unkinded() {
        // Captures only cover [0, 5); input is 10 bytes — simulating
        // a partial parse where the parser bailed at byte 5.
        let input = "abcdefghij";
        let kinds = vec!["k".to_string()];
        let captures = vec![cap(0, 0, 5)];
        let segs = collect(input, &captures, &kinds);
        assert_tiles(input, &segs);
        assert_eq!(segs, vec![(0, 5, Some("k".to_string())), (5, 10, None),]);
    }

    #[test]
    fn adjacent_captures_emit_back_to_back_kinded_segments() {
        // 0..3 kind=a, 3..6 kind=b, 6..9 unkinded gap.
        let input = "abcdefghi";
        let kinds = vec!["a".to_string(), "b".to_string()];
        let captures = vec![cap(0, 0, 3), cap(1, 3, 6)];
        let segs = collect(input, &captures, &kinds);
        assert_tiles(input, &segs);
        assert_eq!(
            segs,
            vec![
                (0, 3, Some("a".to_string())),
                (3, 6, Some("b".to_string())),
                (6, 9, None),
            ]
        );
    }
}
