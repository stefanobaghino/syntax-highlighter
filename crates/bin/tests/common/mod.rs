//! JSON-line field extractor shared by the `pegc` / `pegdb` integration
//! suites. Kept in lockstep with the runtime-side compiler crate's own
//! `common::json_field_str` so JSONL invariants are validated against
//! one parser regardless of which binary produced the output.

#![allow(dead_code)]

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
