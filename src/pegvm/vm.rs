use std::collections::HashMap;
use std::mem::MaybeUninit;

use super::instruction::{CaptureKind, Instruction, MemoId, RuleKind};
use super::slab::Slab;

/// One half-open byte span `start..end` tagged with a [`CaptureKind`].
///
/// Emitted in `CaptureBegin` order: `start`-ascending, with a parent
/// capture appearing before any of its children (the outer rule's
/// `CaptureBegin` fires before any inner one's). `CaptureEnd` only
/// fills in the matching capture's `end` — it doesn't reorder.
/// Consumers that reconstruct nesting can walk a stack keyed on `end`
/// (see `walk` in `src/walk.rs` for the canonical traversal).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    pub kind: CaptureKind,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
enum StackEntry {
    Backtrack {
        ip: usize,
        sp: usize,
        snapshot: CaptureSnapshot,
    },
    Return {
        ip: usize,
    },
    /// Frame for an in-flight memoized rule call. Pushed by `RuleEnter`'s
    /// `RuleKind::Memo` miss path, popped by `MemoClose` on success (which
    /// records the entry) or by `fail()` when the rule escapes via failure
    /// (which records a failure entry). Holds enough state to locate the
    /// cache slot (`memo_id`, `start_sp`) and to extract the captures
    /// produced inside the rule (`snapshot`).
    Memo {
        memo_id: MemoId,
        start_sp: usize,
        snapshot: CaptureSnapshot,
    },
    /// Frame for an in-flight left-recursive rule invocation. Pushed by
    /// `RuleEnter`'s `RuleKind::Lr` miss path on the first entry at a
    /// given `sp`; popped by `LRTail` when the seed-and-grow loop
    /// converges, or by `fail()` (which uses the `seed` to decide between
    /// rescuing the rule with the prior seed or continuing to unwind).
    /// On convergence `LRTail` writes the seed to `self.memo` (subject to
    /// the threshold filter); failure entries for LR rules are not
    /// cached yet (#48 scoped to converged seeds).
    LFrame {
        memo_id: MemoId,
        start_sp: usize,
        snapshot: CaptureSnapshot,
        return_addr: usize,
        seed: Option<LSeed>,
    },
}

/// Successful seed of a left-recursive rule's prior iteration: the `sp` the
/// body matched up to, and the closed captures it produced past the
/// `LFrame`'s baseline snapshot. Replayed verbatim on recursive entries
/// (where the recursive call must appear to have returned this seed) and on
/// the convergence step (where the rule itself exits with this match).
#[derive(Debug, Clone)]
struct LSeed {
    end_sp: usize,
    captures: Vec<Capture>,
}

/// Outcome of running a compiled [`Program`](super::Program) against an input.
///
/// - `complete == true`: the VM reached `End`. `matched` is the `sp` at that
///   point, which may be less than `input.len()` if the grammar is designed
///   to stop early (no trailing `!.`).
/// - `complete == false`: the VM exhausted its backtrack stack without
///   reaching `End`. `matched` is the farthest input position the VM ever
///   reached before retreating (Ford 2004's "farthest failure position"),
///   and `captures` are the captures that were valid at that point — any
///   captures left open at `matched` are closed there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    pub matched: usize,
    pub captures: Vec<Capture>,
    pub complete: bool,
}

/// Diagnostic counters for the memoization cache, exposed via
/// [`VM::run_with_memo_stats`]. Primarily used by tests and benchmarks to
/// prove the cache is doing its job.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoStats {
    /// Number of distinct `(memo_id, start_sp)` pairs in the table at the
    /// end of the run. Each entry represents one cached rule outcome
    /// (success or failure).
    pub entries: usize,
    /// Number of times `RuleEnter`'s shared cache-hit prologue resolved
    /// via a cached entry instead of re-executing the rule body. Counts
    /// hits for both `RuleKind::Memo` and `RuleKind::Lr` rules.
    pub hits: usize,
    /// Number of `RuleEnter` cache misses on `RuleKind::Memo` rules — i.e.
    /// non-LR rules that had to execute the body. LR-rule misses are not
    /// counted here because the LR miss path may resolve via a live
    /// `LFrame` (recursive entry) rather than executing the body. With
    /// the memo threshold at 0, every Memo miss produces an entry; with
    /// a non-zero threshold, successful miss bodies shorter than the
    /// threshold are not written back, so `entries` and `misses` diverge.
    pub misses: usize,
}

pub struct VM<'p, 'i> {
    program: &'p [Instruction],
    input: &'i [u8],
    ip: usize,
    sp: usize,
    stack: Vec<StackEntry>,
    /// Speculative-capture arena. `OpenCapture`s live in a parent-linked
    /// chain of fixed-size [`Chunk`]s (size [`CHUNK_SIZE`]). The chain
    /// from `current_chunk` back to `root_chunk` is the live capture
    /// stream in `CaptureBegin` order. Pushes spill into a freshly
    /// allocated child chunk when the current chunk fills; restores walk
    /// the chain leaf-first, freeing chunks back to the slab. Replaces
    /// the flat-Vec design and its `protect_max_captures` clone-on-fail
    /// (which was 73.6 % self-time on jquery.js) with chain-walk
    /// preservation that pays only when the watermark is endangered.
    chunks: Slab<Chunk>,
    current_chunk: ChunkId,
    /// Origin of the chain. Allocated in `VM::new`; never freed. Pushes
    /// from a freshly constructed VM land in `root_chunk` until it fills.
    root_chunk: ChunkId,
    /// Monotone-increasing counter assigned to each `Chunk` at slab-insert
    /// time. Two purposes: (a) detect slab-slot reuse — a `CaptureSnapshot`'s
    /// `chunk_seq` must match the live chunk's `seq`, otherwise the slot
    /// was freed and a different chunk now occupies it; (b) total-order
    /// chunks for `MaxSnap`'s endangerment check.
    next_seq: u64,
    max_sp: usize,
    /// Farthest-failure capture state. Stores `(chunk, offset)` at the
    /// moment `sp` reached `max_sp`; the preservation copy in
    /// `MaxSnap.saved` is materialised only when `restore_to` is about
    /// to free the snapshot's chunk (or truncate within it past
    /// `snapshot.offset`). On a successful parse the snapshot is never
    /// materialised — what was 73.6 % self-time as `protect_max_captures`
    /// becomes a single integer comparison.
    max_capture: Option<MaxSnap>,
    /// Packrat memo table, keyed by `(memo_id, start_sp)`. Populated by
    /// `MemoClose` on success and by `fail()` on failure escape (Commit 5).
    memo: HashMap<(MemoId, usize), MemoEntry>,
    /// Running count of resolved cache hits, exposed via `MemoStats`.
    memo_hits: usize,
    /// Running count of `RuleEnter` cache misses on `RuleKind::Memo`
    /// rules, exposed via `MemoStats`. See [`MemoStats::misses`].
    memo_misses: usize,
    /// Minimum successful-span length (in bytes) for which `MemoClose` (i.e.
    /// `RuleKind::Memo` rules) will write the outcome back to the cache.
    /// Default is [`Self::DEFAULT_MEMO_THRESHOLD`]; `0` disables the filter
    /// and restores pure packrat behavior. Non-zero values skip tiny
    /// leaf-rule entries that pay lookup cost without a meaningful storage
    /// win — see GPeg (default 512) and Yedidia §5.2.4 (knee near 4096).
    /// `LRTail` (`RuleKind::Lr` seed commit) ignores this filter — the
    /// seed-and-grow loop relies on the seed cache to short-circuit
    /// subsequent visits, and filtering short LR seeds out causes O(2^N)
    /// re-descent in deep LR cascades (issue #55). Failure entries in
    /// `fail()` are also not filtered; their value is short-circuiting.
    memo_threshold: usize,
    /// Per-rule-invocation watermark of the farthest input position examined
    /// since the enclosing `StackEntry::Memo` frame was pushed. One entry
    /// per live `Memo` frame, pushed/popped in lockstep with them. The
    /// topmost value is bumped by every read site via `track_read`; at
    /// `MemoClose` or the `Memo` arm of `fail()` the popped value becomes
    /// the outgoing entry's `examined_max` and is then merged into the new
    /// top (the parent rule's watermark). Needed for incremental parsing:
    /// an edit at position `p` invalidates a cached entry iff
    /// `p < entry.examined_max`, so the bound must reflect lookahead reads
    /// past `end_sp` as well as the consumed span.
    memo_examined: Vec<usize>,
}

