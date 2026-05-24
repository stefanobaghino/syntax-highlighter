//! Grammar source → [`crate::pegvm::Program`] compiler.
//!
//! Symmetric counterpart to [`crate::pegvm`]: `pegvm` *runs* bytecode,
//! `pegc` *compiles* grammar source to bytecode. Depends on `pegvm`
//! (for [`Program`], [`Instruction`](crate::pegvm::Instruction), etc.);
//! `pegvm` has no reverse dependency.
//!
//! # Deep-module entry point
//!
//! [`compile`] takes grammar source and returns a runnable
//! [`Program`] with the two-step parse + compile folded into one
//! call. [`Error`] unifies the parse and compile failures behind one
//! type. This is the shape the vast majority of callers want.
//!
//! ```no_run
//! # use syntax_highlighter::pegc;
//! let program = pegc::compile(include_str!("../../grammars/json.peg")).unwrap();
//! ```
//!
//! # Lower-level surface
//!
//! Callers who need finer composition — inspecting the AST between
//! parse and compile, compiling a single pattern without a named
//! grammar, or building a [`Grammar`] from a hand-built rule map in a
//! test — can reach for the primitives directly: [`parse`],
//! [`Grammar::new`], [`Grammar::compile`], [`compile_pattern`], and
//! [`Pattern`].

pub mod analysis;
pub mod compiler;
pub mod parser;
pub mod pattern;
pub mod unicode_properties;

pub use analysis::{tally_non_terminal_refs, LintFinding, LintKind};
pub use compiler::{compile_pattern, CompileError};
pub use parser::{parse, Grammar, ParseError, RuleHeader};
pub use pattern::{Pattern, Span};

use crate::pegvm::Program;

/// Unified failure mode for [`compile`]. Wraps the distinct
/// grammar-source [`ParseError`] and bytecode [`CompileError`] types
/// so one-step callers learn one error type rather than two.
#[derive(Debug)]
pub enum Error {
    Parse(ParseError),
    Compile(CompileError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "{}", e),
            Error::Compile(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::Parse(e)
    }
}

impl From<CompileError> for Error {
    fn from(e: CompileError) -> Self {
        Error::Compile(e)
    }
}

/// Compile grammar source straight to a runnable [`Program`]. The
/// deep one-step entry point — folds [`parse`] and [`Grammar::compile`]
/// into a single call and unifies their error types behind [`Error`].
pub fn compile(source: &str) -> Result<Program, Error> {
    Ok(parse(source)?.compile()?)
}
