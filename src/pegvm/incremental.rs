//! Persistent memoization cache for incremental reparsing.
//!
//! The [`VM`] in this crate already memoizes every rule invocation during
//! a single run. Incremental parsing reuses that memo across edits: after
//! the user changes the input, most cached entries are still valid and
//! only the ones whose examined-span crosses the edit point need to be
//! recomputed. [`MemoCache`] is the ownership wrapper around that memo
//! between runs; [`MemoCache::apply_edit`] is the invalidation protocol.
//!
//! # Correctness
//!
//! Each memo entry carries its `examined_max` — the farthest input
//! position the rule's execution ever looked at (see
//! [`crate::pegvm::vm`]'s `MemoEntry` doc). For an edit that replaces
//! bytes `[edit.start, edit.old_end)` with content that produces the new
//! range `[edit.start, edit.new_end)`:
//!
//! - An entry at `(memo_id, start_sp)` is **invalidated** iff
//!   `start_sp < edit.old_end && examined_max > edit.start`. This covers
//!   both entries that started inside the edit region (their starting
//!   byte has changed) and entries that started before but peeked into
//!   it (their outcome might have depended on the changed bytes).
//! - An entry with `start_sp >= edit.old_end` is **shifted** by
//!   `delta = new_end - old_end` so its key, `end_sp`, `examined_max`,
//!   and capture offsets land on the equivalent position in the new
//!   input.
//! - An entry with `start_sp < edit.old_end && examined_max <= edit.start`
//!   lives entirely before the edit region — it is kept unshifted.
//!
//! Rebuilding the HashMap on every edit is `O(|cache|)`. Fine on the
//! working set sizes we care about; a position-indexed structure (e.g.
//! `BTreeMap<usize, ...>`) is a future optimization if cache size grows.
//!
//! Note for future work: when left-recursion support lands, its L-table
//! will need its own invalidation pass in parallel with this one.

use std::collections::HashMap;

use super::instruction::MemoId;
use super::vm::MemoEntry;

/// A description of a single edit to the parser's input. The replacement
/// content itself is not stored — the cache only needs the byte offsets.
///
/// - **Insertion** of `k` bytes at position `p`: `start = old_end = p`,
///   `new_end = p + k`. Use [`Edit::insertion`].
/// - **Deletion** of `[a, b)`: `start = a`, `old_end = b`, `new_end = a`.
///   Use [`Edit::deletion`].
/// - **Replacement** of `[a, b)` with `k` bytes: `start = a`,
///   `old_end = b`, `new_end = a + k`. Use [`Edit::replacement`].
///
/// Invariant: `start <= old_end` and `start <= new_end`. An edit where
/// `start == old_end == new_end` is a no-op and
/// [`MemoCache::apply_edit`] returns without touching the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edit {
    pub start: usize,
    pub old_end: usize,
    pub new_end: usize,
}

impl Edit {
    pub fn insertion(at: usize, inserted_len: usize) -> Self {
        Edit {
            start: at,
            old_end: at,
            new_end: at + inserted_len,
        }
    }

    pub fn deletion(start: usize, old_end: usize) -> Self {
        debug_assert!(start <= old_end, "Edit::deletion: start must be <= old_end");
        Edit {
            start,
            old_end,
            new_end: start,
        }
    }

    pub fn replacement(start: usize, old_end: usize, new_len: usize) -> Self {
        debug_assert!(
            start <= old_end,
            "Edit::replacement: start must be <= old_end"
        );
        Edit {
            start,
            old_end,
            new_end: start + new_len,
        }
    }

    fn delta(self) -> isize {
        self.new_end as isize - self.old_end as isize
    }

    fn is_noop(self) -> bool {
        self.start == self.old_end && self.start == self.new_end
    }
}