/// A memo-table entry for a rule invocation at a specific input position.
///
/// `end_sp.is_some()` encodes success and carries the sp at which the rule
/// finished matching, plus the captures it produced (already closed).
/// `end_sp.is_none()` encodes a cached failure — future hits at the same
/// `(memo_id, start_sp)` enter `fail()` directly without re-running the
/// rule body.
///
/// `examined_max` is the farthest input position the rule's execution ever
/// looked at, including lookahead past `end_sp` (`&expr` / `!expr`) and
/// failed reads. It is always `>= start_sp` (the key's second component),
/// and for successful entries `>= end_sp`. Incremental parsing uses it as
/// the invalidation bound: an edit touching any byte before
/// `examined_max` may change the rule's outcome, so the entry is stale.
///
/// Fields are `pub(crate)` so the `incremental` module (which holds the
/// `MemoCache` that reuses entries across edits) can read and shift them.
/// Outside the crate the type is opaque.
#[derive(Debug, Clone)]
pub(crate) struct MemoEntry {
    pub(crate) end_sp: Option<usize>,
    pub(crate) examined_max: usize,
    pub(crate) captures: Vec<Capture>,
}

#[derive(Debug, Clone, Copy)]
struct OpenCapture {
    kind: CaptureKind,
    start: usize,
    end: Option<usize>,
}

/// Number of `OpenCapture`s a single `Chunk` can hold inline. Sized so
/// that a chunk is ≈1 KiB on a 64-bit target (32 × 32 B = 1024 B for
/// the captures array, plus ≈18 B of bookkeeping). Larger values reduce
/// chain depth (and thus per-restore walk length) but raise the
/// per-chunk slab footprint and the per-clone cost when an endangered
/// `MaxSnap` materialises. 32 is the starting point; sweep if perf is
/// sensitive.
const CHUNK_SIZE: usize = 32;

/// Index into the `chunks` slab. Stable across pushes; reused after
/// `Slab::remove` (the `seq` field on each saved snapshot detects
/// reuse). 32 bits is plenty — the largest realistic input observed
/// keeps live chunks under ~10⁵ even with deep backtracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChunkId(u32);

/// One node of the chunked capture chain. Slots `[..len]` are
/// initialised; the remainder are `MaybeUninit::uninit()` and may not
/// be read until written by `push_capture`. `parent` is `None` only
/// for `root_chunk`, allocated in [`VM::new`]; every other chunk is
/// linked back to a chunk that existed at allocation time. `seq` is a
/// monotone-increasing per-VM counter, assigned at slab-insert time.
struct Chunk {
    captures: [MaybeUninit<OpenCapture>; CHUNK_SIZE],
    len: u16,
    parent: Option<ChunkId>,
    seq: u64,
}

impl Chunk {
    fn new(parent: Option<ChunkId>, seq: u64) -> Self {
        Chunk {
            captures: std::array::from_fn(|_| MaybeUninit::uninit()),
            len: 0,
            parent,
            seq,
        }
    }

    /// Initialised prefix as a `&[OpenCapture]` slice. Reads only the
    /// `len` slots that `push_capture` has written.
    fn slots(&self) -> &[OpenCapture] {
        // SAFETY: slots `..len` are initialised by `push_capture`; the
        // `MaybeUninit<T>` and `T` have identical layouts, and the
        // pointer cast respects alignment of `OpenCapture`.
        unsafe {
            std::slice::from_raw_parts(
                self.captures.as_ptr() as *const OpenCapture,
                self.len as usize,
            )
        }
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        for slot in &mut self.captures[..self.len as usize] {
            // SAFETY: slots `..len` are initialised by `push_capture`.
            unsafe { slot.assume_init_drop() };
        }
    }
}

/// Saved state of "everything that has been captured so far": the
/// chunk the writer was in, plus the in-chunk offset at save time.
/// Captures up to this point form `chunk.slots()[..offset]` **plus**
/// `chunk`'s ancestor chain.
#[derive(Debug, Clone, Copy)]
struct CaptureSnapshot {
    chunk: ChunkId,
    offset: u16,
    chunk_seq: u64,
}

/// Farthest-failure capture state, materialised lazily. `snapshot`
/// points at the live chunk reached at `max_sp`; `saved` is the
/// preservation copy, populated only the first time `restore_to` would
/// drop captures the snapshot still depends on (either freeing
/// `snapshot.chunk` outright, or truncating it below `snapshot.offset`).
/// On `sp > max_sp` the snapshot is replaced and `saved` cleared.
struct MaxSnap {
    snapshot: CaptureSnapshot,
    saved: Option<Vec<OpenCapture>>,
}

impl<'p, 'i> VM<'p, 'i> {
    /// Default memo-threshold applied by [`VM::new`]. Picked from the sweep
    /// at `benches/memo.rs`: the time-vs-entries curve is flat from ~32
    /// bytes upward on every shipped grammar, so any value in that range
    /// is defensible. 128 matches GPeg's benchmark reference point and
    /// stays conservative against hardware and corpus variation.
    pub const DEFAULT_MEMO_THRESHOLD: usize = 128;

    pub fn new(program: &'p [Instruction], input: &'i [u8]) -> Self {
        let mut chunks: Slab<Chunk> = Slab::new();
        // Root chunk gets seq 0; subsequent allocations bump `next_seq`.
        // The 0-baseline matters for the endangerment check: any snapshot
        // taken on a non-root chunk has `chunk_seq > 0`, so a restore
        // back through root correctly triggers preservation.
        let root_id = chunks.insert(Chunk::new(None, 0));
        let root_chunk = ChunkId(root_id);
        VM {
            program,
            input,
            ip: 0,
            sp: 0,
            stack: Vec::new(),
            chunks,
            current_chunk: root_chunk,
            root_chunk,
            next_seq: 1,
            max_sp: 0,
            max_capture: None,
            memo: HashMap::new(),
            memo_hits: 0,
            memo_misses: 0,
            memo_threshold: Self::DEFAULT_MEMO_THRESHOLD,
            memo_examined: Vec::new(),
        }
    }

