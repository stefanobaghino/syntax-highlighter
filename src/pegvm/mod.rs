pub mod incremental;
pub mod instruction;
pub mod program;
mod slab;
pub mod vm;

pub use incremental::{Edit, MemoCache};
pub use instruction::{CaptureKind, CharSet, Instruction, Label, MemoId, RuleKind};
pub use program::Program;
pub use vm::{Capture, MatchResult, MemoStats, VM};
