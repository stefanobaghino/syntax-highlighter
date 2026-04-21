//! Deep incremental-parsing abstraction built on [`crate::pegvm`].
//!
//! [`Parser`] compiles a grammar once, owns the current input, and
//! carries a [`MemoCache`] across edits so re-parsing after a small
//! change touches only the memo entries whose examined spans cross the
//! edit point (see [`crate::pegvm::incremental`] for the invalidation
//! protocol). The first [`set_input`](Parser::set_input) is a cold
//! full parse; each [`edit`](Parser::edit) / [`append`](Parser::append)
//! is a warm incremental reparse.
//!
//! This module hides the two-step grammar-source → bytecode pipeline
//! ([`crate::grammar::parse`] + [`crate::grammar::compile`]) and the
//! stateful VM wiring behind a single type. Callers that need tighter
//! composition (e.g. to reuse one compiled [`Program`] across multiple
//! Parser instances) can drop to those lower-level modules directly.
//!
//! # Byte-oriented input
//!
//! PEG matching is byte-oriented, so `Parser` accepts
//! [`Vec<u8>`] / `&[u8]` rather than `String` / `&str`. UTF-8 handling
//! is the caller's concern; the ANSI-coloring wrapper in
//! [`crate::highlight`] handles the `str`/`bytes` conversion at its
//! entry points, backed by the invariant that every mutation enters as
//! valid UTF-8.

use crate::grammar::{compile, parse, CompileError, ParseError};
use crate::pegvm::{incremental::Edit, Capture, MemoCache, MemoStats, Program, VM};