    /// Override the default memo threshold. `bytes = 0` disables the
    /// filter and restores pure packrat behavior (useful for tests that
    /// exercise the caching mechanism on small inputs). See the
    /// `memo_threshold` field doc for the rationale.
    pub fn with_memo_threshold(mut self, bytes: usize) -> Self {
        self.memo_threshold = bytes;
        self
    }

    /// Construct a VM whose memo table is pre-populated with `cache`.
    /// Entries the seeded cache agrees with the program on (same rule +
    /// position) will be served as hits without re-executing the rule
    /// body — that is the entire point of incremental parsing. The
    /// caller is responsible for ensuring the cache was built against
    /// either this same input or a prior input plus a correctly applied
    /// [`MemoCache::apply_edit`](super::incremental::MemoCache::apply_edit).
    pub fn new_with_cache(
        program: &'p [Instruction],
        input: &'i [u8],
        mut cache: super::incremental::MemoCache,
    ) -> Self {
        let mut vm = Self::new(program, input);
        vm.memo = cache.take();
        vm
    }

    pub fn run(self) -> MatchResult {
        self.run_with_memo_stats().0
    }

    pub fn run_with_memo_stats(self) -> (MatchResult, MemoStats) {
        let (result, stats, _memo) = self.run_core();
        (result, stats)
    }

    /// Run the VM to completion and return the populated memo as a
    /// [`MemoCache`](super::incremental::MemoCache) so the caller can
    /// carry it across an edit. Equivalent to `run_with_memo_stats`
    /// plus a rewrap; the cache retains every entry recorded during
    /// this run, including entries seeded in via [`new_with_cache`]
    /// that survived the parse.
    pub fn run_with_cache(self) -> (MatchResult, MemoStats, super::incremental::MemoCache) {
        let (result, stats, memo) = self.run_core();
        let mut cache = super::incremental::MemoCache::new();
        cache.install(memo);
        (result, stats, cache)
    }

