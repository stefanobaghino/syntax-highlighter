//! Incremental-reparse sweep: measures the warm reparse time after a
//! small edit against the cold full-parse baseline, across every
//! shipped grammar and fixture size × a fixed set of edit shapes.
//!
//! Invoke with `cargo bench --bench incremental`.
//!
//! Like `benches/memo.rs`, this is a custom harness: median-of-N wall
//! clock, no criterion, `pegvm` stays dependency-free. Output is a
//! human-readable table on stdout and a JSONL machine-readable stream
//! at `target/bench-results/incremental.jsonl` (read by the
//! `memo-bench` skill's compare script).
//!
//! Per-cell correctness guard: the warm-reparse captures must match
//! a fresh full-parse on the post-edit input exactly. A divergence
//! aborts the bench before any timing is recorded.

use std::time::Instant;

use syntax_highlighter::highlight::Highlighter;

#[path = "common.rs"]
mod common;

use common::{json_f64, json_num, json_str, median, open_jsonl, write_jsonl_row};

const JSON_GRAMMAR: &str = include_str!("../grammars/json.peg");
const TOML_GRAMMAR: &str = include_str!("../grammars/toml.peg");
const SQL_GRAMMAR: &str = include_str!("../grammars/sqlite.peg");
const RUST_GRAMMAR: &str = include_str!("../grammars/rust.peg");
const JS_GRAMMAR: &str = include_str!("../grammars/javascript.peg");
const GO_GRAMMAR: &str = include_str!("../grammars/go.peg");
const C_GRAMMAR: &str = include_str!("../grammars/c.peg");

const JSON_SMALL: &str = include_str!("fixtures/small.json");
const JSON_MEDIUM: &str = include_str!("fixtures/medium.json");
const JSON_LARGE: &str = include_str!("fixtures/large.json");
const JSON_XLARGE: &str = include_str!("fixtures/xlarge.json");

const TOML_SMALL: &str = include_str!("fixtures/small.toml");
const TOML_MEDIUM: &str = include_str!("fixtures/medium.toml");
const TOML_LARGE: &str = include_str!("fixtures/large.toml");
const TOML_XLARGE: &str = include_str!("fixtures/xlarge.toml");

const SQL_SMALL: &str = include_str!("fixtures/small.sql");
const SQL_MEDIUM: &str = include_str!("fixtures/medium.sql");
const SQL_LARGE: &str = include_str!("fixtures/large.sql");
const SQL_XLARGE: &str = include_str!("fixtures/xlarge.sql");

const RUST_SMALL: &str = include_str!("fixtures/small.rs");
const RUST_MEDIUM: &str = include_str!("fixtures/medium.rs");
const RUST_LARGE: &str = include_str!("fixtures/large.rs");
const RUST_XLARGE: &str = include_str!("fixtures/xlarge.rs");

const JS_SMALL: &str = include_str!("fixtures/small.js");
const JS_MEDIUM: &str = include_str!("fixtures/medium.js");
const JS_LARGE: &str = include_str!("fixtures/large.js");
const JS_XLARGE: &str = include_str!("fixtures/xlarge.js");

const GO_SMALL: &str = include_str!("fixtures/small.go");
const GO_MEDIUM: &str = include_str!("fixtures/medium.go");
const GO_LARGE: &str = include_str!("fixtures/large.go");
const GO_XLARGE: &str = include_str!("fixtures/xlarge.go");

const C_SMALL: &str = include_str!("fixtures/small.c");
const C_MEDIUM: &str = include_str!("fixtures/medium.c");
const C_LARGE: &str = include_str!("fixtures/large.c");
const C_XLARGE: &str = include_str!("fixtures/xlarge.c");

const RUNS_PER_CELL: usize = 11;

#[derive(Clone, Copy)]
enum EditKind {
    /// Append a single character at the end. Simulates streaming
    /// input arriving one char at a time.
    AppendOneChar,
    /// Insert 10 bytes near the middle. Simulates typing in the
    /// middle of a document.
    InsertMid10,
    /// Delete 10 bytes near the middle. Simulates selecting-and-
    /// deleting a small span.
    DeleteMid10,
}

