//! Shared helpers for the custom bench harnesses under `benches/`.
//!
//! Included from each bench file via `#[path = "common.rs"]` — Cargo
//! treats each bench as its own crate root, so this is the lightest
//! way to share code without promoting it to the library.
//!
//! The JSONL emitter is the mechanism that makes commit-by-commit
//! performance tracking possible (the `memo-bench` skill's compare
//! script reads these files). Every row is one self-contained JSON
//! object on a line; keys identify the cell, numeric fields carry
//! the measurement.

use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

pub const BENCH_RESULTS_DIR: &str = "target/bench-results";

/// Open (truncating) a JSONL file under [`BENCH_RESULTS_DIR`] for the
/// named bench. Caller writes one row per `writeln!` using
/// [`format_row`]. Returns `None` if the output directory can't be
/// created — benches still complete, but skip machine output.
pub fn open_jsonl(bench_name: &str) -> Option<BufWriter<File>> {
    let mut path = PathBuf::from(BENCH_RESULTS_DIR);
    if create_dir_all(&path).is_err() {
        return None;
    }
    path.push(format!("{bench_name}.jsonl"));
    File::create(&path).ok().map(BufWriter::new)
}

/// Serialize an iterator of `(key, value)` pairs as a one-line JSON
/// object. Values are already-formatted JSON scalars (strings must be
/// double-quoted by the caller). Keeps the dependency surface at zero.
///
/// This is deliberately minimal — no escaping of string values beyond
/// what the caller provides. All bench cells use ASCII-only string
/// fields (grammar/fixture/edit-kind labels are chosen that way) so
/// no escaping is needed.
pub fn write_jsonl_row(
    out: &mut BufWriter<File>,
    fields: &[(&str, String)],
) -> std::io::Result<()> {
    let mut line = String::from("{");
    for (i, (k, v)) in fields.iter().enumerate() {
        if i > 0 {
            line.push(',');
        }
        line.push('"');
        line.push_str(k);
        line.push_str("\":");
        line.push_str(v);
    }
    line.push('}');
    writeln!(out, "{line}")
}

pub fn json_str(s: &str) -> String {
    format!("\"{s}\"")
}

pub fn json_num<T: std::fmt::Display>(n: T) -> String {
    format!("{n}")
}

pub fn json_f64(n: f64) -> String {
    // Bench cells produce microsecond numbers; 3 decimals is enough
    // and keeps the files grep-able / human-readable.
    format!("{n:.3}")
}

pub fn median(mut xs: Vec<u128>) -> u128 {
    xs.sort_unstable();
    xs[xs.len() / 2]
}
