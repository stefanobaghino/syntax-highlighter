//! FOLLOW-set analysis cost per shipped grammar.
//!
//! Measures `pegc::analysis::compute_follow` on each bundled grammar.
//! Output: one JSONL row per grammar `(grammar, rules, follow_entries, time_us)`.
//!
//! Run with `cargo bench --bench follow_set`. Output written to
//! `target/bench-results/follow_set.jsonl` for commit-by-commit comparison.
//!
//! Methodology mirrors `benches/memo.rs`: `std::time::Instant`,
//! median-of-`RUNS_PER_CELL`, no statistical framework. FOLLOW analysis
//! is a one-shot static pass — its cost is below grammar compile time by
//! a comfortable margin on every shipped grammar.

use std::time::Instant;

use syntax_highlighter::pegc;
use syntax_highlighter::pegc::analysis::compute_follow;
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
const STARLARK_GRAMMAR: &str = include_str!("../grammars/starlark.peg");
const YAML_GRAMMAR: &str = include_str!("../grammars/yaml.peg");

const RUNS_PER_CELL: usize = 11;

struct GrammarCase {
    name: &'static str,
    grammar: Grammar,
}

fn parse_case(name: &str, src: &str) -> Grammar {
    pegc::parse(src).unwrap_or_else(|e| panic!("parse {name}: {e}"))
}

/// Run `compute_follow` `runs` times; report median wall time (ns), total
/// rule count, and total follow-entries across all rules.
fn measure(grammar: &Grammar, runs: usize) -> (u128, usize, usize) {
    let mut times = Vec::with_capacity(runs);
    let mut entries = 0;
    for _ in 0..runs {
        let t0 = Instant::now();
        let follow = compute_follow(grammar);
        let dt = t0.elapsed().as_nanos();
        times.push(dt);
        entries = follow.values().map(|s| s.len()).sum();
    }
    (median(times), grammar.rules.len(), entries)
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
        GrammarCase {
            name: "starlark",
            grammar: parse_case("starlark", STARLARK_GRAMMAR),
        },
        GrammarCase {
            name: "yaml",
            grammar: parse_case("yaml", YAML_GRAMMAR),
        },
    ];

    let mut jsonl = open_jsonl("follow_set");
    println!(
        "{:<12} {:>6} {:>14} {:>12}",
        "grammar", "rules", "follow_entries", "time(us)"
    );
    for case in &cases {
        let (median_ns, rules, entries) = measure(&case.grammar, RUNS_PER_CELL);
        let time_us = median_ns as f64 / 1000.0;
        println!(
            "{:<12} {:>6} {:>14} {:>12.1}",
            case.name, rules, entries, time_us
        );
        if let Some(w) = jsonl.as_mut() {
            let row = &[
                ("bench", json_str("follow_set")),
                ("grammar", json_str(case.name)),
                ("rules", json_num(rules)),
                ("follow_entries", json_num(entries)),
                ("time_us", json_f64(time_us)),
            ];
            let _ = write_jsonl_row(w, row);
        }
    }
}
