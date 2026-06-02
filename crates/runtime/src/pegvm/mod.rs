pub mod charset;
pub mod incremental;
pub mod instruction;
pub mod program;
pub mod utf8;
pub mod vm;

pub use charset::{CharSet, CharSetError};
pub use incremental::{Edit, MemoCache};
pub use instruction::{
    ArgSrc, CaptureKind, CmpOp, Instruction, Label, LabelId, MemoId, RuleKind, SetId,
};
pub use program::Program;
pub use vm::{Capture, MatchResult, MemoStats, RecoveryDiagnostic, RecoveryOrigin, VM};

// Re-exported for integration tests that inspect the VM's memo table
// directly. Both are `#[doc(hidden)]` at the definition site; the memo
// table is not a stable surface.
#[doc(hidden)]
pub use vm::{ArgKey, MemoEntry};
