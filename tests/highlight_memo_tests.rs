//! End-to-end demonstration that memoization fires on one of the shipping
//! grammars and that it does not change the observable match result.
//!
//! The small-input tests here pin `with_memo_threshold(0)` so the cache
//! effects they rely on do not depend on the production default. A
//! separate test in this file exercises the default threshold on a
//! realistic (sub-kilobyte) input to guard against regressions in the
//! default itself.

use syntax_highlighter::grammar::{compile, parse};
use syntax_highlighter::pegvm::VM;

const SQLITE_GRAMMAR: &str = include_str!("../grammars/sqlite.peg");
const JSON_GRAMMAR: &str = include_str!("../grammars/json.peg");

#[test]
fn sqlite_select_registers_memo_hits() {
    // The SQL grammar has a lot of ordered alternatives that share prefixes
    // (keywords, identifiers, expressions) — exactly the shape where
    // memoization earns its keep.
    let g = parse(SQLITE_GRAMMAR).unwrap();
    let prog = compile(&g.rules, &g.start).unwrap();

    let input = "SELECT id, name FROM users WHERE active AND id > 10;";
    let (result, stats) = VM::new(&prog.code, input.as_bytes())
        .with_memo_threshold(0)
        .run_with_memo_stats();

    assert!(
        result.complete,
        "SQL grammar should accept the sample input"
    );
    assert!(
        stats.hits > 0,
        "expected memoization to fire on the SQL grammar, got {} hits / {} entries",
        stats.hits,
        stats.entries,
    );
}

/// Default-threshold regression guard: on a realistic ~640-byte SQL
/// input the cache must still fire. The bench (`cargo bench --bench
/// memo`) validates the whole sweep; this test locks in a minimum
/// bar for the shipping default under `cargo test`. If this fails,
/// `VM::DEFAULT_MEMO_THRESHOLD` is too high for the shipped grammars.
#[test]
fn default_threshold_still_fires_on_realistic_sql() {
    let input = include_str!("../benches/fixtures/medium.sql");
    let g = parse(SQLITE_GRAMMAR).unwrap();
    let prog = compile(&g.rules, &g.start).unwrap();

    let (result, stats) = VM::new(&prog.code, input.as_bytes()).run_with_memo_stats();

    assert!(result.complete, "medium.sql must parse to completion");
    assert!(
        stats.hits > 0,
        "default threshold produced 0 hits on {}-byte SQL input; \
         DEFAULT_MEMO_THRESHOLD ({}) may be too high",
        input.len(),
        VM::DEFAULT_MEMO_THRESHOLD,
    );
}

#[test]
fn json_run_records_memo_entries() {
    // JSON is a small grammar and highly structured, so hits may not occur
    // on simple inputs — but every rule call still lands an entry in the
    // table. This test just confirms the cache is wired end-to-end on a
    // shipping grammar.
    let g = parse(JSON_GRAMMAR).unwrap();
    let prog = compile(&g.rules, &g.start).unwrap();

    let input = br#"{"a": 1, "b": [true, null, "x"]}"#;
    let (result, stats) = VM::new(&prog.code, input)
        .with_memo_threshold(0)
        .run_with_memo_stats();

    assert!(result.complete);
    assert!(
        stats.entries > 0,
        "expected memo table to populate on JSON, got {} entries",
        stats.entries,
    );
}
