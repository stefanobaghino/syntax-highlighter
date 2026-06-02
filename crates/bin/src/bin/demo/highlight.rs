//! Demo CLI's ANSI-rendering layer over `Parser`.
//!
//! [`Highlighter`] is a thin wrapper around [`Parser`]: `set_input`
//! and `input` cross the `&str ↔ &[u8]` boundary, and
//! [`highlight`](Highlighter::highlight) consumes
//! [`syntax_highlighter::walk`] to wrap each kinded segment in the
//! corresponding ANSI escape from [`super::theme`].
//!
//! Demo-internal: this module is private to the `demo` binary. The
//! library exposes `Parser` and `walk` as the load-bearing API; ANSI
//! presentation is a demo concern.
//!
//! # Bytecode-only construction
//!
//! [`Highlighter::from_pegb`] takes the AOT-precompiled bytecode
//! emitted by the bin crate's `build.rs`. The demo never compiles a
//! grammar at startup; it embeds one `.pegb` blob per shipped
//! language and forwards the matching one here.
//!
//! # Load-bearing invariant
//!
//! The renderer never reorders, drops, or substitutes input bytes —
//! it only inserts color codes around them, *including* for inputs
//! the VM only partially matches. This is the structural property
//! `walk` guarantees: its segment stream tiles `0..input.len()`
//! exactly. Walker correctness is asserted by unit tests in
//! `src/walk.rs::tests`.

use syntax_highlighter::pegb;
use syntax_highlighter::pegvm::Capture;
use syntax_highlighter::walk::walk;
use syntax_highlighter_compiler::parser::Parser;

use super::theme;

#[derive(Debug)]
pub struct HighlightError(String);

impl std::fmt::Display for HighlightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for HighlightError {}

pub struct Highlighter {
    parser: Parser,
}

impl Highlighter {
    /// Build a highlighter from `pegb`-encoded bytecode embedded by the
    /// bin crate's `build.rs`. Decoding is on the order of microseconds
    /// per grammar; the demo's startup is dominated by stdin/stdout I/O
    /// rather than program loading.
    pub fn from_pegb(bytes: &[u8]) -> Result<Self, HighlightError> {
        let program =
            pegb::decode(bytes).map_err(|e| HighlightError(format!("decode pegb: {e}")))?;
        Ok(Self {
            parser: Parser::from_program(program),
        })
    }

    pub fn set_input(&mut self, input: String) {
        self.parser.set_input(input.into_bytes());
    }

    pub fn input(&self) -> &str {
        std::str::from_utf8(self.parser.input()).expect("UTF-8 invariant violated")
    }

    pub fn highlight(&self) -> String {
        let (_, captures) = self.parser.captures();
        render(self.input(), captures, self.parser.capture_kinds())
    }
}

/// ANSI-color the input by walking captures and wrapping each kinded
/// run with the corresponding theme escape. Sits on top of `walk`;
/// the byte-coverage property comes from there, this layer only
/// inserts ANSI prefix/reset bytes between segments where the active
/// kind transitions.
fn render(input: &str, captures: &[Capture], capture_kinds: &[String]) -> String {
    if captures.is_empty() {
        return input.to_string();
    }
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(bytes.len() + 32 * captures.len());
    let mut current_color: &'static str = "";
    walk(input, captures, capture_kinds, |seg| {
        let new_color = seg.kind.map(theme::color_for).unwrap_or("");
        if new_color != current_color {
            if !current_color.is_empty() {
                out.push_str(theme::RESET);
            }
            if !new_color.is_empty() {
                out.push_str(new_color);
            }
            current_color = new_color;
        }
        out.push_str(
            std::str::from_utf8(&bytes[seg.range])
                .expect("walker segment must align to a UTF-8 boundary"),
        );
    });
    if !current_color.is_empty() {
        out.push_str(theme::RESET);
    }
    out
}

#[cfg(test)]
mod tests {
    //! Renderer-correctness tests on synthetic capture sequences. The
    //! property under test is ANSI-emission shape: each kinded segment
    //! is wrapped in the corresponding theme escape, RESET fires on
    //! kind transitions, and adjacent same-kind segments don't emit
    //! spurious resets. Walker correctness is asserted separately at
    //! `src/walk.rs::tests`; per-grammar kind/range assertions live in
    //! the lib-level `tests/parse_<lang>_tests.rs`.
    use super::{render, theme};
    use syntax_highlighter::pegvm::{Capture, CaptureKind};

    fn cap(kind: u16, start: usize, end: usize) -> Capture {
        Capture {
            kind: CaptureKind(kind),
            start,
            end,
        }
    }

    #[test]
    fn no_captures_emits_input_verbatim() {
        let out = render("hello", &[], &[]);
        assert_eq!(out, "hello", "without captures, render is identity");
    }

    #[test]
    fn single_capture_wraps_in_theme_escape_and_resets() {
        let kinds = vec!["string".to_string()];
        let captures = vec![cap(0, 0, 5)];
        let out = render("hello", &captures, &kinds);
        let color = theme::color_for("string");
        assert!(out.starts_with(color), "expected leading color in {out:?}");
        assert!(
            out.ends_with(theme::RESET),
            "expected trailing RESET in {out:?}"
        );
        assert!(
            out.contains("hello"),
            "input bytes must appear verbatim in {out:?}"
        );
    }

    #[test]
    fn unknown_kind_emits_no_ansi() {
        // theme::color_for falls through to "" for kinds it doesn't recognize.
        // The renderer should emit no escape codes around such captures.
        let kinds = vec!["nonexistent_kind".to_string()];
        let captures = vec![cap(0, 0, 5)];
        let out = render("hello", &captures, &kinds);
        assert!(
            !out.contains('\x1b'),
            "no ANSI expected for unknown kind, got {out:?}"
        );
        assert_eq!(out, "hello");
    }

    #[test]
    fn kind_transition_emits_reset_between() {
        let kinds = vec!["string".to_string(), "number".to_string()];
        let captures = vec![cap(0, 0, 3), cap(1, 3, 6)];
        let out = render("abcdef", &captures, &kinds);
        // The sequence "string-color → abc → RESET → number-color → def → RESET"
        // means RESET appears between the two color escapes.
        let str_color = theme::color_for("string");
        let num_color = theme::color_for("number");
        let pos_str = out.find(str_color).expect("string color present");
        let pos_num = out.find(num_color).expect("number color present");
        let between = &out[pos_str..pos_num];
        assert!(
            between.contains(theme::RESET),
            "expected RESET between kind transitions, got {between:?}"
        );
    }

    #[test]
    fn unkinded_gap_renders_verbatim_no_ansi_around_it() {
        // outer kinded 0..2, gap 2..4 unkinded, kinded 4..6.
        let kinds = vec!["string".to_string()];
        let captures = vec![cap(0, 0, 2), cap(0, 4, 6)];
        let out = render("abcdef", &captures, &kinds);
        // The "cd" gap must appear in the output (verbatim) without
        // ANSI escape codes interleaved between its bytes.
        assert!(out.contains("cd"), "gap text must appear in {out:?}");
    }
}