    /// Core execution loop. Returns the result, stats, and the final memo
    /// table. Package-internal wrapper for tests that need to inspect
    /// per-entry details (notably `examined_max`) and for the public
    /// `run_with_cache` variant that rewraps the table into a
    /// [`MemoCache`](super::incremental::MemoCache).
    fn run_core(mut self) -> (MatchResult, MemoStats, HashMap<(MemoId, usize), MemoEntry>) {
        loop {
            let instr = match self.program.get(self.ip) {
                Some(i) => i,
                None => return self.finalize_partial(),
            };
            match instr {
                Instruction::Char(b) => {
                    self.track_read(1);
                    if self.input.get(self.sp) == Some(b) {
                        self.sp += 1;
                        self.ip += 1;
                    } else if !self.fail() {
                        return self.finalize_partial();
                    }
                }
                Instruction::Set(set) => {
                    self.track_read(1);
                    match self.input.get(self.sp) {
                        Some(b) if set.contains(*b) => {
                            self.sp += 1;
                            self.ip += 1;
                        }
                        _ => {
                            if !self.fail() {
                                return self.finalize_partial();
                            }
                        }
                    }
                }
                Instruction::Any(n) => {
                    let n = *n as usize;
                    self.track_read(n);
                    if self.sp + n <= self.input.len() {
                        self.sp += n;
                        self.ip += 1;
                    } else if !self.fail() {
                        return self.finalize_partial();
                    }
                }
                Instruction::TestChar(b, label) => {
                    self.track_read(1);
                    if self.input.get(self.sp) == Some(b) {
                        self.sp += 1;
                        self.ip += 1;
                    } else {
                        self.ip = label.as_index();
                    }
                }
                Instruction::TestSet(set, label) => {
                    self.track_read(1);
                    match self.input.get(self.sp) {
                        Some(b) if set.contains(*b) => {
                            self.sp += 1;
                            self.ip += 1;
                        }
                        _ => {
                            self.ip = label.as_index();
                        }
                    }
                }
                Instruction::Jump(label) => {
                    self.ip = label.as_index();
                }
                Instruction::Choice(label) => {
                    let snapshot = self.snapshot_now();
                    self.stack.push(StackEntry::Backtrack {
                        ip: label.as_index(),
                        sp: self.sp,
                        snapshot,
                    });
                    self.ip += 1;
                }
                Instruction::Commit(label) => {
                    self.pop_backtrack();
                    self.ip = label.as_index();
                }
                Instruction::PartialCommit(label) => {
                    let new_snap = self.snapshot_now();
                    let top = self.stack.last_mut().expect("PartialCommit on empty stack");
                    match top {
                        StackEntry::Backtrack { sp, snapshot, .. } => {
                            *sp = self.sp;
                            *snapshot = new_snap;
                        }
                        _ => panic!("PartialCommit expected Backtrack on stack top"),
                    }
                    self.ip = label.as_index();
                }
                Instruction::BackCommit(label) => {
                    self.maybe_snapshot();
                    let entry = self.pop_backtrack();
                    if let StackEntry::Backtrack { sp, snapshot, .. } = entry {
                        self.sp = sp;
                        self.restore_to(snapshot);
                    }
                    self.ip = label.as_index();
                }
                Instruction::FailTwice => {
                    self.pop_backtrack();
                    if !self.fail() {
                        return self.finalize_partial();
                    }
                }
                Instruction::Fail => {
                    if !self.fail() {
                        return self.finalize_partial();
                    }
                }
                Instruction::Call(label) => {
                    self.stack.push(StackEntry::Return { ip: self.ip + 1 });
                    self.ip = label.as_index();
                }
                Instruction::Return => {
                    let ret_ip = match self.stack.pop() {
                        Some(StackEntry::Return { ip }) => ip,
                        Some(other) => {
                            panic!("Return expected Return on stack top, got {:?}", other)
                        }
                        None => panic!("Return on empty stack"),
                    };
                    self.ip = ret_ip;
                }
                Instruction::RuleEnter(memo_id, kind, return_label) => {
                    // Shared cache-hit prologue. Both kinds probe the same
                    // packrat slot; only the post-miss frame layout differs.
                    if let Some(entry) = self.memo.get(&(*memo_id, self.sp)) {
                        self.memo_hits += 1;
                        let hit_examined = entry.examined_max;
                        match entry.end_sp {
                            Some(end_sp) => {
                                // Success hit: replay captures already-closed
                                // (the enclosing `CaptureEnd`'s reverse-walk
                                // binds to the innermost still-open capture
                                // and skips over them).
                                let replay = entry.captures.clone();
                                self.bump_top_memo_examined(hit_examined);
                                for c in replay {
                                    self.push_capture(OpenCapture {
                                        kind: c.kind,
                                        start: c.start,
                                        end: Some(c.end),
                                    });
                                }
                                self.sp = end_sp;
                                self.ip = return_label.as_index();
                                // Hit advances sp past code that didn't run;
                                // farthest-failure bookkeeping must see it.
                                self.maybe_snapshot();
                            }
                            None => {
                                // Cached failure: enter fail() immediately
                                // without re-executing the rule body or
                                // pushing any frame.
                                self.bump_top_memo_examined(hit_examined);
                                if !self.fail() {
                                    return self.finalize_partial();
                                }
                            }
                        }
                        continue;
                    }
                    // Cache miss — frame layout depends on the kind.
                    match kind {
                        RuleKind::Memo => {
                            self.memo_misses += 1;
                            let snapshot = self.snapshot_now();
                            self.stack.push(StackEntry::Memo {
                                memo_id: *memo_id,
                                start_sp: self.sp,
                                snapshot,
                            });
                            // Parallel watermark for this frame. Starts at the
                            // rule's entry sp; read sites bump it as the body
                            // executes. Popped together with the `Memo` frame
                            // at `MemoClose` or in `fail()`'s `Memo` arm.
                            self.memo_examined.push(self.sp);
                            self.ip += 1;
                        }
                        RuleKind::Lr => {
                            // Walk the stack top-down for an in-flight LFrame
                            // at the same (memo_id, sp). The packrat probe
                            // above has already returned None, so any matching
                            // LFrame is a current recursive re-entry rather
                            // than a stale converged seed.
                            let lookup: Option<Option<LSeed>> =
                                self.stack.iter().rev().find_map(|e| {
                                    if let StackEntry::LFrame {
                                        memo_id: id,
                                        start_sp,
                                        seed,
                                        ..
                                    } = e
                                    {
                                        if *id == *memo_id && *start_sp == self.sp {
                                            return Some(seed.clone());
                                        }
                                    }
                                    None
                                });
                            match lookup {
                                Some(Some(found)) => {
                                    // Recursive entry with a seed: replay
                                    // captures and jump to the rule's Return
                                    // so the caller's `Call`-pushed Return
                                    // frame fires normally.
                                    let replay = found.captures;
                                    self.bump_top_memo_examined(found.end_sp);
                                    for c in replay {
                                        self.push_capture(OpenCapture {
                                            kind: c.kind,
                                            start: c.start,
                                            end: Some(c.end),
                                        });
                                    }
                                    self.sp = found.end_sp;
                                    self.ip = return_label.as_index();
                                    self.maybe_snapshot();
                                }
                                Some(None) => {
                                    // Recursive entry with no seed yet — the
                                    // rule has not succeeded once at this
                                    // position, so the recursive call must
                                    // fail (bound 0).
                                    if !self.fail() {
                                        return self.finalize_partial();
                                    }
                                }
                                None => {
                                    // First entry at this sp — push an LFrame
                                    // and a paired memo_examined watermark,
                                    // then fall through to the body.
                                    let snapshot = self.snapshot_now();
                                    self.stack.push(StackEntry::LFrame {
                                        memo_id: *memo_id,
                                        start_sp: self.sp,
                                        snapshot,
                                        return_addr: return_label.as_index(),
                                        seed: None,
                                    });
                                    self.memo_examined.push(self.sp);
                                    self.ip += 1;
                                }
                            }
                        }
                    }
                }
                Instruction::MemoClose(memo_id) => {
                    let (top_id, start_sp, snapshot) = match self.stack.pop() {
                        Some(StackEntry::Memo {
                            memo_id,
                            start_sp,
                            snapshot,
                        }) => (memo_id, start_sp, snapshot),
                        other => {
                            panic!("MemoClose expected Memo on stack top, got {:?}", other)
                        }
                    };
                    debug_assert_eq!(
                        top_id, *memo_id,
                        "MemoClose id mismatch: expected {:?}, found {:?}",
                        memo_id, top_id,
                    );
                    debug_assert!(
                        start_sp <= self.sp,
                        "MemoClose: rule body retreated past start_sp ({} > {})",
                        start_sp,
                        self.sp,
                    );
                    // Pop this frame's examined watermark. Every `Memo`
                    // frame push (in `RuleEnter`'s Memo-kind miss path)
                    // is paired with a `memo_examined` push, and this is
                    // the only non-failure pop site, so the stacks must
                    // align.
                    let examined_max = self
                        .memo_examined
                        .pop()
                        .expect("MemoClose: memo_examined underflow");
                    debug_assert!(
                        examined_max >= self.sp,
                        "MemoClose: examined_max ({}) fell behind end_sp ({})",
                        examined_max,
                        self.sp,
                    );
                    // Propagate up so the parent rule's watermark covers
                    // everything this one examined.
                    self.bump_top_memo_examined(examined_max);
                    // Snapshot the captures produced inside the rule. All of
                    // them must be closed at this point — PEG captures are
                    // lexically scoped to `@name{...}` and cannot straddle a
                    // rule boundary, so any `OpenCapture` with `end.is_none()`
                    // would be a compiler bug.
                    let body_caps = self.collect_captures_since(snapshot);
                    let rule_captures: Vec<Capture> = body_caps
                        .into_iter()
                        .map(|c| {
                            debug_assert!(
                                c.end.is_some(),
                                "MemoClose: open capture straddles rule boundary"
                            );
                            Capture {
                                kind: c.kind,
                                start: c.start,
                                end: c.end.unwrap_or(self.sp),
                            }
                        })
                        .collect();
                    self.cache_success(
                        RuleKind::Memo,
                        *memo_id,
                        start_sp,
                        self.sp,
                        examined_max,
                        rule_captures,
                    );
                    self.ip += 1;
                }
                Instruction::LRTail(memo_id, body_start) => {
                    // Peek the topmost LFrame. The body just succeeded; we
                    // must decide between growing (re-iterate) and
                    // accepting (commit and fall through to Return).
                    let top_idx = self
                        .stack
                        .iter()
                        .rposition(|e| matches!(e, StackEntry::LFrame { .. }))
                        .expect("LRTail without an enclosing LFrame");
                    let StackEntry::LFrame {
                        memo_id: top_id,
                        start_sp,
                        snapshot,
                        return_addr: _,
                        seed,
                    } = &mut self.stack[top_idx]
                    else {
                        unreachable!("rposition matched LFrame")
                    };
                    debug_assert_eq!(
                        *top_id, *memo_id,
                        "LRTail id mismatch: expected {:?}, found {:?}",
                        memo_id, top_id,
                    );
                    let start_sp = *start_sp;
                    let snapshot = *snapshot;
                    let body_end_sp = self.sp;
                    let grew = match seed {
                        None => body_end_sp > start_sp,
                        Some(prev) => body_end_sp > prev.end_sp,
                    };
                    if grew {
                        // Snapshot the iteration's captures (closed at
                        // body_end_sp where any still-open ones land) and
                        // store as the new seed; rewind for re-iteration.
                        let body_caps = self.collect_captures_since(snapshot);
                        let new_caps: Vec<Capture> = body_caps
                            .into_iter()
                            .map(|c| Capture {
                                kind: c.kind,
                                start: c.start,
                                end: c.end.unwrap_or(body_end_sp),
                            })
                            .collect();
                        // Re-borrow the LFrame to install the new seed —
                        // the immutable `collect_captures_since` borrow
                        // above ended.
                        if let StackEntry::LFrame { seed, .. } = &mut self.stack[top_idx] {
                            *seed = Some(LSeed {
                                end_sp: body_end_sp,
                                captures: new_caps,
                            });
                        } else {
                            unreachable!("LFrame slot reaffirmed")
                        }
                        self.restore_to(snapshot);
                        self.sp = start_sp;
                        self.ip = body_start.as_index();
                    } else {
                        // No growth — accept the prior seed. If the seed
                        // is still None here (body matched empty on first
                        // try), the rule succeeds with an empty match.
                        let final_seed = seed.take();
                        // Pop the LFrame and the paired memo_examined.
                        let _frame = self.stack.remove(top_idx);
                        let examined = self
                            .memo_examined
                            .pop()
                            .expect("LRTail: memo_examined underflow");
                        self.bump_top_memo_examined(examined);
                        // Restore captures to the LFrame baseline, replay
                        // seed captures (if any), and extract the captures
                        // vec for the cache write so we don't have to
                        // re-clone.
                        self.restore_to(snapshot);
                        let (final_sp, seed_captures) = match final_seed {
                            Some(s) => {
                                for c in &s.captures {
                                    self.push_capture(OpenCapture {
                                        kind: c.kind,
                                        start: c.start,
                                        end: Some(c.end),
                                    });
                                }
                                (s.end_sp, s.captures)
                            }
                            None => (start_sp, Vec::new()),
                        };
                        self.sp = final_sp;
                        // Cache the converged seed. `examined` is the
                        // high-water mark across every iteration of the
                        // seed-and-grow loop (RuleEnter's miss path pushed
                        // one memo_examined slot at entry; growth iterations
                        // don't pop it, only this commit does), so it is the
                        // correct invalidation bound for
                        // `MemoCache::apply_edit`. Failure caching for LR
                        // rules is not yet implemented (#48 scoped to
                        // converged seeds only).
                        self.cache_success(
                            RuleKind::Lr,
                            *memo_id,
                            start_sp,
                            final_sp,
                            examined,
                            seed_captures,
                        );
                        self.maybe_snapshot();
                        self.ip += 1;
                    }
                }
                Instruction::CaptureBegin(kind) => {
                    self.push_capture(OpenCapture {
                        kind: *kind,
                        start: self.sp,
                        end: None,
                    });
                    self.ip += 1;
                }
                Instruction::CaptureEnd => {
                    self.close_top_capture(self.sp);
                    self.ip += 1;
                }
                Instruction::End => {
                    let stats = MemoStats {
                        entries: self.memo.len(),
                        hits: self.memo_hits,
                        misses: self.memo_misses,
                    };
                    let all = self.collect_all_captures();
                    return (
                        MatchResult {
                            matched: self.sp,
                            captures: close_captures(all, self.sp),
                            complete: true,
                        },
                        stats,
                        self.memo,
                    );
                }
            }
        }
    }

