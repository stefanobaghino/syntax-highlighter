//! Character set: an immutable sorted interval list over the Unicode
//! scalar value space, stored as `(char, char)` ranges.
//!
//! Used by the [`Instruction::CharSet`](super::Instruction::CharSet)
//! opcode for membership tests after a UTF-8 decode.
//!
//! `char` is the right representation here: Rust's `char` type already
//! enforces the Unicode scalar value invariant (no surrogates, no values
//! above `U+10FFFF`), so the constructor can't be asked to admit an
//! impossible value. The only construction error left is an inverted
//! range (`lo > hi`).

use std::fmt;

/// Immutable sorted, non-overlapping interval list over Unicode scalar
/// values. Internal representation is a `Vec<(char, char)>` of inclusive
/// `[lo, hi]` ranges with `lo <= hi`, sorted ascending with no adjacency
/// (adjacent ranges merge at construction).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CharSet {
    ranges: Vec<(char, char)>,
}

/// Construction-time error for inverted ranges. Out-of-range and
/// surrogate values are impossible by construction — `char` rejects them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CharSetError {
    /// `lo > hi` in a constructed range.
    InvertedRange(char, char),
}

impl fmt::Display for CharSetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            CharSetError::InvertedRange(lo, hi) => {
                write!(
                    f,
                    "inverted code-point range U+{:04X}-U+{:04X} (lo > hi)",
                    lo as u32, hi as u32
                )
            }
        }
    }
}

impl std::error::Error for CharSetError {}

impl CharSet {
    /// Empty set. Matches no code points.
    pub fn empty() -> Self {
        CharSet { ranges: Vec::new() }
    }

    /// Full set: every Unicode scalar value. Equivalent to `\p{Any}`.
    ///
    /// Stored as a single `('\0', char::MAX)` range — the surrogate gap
    /// is implicit (`char` can't hold a surrogate, so `contains_char`
    /// rejects them anyway). `from_sorted_dedup` bridges adjacent
    /// ranges across the gap, so this is the canonical merged form.
    pub fn any() -> Self {
        CharSet {
            ranges: vec![('\0', char::MAX)],
        }
    }

    /// Build from a list of inclusive ranges. The input does not need to
    /// be sorted or non-overlapping; the constructor normalises (sorts,
    /// merges overlaps and adjacencies). Errors on inverted ranges.
    pub fn from_ranges(input: &[(char, char)]) -> Result<Self, CharSetError> {
        let mut ranges = Vec::with_capacity(input.len());
        for &(lo, hi) in input {
            if lo > hi {
                return Err(CharSetError::InvertedRange(lo, hi));
            }
            ranges.push((lo, hi));
        }
        Ok(CharSet::from_sorted_dedup(ranges))
    }

    /// Single-range convenience constructor.
    pub fn single_range(lo: char, hi: char) -> Result<Self, CharSetError> {
        Self::from_ranges(&[(lo, hi)])
    }

    /// Singleton constructor: the set `{c}`.
    pub fn singleton(c: char) -> Self {
        CharSet {
            ranges: vec![(c, c)],
        }
    }

    /// Build from a list of individual characters. Each char becomes a
    /// `(c, c)` range; the constructor normalises (sorts, merges
    /// adjacencies) like [`from_ranges`](Self::from_ranges).
    pub fn from_chars(chars: &[char]) -> Self {
        let ranges: Vec<(char, char)> = chars.iter().map(|&c| (c, c)).collect();
        CharSet::from_ranges(&ranges).expect("singleton ranges are never inverted")
    }

    /// True iff the code point is in the set. Binary search; O(log n) in
    /// the number of ranges.
    pub fn contains_char(&self, c: char) -> bool {
        match self.ranges.binary_search_by(|&(lo, _)| lo.cmp(&c)) {
            Ok(_) => true,
            Err(0) => false,
            Err(idx) => {
                let (_, hi) = self.ranges[idx - 1];
                hi >= c
            }
        }
    }

    /// True iff the set contains the code point identified by `c`.
    /// Returns `false` if `c` is not a valid Unicode scalar value
    /// (surrogate or above `U+10FFFF`).
    pub fn contains(&self, c: u32) -> bool {
        char::from_u32(c).is_some_and(|ch| self.contains_char(ch))
    }

    /// Union with another set. Returns a new set.
    pub fn union(&self, other: &CharSet) -> CharSet {
        let mut merged = Vec::with_capacity(self.ranges.len() + other.ranges.len());
        merged.extend_from_slice(&self.ranges);
        merged.extend_from_slice(&other.ranges);
        CharSet::from_sorted_dedup(merged)
    }

