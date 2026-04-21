//! Grammar source → [`crate::pegvm::Program`] pipeline.
//!
//! Separates the grammar-parsing and compilation concerns from the
//! bytecode VM primitives in [`crate::pegvm`]. The VM is a candidate
//! for extraction (see `ARCHITECTURE.md`); keeping grammar parsing out
//! of it preserves that option.
//!
//! Entry points:
//!
//! - [`parse`] — grammar source text → [`Grammar`] (rules AST).
//! - [`compile`] — [`Grammar`] rules → a runnable
//!   [`crate::pegvm::Program`].
//! - [`compile_pattern`] — single [`Pattern`] (no non-terminal refs) →
//!   [`crate::pegvm::Program`].

pub mod compiler;
pub mod parser;
pub mod pattern;

pub use compiler::{compile, compile_pattern, CompileError};
pub use parser::{parse, Grammar, ParseError};
pub use pattern::Pattern;
