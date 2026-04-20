//! ANSI-colored syntax highlighting on top of [`crate::pegvm`].
//!
//! [`Highlighter`] compiles a PEG grammar once, then renders highlighted
//! output for any input by running the VM and translating its captures
//! into ANSI escape codes.
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
//! Stripping the ANSI escape codes from `highlight(input)` must yield the
//! original `input` unchanged, *including* for inputs the VM only
//! partially matches. The renderer never reorders, drops, or substitutes
//! input bytes — it only inserts color codes around them. The integration
//! tests assert this directly with a `strip_ansi` helper; any change to
//! the renderer must preserve the property.

pub mod theme;

use crate::pegvm::{
    compile_grammar, parse_grammar, Capture, CompileError, ParseError, Program, VM,
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

pub struct Highlighter {
    program: Program,
}

impl Highlighter {
    pub fn new(grammar_source: &str) -> Result<Self, HighlightError> {
        let g = parse_grammar(grammar_source)?;
        let program = compile_grammar(&g.rules, &g.start)?;
        Ok(Highlighter { program })
    }

    /// Run the VM and return raw captures alongside how many bytes matched.
    /// Useful for tests and debugging.
    ///
    /// On partial match (VM failed to reach `End`), the returned `matched`
    /// is the farthest input position reached and `captures` are the spans
    /// valid at that point — so the renderer naturally styles the valid
    /// prefix and emits the unparseable tail plain.
    pub fn captures(&self, input: &str) -> (usize, Vec<Capture>) {
        let r = VM::new(&self.program.code, input.as_bytes()).run();
        (r.matched, r.captures)
    }

    pub fn capture_kinds(&self) -> &[String] {
        &self.program.capture_kinds
    }

    pub fn highlight(&self, input: &str) -> String {
        let (_, captures) = self.captures(input);
        render(input, &captures, &self.program.capture_kinds)
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