    /// Complement of the set with respect to the Unicode scalar value
    /// space. Walks each of the two universe stretches that bracket the
    /// surrogate gap.
    pub fn negate(&self) -> CharSet {
        let universe: [(char, char); 2] = [('\0', '\u{D7FF}'), ('\u{E000}', char::MAX)];
        let mut out: Vec<(char, char)> = Vec::new();
        for &(u_lo, u_hi) in &universe {
            let mut cursor = u_lo;
            let mut cursor_consumed = false;
            for &(r_lo, r_hi) in &self.ranges {
                if r_hi < cursor {
                    continue;
                }
                if r_lo > u_hi {
                    break;
                }
                let lo = r_lo.max(u_lo);
                let hi = r_hi.min(u_hi);
                if cursor < lo {
                    out.push((cursor, prev_char(lo)));
                }
                match next_char(hi) {
                    Some(c) if c <= u_hi => {
                        cursor = c;
                    }
                    _ => {
                        cursor_consumed = true;
                        break;
                    }
                }
            }
            if !cursor_consumed && cursor <= u_hi {
                out.push((cursor, u_hi));
            }
        }
        CharSet::from_sorted_dedup(out)
    }

    /// Borrow the internal sorted range list. Public so pegb's
    /// serializer can round-trip the representation.
    pub fn ranges(&self) -> &[(char, char)] {
        &self.ranges
    }

    /// Number of disjoint intervals in the set.
    pub fn interval_count(&self) -> usize {
        self.ranges.len()
    }

    /// True iff the set matches no code points.
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// Merge sorted, possibly-overlapping ranges into a canonical
    /// non-overlapping non-adjacent representation. Caller is
    /// responsible for ensuring each input range is well-formed
    /// (`lo <= hi`); this helper does no validation.
    fn from_sorted_dedup(mut ranges: Vec<(char, char)>) -> Self {
        if ranges.is_empty() {
            return CharSet { ranges };
        }
        ranges.sort_by_key(|&(lo, _)| lo);
        let mut out: Vec<(char, char)> = Vec::with_capacity(ranges.len());
        for (lo, hi) in ranges {
            if let Some(last) = out.last_mut() {
                // Merge if the new range overlaps the last or is
                // adjacent (no gap, or just the surrogate gap between).
                let bridges_surrogate_gap = last.1 == '\u{D7FF}' && lo == '\u{E000}';
                if (lo as u32) <= (last.1 as u32).saturating_add(1) || bridges_surrogate_gap {
                    if hi > last.1 {
                        last.1 = hi;
                    }
                    continue;
                }
            }
            out.push((lo, hi));
        }
        CharSet { ranges: out }
    }
}

/// Next-greater scalar value, or `None` at `char::MAX`. Skips the
/// surrogate gap.
fn next_char(c: char) -> Option<char> {
    let n = c as u32 + 1;
    if n == 0xD800 {
        Some('\u{E000}')
    } else {
        char::from_u32(n)
    }
}

/// Previous scalar value. Caller must ensure `c != '\0'`. Skips the
/// surrogate gap.
fn prev_char(c: char) -> char {
    let n = c as u32;
    debug_assert!(n != 0, "prev_char: underflow at U+0000");
    if n == 0xE000 {
        '\u{D7FF}'
    } else {
        char::from_u32(n - 1).expect("prev_char: skipped surrogate gap")
    }
}

