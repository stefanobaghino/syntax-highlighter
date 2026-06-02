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

/// Opaque tag identifying a character set. Assigned by the compiler
/// when it interns a [`super::CharSet`] into
/// [`super::Program::char_sets`]. The VM uses it as an index into that
/// table when dispatching [`Instruction::CharSet`]. Distinct namespace
/// from [`CaptureKind`] and [`LabelId`].
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct SetId(pub u16);

/// Opaque tag identifying a recovery-scope label. Assigned by
/// `Compiler::intern_label` and threaded through each
/// `RecoverScopeBegin` instruction. The label is a diagnostic tag
/// only: it flows into `RecoveryDiagnostic.label` so `pegdb
/// recoveries explain` clusters firings by it. The name resolves via
/// [`crate::pegvm::Program::label_kinds`]. Distinct namespace from
/// `CaptureKind` so a label and a capture kind with the same name
/// don't collide.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct LabelId(pub u16);

/// Relation tested by [`Instruction::IndentCmp`] between a measured
/// indentation width (left) and a reference column (right). All three
/// are the relations the declarative indent combinators desugar to:
/// `deeper` → [`CmpOp::Gt`], `at_least` → [`CmpOp::Ge`], `same` →
/// [`CmpOp::Eq`]. A false relation routes through `fail()` like any
/// other match miss.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmpOp {
    /// Strictly deeper: `measured > reference`.
    Gt,
    /// At least as deep: `measured >= reference`.
    Ge,
    /// Exactly aligned: `measured == reference`.
    Eq,
}

/// Source of an integer argument for [`Instruction::ArgPush`],
/// [`Instruction::IndentCmp`], and the rule-call convention. Either a
/// compile-time literal column or a read from the current activation's
/// local slots (params first, then `as`-bound indent widths).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgSrc {
    /// A literal column value (e.g. the `0` in `block(0)`).
    Lit(i32),
    /// Read activation slot `n` of the current rule frame: index
    /// `locals_base + n` into the VM's locals arena.
    Local(u8),
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    /// Consume one input byte exactly equal to the payload. Byte-faithful
    /// for literal matching (each byte of a UTF-8-encoded literal compiles
    /// to one of these); the code-point-aware opcodes are `CharSet` and
    /// `Any`.
    Byte(u8),
    /// Consume one Unicode scalar at the current input pointer
    /// (1..=4 bytes via WHATWG UTF-8 decode). On invalid UTF-8
    /// emits a `RecoveryOrigin::Utf8` capture and skips the maximal
    /// invalid prefix; failure at EOF routes through `fail()`.
    Any,
    /// Match one Unicode scalar against the set at
    /// [`super::Program::char_sets`]`[id.0]`.
    ///
    /// Decodes UTF-8 at the current input pointer (1..=4 bytes,
    /// WHATWG-style). On a valid decode whose scalar is in the set,
    /// advances by the decoded byte width. On a valid decode whose
    /// scalar is not in the set, fails like any other match opcode.
    /// On invalid UTF-8 the opcode **succeeds-by-recovery**: emits a
    /// `recovery`-kind capture with [`super::RecoveryOrigin::Utf8`]
    /// over the maximal invalid prefix (coalesced with an immediately
    /// preceding UTF-8 recovery span), advances past the bad bytes,
    /// and proceeds as if a code point had matched.
    CharSet(SetId),

    Jump(Label),
    Choice(Label),
    Commit(Label),
    PartialCommit(Label),
    BackCommit(Label),
    FailTwice,
    Fail,

    Call(Label),
    Return,

    /// Measure the indentation run `[ \t]*` at the current input pointer,
    /// **consuming it forward** (advancing `sp`, with each byte counted
    /// against the enclosing rule's `examined_max` watermark), and store
    /// the column width into activation slot `n` (index `locals_base + n`
    /// of the VM's locals arena, growing it with zeros if `n` is past the
    /// current frame's end). One column per space *or* tab — a documented
    /// simplification (no tab-stop expansion).
    ///
    /// Forward-consuming is the incremental-soundness lynchpin: because
    /// the whitespace is read through the normal `track_read` path, an
    /// edit anywhere in the measured run is covered by the memo entry's
    /// `examined_max`, so warm reparses invalidate correctly. A backward
    /// scan to the previous newline would create a dependency on bytes
    /// before `start_sp` that the incremental invalidator cannot see.
    IndentMeasure(u8),
    /// Compare activation slot `n` (a previously [`IndentMeasure`]d width)
    /// against a reference column under the [`CmpOp`] relation. The
    /// reference comes from the [`ArgSrc`] (a literal or another local).
    /// A false relation routes to `fail()`; a true one falls through. The
    /// slot is left intact so a single measured width can be both compared
    /// and (via the same slot) referenced later — e.g. `deeper(outer) as i`
    /// measures into `i`, compares `i > outer`, and leaves `i` bound for
    /// the suite's `same(i)` lines.
    ///
    /// [`IndentMeasure`]: Instruction::IndentMeasure
    IndentCmp(CmpOp, u8, ArgSrc),
    /// Push one integer argument onto the VM's argument stack ahead of a
    /// `Call`/`RuleEnter` to a parameterized rule. The matching
    /// `RuleEnter` pops `argc` of these to form its memo key and seed the
    /// callee's parameter slots. Emitted in left-to-right argument order,
    /// so the callee's slot `0` is the first argument.
    ArgPush(ArgSrc),

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
    ///
    /// The trailing `u8` is the rule's **arg count** (`argc`). The
    /// prologue pops that many values off the VM's argument stack (pushed
    /// by the caller's `ArgPush` run) to form the [`MemoId`]-keyed memo
    /// probe's argument component, and — on a miss — seeds the callee's
    /// first `argc` activation slots with them. `argc = 0` is exactly the
    /// pre-parameterization behavior: no args popped, an empty
    /// (allocation-free) arg key, and no slot seeding — the hot path every
    /// non-indentation grammar takes.
    RuleEnter(MemoId, RuleKind, Label, u8),
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
    /// recoveries explain` to cluster recoveries by label alongside
    /// rule stack.
    RecoverScopeBegin(LabelId),
    /// Materialize the topmost `RecoverScope`'s deepest-progress
    /// captures into the live capture buffer past the iteration's
    /// `baseline_capture_len`, and advance `sp` to `scoped_max_sp`.
    /// Emitted at the head of `p*^`'s recovery branch so the following
    /// `Any` emits a recovery span starting *after* the partially
    /// matched prefix instead of swallowing it. Does not pop the scope.
    RecoverToScopedMax,
    /// Pop the topmost `RecoverScope` frame. Emitted on every edge that
    /// leaves a `p*^` iteration so each `RecoverScopeBegin` is
    /// brace-matched.
    RecoverScopeEnd,

    End,
}
