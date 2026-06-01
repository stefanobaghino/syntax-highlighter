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
//! This module hides the grammar-source → bytecode pipeline
//! ([`crate::pegc::compile`]) and the stateful VM wiring behind a
//! single type. Callers that need tighter composition (e.g. to reuse
//! one compiled [`Program`] across multiple Parser instances) can
//! drop to those lower-level modules directly.
//!
//! # Byte-oriented input
//!
//! PEG matching is byte-oriented, so `Parser` accepts
//! [`Vec<u8>`] / `&[u8]` rather than `String` / `&str`. UTF-8 handling
//! is the caller's concern; callers that ingest `&str` should preserve
//! the UTF-8 invariant at their own boundary.

use crate::pegc;
use crate::pegvm::{
    incremental::Edit, Capture, MemoCache, MemoStats, Program, RecoveryDiagnostic, VM,
};

/// Failure mode for [`Parser::new`]. A thin wrapper around
/// [`pegc::Error`] so callers learn one parser-shaped type rather
/// than reaching into [`pegc`] for its error vocabulary.
#[derive(Debug)]
pub struct ParserError(pub pegc::Error);

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ParserError {}

impl From<pegc::Error> for ParserError {
    fn from(e: pegc::Error) -> Self {
        ParserError(e)
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
    complete: bool,
    captures: Vec<Capture>,
    last_stats: MemoStats,
    /// When `true`, `reparse` builds the VM with
    /// [`VM::with_track_recovery_diagnostics`] and the resulting
    /// per-recovery records land in [`Parser::recovery_diagnostics`].
    /// Default `false`; consumers (e.g. `pegdb`) opt in via
    /// [`Parser::with_track_recovery_diagnostics`].
    track_recovery_diagnostics: bool,
    recovery_diagnostics: Vec<RecoveryDiagnostic>,
}

impl Parser {
    /// Compile `grammar_source` and construct an empty-input parser.
    /// Compilation runs once; subsequent `set_input` / `edit` calls
    /// reuse the same [`Program`].
    pub fn new(grammar_source: &str) -> Result<Self, ParserError> {
        Ok(Self::from_program(pegc::compile(grammar_source)?))
    }

    /// Construct an empty-input parser from a pre-compiled [`Program`].
    /// Skips the grammar-source pipeline — useful when the program came
    /// from [`crate::pegb::decode`], from a hand-built fixture, or from
    /// any other producer. The caller is responsible for the program's
    /// shape; there's no validation beyond what the producer already
    /// did.
    pub fn from_program(program: Program) -> Self {
        Self {
            program,
            input: Vec::new(),
            cache: MemoCache::new(),
            matched: 0,
            complete: true,
            captures: Vec::new(),
            last_stats: MemoStats::default(),
            track_recovery_diagnostics: false,
            recovery_diagnostics: Vec::new(),
        }
    }

