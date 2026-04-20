/// Index into a compiled `Program`'s instruction array — i.e. a code address.
///
/// A newtype rather than a bare `usize` so it cannot be silently confused with
/// the subject pointer (`sp`), an array length, or any other index. Wrapping
/// is a `#[repr(transparent)]` newtype: zero runtime cost, identical layout to
/// `usize`. See `CLAUDE.md` for the project-wide convention behind this choice.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Label(pub usize);

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
/// by `compile_grammar`. The VM uses it as an index into its memo table.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct MemoId(pub u32);

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

    /// Rule-level memoization prologue. On a cache hit the VM advances `sp`
    /// to the cached end and jumps to the `Label` (the rule's `Return`
    /// address); on a miss it pushes a `StackEntry::Memo` frame and falls
    /// through to the rule body.
    MemoOpen(MemoId, Label),
    /// Rule-level memoization epilogue. Pops the matching `StackEntry::Memo`
    /// frame and records a success entry for this rule at the frame's
    /// `start_sp`.
    MemoClose(MemoId),

    CaptureBegin(CaptureKind),
    CaptureEnd,

    End,
}