    fn pop_backtrack(&mut self) -> StackEntry {
        match self.stack.pop() {
            Some(e @ StackEntry::Backtrack { .. }) => e,
            other => panic!("expected Backtrack on stack top, got {:?}", other),
        }
    }

    fn fail(&mut self) -> bool {
        self.maybe_snapshot();
        while let Some(entry) = self.stack.pop() {
            // Explicit match on all three variants — silently dropping a
            // `Memo` frame would skip caching its failure and leak
            // re-executions on future hits at the same sp.
            match entry {
                StackEntry::Backtrack { ip, sp, snapshot } => {
                    self.ip = ip;
                    self.sp = sp;
                    self.restore_to(snapshot);
                    return true;
                }
                StackEntry::Memo {
                    memo_id,
                    start_sp,
                    snapshot: _,
                } => {
                    // Pop the paired examined watermark. See
                    // `RuleEnter`'s Memo-kind miss path — every
                    // `StackEntry::Memo` push is twinned with a
                    // `memo_examined` push.
                    let examined_max = self
                        .memo_examined
                        .pop()
                        .expect("fail(): memo_examined underflow on Memo frame");
                    // Cache the failure so a future call at the same sp
                    // short-circuits into `fail()` without re-executing the
                    // body. Captures produced inside the rule will be
                    // truncated by whichever `Backtrack` ultimately catches
                    // this unwind (its `snapshot` was taken *before*
                    // `RuleEnter`'s Memo-kind miss path pushed this frame).
                    self.memo.insert(
                        (memo_id, start_sp),
                        MemoEntry {
                            end_sp: None,
                            examined_max,
                            captures: Vec::new(),
                        },
                    );
                    // Propagate to the parent rule: a failure here depended
                    // on input through `examined_max`, and any caller that
                    // ultimately retries a different path still saw those
                    // bytes.
                    self.bump_top_memo_examined(examined_max);
                }
                StackEntry::Return { .. } => {
                    // Rule-call frame unwinding past its caller. The caller's
                    // Backtrack (if any) is deeper on the stack and will be
                    // found by continued popping.
                }
                StackEntry::LFrame {
                    memo_id: _,
                    start_sp: _,
                    snapshot,
                    return_addr,
                    seed,
                } => {
                    // Pop the paired examined watermark. Successful LR
                    // converged seeds are cached at LRTail; LR-rule
                    // failures (this arm with seed=None) are not cached
                    // yet — symmetric with `fail()`'s `Memo` arm but
                    // deferred per #48's scoping. The watermark still
                    // flows up to the parent rule's frame.
                    let examined_max = self
                        .memo_examined
                        .pop()
                        .expect("fail(): memo_examined underflow on LFrame");
                    self.bump_top_memo_examined(examined_max);
                    if let Some(s) = seed {
                        // Body failed on a re-iteration after the seed
                        // already grew at least once. Bounded LR
                        // semantics: accept the prior seed as the rule's
                        // match. Restore captures to the LFrame baseline,
                        // replay the seed's closed captures, set sp to
                        // seed.end_sp, and jump to the rule's Return.
                        // Returning `true` from `fail()` resumes execution
                        // at `self.ip` — the Return frame the caller's
                        // `Call` pushed is still on the stack and will
                        // pop normally.
                        self.restore_to(snapshot);
                        for c in &s.captures {
                            self.push_capture(OpenCapture {
                                kind: c.kind,
                                start: c.start,
                                end: Some(c.end),
                            });
                        }
                        self.sp = s.end_sp;
                        self.ip = return_addr;
                        self.maybe_snapshot();
                        return true;
                    }
                    // No seed yet — the LR rule has failed without ever
                    // growing past bound 0. Continue unwinding past this
                    // frame; the captures-truncate happens at whichever
                    // Backtrack ultimately catches the unwind.
                }
            }
        }
        false
    }