impl EditKind {
    fn label(&self) -> &'static str {
        match self {
            EditKind::AppendOneChar => "append_1c",
            EditKind::InsertMid10 => "insert_10",
            EditKind::DeleteMid10 => "delete_10",
        }
    }

    /// Apply this edit to `inc`. Each bench iteration must call this on
    /// a freshly primed highlighter so the cache reflects exactly one
    /// edit's worth of invalidation.
    fn apply(&self, inc: &mut Highlighter) {
        match self {
            EditKind::AppendOneChar => inc.append(" "),
            EditKind::InsertMid10 => {
                let at = pick_boundary(inc.input(), inc.input().len() / 2);
                inc.edit(at, at, "          ");
            }
            EditKind::DeleteMid10 => {
                let a = pick_boundary(inc.input(), inc.input().len() / 2);
                let mut b = a + 10;
                while b < inc.input().len() && !inc.input().is_char_boundary(b) {
                    b += 1;
                }
                if b > inc.input().len() {
                    b = inc.input().len();
                }
                inc.edit(a, b, "");
            }
        }
    }
}

/// Walk `pos` forward to the next char boundary if it isn't on one.
/// Fixtures are ASCII-only today, but keep the helper robust.
fn pick_boundary(s: &str, mut pos: usize) -> usize {
    while pos < s.len() && !s.is_char_boundary(pos) {
        pos += 1;
    }
    pos
}

struct Cell {
    grammar: &'static str,
    fixture_label: &'static str,
    edit_kind: EditKind,
    cold_ns: u128,
    warm_ns: u128,
    speedup: f64,
    warm_hits: usize,
    warm_misses: usize,
    warm_hit_rate: f64,
}

fn run_cell(
    grammar_src: &str,
    grammar_label: &'static str,
    fixture_label: &'static str,
    fixture: &str,
    edit_kind: EditKind,
) -> Cell {
    // Correctness guard: one warm iteration and one fresh full parse
    // must agree bit-for-bit on captures before we trust any timings.
    {
        let mut inc = Highlighter::new(grammar_src).unwrap();
        inc.set_input(fixture.to_string());
        edit_kind.apply(&mut inc);
        let mut fresh = Highlighter::new(grammar_src).unwrap();
        fresh.set_input(inc.input().to_string());
        assert_eq!(
            inc.captures(),
            fresh.captures(),
            "warm-reparse captures diverged from fresh parse: {}/{}/{}",
            grammar_label,
            fixture_label,
            edit_kind.label(),
        );
    }

    // Build the post-edit input once outside the timed regions.
    let post_edit = {
        let mut inc = Highlighter::new(grammar_src).unwrap();
        inc.set_input(fixture.to_string());
        edit_kind.apply(&mut inc);
        inc.input().to_string()
    };

    // Cold: each iteration drops the cache and full-parses the
    // post-edit input from scratch. `set_input` performs a cold
    // parse by construction (new MemoCache, every rule invocation
    // is a miss).
    let mut cold_times = Vec::with_capacity(RUNS_PER_CELL);
    let mut fresh = Highlighter::new(grammar_src).unwrap();
    for _ in 0..RUNS_PER_CELL {
        let t0 = Instant::now();
        fresh.set_input(post_edit.clone());
        cold_times.push(t0.elapsed().as_nanos());
    }

    // Warm: reset to post-initial-parse state, apply edit, time just
    // the edit (which does the incremental reparse).
    let mut warm_times = Vec::with_capacity(RUNS_PER_CELL);
    let mut last_stats = syntax_highlighter::pegvm::MemoStats::default();
    for _ in 0..RUNS_PER_CELL {
        let mut inc = Highlighter::new(grammar_src).unwrap();
        inc.set_input(fixture.to_string());
        let t0 = Instant::now();
        edit_kind.apply(&mut inc);
        warm_times.push(t0.elapsed().as_nanos());
        last_stats = inc.last_stats();
    }

    let cold_ns = median(cold_times);
    let warm_ns = median(warm_times);
    let speedup = cold_ns as f64 / warm_ns as f64;
    let lookups = last_stats.hits + last_stats.misses;
    let warm_hit_rate = if lookups == 0 {
        0.0
    } else {
        last_stats.hits as f64 / lookups as f64
    };

    Cell {
        grammar: grammar_label,
        fixture_label,
        edit_kind,
        cold_ns,
        warm_ns,
        speedup,
        warm_hits: last_stats.hits,
        warm_misses: last_stats.misses,
        warm_hit_rate,
    }
}

fn print_header() {
    println!(
        "{:<6} {:<7} {:<10} {:>10} {:>10} {:>8} {:>8} {:>8} {:>9}",
        "gram", "fixture", "edit", "cold(us)", "warm(us)", "speedup", "hits", "misses", "hit_rate"
    );
}