impl fmt::Debug for CharSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CharSet[")?;
        let mut first = true;
        for &(lo, hi) in &self.ranges {
            if !first {
                write!(f, ",")?;
            }
            first = false;
            if lo == hi {
                write!(f, "U+{:04X}", lo as u32)?;
            } else {
                write!(f, "U+{:04X}-U+{:04X}", lo as u32, hi as u32)?;
            }
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_contains_nothing() {
        let s = CharSet::empty();
        assert!(!s.contains_char('\0'));
        assert!(!s.contains_char('A'));
        assert!(!s.contains_char(char::MAX));
        assert!(s.is_empty());
        assert_eq!(s.interval_count(), 0);
    }

    #[test]
    fn any_contains_all_non_surrogates() {
        let s = CharSet::any();
        assert!(s.contains_char('\0'));
        assert!(s.contains_char('A'));
        assert!(s.contains_char('世'));
        assert!(s.contains_char(char::MAX));
        // Surrogates aren't representable as `char`; `contains` rejects
        // their u32 form.
        assert!(!s.contains(0xD800));
        // The surrogate gap is implicit — `any()` is one range from
        // `\0` to `char::MAX` since `char` can't hold a surrogate.
        assert_eq!(s.interval_count(), 1);
    }

    #[test]
    fn singleton() {
        let s = CharSet::singleton('世');
        assert!(s.contains_char('世'));
        assert!(!s.contains_char('\u{4E15}'));
        assert!(!s.contains_char('\u{4E17}'));
        assert_eq!(s.interval_count(), 1);
    }

    #[test]
    fn ascii_range() {
        let s = CharSet::single_range('A', 'Z').unwrap();
        assert!(s.contains_char('A'));
        assert!(s.contains_char('M'));
        assert!(s.contains_char('Z'));
        assert!(!s.contains_char('@'));
        assert!(!s.contains_char('['));
    }

    #[test]
    fn from_ranges_normalises_unsorted() {
        let s = CharSet::from_ranges(&[('a', 'z'), ('0', '9'), ('A', 'Z')]).unwrap();
        assert_eq!(s.interval_count(), 3);
        assert_eq!(s.ranges(), &[('0', '9'), ('A', 'Z'), ('a', 'z')]);
    }

    #[test]
    fn from_ranges_merges_overlapping() {
        let s = CharSet::from_ranges(&[('\u{10}', '\u{20}'), ('\u{15}', '\u{25}')]).unwrap();
        assert_eq!(s.interval_count(), 1);
        assert_eq!(s.ranges(), &[('\u{10}', '\u{25}')]);
    }

    #[test]
    fn from_ranges_merges_adjacent() {
        let s = CharSet::from_ranges(&[('\u{10}', '\u{20}'), ('\u{21}', '\u{30}')]).unwrap();
        assert_eq!(s.interval_count(), 1);
        assert_eq!(s.ranges(), &[('\u{10}', '\u{30}')]);
    }

    #[test]
    fn inverted_rejected() {
        assert_eq!(
            CharSet::single_range('\u{20}', '\u{10}'),
            Err(CharSetError::InvertedRange('\u{20}', '\u{10}'))
        );
    }

    #[test]
    fn surrogate_gap_bridged_by_adjacent_ranges() {
        // Two ranges that touch across the surrogate gap merge into
        // one — the gap is structural, not a real discontinuity in
        // the class's coverage.
        let s = CharSet::from_ranges(&[('\0', '\u{D7FF}'), ('\u{E000}', '\u{FFFF}')]).unwrap();
        assert_eq!(s.interval_count(), 1);
        assert_eq!(s.ranges(), &[('\0', '\u{FFFF}')]);
    }

    #[test]
    fn union_simple() {
        let a = CharSet::from_ranges(&[('\u{10}', '\u{20}')]).unwrap();
        let b = CharSet::from_ranges(&[('\u{30}', '\u{40}')]).unwrap();
        let u = a.union(&b);
        assert_eq!(u.ranges(), &[('\u{10}', '\u{20}'), ('\u{30}', '\u{40}')]);
    }

    #[test]
    fn union_merges_adjacent() {
        let a = CharSet::from_ranges(&[('\u{10}', '\u{1F}')]).unwrap();
        let b = CharSet::from_ranges(&[('\u{20}', '\u{30}')]).unwrap();
        let u = a.union(&b);
        assert_eq!(u.ranges(), &[('\u{10}', '\u{30}')]);
    }

    #[test]
    fn negate_empty_is_any() {
        let neg = CharSet::empty().negate();
        assert_eq!(neg, CharSet::any());
    }

    #[test]
    fn negate_any_is_empty() {
        let neg = CharSet::any().negate();
        assert_eq!(neg, CharSet::empty());
    }

    #[test]
    fn negate_single_range() {
        let s = CharSet::from_ranges(&[('A', 'Z')]).unwrap();
        let n = s.negate();
        assert!(n.contains_char('@'));
        assert!(!n.contains_char('A'));
        assert!(!n.contains_char('Z'));
        assert!(n.contains_char('['));
        assert!(n.contains_char(char::MAX));
    }

    #[test]
    fn contains_boundary() {
        let s = CharSet::from_ranges(&[('\u{80}', char::MAX)]).unwrap();
        assert!(!s.contains_char('\u{7F}'));
        assert!(s.contains_char('\u{80}'));
        assert!(s.contains_char(char::MAX));
    }

    #[test]
    fn debug_format() {
        let s = CharSet::from_ranges(&[('A', 'Z'), ('a', 'a')]).unwrap();
        let dbg = format!("{:?}", s);
        assert_eq!(dbg, "CharSet[U+0041-U+005A,U+0061]");
    }
}