    /// Success-entry insert, with kind-aware threshold policy. Both
    /// `MemoClose` and `LRTail`'s no-growth branch route through this
    /// helper so the policy lives in one place.
    ///
    /// `RuleKind::Memo` (non-LR): apply the `memo_threshold` filter —
    /// skip caching when the matched span is shorter than the
    /// threshold. Tracks GPeg's 512-byte default and Yedidia §5.2.4;
    /// short Memo entries are pure overhead because the hash insert +
    /// lookup cost more than re-executing the rule body.
    ///
    /// `RuleKind::Lr`: always cache, ignoring the threshold. Caching
    /// the converged seed is part of the seed-and-grow algorithm's
    /// short-circuit on subsequent visits at the same `sp`; filtering
    /// short seeds out causes an O(2^N) cascade re-descent in deep LR
    /// ladders (the iter-2 second-alt fallback re-invokes the next
    /// level on operator-failure, which would otherwise have been a
    /// memo hit). See issue #55.
    ///
    /// The captures source differs between callers (live buffer at
    /// `MemoClose`; already-closed `LSeed::captures` at `LRTail`);
    /// both pre-process to a closed `Vec<Capture>` before calling.
    fn cache_success(
        &mut self,
        kind: RuleKind,
        memo_id: MemoId,
        start_sp: usize,
        end_sp: usize,
        examined_max: usize,
        captures: Vec<Capture>,
    ) {
        let should_cache = match kind {
            RuleKind::Lr => true,
            RuleKind::Memo => end_sp - start_sp >= self.memo_threshold,
        };
        if should_cache {
            self.memo.insert(
                (memo_id, start_sp),
                MemoEntry {
                    end_sp: Some(end_sp),
                    examined_max,
                    captures,
                },
            );
        }
    }

    /// Bump the in-flight rule's examined watermark to at least `pos`.
    /// Invoked from every read site (Char/Set/Any/TestChar/TestSet) and
    /// from memo-hit propagation, so a rule's `MemoEntry.examined_max`
    /// reflects lookahead and failed reads past `end_sp`. A no-op when no
    /// `Memo` frame is live (top-level program reads don't feed any
    /// cached entry).
    fn bump_top_memo_examined(&mut self, pos: usize) {
        if let Some(top) = self.memo_examined.last_mut() {
            if pos > *top {
                *top = pos;
            }
        }
    }

    /// Record that the current instruction examines `n` bytes starting at
    /// `self.sp`. The probe happens at the current `sp` whether the read
    /// succeeds (consume) or fails (EOF or mismatch) — in both cases the
    /// outcome depends on bytes up to `sp + n`, so the watermark must
    /// advance there. `n = 1` for Char/Set/TestChar/TestSet; `n` for Any.
    fn track_read(&mut self, n: usize) {
        self.bump_top_memo_examined(self.sp + n);
    }

    /// O(1) snapshot of the current write position: `(current_chunk,
    /// chunk.len, chunk.seq)`. The `seq` lets snapshot consumers detect
    /// slab-slot reuse (`Slab::remove` may reclaim a chunk's slot, and a
    /// later `Slab::insert` would assign a different chunk to it).
    fn snapshot_now(&self) -> CaptureSnapshot {
        let cur = self.chunks.get(self.current_chunk.0);
        CaptureSnapshot {
            chunk: self.current_chunk,
            offset: cur.len,
            chunk_seq: cur.seq,
        }
    }

    /// Allocate a fresh chunk parented at `parent`, advancing the
    /// per-VM seq counter so every chunk gets a unique seq.
    fn allocate_chunk(&mut self, parent: ChunkId) -> ChunkId {
        let seq = self.next_seq;
        self.next_seq += 1;
        ChunkId(self.chunks.insert(Chunk::new(Some(parent), seq)))
    }

    /// Append one `OpenCapture`, allocating a new child chunk when the
    /// current one fills. The active chain extends from `current_chunk`
    /// (now the just-written chunk) back through parent pointers to
    /// `root_chunk`.
    fn push_capture(&mut self, cap: OpenCapture) {
        let cur_id = self.current_chunk.0;
        let cur = self.chunks.get_mut(cur_id);
        if (cur.len as usize) < CHUNK_SIZE {
            cur.captures[cur.len as usize].write(cap);
            cur.len += 1;
        } else {
            let parent = self.current_chunk;
            let new = self.allocate_chunk(parent);
            let nc = self.chunks.get_mut(new.0);
            nc.captures[0].write(cap);
            nc.len = 1;
            self.current_chunk = new;
        }
    }

    /// Close the innermost still-open capture by setting its `end` to
    /// `sp`. Walks current_chunk leaf-first, scanning captures back to
    /// front; on a chunk full of closed captures, follows the parent
    /// pointer. Panics if no open capture exists — the compiler must
    /// guarantee `CaptureBegin`/`CaptureEnd` balance per rule.
    fn close_top_capture(&mut self, sp: usize) {
        let mut cur_id = self.current_chunk.0;
        loop {
            let cur = self.chunks.get_mut(cur_id);
            for i in (0..cur.len as usize).rev() {
                // SAFETY: slots `..len` are initialised by `push_capture`.
                let slot = unsafe { cur.captures[i].assume_init_mut() };
                if slot.end.is_none() {
                    slot.end = Some(sp);
                    return;
                }
            }
            match cur.parent {
                Some(p) => cur_id = p.0,
                None => panic!("CaptureEnd without matching CaptureBegin"),
            }
        }
    }

    /// Update the farthest-failure snapshot if `sp` has advanced past the
    /// previous maximum. Called from the only two sites where `sp`
    /// retreats — [`fail`](Self::fail) and the `BackCommit` handler —
    /// plus defensively at finalize time. Between retreats `sp` is
    /// monotone non-decreasing, so these hooks capture the true deepest
    /// point.
    ///
    /// O(1) per call: stores a `(chunk, offset, seq)` triple. Captures
    /// are materialised lazily via `restore_to`'s endangerment check
    /// when (and only when) a backtrack would drop captures the snapshot
    /// still depends on. Successful parses never pay the materialisation
    /// cost.
    fn maybe_snapshot(&mut self) {
        if self.sp > self.max_sp {
            self.max_sp = self.sp;
            self.max_capture = Some(MaxSnap {
                snapshot: self.snapshot_now(),
                saved: None,
            });
        }
    }

