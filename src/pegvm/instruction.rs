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

    End,
}