    /// Opt into per-recovery diagnostic capture. When enabled,
    /// subsequent `set_input` / `edit` / `append` calls populate
    /// [`Parser::recovery_diagnostics`] with one
    /// [`RecoveryDiagnostic`] per `kind == "recovery"` capture. The
    /// flag is sticky — set it once at construction and every reparse
    /// honours it. Default is `false` (highlighter behaviour).
    pub fn with_track_recovery_diagnostics(mut self, enable: bool) -> Self {
        self.track_recovery_diagnostics = enable;
        self
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

    /// True iff the most recent parse reached `End` (the input fully
    /// matched the grammar). Distinct from `captures().0 == input.len()`,
    /// which can hold for a partial parse whose farthest-reached
    /// position happens to equal the input length.
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn capture_kinds(&self) -> &[String] {
        &self.program.capture_kinds
    }

    /// Rule names indexed by `MemoId.0` — see
    /// [`crate::pegvm::Program::rule_names`]. Used by diagnostic
    /// consumers to resolve [`RecoveryDiagnostic::rule_stack`] entries
    /// back to source-grammar names.
    pub fn rule_names(&self) -> &[String] {
        &self.program.rule_names
    }

    /// Label names indexed by `LabelId.0` — see
    /// [`crate::pegvm::Program::label_kinds`]. Used by diagnostic
    /// consumers to resolve [`crate::pegvm::RecoveryDiagnostic::label`]
    /// back to the source `^label` name.
    pub fn label_kinds(&self) -> &[String] {
        &self.program.label_kinds
    }

    /// Per-rule ignore flags indexed by `MemoId.0` — see
    /// [`crate::pegvm::Program::rule_is_ignore`]. `pegdb recoveries explain`
    /// uses this to drop trailing ignore frames from the rule_stack
    /// when picking the displayed leaf of a recovery cluster.
    pub fn rule_is_ignore(&self) -> &[bool] {
        &self.program.rule_is_ignore
    }

    /// Per-recovery diagnostic records from the most recent parse.
    /// Empty unless the parser was built with
    /// [`Parser::with_track_recovery_diagnostics(true)`]. Aligned with
    /// the `kind == "recovery"` captures in
    /// [`Parser::captures`] in emission order; each diagnostic's
    /// [`RecoveryDiagnostic::capture_index`] points back into the
    /// captures slice.
    pub fn recovery_diagnostics(&self) -> &[RecoveryDiagnostic] {
        &self.recovery_diagnostics
    }

    /// Memo cache diagnostics from the most recent parse. `hits` /
    /// `misses` reflect the warm re-parse's consumption of the seeded
    /// cache; `entries` is the post-parse cache size.
    pub fn last_stats(&self) -> MemoStats {
        self.last_stats
    }

    fn reparse(&mut self) {
        let seeded = std::mem::take(&mut self.cache);
        let recovery_kind = self
            .program
            .capture_kinds
            .iter()
            .position(|k| k == "recovery")
            .map(|i| {
                crate::pegvm::CaptureKind(u16::try_from(i).expect("capture_kinds index fits u16"))
            });
        let (result, stats, cache_after) = VM::new_with_cache(&self.program, &self.input, seeded)
            .with_recovery_capture_kind(recovery_kind)
            .with_track_recovery_diagnostics(self.track_recovery_diagnostics)
            .run_with_cache();
        self.cache = cache_after;
        self.matched = result.matched;
        self.complete = result.complete;
        self.captures = result.captures;
        self.recovery_diagnostics = result.recovery_diagnostics;
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
    /// delete must not short-circuit `RuleEnter` to `fail()` without
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
    fn is_complete_distinguishes_full_parse_from_truncation() {
        let mut p = Parser::new(JSON_GRAMMAR).unwrap();
        p.set_input(br#"{"a": 1}"#.to_vec());
        assert_eq!(p.captures().0, 8);
        assert!(p.is_complete(), "full parse must report complete");

        // Truncate the input mid-object: missing closing brace.
        p.set_input(br#"{"a": 1"#.to_vec());
        assert!(
            !p.is_complete(),
            "truncated input must report partial parse"
        );
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

    /// `Parser::from_program` and `Parser::new` produce equivalent
    /// parsers when given matching inputs — the `from_program` path
    /// skips `pegc::compile` but otherwise must behave identically.
    #[test]
    fn from_program_matches_new_on_same_grammar() {
        let program = pegc::compile(JSON_GRAMMAR).unwrap();
        let mut from_prog = Parser::from_program(program);
        let mut from_src = Parser::new(JSON_GRAMMAR).unwrap();

        let input = br#"{"a": 1, "b": [true, null]}"#.to_vec();
        from_prog.set_input(input.clone());
        from_src.set_input(input);

        assert_eq!(from_prog.captures(), from_src.captures());
        assert_eq!(from_prog.is_complete(), from_src.is_complete());
        assert_eq!(from_prog.capture_kinds(), from_src.capture_kinds());
    }
}