    /// Restore the capture state to `target`: free every chunk on the
    /// chain from `current_chunk` back to `target.chunk` (exclusive),
    /// then truncate `target.chunk.captures` back to `target.offset`.
    /// Sets `current_chunk = target.chunk`.
    ///
    /// If the `MaxSnap` snapshot still depends on captures we are about
    /// to drop — either its chunk lives in the freed segment, or the
    /// truncate would shrink its chunk past `snapshot.offset` —
    /// materialise its captures first via `collect_captures_alive_at`.
    /// Both endangerment cases are detected by a `(seq, offset)`
    /// comparison; the in-chunk case matters because pushes during a
    /// Choice's body may extend `target.chunk` past `target.offset`
    /// before the watermark moves to a deeper chunk.
    fn restore_to(&mut self, target: CaptureSnapshot) {
        let needs_save = match &self.max_capture {
            Some(m) if m.saved.is_none() => {
                let s = m.snapshot;
                target.chunk_seq < s.chunk_seq
                    || (target.chunk_seq == s.chunk_seq && target.offset < s.offset)
            }
            _ => false,
        };
        if needs_save {
            let snap = self
                .max_capture
                .as_ref()
                .expect("needs_save implies max_capture")
                .snapshot;
            let saved = self.collect_captures_alive_at(snap);
            self.max_capture
                .as_mut()
                .expect("needs_save implies max_capture")
                .saved = Some(saved);
        }
        // Walk the chain freeing chunks until current_chunk == target.chunk.
        while self.current_chunk != target.chunk {
            let cur_id = self.current_chunk.0;
            let parent = self
                .chunks
                .get(cur_id)
                .parent
                .expect("restore_to: walked past root looking for target chunk");
            // `Slab::remove` runs `Chunk::drop`, which `assume_init_drop`s
            // every initialised slot. No leaks.
            self.chunks.remove(cur_id);
            self.current_chunk = parent;
        }
        let cur = self.chunks.get_mut(self.current_chunk.0);
        debug_assert_eq!(
            cur.seq, target.chunk_seq,
            "restore_to: target chunk seq mismatch (slab slot reused)"
        );
        // Drop captures in `[target.offset, cur.len)` — they were
        // pushed after the snapshot and are no longer reachable.
        for i in (target.offset as usize)..(cur.len as usize) {
            // SAFETY: slots in `..cur.len` are initialised.
            unsafe { cur.captures[i].assume_init_drop() };
        }
        cur.len = target.offset;
    }

    /// Walk the chunk chain from `current_chunk` back to `snap.chunk`
    /// (inclusive), collecting captures pushed **since** the snapshot —
    /// i.e. `snap.chunk.slots()[snap.offset..]` plus the captures of
    /// every chunk added past it. Used by `MemoClose` and `LRTail`'s
    /// growth path.
    ///
    /// Two passes: sum lengths to pre-allocate the result, then copy.
    fn collect_captures_since(&self, snap: CaptureSnapshot) -> Vec<OpenCapture> {
        // Walk leaf-first, recording each chunk we'll need to read.
        let mut chain: Vec<ChunkId> = Vec::new();
        let mut cur = self.current_chunk;
        loop {
            chain.push(cur);
            if cur == snap.chunk {
                break;
            }
            cur = self
                .chunks
                .get(cur.0)
                .parent
                .expect("collect_captures_since: snap.chunk not in current chain");
        }
        debug_assert_eq!(
            self.chunks.get(snap.chunk.0).seq,
            snap.chunk_seq,
            "collect_captures_since: snap.chunk seq mismatch (slab slot reused)"
        );
        let mut total: usize = 0;
        for (i, id) in chain.iter().enumerate() {
            let c = self.chunks.get(id.0);
            // The deepest entry (i == chain.len() - 1) is `snap.chunk`,
            // and only its post-snapshot suffix counts.
            let take = if i == chain.len() - 1 {
                (c.len as usize).saturating_sub(snap.offset as usize)
            } else {
                c.len as usize
            };
            total += take;
        }
        let mut out: Vec<OpenCapture> = Vec::with_capacity(total);
        for id in chain.iter().rev() {
            let c = self.chunks.get(id.0);
            if *id == snap.chunk {
                out.extend_from_slice(&c.slots()[snap.offset as usize..]);
            } else {
                out.extend_from_slice(c.slots());
            }
        }
        out
    }

    /// Walk **up** from `snap.chunk` to the root, collecting captures
    /// that were **alive at the moment of the snapshot** — i.e.
    /// `snap.chunk.slots()[..snap.offset]` plus the full `slots()` of
    /// every ancestor (each ancestor's captures stop growing the moment
    /// the writer moves to a child chunk, so the live `len` *is* its
    /// alive-at-snap count). Used by `finalize_partial` and
    /// `restore_to`'s `MaxSnap` preservation path — the snapshot's
    /// "below" timeline, dual to `collect_captures_since`'s "above".
    fn collect_captures_alive_at(&self, snap: CaptureSnapshot) -> Vec<OpenCapture> {
        debug_assert_eq!(
            self.chunks.get(snap.chunk.0).seq,
            snap.chunk_seq,
            "collect_captures_alive_at: snap.chunk seq mismatch (slab slot reused)"
        );
        let mut chain: Vec<ChunkId> = Vec::new();
        let mut cur = snap.chunk;
        loop {
            chain.push(cur);
            match self.chunks.get(cur.0).parent {
                Some(p) => cur = p,
                None => break,
            }
        }
        let mut total: usize = 0;
        for (i, id) in chain.iter().enumerate() {
            let c = self.chunks.get(id.0);
            let take = if i == 0 {
                snap.offset as usize
            } else {
                c.len as usize
            };
            total += take;
        }
        let mut out: Vec<OpenCapture> = Vec::with_capacity(total);
        for id in chain.iter().rev() {
            let c = self.chunks.get(id.0);
            if *id == snap.chunk {
                out.extend_from_slice(&c.slots()[..snap.offset as usize]);
            } else {
                out.extend_from_slice(c.slots());
            }
        }
        out
    }

    /// Collect every live capture from `root_chunk` to `current_chunk`,
    /// in `CaptureBegin` order. Used by the `End` instruction to
    /// extract the final result on a successful parse.
    fn collect_all_captures(&self) -> Vec<OpenCapture> {
        let root_seq = self.chunks.get(self.root_chunk.0).seq;
        self.collect_captures_since(CaptureSnapshot {
            chunk: self.root_chunk,
            offset: 0,
            chunk_seq: root_seq,
        })
    }

    fn finalize_partial(mut self) -> (MatchResult, MemoStats, HashMap<(MemoId, usize), MemoEntry>) {
        self.maybe_snapshot();
        let stats = MemoStats {
            entries: self.memo.len(),
            hits: self.memo_hits,
            misses: self.memo_misses,
        };
        // Materialise the farthest-failure captures. If `restore_to`
        // already preserved them (because a fail dropped captures the
        // snapshot needed), use that copy. Otherwise the snapshot still
        // points at live chunks — walk **up** the ancestor chain.
        let max_captures = match self.max_capture.take() {
            Some(MaxSnap {
                saved: Some(saved), ..
            }) => saved,
            Some(MaxSnap {
                snapshot,
                saved: None,
            }) => self.collect_captures_alive_at(snapshot),
            None => Vec::new(),
        };
        let result = MatchResult {
            matched: self.max_sp,
            captures: close_captures(max_captures, self.max_sp),
            complete: false,
        };
        (result, stats, self.memo)
    }
}

fn close_captures(open: Vec<OpenCapture>, close_at: usize) -> Vec<Capture> {
    open.into_iter()
        .map(|c| Capture {
            kind: c.kind,
            start: c.start,
            end: c.end.unwrap_or(close_at),
        })
        .collect()
}

#[cfg(test)]
mod examined_max_tests {
    //! Verify that every memo entry records the farthest input position
    //! its rule invocation ever examined, including lookahead past
    //! `end_sp` and failed reads. Incremental parsing's invalidation
    //! predicate (`edit.start <= entry.examined_max`) depends on this
    //! bound being tight-enough to be useful and safe-at-minimum to be
    //! correct.
    //!
    //! Tests access `VM::run_core` directly to inspect the private
    //! memo map — post-run the public `run_with_memo_stats` has already
    //! discarded it.
    use super::*;
    use crate::pegc::{Grammar, Pattern};
    use std::collections::HashMap;

