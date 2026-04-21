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
    /// Number of memoized rules. Each rule gets a distinct `MemoId` in the
    /// range `0..memo_count`, assigned in compilation order, so the VM can
    /// size its memo table once up front.
    pub memo_count: usize,
}
