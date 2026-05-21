//! Partial-match-leniency lint cost per shipped grammar.
//!
//! Measures `pegc::analysis::lint_partial_match` on each bundled
//! grammar. Output: one JSONL row per grammar `(grammar, rules,
//! findings, time_us)`.
//!
//! Run with `cargo bench --bench lint`. Output written to
//! `target/bench-results/lint.jsonl` for commit-by-commit comparison.
//!
//! Methodology mirrors `benches/follow_set.rs`: `std::time::Instant`,
//! median-of-`RUNS_PER_CELL`. The lint is a one-shot static pass;
//! callers run it via the library API, so the absolute cost matters
//! more than per-rule scaling.

use std::time::Instant;

use syntax_highlighter::pegc;
use syntax_highlighter::pegc::analysis::lint_partial_match;
use syntax_highlighter::pegc::Grammar;

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
const CSS_GRAMMAR: &str = include_str!("../grammars/css.peg");

const RUNS_PER_CELL: usize = 11;

struct GrammarCase {
    name: &'static str,
    grammar: Grammar,
}

fn parse_case(name: &str, src: &str) -> Grammar {
    pegc::parse(src).unwrap_or_else(|e| panic!("parse {name}: {e}"))
}

fn measure(grammar: &Grammar, runs: usize) -> (u128, usize, usize) {
    let mut times = Vec::with_capacity(runs);
    let mut findings = 0;
    for _ in 0..runs {
        let t0 = Instant::now();
        let f = lint_partial_match(grammar);
        let dt = t0.elapsed().as_nanos();
        times.push(dt);
        findings = f.len();
    }
    (median(times), grammar.rules.len(), findings)
}

fn main() {
    let cases = [
        GrammarCase {
            name: "json",
            grammar: parse_case("json", JSON_GRAMMAR),
        },
        GrammarCase {
            name: "toml",
            grammar: parse_case("toml", TOML_GRAMMAR),
        },
        GrammarCase {
            name: "sql",
            grammar: parse_case("sql", SQL_GRAMMAR),
        },
        GrammarCase {
            name: "rust",
            grammar: parse_case("rust", RUST_GRAMMAR),
        },
        GrammarCase {
            name: "javascript",
            grammar: parse_case("javascript", JS_GRAMMAR),
        },
        GrammarCase {
            name: "go",
            grammar: parse_case("go", GO_GRAMMAR),
        },
        GrammarCase {
            name: "c",
            grammar: parse_case("c", C_GRAMMAR),
        },
        GrammarCase {
            name: "css",
            grammar: parse_case("css", CSS_GRAMMAR),
        },
    ];

    let mut jsonl = open_jsonl("lint");
    println!(
        "{:<12} {:>6} {:>8} {:>12}",
        "grammar", "rules", "findings", "time(us)"
    );
    for case in &cases {
        let (median_ns, rules, findings) = measure(&case.grammar, RUNS_PER_CELL);
        let time_us = median_ns as f64 / 1000.0;
        println!(
            "{:<12} {:>6} {:>8} {:>12.1}",
            case.name, rules, findings, time_us
        );
        if let Some(w) = jsonl.as_mut() {
            let row = &[
                ("bench", json_str("lint")),
                ("grammar", json_str(case.name)),
                ("rules", json_num(rules)),
                ("findings", json_num(findings)),
                ("time_us", json_f64(time_us)),
            ];
            let _ = write_jsonl_row(w, row);
        }
    }
}