fn print_cell(c: &Cell) {
    println!(
        "{:<6} {:<7} {:<10} {:>10.1} {:>10.1} {:>8.2} {:>8} {:>8} {:>8.1}%",
        c.grammar,
        c.fixture_label,
        c.edit_kind.label(),
        c.cold_ns as f64 / 1000.0,
        c.warm_ns as f64 / 1000.0,
        c.speedup,
        c.warm_hits,
        c.warm_misses,
        c.warm_hit_rate * 100.0,
    );
}

fn emit_jsonl(out: &mut Option<std::io::BufWriter<std::fs::File>>, c: &Cell) {
    let Some(w) = out else { return };
    let row = &[
        ("bench", json_str("incremental")),
        ("grammar", json_str(c.grammar)),
        ("fixture", json_str(c.fixture_label)),
        ("edit_kind", json_str(c.edit_kind.label())),
        ("cold_us", json_f64(c.cold_ns as f64 / 1000.0)),
        ("warm_us", json_f64(c.warm_ns as f64 / 1000.0)),
        ("speedup", json_f64(c.speedup)),
        ("warm_hits", json_num(c.warm_hits)),
        ("warm_misses", json_num(c.warm_misses)),
        ("warm_hit_rate", json_f64(c.warm_hit_rate)),
        ("runs", json_num(RUNS_PER_CELL)),
    ];
    let _ = write_jsonl_row(w, row);
}

struct GrammarCase {
    label: &'static str,
    source: &'static str,
    fixtures: &'static [(&'static str, &'static str)],
}

fn main() {
    let cases: &[GrammarCase] = &[
        GrammarCase {
            label: "json",
            source: JSON_GRAMMAR,
            fixtures: &[
                ("small", JSON_SMALL),
                ("medium", JSON_MEDIUM),
                ("large", JSON_LARGE),
                ("xlarge", JSON_XLARGE),
            ],
        },
        GrammarCase {
            label: "toml",
            source: TOML_GRAMMAR,
            fixtures: &[
                ("small", TOML_SMALL),
                ("medium", TOML_MEDIUM),
                ("large", TOML_LARGE),
                ("xlarge", TOML_XLARGE),
            ],
        },
        GrammarCase {
            label: "sql",
            source: SQL_GRAMMAR,
            fixtures: &[
                ("small", SQL_SMALL),
                ("medium", SQL_MEDIUM),
                ("large", SQL_LARGE),
                ("xlarge", SQL_XLARGE),
            ],
        },
        GrammarCase {
            label: "rust",
            source: RUST_GRAMMAR,
            fixtures: &[
                ("small", RUST_SMALL),
                ("medium", RUST_MEDIUM),
                ("large", RUST_LARGE),
                ("xlarge", RUST_XLARGE),
            ],
        },
        GrammarCase {
            label: "js",
            source: JS_GRAMMAR,
            fixtures: &[
                ("small", JS_SMALL),
                ("medium", JS_MEDIUM),
                ("large", JS_LARGE),
                ("xlarge", JS_XLARGE),
            ],
        },
        GrammarCase {
            label: "go",
            source: GO_GRAMMAR,
            fixtures: &[
                ("small", GO_SMALL),
                ("medium", GO_MEDIUM),
                ("large", GO_LARGE),
                ("xlarge", GO_XLARGE),
            ],
        },
        GrammarCase {
            label: "c",
            source: C_GRAMMAR,
            fixtures: &[
                ("small", C_SMALL),
                ("medium", C_MEDIUM),
                ("large", C_LARGE),
                ("xlarge", C_XLARGE),
            ],
        },
    ];
    let kinds = [
        EditKind::AppendOneChar,
        EditKind::InsertMid10,
        EditKind::DeleteMid10,
    ];

    println!(
        "incremental reparse sweep — {} runs/cell; cold = fresh Highlighter parse of post-edit input",
        RUNS_PER_CELL
    );
    println!();
    print_header();

    let mut jsonl = open_jsonl("incremental");
    for case in cases {
        for (fixture_label, fixture) in case.fixtures {
            for kind in &kinds {
                // Edits require at least ~20 bytes to find a middle.
                // Skip the small fixtures for mid-range edits so we
                // don't panic on empty ranges.
                if fixture.len() < 20 && !matches!(kind, EditKind::AppendOneChar) {
                    continue;
                }
                let cell = run_cell(case.source, case.label, fixture_label, fixture, *kind);
                print_cell(&cell);
                emit_jsonl(&mut jsonl, &cell);
            }
        }
    }
}
