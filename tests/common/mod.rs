//! Shared test helpers.
//!
//! Two families of helpers sit here behind a single `#[path]` import:
//!
//! - Parser/walk wrappers ([`segments`], [`kind_at`], [`matched_len`],
//!   [`is_complete`]) used by the per-grammar `tests/parse_<lang>_tests.rs`
//!   files.
//! - A minimal JSON-line field extractor ([`json_field_str`]) used by
//!   the `tests/pegc_tests.rs` and `tests/pegdb_tests.rs` integration
//!   suites to validate JSONL output without pulling in a JSON
//!   dependency.
//!
//! Each consuming test file declares this with
//! `#[path = "common/mod.rs"] mod common;`.

#![allow(dead_code)]

use std::ops::Range;

use syntax_highlighter::parser::Parser;
use syntax_highlighter::walk::walk;

/// Compile `grammar`, parse `input`, and collect the walker's segment
/// stream. Each segment is `(range, kind)`: `kind` is `Some(name)`
/// when a capture covers those bytes, `None` for unkinded gaps.
pub fn segments(grammar: &str, input: &str) -> Vec<(Range<usize>, Option<String>)> {
    let mut p = Parser::new(grammar).expect("grammar compiles");
    p.set_input(input.as_bytes().to_vec());
    let kinds = p.capture_kinds();
    let (_, captures) = p.captures();
    let view = std::str::from_utf8(p.input()).expect("UTF-8 input");
    let mut out = Vec::new();
    walk(view, captures, kinds, |seg| {
        out.push((seg.range, seg.kind.map(|k| k.to_string())));
    });
    out
}

/// Find the segment covering byte `byte` and return its kind. Returns
/// `None` when the byte is in an unkinded gap or out of range.
pub fn kind_at(grammar: &str, input: &str, byte: usize) -> Option<String> {
    segments(grammar, input)
        .into_iter()
        .find(|(r, _)| r.start <= byte && r.end > byte)
        .and_then(|(_, k)| k)
}

/// Total bytes the parser matched (i.e. the position it reached
/// before either accepting or bailing).
pub fn matched_len(grammar: &str, input: &str) -> usize {
    let mut p = Parser::new(grammar).expect("grammar compiles");
    p.set_input(input.as_bytes().to_vec());
    let (matched, _) = p.captures();
    matched
}

/// True iff the parser reached End on the input (full match).
pub fn is_complete(grammar: &str, input: &str) -> bool {
    let mut p = Parser::new(grammar).expect("grammar compiles");
    p.set_input(input.as_bytes().to_vec());
    p.is_complete()
}

/// Extract the value for a `"key":` substring out of a flat single-line
/// JSON object, returning the raw substring (still JSON-encoded) up to
/// the matching `,` or `}`. Handles nested arrays and objects in the
/// value, but not pretty-printed JSON. Adequate for the shapes `pegc`
/// and `pegdb` emit today. Not a general JSON parser.
pub fn json_field_str<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\":");
    let start = line.find(&needle)? + needle.len();
    let mut depth_array = 0i32;
    let mut depth_object = 0i32;
    let mut in_string = false;
    let mut escape = false;
    let bytes = line.as_bytes();
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        if in_string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'[' => depth_array += 1,
            b']' => depth_array -= 1,
            b'{' => depth_object += 1,
            b'}' if depth_object > 0 => depth_object -= 1,
            b',' | b'}' if depth_array == 0 && depth_object == 0 => return Some(&line[start..i]),
            _ => {}
        }
    }
    None
}
