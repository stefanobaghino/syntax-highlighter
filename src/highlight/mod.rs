//! ANSI-colored syntax highlighting on top of [`crate::pegvm`].
//!
//! [`Highlighter`] compiles a PEG grammar once, owns the current input,
//! and carries a [`MemoCache`](crate::pegvm::MemoCache) across edits so
//! re-highlighting after a small change touches only the memo entries
//! whose spans cross the edit point (see
//! [`crate::pegvm::incremental`] for the invalidation protocol). The
//! first [`set_input`](Highlighter::set_input) is a cold full parse;
//! each [`edit`](Highlighter::edit) / [`append`](Highlighter::append)
//! is a warm incremental reparse.
//!
//! # Usage
//!
//! ```no_run
//! # use syntax_highlighter::highlight::Highlighter;
//! let mut h = Highlighter::new(include_str!("../../grammars/json.peg")).unwrap();
//! h.set_input(r#"{"a":1}"#.into());
//! h.edit(4, 5, "42");          // replace `1` with `42`
//! h.append(", \"b\": true");   // streaming append
//! println!("{}", h.highlight());
//! ```
//!
//! Equivalence guarantee: for any input `I` and any sequence of edits
//! transforming it into `I'`, calling [`highlight`](Highlighter::highlight)
//! after the edits must produce the same bytes as constructing a fresh
//! [`Highlighter`] and calling `set_input(I')` then `highlight()`. The
//! integration tests in `tests/incremental_tests.rs` cover this across
//! shipped grammars and edit sequences.
//!
//! # Rendering strategy
//!
//! The renderer walks captures as Begin/End events, maintains a stack of
//! currently-open captures, and applies **"innermost capture wins"**
//! coloring. PEG nesting guarantees captures form a forest (no overlap),
//! which is why a stack works rather than an interval tree.
//!
//! # Partial-match rendering
//!
//! When the VM cannot match the full input, it still returns captures
//! valid at the farthest position it reached (see `MatchResult` in
//! [`crate::pegvm`]). The renderer styles `input[..matched]` using those
//! captures and emits `input[matched..]` plain. This is why malformed or
//! in-progress inputs show a styled prefix followed by a plain tail,
//! rather than the whole input going unstyled on the first character
//! that fails to parse.
//!
//! # Load-bearing invariant
//!
//! Stripping the ANSI escape codes from [`highlight`](Highlighter::highlight)
//! must yield the current [`input`](Highlighter::input) unchanged,
//! *including* for inputs the VM only partially matches. The renderer
//! never reorders, drops, or substitutes input bytes — it only inserts
//! color codes around them. The integration tests assert this directly
//! with a `strip_ansi` helper; any change to the renderer must preserve
//! the property.

pub mod theme;

use crate::pegvm::{
    compile_grammar, incremental::Edit, parse_grammar, Capture, CompileError, MemoCache, MemoStats,
    ParseError, Program, VM,
};

#[derive(Debug)]
pub enum HighlightError {
    Parse(ParseError),
    Compile(CompileError),
}