/// Failure mode for [`Parser::new`]. Unifies the grammar-source
/// parsing and bytecode-compilation error types behind one public
/// enum so callers learn one type rather than two.
#[derive(Debug)]
pub enum ParserError {
    Parse(ParseError),
    Compile(CompileError),
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::Parse(e) => write!(f, "{}", e),
            ParserError::Compile(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ParserError {}

impl From<ParseError> for ParserError {
    fn from(e: ParseError) -> Self {
        ParserError::Parse(e)
    }
}

impl From<CompileError> for ParserError {
    fn from(e: CompileError) -> Self {
        ParserError::Compile(e)
    }
}

/// Stateful, always-incremental parser. Holds the compiled program,
/// the current input, the carry-across memo cache, and the most recent
/// parse result. Every mutating method (`set_input`, `edit`, `append`)
/// re-parses synchronously and refreshes the cached captures; read
/// methods never parse.
pub struct Parser {
    program: Program,
    input: Vec<u8>,
    cache: MemoCache,
    matched: usize,
    captures: Vec<Capture>,
    last_stats: MemoStats,
}

impl Parser {
    /// Compile `grammar_source` and construct an empty-input parser.
    /// Compilation runs once; subsequent `set_input` / `edit` calls
    /// reuse the same [`Program`].
    pub fn new(grammar_source: &str) -> Result<Self, ParserError> {
        let g = parse(grammar_source)?;
        let program = compile(&g.rules, &g.start)?;
        Ok(Self {
            program,
            input: Vec::new(),
            cache: MemoCache::new(),
            matched: 0,
            captures: Vec::new(),
            last_stats: MemoStats::default(),
        })
    }

    /// Replace the entire input and perform a cold parse. Discards any
    /// prior cache — use this when the new input is unrelated to the
    /// old one. For small changes, prefer [`edit`](Self::edit) or
    /// [`append`](Self::append).
    pub fn set_input(&mut self, input: Vec<u8>) {
        self.input = input;
        self.cache = MemoCache::new();
        self.reparse();
    }

    /// Replace `input[start..old_end]` with `replacement` and re-parse
    /// incrementally. Entries in the carry-across cache whose examined
    /// span crosses `start` are dropped; the rest are shifted to
    /// reflect the new byte offsets and served as hits on the warm
    /// parse.
    ///
    /// Panics if `start > old_end` or `old_end > self.input.len()`.
    pub fn edit(&mut self, start: usize, old_end: usize, replacement: &[u8]) {
        assert!(start <= old_end, "edit: start must be <= old_end");
        assert!(
            old_end <= self.input.len(),
            "edit: old_end ({}) past input.len() ({})",
            old_end,
            self.input.len()
        );
        let edit = Edit::replacement(start, old_end, replacement.len());
        self.input
            .splice(start..old_end, replacement.iter().copied());
        self.cache.apply_edit(edit);
        self.reparse();
    }

    /// Streaming append: convenience for edits at `self.input.len()`.
    pub fn append(&mut self, text: &[u8]) {
        let at = self.input.len();
        self.edit(at, at, text);
    }

    pub fn input(&self) -> &[u8] {
        &self.input
    }

    /// Captures from the most recent parse, alongside how many bytes
    /// matched. On partial match (VM failed to reach `End`), `matched`
    /// is the farthest input position reached and the captures are the
    /// spans valid at that point.
    pub fn captures(&self) -> (usize, &[Capture]) {
        (self.matched, &self.captures)
    }

    pub fn capture_kinds(&self) -> &[String] {
        &self.program.capture_kinds
    }

    /// Memo cache diagnostics from the most recent parse. `hits` /
    /// `misses` reflect the warm re-parse's consumption of the seeded
    /// cache; `entries` is the post-parse cache size.
    pub fn last_stats(&self) -> MemoStats {
        self.last_stats
    }

    fn reparse(&mut self) {
        let seeded = std::mem::take(&mut self.cache);
        let (result, stats, cache_after) =
            VM::new_with_cache(&self.program.code, &self.input, seeded).run_with_cache();
        self.cache = cache_after;
        self.matched = result.matched;
        self.captures = result.captures;
        self.last_stats = stats;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOML_GRAMMAR: &str = include_str!("../grammars/toml.peg");
    const JSON_GRAMMAR: &str = include_str!("../grammars/json.peg");

    /// Oracle: a second `Parser` built from scratch must agree with
    /// the incremental one on captures and `matched` at every
    /// observable step.
    fn assert_equivalent(inc: &Parser, grammar: &str, tag: &str) {
        let mut fresh = Parser::new(grammar).expect("grammar compiles");
        fresh.set_input(inc.input().to_vec());
        assert_eq!(
            inc.captures(),
            fresh.captures(),
            "captures diverged at {tag}"
        );
    }

    #[test]
    fn set_input_cold_parse_produces_captures() {
        let mut p = Parser::new(JSON_GRAMMAR).unwrap();
        p.set_input(br#"{"a": 1}"#.to_vec());
        let (matched, caps) = p.captures();
        assert_eq!(matched, 8);
        assert!(!caps.is_empty());
    }

    #[test]
    fn edit_then_append_match_fresh_parse() {
        let mut inc = Parser::new(JSON_GRAMMAR).unwrap();
        inc.set_input(br#"{"a": 1}"#.to_vec());
        assert_equivalent(&inc, JSON_GRAMMAR, "initial");

        // Replace the `1` with `42`.
        inc.edit(6, 7, b"42");
        assert_equivalent(&inc, JSON_GRAMMAR, "after replace");

        // Streaming append shape; whitespace is valid JSON trailing.
        inc.append(b" ");
        assert_equivalent(&inc, JSON_GRAMMAR, "after append");
    }

    /// Regression pin: renaming a TOML section to something malformed
    /// leaves a top-level failure entry in the cache. A subsequent
    /// delete must not short-circuit `MemoOpen` to `fail()` without
    /// updating `max_sp` / `max_captures`, otherwise the incremental
    /// parse reports `matched == 0` while a fresh parse on the same
    /// input reaches the end of the valid prefix. Fix:
    /// `MemoCache::apply_edit` drops failure entries unconditionally.
    #[test]
    fn toml_section_rename_then_delete_preserves_max_sp_after_partial_parse() {
        let mut inc = Parser::new(TOML_GRAMMAR).unwrap();
        inc.set_input(b"[package]\nname = \"demo\"\nversion = \"0.1.0\"\n".to_vec());

        let pos = inc.input().iter().position(|&b| b == b'[').unwrap();
        inc.edit(pos, pos + 1, b"!");
        assert_equivalent(&inc, TOML_GRAMMAR, "after break");

        inc.edit(pos, pos + 1, b"");
        assert_equivalent(&inc, TOML_GRAMMAR, "after second edit");
    }

    #[test]
    fn last_stats_cold_parse_records_only_misses() {
        let mut p = Parser::new(JSON_GRAMMAR).unwrap();
        p.set_input(br#"{"a": 1, "b": 2}"#.to_vec());
        let cold = p.last_stats();
        assert_eq!(cold.hits, 0, "cold parse starts from an empty cache");
        assert!(
            cold.misses > 0,
            "expected at least one lookup, got {cold:?}"
        );
    }
}