/// Memoization table carried across incremental reparses. Opaque wrapper
/// around the same `HashMap<(MemoId, usize), MemoEntry>` the VM uses
/// internally. Construct via [`MemoCache::new`]; hand to
/// `VM::new_with_cache` (arriving in a follow-up commit) to seed the
/// next run, and reclaim the post-run table via `VM::into_cache`.
#[derive(Debug, Default, Clone)]
pub struct MemoCache {
    table: HashMap<(MemoId, usize), MemoEntry>,
}

impl MemoCache {
    pub fn new() -> Self {
        MemoCache::default()
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Apply `edit` to every cached entry: drop the ones whose examined
    /// span crosses the edit point, shift the ones that follow it, and
    /// leave untouched the ones that finished before it. See the
    /// module-level docs for the full correctness argument.
    pub fn apply_edit(&mut self, edit: Edit) {
        if edit.is_noop() {
            return;
        }
        let delta = edit.delta();
        let mut next = HashMap::with_capacity(self.table.len());
        for ((memo_id, start_sp), mut entry) in self.table.drain() {
            if invalidates(edit, start_sp, &entry) {
                continue;
            }
            if start_sp >= edit.old_end {
                let new_start_sp = start_sp.wrapping_add_signed(delta);
                shift_entry(&mut entry, delta);
                next.insert((memo_id, new_start_sp), entry);
            } else {
                // Entry lives entirely before the edit; keep unshifted.
                next.insert((memo_id, start_sp), entry);
            }
        }
        self.table = next;
    }

    /// Drain the underlying table; leaves `self` empty. Package-internal
    /// hook for `VM::new_with_cache` (next commit) to seed its memo
    /// directly without an extra copy.
    #[allow(dead_code)] // consumed by the VM wiring in the next commit
    pub(crate) fn take(&mut self) -> HashMap<(MemoId, usize), MemoEntry> {
        std::mem::take(&mut self.table)
    }

    /// Replace the underlying table. Package-internal hook for
    /// `VM::into_cache` (next commit) to return a populated cache after
    /// a run.
    #[allow(dead_code)] // consumed by the VM wiring in the next commit
    pub(crate) fn install(&mut self, table: HashMap<(MemoId, usize), MemoEntry>) {
        self.table = table;
    }

    /// Package-internal read access for tests to assert on per-entry
    /// structure without leaking `MemoEntry`.
    #[cfg(test)]
    pub(crate) fn get(&self, memo_id: MemoId, start_sp: usize) -> Option<&MemoEntry> {
        self.table.get(&(memo_id, start_sp))
    }
}

fn invalidates(edit: Edit, start_sp: usize, entry: &MemoEntry) -> bool {
    start_sp < edit.old_end && entry.examined_max > edit.start
}

fn shift_entry(entry: &mut MemoEntry, delta: isize) {
    if let Some(ref mut end_sp) = entry.end_sp {
        *end_sp = end_sp.wrapping_add_signed(delta);
    }
    entry.examined_max = entry.examined_max.wrapping_add_signed(delta);
    for c in &mut entry.captures {
        c.start = c.start.wrapping_add_signed(delta);
        c.end = c.end.wrapping_add_signed(delta);
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the invalidation predicate and shift math. End-to-end
    //! parse-edit-reparse equivalence tests arrive once the VM can be
    //! seeded from a `MemoCache` (next commit).
    use super::*;
    use crate::pegvm::vm::{Capture, MemoEntry};
    use crate::pegvm::CaptureKind;

    fn cap(start: usize, end: usize) -> Capture {
        Capture {
            kind: CaptureKind(0),
            start,
            end,
        }
    }

    fn success(end_sp: usize, examined_max: usize, caps: Vec<Capture>) -> MemoEntry {
        MemoEntry {
            end_sp: Some(end_sp),
            examined_max,
            captures: caps,
        }
    }

    fn failure(examined_max: usize) -> MemoEntry {
        MemoEntry {
            end_sp: None,
            examined_max,
            captures: Vec::new(),
        }
    }

    fn populate(pairs: Vec<((MemoId, usize), MemoEntry)>) -> MemoCache {
        let mut cache = MemoCache::new();
        let mut table = HashMap::new();
        for (k, v) in pairs {
            table.insert(k, v);
        }
        cache.install(table);
        cache
    }

    #[test]
    fn noop_edit_leaves_everything_intact() {
        let mut cache = populate(vec![
            ((MemoId(0), 0), success(10, 12, vec![cap(0, 10)])),
            ((MemoId(1), 5), success(8, 8, vec![])),
        ]);
        cache.apply_edit(Edit {
            start: 7,
            old_end: 7,
            new_end: 7,
        });
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(MemoId(0), 0).unwrap().examined_max, 12);
        assert_eq!(cache.get(MemoId(1), 5).unwrap().end_sp, Some(8));
    }

    #[test]
    fn entry_fully_before_edit_is_kept_unshifted() {
        // entry at 0..5, examined_max=5. Edit at 10..10 inserting 3 bytes.
        let mut cache = populate(vec![((MemoId(0), 0), success(5, 5, vec![cap(1, 4)]))]);
        cache.apply_edit(Edit::insertion(10, 3));
        let e = cache.get(MemoId(0), 0).expect("must survive");
        assert_eq!(e.end_sp, Some(5));
        assert_eq!(e.examined_max, 5);
        assert_eq!(e.captures, vec![cap(1, 4)]);
    }

    #[test]
    fn entry_starting_after_edit_is_shifted() {
        // entry starts at sp=20, examined_max=30. Edit inserts 5 bytes at sp=5.
        let mut cache = populate(vec![((MemoId(2), 20), success(25, 30, vec![cap(22, 24)]))]);
        cache.apply_edit(Edit::insertion(5, 5));
        assert!(cache.get(MemoId(2), 20).is_none(), "old key gone");
        let e = cache
            .get(MemoId(2), 25)
            .expect("entry present under shifted key");
        assert_eq!(e.end_sp, Some(30));
        assert_eq!(e.examined_max, 35);
        assert_eq!(e.captures, vec![cap(27, 29)]);
    }

    #[test]
    fn deletion_shifts_entries_left() {
        // entry at sp=20, examined_max=30. Delete bytes 5..10 (5 bytes removed).
        let mut cache = populate(vec![((MemoId(0), 20), success(25, 30, vec![cap(20, 25)]))]);
        cache.apply_edit(Edit::deletion(5, 10));
        let e = cache
            .get(MemoId(0), 15)
            .expect("entry present under shifted key");
        assert_eq!(e.end_sp, Some(20));
        assert_eq!(e.examined_max, 25);
        assert_eq!(e.captures, vec![cap(15, 20)]);
    }

    #[test]
    fn entry_spanning_edit_is_invalidated() {
        // entry starts at 0, examined up to 20. Edit at 10..12.
        // examined_max (20) > edit.start (10) AND start_sp (0) < old_end (12).
        let mut cache = populate(vec![((MemoId(0), 0), success(18, 20, vec![cap(0, 18)]))]);
        cache.apply_edit(Edit::replacement(10, 12, 2));
        assert_eq!(cache.len(), 0, "entry must be dropped");
    }

    #[test]
    fn entry_starting_inside_edit_is_invalidated() {
        // Even if examined_max never reaches past start, an entry whose
        // starting byte sits inside the edit region is stale — the byte
        // at start_sp may have changed. The predicate catches this
        // because examined_max >= start_sp > edit.start.
        let mut cache = populate(vec![((MemoId(0), 8), success(10, 10, vec![]))]);
        cache.apply_edit(Edit::replacement(5, 12, 7));
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn entry_ending_at_edit_boundary_is_kept() {
        // Entry examined up to exactly edit.start (exclusive: examined_max == 10,
        // reads go through position 9). The read at position 9 doesn't overlap
        // with bytes 10.., so the entry is valid.
        let mut cache = populate(vec![((MemoId(0), 0), success(10, 10, vec![]))]);
        cache.apply_edit(Edit::replacement(10, 12, 5));
        let e = cache.get(MemoId(0), 0).expect("entry must survive");
        assert_eq!(e.examined_max, 10);
    }

    #[test]
    fn failure_entry_with_eof_read_is_invalidated_on_append() {
        // Append scenario: old input had length 5. The failing rule
        // probed position 5 (EOF), which bumped its watermark to 6 —
        // `track_read` fires *before* the bounds check, so a read at
        // `sp = 5` registers the intent to examine position 5 even
        // when `input.get(5)` returns None. Appending bytes at
        // position 5 changes what lives there, so the cached failure
        // is stale. Predicate: `0 < 5 && 6 > 5` → drop.
        let mut cache = populate(vec![((MemoId(0), 0), failure(6))]);
        cache.apply_edit(Edit::insertion(5, 3));
        assert_eq!(cache.len(), 0, "EOF-sensitive failure must be dropped");
    }

    #[test]
    fn failure_entry_with_bounded_read_survives_append() {
        // Failure entry examined only up to position 3 (< old_input_len).
        // Appending bytes past the examined range doesn't affect it.
        let mut cache = populate(vec![((MemoId(0), 0), failure(3))]);
        cache.apply_edit(Edit::insertion(5, 3));
        let e = cache.get(MemoId(0), 0).expect("failure must survive");
        assert_eq!(e.end_sp, None);
        assert_eq!(e.examined_max, 3);
    }

    #[test]
    fn insert_at_zero_shifts_everything() {
        // Edit::insertion(0, k) shifts every entry by k and invalidates
        // nothing (examined_max >= start_sp >= 0; start_sp < 0 is
        // impossible, so the `start_sp < edit.old_end == 0` leg of the
        // predicate is always false).
        let mut cache = populate(vec![
            ((MemoId(0), 0), success(5, 5, vec![cap(0, 5)])),
            ((MemoId(1), 10), success(15, 18, vec![cap(11, 14)])),
        ]);
        cache.apply_edit(Edit::insertion(0, 4));
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(MemoId(0), 4).unwrap().end_sp, Some(9));
        assert_eq!(cache.get(MemoId(0), 4).unwrap().captures, vec![cap(4, 9)]);
        assert_eq!(cache.get(MemoId(1), 14).unwrap().examined_max, 22);
    }

    #[test]
    fn replacement_of_equal_length_does_not_shift_but_invalidates() {
        // delta = 0, but entries whose examined_max > edit.start must
        // still be invalidated — the byte content within [start, end)
        // has changed.
        let mut cache = populate(vec![
            ((MemoId(0), 0), success(20, 20, vec![cap(0, 20)])), // spans edit
            ((MemoId(1), 25), success(30, 30, vec![cap(25, 30)])), // after edit
        ]);
        cache.apply_edit(Edit::replacement(10, 15, 5));
        assert!(cache.get(MemoId(0), 0).is_none(), "spanning entry dropped");
        // After-edit entry unshifted (delta=0) and intact.
        let e = cache.get(MemoId(1), 25).expect("after-edit entry survives");
        assert_eq!(e.end_sp, Some(30));
        assert_eq!(e.captures, vec![cap(25, 30)]);
    }

    #[test]
    fn same_start_sp_multiple_memo_ids_shift_together() {
        // Two rules memoized at the same position get independent keys;
        // shift them both.
        let mut cache = populate(vec![
            ((MemoId(0), 10), success(15, 15, vec![])),
            ((MemoId(1), 10), success(12, 12, vec![])),
        ]);
        cache.apply_edit(Edit::insertion(5, 2));
        assert!(cache.get(MemoId(0), 10).is_none());
        assert!(cache.get(MemoId(1), 10).is_none());
        assert_eq!(cache.get(MemoId(0), 12).unwrap().end_sp, Some(17));
        assert_eq!(cache.get(MemoId(1), 12).unwrap().end_sp, Some(14));
    }
}