impl std::fmt::Display for HighlightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HighlightError::Parse(e) => write!(f, "{}", e),
            HighlightError::Compile(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for HighlightError {}

impl From<ParseError> for HighlightError {
    fn from(e: ParseError) -> Self {
        HighlightError::Parse(e)
    }
}

impl From<CompileError> for HighlightError {
    fn from(e: CompileError) -> Self {
        HighlightError::Compile(e)
    }
}

/// Stateful, always-incremental highlighter. Holds the compiled program,
/// the current input, the carry-across memo cache, and the most recent
/// parse result. Every mutating method (`set_input`, `edit`, `append`)
/// re-parses synchronously and refreshes the cached captures; read
/// methods (`highlight`, `captures`, `input`) never parse.
pub struct Highlighter {
    program: Program,
    input: String,
    cache: MemoCache,
    matched: usize,
    captures: Vec<Capture>,
    last_stats: MemoStats,
}

impl Highlighter {
    pub fn new(grammar_source: &str) -> Result<Self, HighlightError> {
        let g = parse_grammar(grammar_source)?;
        let program = compile_grammar(&g.rules, &g.start)?;
        Ok(Self {
            program,
            input: String::new(),
            cache: MemoCache::new(),
            matched: 0,
            captures: Vec::new(),
            last_stats: MemoStats::default(),
        })
    }

    /// Replace the entire input and perform a cold parse. Discards any
    /// prior cache — use this when the new input is unrelated to the
    /// old one (e.g. loading a different file). For small edits to the
    /// current input, prefer [`edit`](Self::edit) or
    /// [`append`](Self::append) instead.
    pub fn set_input(&mut self, input: String) {
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
    pub fn edit(&mut self, start: usize, old_end: usize, replacement: &str) {
        assert!(start <= old_end, "edit: start must be <= old_end");
        assert!(
            old_end <= self.input.len(),
            "edit: old_end ({}) past input.len() ({})",
            old_end,
            self.input.len()
        );
        let edit = Edit::replacement(start, old_end, replacement.len());
        self.input.replace_range(start..old_end, replacement);
        self.cache.apply_edit(edit);
        self.reparse();
    }

    /// Streaming append: convenience for edits at `self.input.len()`.
    /// Optimized for the char-by-char case an LLM-streaming UI hits.
    pub fn append(&mut self, text: &str) {
        let at = self.input.len();
        self.edit(at, at, text);
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    /// Raw captures from the most recent parse, alongside how many
    /// bytes matched. Useful for tests and debugging.
    ///
    /// On partial match (VM failed to reach `End`), `matched` is the
    /// farthest input position reached and the captures are the spans
    /// valid at that point — so the renderer naturally styles the valid
    /// prefix and emits the unparseable tail plain.
    pub fn captures(&self) -> (usize, &[Capture]) {
        (self.matched, &self.captures)
    }

    pub fn capture_kinds(&self) -> &[String] {
        &self.program.capture_kinds
    }

    pub fn highlight(&self) -> String {
        render(&self.input, &self.captures, &self.program.capture_kinds)
    }

    /// Memo cache diagnostics from the most recent parse. `hits` /
    /// `misses` reflect the warm re-parse's consumption of the seeded
    /// cache; `entries` is the post-parse cache size. Primarily useful
    /// for benchmarks measuring incremental speedup.
    pub fn last_stats(&self) -> MemoStats {
        self.last_stats
    }

    fn reparse(&mut self) {
        let seeded = std::mem::take(&mut self.cache);
        let (result, stats, cache_after) =
            VM::new_with_cache(&self.program.code, self.input.as_bytes(), seeded).run_with_cache();
        self.cache = cache_after;
        self.matched = result.matched;
        self.captures = result.captures;
        self.last_stats = stats;
    }
}

#[derive(Clone, Copy, Debug)]
enum EventKind {
    Begin,
    End,
}

#[derive(Clone, Copy, Debug)]
struct Event {
    pos: usize,
    kind: EventKind,
    cap_idx: usize,
}

/// Walk through input emitting ANSI-colored output. The active color at each
/// position is determined by the innermost still-open capture (a stack).
fn render(input: &str, captures: &[Capture], capture_kinds: &[String]) -> String {
    if captures.is_empty() {
        return input.to_string();
    }

    let mut events: Vec<Event> = Vec::with_capacity(captures.len() * 2);
    for (i, c) in captures.iter().enumerate() {
        events.push(Event {
            pos: c.start,
            kind: EventKind::Begin,
            cap_idx: i,
        });
        events.push(Event {
            pos: c.end,
            kind: EventKind::End,
            cap_idx: i,
        });
    }

    // Order events at the same position so the stack stays consistent with
    // properly nested captures:
    //   - END events come before BEGIN events (a capture that ends here closes
    //     before a sibling that starts here).
    //   - For ENDs at the same position, inner captures (higher cap_idx) end first.
    //   - For BEGINs at the same position, outer captures (lower cap_idx) start first.
    events.sort_by(|a, b| {
        a.pos.cmp(&b.pos).then_with(|| match (a.kind, b.kind) {
            (EventKind::End, EventKind::Begin) => std::cmp::Ordering::Less,
            (EventKind::Begin, EventKind::End) => std::cmp::Ordering::Greater,
            (EventKind::End, EventKind::End) => b.cap_idx.cmp(&a.cap_idx),
            (EventKind::Begin, EventKind::Begin) => a.cap_idx.cmp(&b.cap_idx),
        })
    });

    let bytes = input.as_bytes();
    let mut out = String::with_capacity(bytes.len() + 32 * captures.len());
    let mut stack: Vec<usize> = Vec::new(); // capture indices, innermost on top
    let mut cursor = 0usize;

    let active_color = |stack: &Vec<usize>| -> &'static str {
        stack
            .last()
            .map(|&i| theme::color_for(&capture_kinds[captures[i].kind.0 as usize]))
            .unwrap_or("")
    };

    let mut current_color: &'static str = "";

    for ev in events {
        // Emit input slice from cursor to ev.pos under the current color.
        if ev.pos > cursor {
            let new_color = active_color(&stack);
            if new_color != current_color {
                if !current_color.is_empty() {
                    out.push_str(theme::RESET);
                }
                if !new_color.is_empty() {
                    out.push_str(new_color);
                }
                current_color = new_color;
            }
            out.push_str(std::str::from_utf8(&bytes[cursor..ev.pos]).unwrap_or(""));
            cursor = ev.pos;
        }
        match ev.kind {
            EventKind::Begin => stack.push(ev.cap_idx),
            EventKind::End => {
                // With proper PEG nesting, the matching capture is on top of the stack.
                if let Some(top) = stack.last() {
                    if *top == ev.cap_idx {
                        stack.pop();
                    } else {
                        // Defensive: search and remove (shouldn't happen for valid grammars).
                        if let Some(pos) = stack.iter().rposition(|&i| i == ev.cap_idx) {
                            stack.remove(pos);
                        }
                    }
                }
            }
        }
    }

    // Trailing input after the last event.
    if cursor < bytes.len() {
        if !current_color.is_empty() {
            out.push_str(theme::RESET);
            current_color = "";
        }
        let _ = current_color;
        out.push_str(std::str::from_utf8(&bytes[cursor..]).unwrap_or(""));
    } else if !current_color.is_empty() {
        out.push_str(theme::RESET);
    }

    out
}
