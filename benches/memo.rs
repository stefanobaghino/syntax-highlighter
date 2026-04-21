//! Memoization threshold sweep.
//!
//! Runs every shipped grammar against three fixture sizes across a spread of
//! `memo_threshold` values, and reports `(time, entries, hits, misses,
//! hit_rate)` per cell. The output is meant for eyeballing the knee where
//! the memo table stops paying off — see README.md roadmap entry for
//! *threshold-based memoization*.
//!
//! Invoke with `cargo bench --bench memo`.
//!
//! This harness intentionally uses `std::time::Instant` and median-of-N
//! rather than a statistical framework (criterion et al.). The threshold
//! knee is order-of-magnitude; we do not need sub-percent precision, and
//! the `pegvm` half of the crate is dependency-free by design.

use std::time::Instant;

use syntax_highlighter::pegc;
use syntax_highlighter::pegvm::{Program, VM};

#[path = "common.rs"]
mod common;

use common::{json_f64, json_num, json_str, median, open_jsonl, write_jsonl_row};

const JSON_GRAMMAR: &str = include_str!("../grammars/json.peg");
const TOML_GRAMMAR: &str = include_str!("../grammars/toml.peg");
const SQL_GRAMMAR: &str = include_str!("../grammars/sqlite.peg");

const JSON_SMALL: &str = include_str!("fixtures/small.json");
const JSON_MEDIUM: &str = include_str!("fixtures/medium.json");
const JSON_LARGE: &str = include_str!("fixtures/large.json");

const TOML_SMALL: &str = include_str!("fixtures/small.toml");
const TOML_MEDIUM: &str = include_str!("fixtures/medium.toml");
const TOML_LARGE: &str = include_str!("fixtures/large.toml");

const SQL_SMALL: &str = include_str!("fixtures/small.sql");
const SQL_MEDIUM: &str = include_str!("fixtures/medium.sql");
const SQL_LARGE: &str = include_str!("fixtures/large.sql");

const THRESHOLDS: &[usize] = &[0, 32, 64, 128, 256, 512, 1024, 2048, 4096];
const RUNS_PER_CELL: usize = 11;

struct GrammarCase {
    name: &'static str,
    program: Program,
    inputs: Vec<(&'static str, &'static [u8])>,
}

fn compile_case(name: &str, src: &str) -> Program {
    pegc::compile(src).unwrap_or_else(|e| panic!("compile {name}: {e}"))
}

/// Run one cell: `runs` executions of the VM at a fixed threshold. Returns
/// the median wall time (nanoseconds), plus a single `(matched, complete,
/// stats)` tuple — all runs are deterministic, so one sample suffices.
fn sweep_cell(
    program: &Program,
    input: &[u8],
    threshold: usize,
    runs: usize,
) -> (u128, usize, bool, (usize, usize, usize)) {
    let mut times = Vec::with_capacity(runs);
    let mut matched = 0;
    let mut complete = false;
    let mut stats = (0, 0, 0);
    for _ in 0..runs {
        let t0 = Instant::now();
        let (result, s) = VM::new(&program.code, input)
            .with_memo_threshold(threshold)
            .run_with_memo_stats();
        let dt = t0.elapsed().as_nanos();
        times.push(dt);
        matched = result.matched;
        complete = result.complete;
        stats = (s.entries, s.hits, s.misses);
    }
    (median(times), matched, complete, stats)
}

fn print_grammar_table(case: &GrammarCase, jsonl: &mut Option<std::io::BufWriter<std::fs::File>>) {
    println!("=== {} ===", case.name);
    println!(
        "{:<8} {:>6} {:>10} {:>8} {:>8} {:>8} {:>9}",
        "input", "thresh", "time(us)", "entries", "hits", "misses", "hit_rate"
    );

    for (label, input) in &case.inputs {
        // Baseline run with threshold=0 establishes the reference parse
        // outcome. The sweep then asserts every other threshold produces
        // byte-identical `(matched, complete)` — the threshold filter must
        // not change observable parse behavior.
        let (_, ref_matched, ref_complete, _) = sweep_cell(&case.program, input, 0, 1);
        assert!(
            ref_complete,
            "{}: fixture {:?} did not parse to completion (matched={}/{})",
            case.name,
            label,
            ref_matched,
            input.len()
        );

        for &thr in THRESHOLDS {
            let (median_ns, matched, complete, (entries, hits, misses)) =
                sweep_cell(&case.program, input, thr, RUNS_PER_CELL);
            assert_eq!(
                (matched, complete),
                (ref_matched, ref_complete),
                "threshold {} changed parse outcome on {} / {}",
                thr,
                case.name,
                label
            );
            let lookups = hits + misses;
            let hit_rate = if lookups == 0 {
                0.0
            } else {
                hits as f64 / lookups as f64
            };
            println!(
                "{:<8} {:>6} {:>10.1} {:>8} {:>8} {:>8} {:>8.1}%",
                label,
                thr,
                median_ns as f64 / 1000.0,
                entries,
                hits,
                misses,
                hit_rate * 100.0
            );
            if let Some(w) = jsonl.as_mut() {
                let row = &[
                    ("bench", json_str("memo")),
                    ("grammar", json_str(case.name)),
                    ("fixture", json_str(label)),
                    ("threshold", json_num(thr)),
                    ("time_us", json_f64(median_ns as f64 / 1000.0)),
                    ("entries", json_num(entries)),
                    ("hits", json_num(hits)),
                    ("misses", json_num(misses)),
                    ("hit_rate", json_f64(hit_rate)),
                    ("runs", json_num(RUNS_PER_CELL)),
                ];
                let _ = write_jsonl_row(w, row);
            }
        }
        println!();
    }
}

fn main() {
    let cases = [
        GrammarCase {
            name: "json",
            program: compile_case("json", JSON_GRAMMAR),
            inputs: vec![
                ("small", JSON_SMALL.as_bytes()),
                ("medium", JSON_MEDIUM.as_bytes()),
                ("large", JSON_LARGE.as_bytes()),
            ],
        },
        GrammarCase {
            name: "toml",
            program: compile_case("toml", TOML_GRAMMAR),
            inputs: vec![
                ("small", TOML_SMALL.as_bytes()),
                ("medium", TOML_MEDIUM.as_bytes()),
                ("large", TOML_LARGE.as_bytes()),
            ],
        },
        GrammarCase {
            name: "sql",
            program: compile_case("sql", SQL_GRAMMAR),
            inputs: vec![
                ("small", SQL_SMALL.as_bytes()),
                ("medium", SQL_MEDIUM.as_bytes()),
                ("large", SQL_LARGE.as_bytes()),
            ],
        },
    ];

    println!(
        "memo threshold sweep — {} runs/cell, thresholds = {:?}",
        RUNS_PER_CELL, THRESHOLDS
    );
    println!();

    let mut jsonl = open_jsonl("memo");
    for case in &cases {
        print_grammar_table(case, &mut jsonl);
    }
}
