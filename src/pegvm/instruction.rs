/// Index into a compiled `Program`'s instruction array — i.e. a code address.
///
/// A newtype rather than a bare integer so it cannot be silently confused with
/// the subject pointer (`sp`), an array length, or any other index. The inner
/// width is `u32`: smaller than `usize` on 64-bit targets, which shrinks every
/// `Instruction` variant carrying a Label and is plenty for any realistic
/// program (the largest grammar in this repo compiles to ~6 K instructions,
/// well below `u32::MAX`). Consumers reach for [`Label::as_index`] at the
/// boundary where a Label flows into the instruction pointer or the
/// backtrack stack.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Label(pub u32);

impl Label {
    /// Widen the inner `u32` to `usize` for indexing. The conversion is
    /// always lossless on every target Rust supports (`usize` is at least
    /// 32 bits everywhere). Centralizes the cast at the type's boundary
    /// so VM dispatch sites read as `self.ip = label.as_index()` rather
    /// than scattering `label.0 as usize` across the loop body.
    #[inline]
    pub const fn as_index(self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Debug for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "L{}", self.0)
    }
}

/// Opaque tag identifying a capture kind. The VM does not interpret it; it
/// flows from `Compiler::intern_capture` (which assigns a fresh id per name)
/// through the bytecode and out as part of each emitted `Capture`. The
/// highlighter looks the id up in the `Program::capture_kinds` table to get
/// the original name.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct CaptureKind(pub u16);

/// Opaque tag identifying a memoized rule. Assigned 1:1 with rule addresses
/// by [`crate::pegc::Grammar::compile`]. The VM uses it as an index into
/// its memo table.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct MemoId(pub u32);

/// Opaque tag identifying a recovery-scope label. Assigned by
/// `Compiler::intern_label` and threaded through each
/// `RecoverScopeBegin` instruction. The label is a diagnostic tag
/// only: it flows into `RecoveryDiagnostic.label` so `pegdb
/// explain-recoveries` clusters firings by it. The name resolves via
/// [`crate::pegvm::Program::label_kinds`]. Distinct namespace from
/// `CaptureKind` so a label and a capture kind with the same name
/// don't collide.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct LabelId(pub u16);

/// Discriminator on a [`Instruction::RuleEnter`] selecting the post-cache-miss
/// behavior. The cache-hit prologue is identical for both kinds; only the
/// miss path differs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleKind {
    /// Plain packrat memoization. On a cache miss the VM pushes a
    /// `StackEntry::Memo` frame and falls through to the rule body;
    /// `MemoClose` commits a success entry, `fail()`'s `Memo` arm
    /// commits a failure entry on escape.
    Memo,
    /// Bounded left recursion. On a cache miss the VM walks the live
    /// stack for an in-flight `LFrame` at the same `(memo_id, sp)`;
    /// a recursive re-entry replays the prior seed (or `fail()`s if no
    /// seed exists yet). First entry pushes a fresh `LFrame`; `LRTail`
    /// commits the converged seed.
    Lr,
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CharSet([u8; 32]);

impl CharSet {
    pub const fn empty() -> Self {
        CharSet([0; 32])
    }

    pub fn full() -> Self {
        CharSet([0xFF; 32])
    }

    pub fn contains(&self, byte: u8) -> bool {
        self.0[(byte / 8) as usize] & (1 << (byte % 8)) != 0
    }

    pub fn add(&mut self, byte: u8) {
        self.0[(byte / 8) as usize] |= 1 << (byte % 8);
    }

    /// Adds every byte value in the inclusive range `lo..=hi` to the set.
    ///
    /// Callers are expected to pass `lo <= hi`. An inverted range would silently
    /// be a no-op (because `lo..=hi` is empty in that case), which would hide a
    /// caller bug — hence the debug-build assertion. In release builds, inverted
    /// input still does nothing. For a single byte, prefer `add(b)`.
    pub fn add_range(&mut self, lo: u8, hi: u8) {
        debug_assert!(
            lo <= hi,
            "CharSet::add_range: inverted range lo=0x{:02x} hi=0x{:02x} (use add() for a single byte)",
            lo,
            hi
        );
        for b in lo..=hi {
            self.add(b);
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut s = CharSet::empty();
        for &b in bytes {
            s.add(b);
        }
        s
    }

    pub fn from_ranges(ranges: &[(u8, u8)]) -> Self {
        let mut s = CharSet::empty();
        for &(lo, hi) in ranges {
            s.add_range(lo, hi);
        }
        s
    }

    pub fn negate(&self) -> Self {
        let mut out = CharSet::empty();
        for i in 0..32 {
            out.0[i] = !self.0[i];
        }
        out
    }

    pub fn union(&self, other: &CharSet) -> Self {
        let mut out = CharSet::empty();
        for i in 0..32 {
            out.0[i] = self.0[i] | other.0[i];
        }
        out
    }

    /// Borrow the raw 256-bit bitmap. The set encodes which of the 256
    /// possible `u8` values it contains; bit `b % 8` of byte `b / 8` is
    /// set iff `contains(b)`. Used by `pegb` to round-trip a `CharSet`
    /// through bytes; not generally useful otherwise.
    pub const fn bitmap(&self) -> &[u8; 32] {
        &self.0
    }

    /// Construct a `CharSet` from a raw 256-bit bitmap. Inverse of
    /// `bitmap()`. Distinct from `from_bytes(&[u8])`, which takes a list
    /// of byte *values* to insert; this takes the bitmap directly.
    pub const fn from_bitmap(bytes: [u8; 32]) -> Self {
        CharSet(bytes)
    }
}

impl std::fmt::Debug for CharSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CharSet[")?;
        let mut first = true;
        let mut i = 0u16;
        while i < 256 {
            if self.contains(i as u8) {
                let lo = i;
                while i < 256 && self.contains(i as u8) {
                    i += 1;
                }
                let hi = i - 1;
                if !first {
                    write!(f, ",")?;
                }
                first = false;
                if lo == hi {
                    write!(f, "{}", fmt_byte(lo as u8))?;
                } else {
                    write!(f, "{}-{}", fmt_byte(lo as u8), fmt_byte(hi as u8))?;
                }
            } else {
                i += 1;
            }
        }
        write!(f, "]")
    }
}

