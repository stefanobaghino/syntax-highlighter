use super::charset::CharSet;
use super::instruction::Instruction;

/// Runnable bytecode produced by the grammar compiler. A `Program` is
/// grammar-agnostic at runtime: the [`VM`](crate::pegvm::VM) only needs
/// the instructions, the interned capture names, and the memo table
/// size. Grammar source parsing and compilation live in
/// [`crate::grammar`].
#[derive(Debug, Clone)]
pub struct Program {
    pub code: Vec<Instruction>,
    pub capture_kinds: Vec<String>,
    /// Number of rules in the grammar. Each rule gets a distinct `MemoId`
    /// in the range `0..rule_count`, assigned by [`crate::pegc`] in
    /// compilation order. Today this is read by `pegc stats` and a
    /// handful of compiler tests; the VM uses a `HashMap`-backed
    /// [`crate::pegvm::MemoCache`] and doesn't size by `rule_count`.
    /// [`crate::pegb::decode`] reconstructs the value from the
    /// instruction stream rather than carrying it on the wire.
    pub rule_count: usize,
    /// Compile-time rule names, indexed by `MemoId.0`. Populated by
    /// [`crate::pegc`] in the same order rules receive their `MemoId`s
    /// (start rule first, then alphabetical). Used by diagnostic
    /// consumers (e.g. `pegdb recoveries dump`'s `rule_stack` column
    /// and `pegdb recoveries explain`'s leaf display) to render
    /// rule-stack snapshots back into human-readable names. Length is
    /// exactly `rule_count`.
    pub rule_names: Vec<String>,
    /// Compile-time label names, indexed by `LabelId.0`. Populated by
    /// [`crate::pegc`] in the order labels are interned (which is
    /// deterministic given a fixed compile-time pattern walk). The
    /// `recoveries` subcommands resolve the label of a labeled
    /// recovery firing back to its source name. Empty for grammars
    /// that don't use labeled failures.
    pub label_kinds: Vec<String>,
    /// Per-rule trivia flags, indexed by `MemoId.0`. `true` when the
    /// rule is reachable from a `trivia <- …` reserved-name root in
    /// the grammar; a rule containing a recovery catch is pinned out
    /// of inference so the catch's frame stays visible. The
    /// `recoveries` subcommands consult this to drop trailing trivia
    /// frames from the rule_stack when picking the displayed leaf of
    /// a recovery cluster or span. Length is exactly `rule_count`.
    pub rule_is_trivia: Vec<bool>,
    /// Character sets interned by the compiler. Indexed by
    /// [`crate::pegvm::SetId`]; each
    /// [`crate::pegvm::Instruction::CharSet`] payload is the index of
    /// one entry here. Empty for grammars that don't use any character
    /// class. The VM dispatch arm pays a single slice-index lookup per
    /// `CharSet` instruction.
    pub char_sets: Vec<CharSet>,
}