    fn rule(rules: &mut HashMap<String, Pattern>, name: &str, pat: Pattern) {
        rules.insert(name.into(), pat);
    }

    #[test]
    fn and_predicate_success_records_examined_past_end_sp() {
        // start <- "x" &"y"   against "xy"
        // Consumed span is "x" (end_sp = 1) but the &"y" lookahead
        // read position 1, so examined_max must be 2.
        let mut rules = HashMap::new();
        rule(
            &mut rules,
            "start",
            Pattern::seq(vec![
                Pattern::literal("x"),
                Pattern::AndPredicate(Box::new(Pattern::literal("y"))),
            ]),
        );
        let prog = Grammar::new(rules, "start").compile().unwrap();
        let (result, _stats, memo) = VM::new(&prog.code, b"xy").with_memo_threshold(0).run_core();
        assert!(result.complete);
        assert_eq!(result.matched, 1, "only 'x' is consumed");
        let entry = memo
            .get(&(MemoId(0), 0))
            .expect("start's memo entry missing");
        assert_eq!(entry.end_sp, Some(1));
        assert_eq!(
            entry.examined_max, 2,
            "&\"y\" read past end_sp; examined_max must reflect it"
        );
    }

    #[test]
    fn and_predicate_failure_records_examined_up_to_failed_read() {
        // start <- "x" &"z"   against "xy"
        // "x" succeeds (sp=1), then &"z" reads 'y' at sp=1 and fails.
        // The failure entry for start at sp=0 must remember that
        // position 1 was examined, so an edit there can invalidate it.
        let mut rules = HashMap::new();
        rule(
            &mut rules,
            "start",
            Pattern::seq(vec![
                Pattern::literal("x"),
                Pattern::AndPredicate(Box::new(Pattern::literal("z"))),
            ]),
        );
        let prog = Grammar::new(rules, "start").compile().unwrap();
        let (result, _stats, memo) = VM::new(&prog.code, b"xy").with_memo_threshold(0).run_core();
        assert!(!result.complete, "overall parse must fail");
        let entry = memo
            .get(&(MemoId(0), 0))
            .expect("start's failure entry missing");
        assert_eq!(entry.end_sp, None);
        assert_eq!(
            entry.examined_max, 2,
            "failed read of 'z' at position 1 examined position 1+1"
        );
    }

    #[test]
    fn nested_rule_examined_max_propagates_to_caller() {
        // outer <- inner "y"
        // inner <- "x"
        // against "xy".  inner's entry: end_sp=1, examined_max=1.
        // outer's entry: end_sp=2, examined_max=2 (reads 'y' at sp=1).
        // The propagation test: inner's examined_max (1) must flow into
        // outer's watermark when inner's MemoClose pops.
        let mut rules = HashMap::new();
        rule(
            &mut rules,
            "outer",
            Pattern::seq(vec![
                Pattern::NonTerminal("inner".into()),
                Pattern::literal("y"),
            ]),
        );
        rule(&mut rules, "inner", Pattern::literal("x"));
        let prog = Grammar::new(rules, "outer").compile().unwrap();
        let (result, _stats, memo) = VM::new(&prog.code, b"xy").with_memo_threshold(0).run_core();
        assert!(result.complete);
        assert_eq!(result.matched, 2);
        // outer is start → MemoId(0); inner is the other → MemoId(1).
        let outer = memo.get(&(MemoId(0), 0)).expect("outer entry missing");
        assert_eq!(outer.end_sp, Some(2));
        assert_eq!(outer.examined_max, 2);
        let inner = memo.get(&(MemoId(1), 0)).expect("inner entry missing");
        assert_eq!(inner.end_sp, Some(1));
        assert_eq!(inner.examined_max, 1);
    }

    #[test]
    fn memo_hit_propagates_examined_max_to_caller() {
        // Two alternatives both start with the memoized rule X.
        // The first alternative's success call to X populates the
        // cache with examined_max = 1; the second alternative's call
        // is a hit. The hit must still contribute X's examined_max to
        // whichever rule encloses the call.
        //
        // Grammar:
        //   start <- (X "aa") / (X "bb")
        //   X <- "a"
        // Input: "abb" — first alt fails after X matches "a" and "aa"
        // fails on "bb"; backtrack to second alt, which hits X's cache
        // and then matches "bb".
        let mut rules = HashMap::new();
        rule(
            &mut rules,
            "start",
            Pattern::choice(vec![
                Pattern::seq(vec![
                    Pattern::NonTerminal("X".into()),
                    Pattern::literal("aa"),
                ]),
                Pattern::seq(vec![
                    Pattern::NonTerminal("X".into()),
                    Pattern::literal("bb"),
                ]),
            ]),
        );
        rule(&mut rules, "X", Pattern::literal("a"));
        let prog = Grammar::new(rules, "start").compile().unwrap();
        let (result, stats, memo) = VM::new(&prog.code, b"abb")
            .with_memo_threshold(0)
            .run_core();
        assert!(result.complete);
        assert_eq!(result.matched, 3);
        assert!(stats.hits >= 1, "second alternative must hit X's cache");
        // start's examined_max must include every byte start's execution
        // ever looked at: position 2 was examined when the first
        // alternative tried "aa" at sp=1 and "bb" at sp=1 needed byte 2.
        let start = memo.get(&(MemoId(0), 0)).expect("start entry missing");
        assert_eq!(start.end_sp, Some(3));
        assert_eq!(
            start.examined_max, 3,
            "start's execution examined up to position 3"
        );
    }

    #[test]
    fn recover_repeat_propagates_examined_max_through_loop_iterations() {
        // start <- ("ab")*^   against "abxab"
        //
        // The recovery loop runs entirely inside start's memo entry.
        // Across iterations the loop reads positions 0, 1 (success),
        // 2 (Char 'a' fails on 'x'), 2 (Any(1) consumes 'x' → sp=3),
        // 3, 4 (success), 5 (Char 'a' at EOF). Whether the failed
        // EOF read at sp=5 contributes depends on track_read's call
        // discipline, so the assertion is a lower bound: every
        // position the loop *successfully* read must appear in
        // examined_max.
        let mut rules = HashMap::new();
        rule(
            &mut rules,
            "start",
            Pattern::RecoverRepeat {
                inner: Box::new(Pattern::literal("ab")),
                recovery_kind: "recovery".into(),
            },
        );
        let prog = Grammar::new(rules, "start").compile().unwrap();
        let (result, _stats, memo) = VM::new(&prog.code, b"abxab")
            .with_memo_threshold(0)
            .run_core();
        assert!(result.complete);
        assert_eq!(result.matched, 5);
        let entry = memo.get(&(MemoId(0), 0)).expect("start entry missing");
        assert_eq!(entry.end_sp, Some(5));
        assert!(
            entry.examined_max >= 5,
            "recovery-loop reads must propagate to enclosing rule's watermark; got {}",
            entry.examined_max
        );
    }
}