fn fmt_byte(b: u8) -> String {
    if b.is_ascii_graphic() || b == b' ' {
        format!("'{}'", b as char)
    } else {
        format!("0x{:02x}", b)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Char(u8),
    Set(CharSet),
    Any(u8),
    TestChar(u8, Label),
    TestSet(CharSet, Label),

    Jump(Label),
    Choice(Label),
    Commit(Label),
    PartialCommit(Label),
    BackCommit(Label),
    FailTwice,
    Fail,

    Call(Label),
    Return,

    /// Rule-entry prologue. Probes the packrat cache at `(memo_id, sp)`:
    /// on a hit replays the cached captures, advances `sp` to the cached
    /// end, and jumps to the `Label` (the rule's `Return` address); on a
    /// miss the post-cache behavior depends on the [`RuleKind`]:
    /// - [`RuleKind::Memo`]: pushes a `StackEntry::Memo` frame and falls
    ///   through to the rule body. `MemoClose` commits the entry on
    ///   success; `fail()`'s `Memo` arm commits a failure entry on escape.
    /// - [`RuleKind::Lr`]: walks the stack for an in-flight `LFrame` at
    ///   `(memo_id, sp)`. A recursive re-entry replays the prior seed
    ///   (`seed: None` ⇒ `fail()`; `seed: Some` ⇒ jump to `Label`); first
    ///   entry pushes a fresh `LFrame { seed: None, return_addr: Label }`
    ///   and falls through. `LRTail` commits the converged seed.
    RuleEnter(MemoId, RuleKind, Label),
    /// Rule-level memoization epilogue. Pops the matching `StackEntry::Memo`
    /// frame and records a success entry for this rule at the frame's
    /// `start_sp` (subject to the memo-threshold filter).
    MemoClose(MemoId),
    /// Left-recursion iteration controller. Sits between the body and the
    /// rule's `Return`. Peeks the topmost `LFrame` (must match `MemoId`).
    /// Decision:
    /// - Body grew (`sp > seed.end_sp`, or first success with seed `None`):
    ///   update seed to `Some(sp, captures-since-baseline)`, rewind to
    ///   `start_sp`, truncate captures to the baseline, jump to the `Label`
    ///   (body start) to re-run.
    /// - No growth: apply the seed (replay captures, set `sp = seed.end_sp`),
    ///   pop the `LFrame`, write the converged seed to the packrat cache
    ///   (subject to the threshold filter), and fall through to `Return`.
    LRTail(MemoId, Label),

    CaptureBegin(CaptureKind),
    CaptureEnd,

    /// Push a fresh `RecoverScope` frame capturing the current
    /// `(sp, captures.len())` as the iteration baseline. Emitted at the
    /// top of every `p*^` iteration and at the head of every catch
    /// (`inner ^label recovery`) so the VM can track the deepest
    /// captures the failed inner attempt produced — see
    /// `RecoverToScopedMax` and `src/pegc/compiler.rs`.
    ///
    /// The `LabelId` tags the scope with the catch's diagnostic label.
    /// `*^` / `*^[cs]` desugar to a catch labeled `"recovery"`, so every
    /// `RecoveryDiagnostic` carries a name. Used by `pegdb
    /// explain-recoveries` to cluster recoveries by label alongside
    /// rule stack.
    RecoverScopeBegin(LabelId),
    /// Materialize the topmost `RecoverScope`'s deepest-progress
    /// captures into the live capture buffer past the iteration's
    /// `baseline_capture_len`, and advance `sp` to `scoped_max_sp`.
    /// Emitted at the head of `p*^`'s recovery branch so the following
    /// `Any(1)` emits a recovery span starting *after* the partially
    /// matched prefix instead of swallowing it. Does not pop the scope.
    RecoverToScopedMax,
    /// Pop the topmost `RecoverScope` frame. Emitted on every edge that
    /// leaves a `p*^` iteration so each `RecoverScopeBegin` is
    /// brace-matched.
    RecoverScopeEnd,

    End,
}
