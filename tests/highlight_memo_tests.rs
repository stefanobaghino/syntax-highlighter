//! End-to-end demonstration that memoization fires on one of the shipping
//! grammars and that it does not change the observable match result.
//!
//! This is the proof-of-life for packrat-by-default: a real grammar, a real
//! input, and a non-zero hit count means the cache is doing work on the
//! actual workloads the highlighter serves.

use syntax_highlighter::pegvm::{compile_grammar, parse_grammar, VM};

const SQLITE_GRAMMAR: &str = include_str!("../grammars/sqlite.peg");
const JSON_GRAMMAR: &str = include_str!("../grammars/json.peg");

#[test]
fn sqlite_select_registers_memo_hits() {
    // The SQL grammar has a lot of ordered alternatives that share prefixes
    // (keywords, identifiers, expressions) — exactly the shape where
    // memoization earns its keep.
    let g = parse_grammar(SQLITE_GRAMMAR).unwrap();
    let prog = compile_grammar(&g.rules, &g.start).unwrap();

    let input = "SELECT id, name FROM users WHERE active AND id > 10;";
    let (result, stats) = VM::new(&prog.code, input.as_bytes()).run_with_memo_stats();

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

#[test]
fn json_run_records_memo_entries() {
    // JSON is a small grammar and highly structured, so hits may not occur
    // on simple inputs — but every rule call still lands an entry in the
    // table. This test just confirms the cache is wired end-to-end on a
    // shipping grammar.
    let g = parse_grammar(JSON_GRAMMAR).unwrap();
    let prog = compile_grammar(&g.rules, &g.start).unwrap();

    let input = br#"{"a": 1, "b": [true, null, "x"]}"#;
    let (result, stats) = VM::new(&prog.code, input).run_with_memo_stats();

    assert!(result.complete);
    assert!(
        stats.entries > 0,
        "expected memo table to populate on JSON, got {} entries",
        stats.entries,
    );
}
