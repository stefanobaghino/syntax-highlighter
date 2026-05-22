//! Integration tests for the incremental-parsing public surface.
//!
//! Two layers:
//!
//! 1. **VM ↔ `MemoCache` round-trip.** Runs a VM, reclaims the
//!    `MemoCache`, seeds a fresh VM with it, confirms the second run
//!    hits the cache and produces identical output.
//! 2. **`Parser` equivalence under edits.** For each shipped grammar,
//!    runs a fixed sequence of edits through one `Parser` and compares
//!    the post-edit captures against a second `Parser` driven only via
//!    `set_input(current_input)`. The equality
//!    `inc.captures() == fresh.captures()` is the strongest
//!    correctness property this crate has — if any edit invalidates
//!    more or fewer entries than it should, this test catches it.
//!
//! Parser is the deep incremental-parsing abstraction. The demo CLI
//! wraps it in a `Highlighter` for ANSI rendering, but that wrapper is
//! demo-internal and adds no state — its faithfulness is a property of
//! `Parser`, not something this suite needs to re-test through a wrapper.

use std::collections::HashMap;

use syntax_highlighter::parser::Parser;
use syntax_highlighter::pegc::{Grammar, Pattern};
use syntax_highlighter::pegvm::{CharSet, MatchResult, MemoCache, VM};

fn two_rule_grammar() -> syntax_highlighter::pegvm::Program {
    // start <- word word
    // word  <- [a-z]+
    // The outer rule has enough span that default threshold (128) would
    // skip its entry on small inputs; tests pin threshold=0 to exercise
    // the mechanism regardless.
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::seq(vec![
            Pattern::nt("word"),
            Pattern::literal(" "),
            Pattern::nt("word"),
        ]),
    );
    rules.insert(
        "word".into(),
        Pattern::repeat_one(Pattern::char_class(CharSet::from_ranges(&[(b'a', b'z')]))),
    );
    Grammar::new(rules, "start").compile().unwrap()
}

#[test]
fn second_parse_with_seeded_cache_hits_every_cached_rule() {
    let prog = two_rule_grammar();

    // Cold parse populates the cache with entries for every memoized
    // rule invocation.
    let (cold_result, cold_stats, cache) = VM::new(&prog.code, b"hello world")
        .with_memo_threshold(0)
        .run_with_cache();
    assert!(cold_result.complete);
    assert_eq!(cold_result.matched, 11);
    assert_eq!(cold_stats.hits, 0, "cold run starts empty");
    assert!(cold_stats.misses > 0, "cold run must produce entries");
    assert_eq!(cache.len(), cold_stats.misses);

    // Warm parse with the same input: every RuleEnter for a rule
    // already in the cache should hit. Misses drop to zero when the
    // seeded cache covers every rule invocation.
    let (warm_result, warm_stats, _warm_cache) =
        VM::new_with_cache(&prog.code, b"hello world", cache)
            .with_memo_threshold(0)
            .run_with_cache();
    assert_eq!(
        warm_result, cold_result,
        "warm re-parse must produce identical output"
    );
    assert!(warm_stats.hits > 0, "warm run should hit the seeded cache");
    assert_eq!(
        warm_stats.misses, 0,
        "seeded cache covered every invocation; expected zero misses"
    );
}

#[test]
fn seeded_cache_survives_second_parse() {
    let prog = two_rule_grammar();
    let (_, _, cache) = VM::new(&prog.code, b"hello world")
        .with_memo_threshold(0)
        .run_with_cache();
    let len_before = cache.len();
    let (_, _, cache_after) = VM::new_with_cache(&prog.code, b"hello world", cache)
        .with_memo_threshold(0)
        .run_with_cache();
    assert_eq!(
        cache_after.len(),
        len_before,
        "re-parse on identical input should neither add nor drop entries"
    );
}

#[test]
fn empty_cache_behaves_like_fresh_vm() {
    let prog = two_rule_grammar();
    let input = b"hi there";
    let fresh: MatchResult = VM::new(&prog.code, input).run();
    let seeded: MatchResult = VM::new_with_cache(&prog.code, input, MemoCache::new()).run();
    assert_eq!(fresh, seeded);
}

#[test]
fn lr_cache_entry_invalidated_by_edit_inside_examined_range() {
    // The LR cache write at LRTail (#48) records examined_max as the
    // high-water mark across every iteration. apply_edit's invalidation
    // predicate (`start_sp < old_end && examined_max > start`) must
    // therefore drop the entry on any edit that touches what the
    // seed-and-grow loop looked at.
    use syntax_highlighter::pegc;
    use syntax_highlighter::pegvm::Edit;
    let prog = pegc::compile("expr <- expr '+' [0-9]+ / [0-9]+").unwrap();

    let (_, _, mut cache) = VM::new(&prog.code, b"1+2+3")
        .with_memo_threshold(0)
        .run_with_cache();
    let entries_before = cache.len();
    assert!(
        entries_before >= 1,
        "cold LR parse should populate the cache, got {entries_before}"
    );

    // Replace byte 0 ('1' → '9'). start_sp=0 < old_end=1 and the
    // entry's examined_max covers byte 0, so the entry must be dropped.
    cache.apply_edit(Edit::replacement(0, 1, 1));
    assert!(
        cache.len() < entries_before,
        "edit in examined range should invalidate the LR entry (before: {}, after: {})",
        entries_before,
        cache.len(),
    );
}

#[test]
fn lr_cache_round_trip_after_edit_matches_fresh_parse() {
    // End-to-end: cold parse + edit + re-parse via the edited cache
    // must produce the same MatchResult as a fresh parse of the new
    // input. This validates that the LR entry's examined_max bound is
    // correct: too small and a stale entry would be reused; too large
    // and an irrelevant edit would invalidate too aggressively
    // (correctness preserved either way, but only the right bound
    // gives an equivalent fresh-parse result on every edit).
    use syntax_highlighter::pegc;
    use syntax_highlighter::pegvm::Edit;
    let prog = pegc::compile("expr <- expr '+' [0-9]+ / [0-9]+").unwrap();

    let (_, _, mut cache) = VM::new(&prog.code, b"1+2+3")
        .with_memo_threshold(0)
        .run_with_cache();
    cache.apply_edit(Edit::replacement(0, 1, 1));

    let (incremental, _, _) = VM::new_with_cache(&prog.code, b"9+2+3", cache)
        .with_memo_threshold(0)
        .run_with_cache();
    let fresh = VM::new(&prog.code, b"9+2+3").with_memo_threshold(0).run();
    assert_eq!(incremental, fresh);
}

const JSON_GRAMMAR: &str = include_str!("../grammars/json.peg");
const TOML_GRAMMAR: &str = include_str!("../grammars/toml.peg");
const SQL_GRAMMAR: &str = include_str!("../grammars/sqlite.peg");

/// Compare the stateful `inc` parser against a fresh `Parser` driven
/// via `set_input(inc.input())`. The only correctness contract exposed
/// to users is that the two must agree bit-for-bit on captures and
/// `matched` at every observable step.
fn assert_equivalent(inc: &Parser, grammar: &str, tag: &str) {
    let mut fresh = Parser::new(grammar).expect("grammar compiles");
    fresh.set_input(inc.input().to_vec());
    assert_eq!(
        inc.captures(),
        fresh.captures(),
        "captures diverged at {tag}"
    );
}

fn parser_for(grammar: &str, input: &str) -> Parser {
    let mut p = Parser::new(grammar).unwrap();
    p.set_input(input.as_bytes().to_vec());
    p
}

#[test]
fn json_incremental_matches_fresh_across_edit_sequence() {
    let mut inc = parser_for(JSON_GRAMMAR, r#"{"a": 1, "b": [true, null, 3]}"#);
    assert_equivalent(&inc, JSON_GRAMMAR, "json initial");

    // Replace the number 1 with a longer number.
    let pos = inc.input().iter().position(|&b| b == b'1').unwrap();
    inc.edit(pos, pos + 1, b"12345");
    assert_equivalent(&inc, JSON_GRAMMAR, "json replace number");

    // Insert a new property before the closing brace.
    let close = inc.input().iter().rposition(|&b| b == b'}').unwrap();
    inc.edit(close, close, br#", "c": "hi""#);
    assert_equivalent(&inc, JSON_GRAMMAR, "json insert property");

    // Delete the "b" array entirely.
    let b_needle = br#", "b""#;
    let b_start = find_subslice(inc.input(), b_needle).unwrap();
    let arr_end = inc.input()[b_start..]
        .iter()
        .position(|&b| b == b']')
        .unwrap()
        + b_start
        + 1;
    inc.edit(b_start, arr_end, b"");
    assert_equivalent(&inc, JSON_GRAMMAR, "json delete array");

    // Replace everything — the set_input fast path.
    inc.set_input(b"null".to_vec());
    assert_equivalent(&inc, JSON_GRAMMAR, "json set_input reset");
}

#[test]
fn toml_incremental_matches_fresh_across_edit_sequence() {
    let mut inc = parser_for(
        TOML_GRAMMAR,
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    );
    assert_equivalent(&inc, TOML_GRAMMAR, "toml initial");

    // Append a new section — streaming pattern.
    inc.append(b"\n[dependencies]\nserde = \"1.0\"\n");
    assert_equivalent(&inc, TOML_GRAMMAR, "toml append section");

    // Rename [package] to [pkg].
    let start = find_subslice(inc.input(), b"[package]").unwrap();
    inc.edit(start, start + "[package]".len(), b"[pkg]");
    assert_equivalent(&inc, TOML_GRAMMAR, "toml rename section");

    // Delete the version line.
    let vstart = find_subslice(inc.input(), b"version").unwrap();
    let vend = inc.input()[vstart..]
        .iter()
        .position(|&b| b == b'\n')
        .unwrap()
        + vstart
        + 1;
    inc.edit(vstart, vend, b"");
    assert_equivalent(&inc, TOML_GRAMMAR, "toml delete version");
}

#[test]
fn sql_incremental_matches_fresh_across_edit_sequence() {
    let mut inc = parser_for(SQL_GRAMMAR, "SELECT id, name FROM users WHERE active = 1;");
    assert_equivalent(&inc, SQL_GRAMMAR, "sql initial");

    // Insert an ORDER BY clause.
    let semi = inc.input().iter().rposition(|&b| b == b';').unwrap();
    inc.edit(semi, semi, b" ORDER BY id DESC");
    assert_equivalent(&inc, SQL_GRAMMAR, "sql insert order by");

    // Replace the table name.
    let users = find_subslice(inc.input(), b"users").unwrap();
    inc.edit(users, users + "users".len(), b"customers");
    assert_equivalent(&inc, SQL_GRAMMAR, "sql rename table");

    // Delete the WHERE clause.
    let where_start = find_subslice(inc.input(), b" WHERE").unwrap();
    let where_end = find_subslice(inc.input(), b" ORDER BY").unwrap();
    inc.edit(where_start, where_end, b"");
    assert_equivalent(&inc, SQL_GRAMMAR, "sql delete where");
}

#[test]
fn sql_incremental_matches_fresh_across_recovery_boundary_edits() {
    // `sql_file`'s top-level `*^` produces `recovery` captures around
    // syntactically-broken chunks. Equivalence with a fresh parse must
    // hold across edits that move, fix, or introduce such chunks — the
    // memo cache must not retain stale entries that span an edit
    // crossing recovery territory.
    let mut inc = parser_for(
        SQL_GRAMMAR,
        "CREATE TABLE t (a INTEGER);\n!!! oops\nSELECT * FROM t;",
    );
    assert_equivalent(&inc, SQL_GRAMMAR, "sql recovery initial");

    // Fix the broken middle by replacing the garbage with a real statement.
    let oops_start = find_subslice(inc.input(), b"!!! oops").unwrap();
    inc.edit(
        oops_start,
        oops_start + "!!! oops".len(),
        b"INSERT INTO t VALUES (1);",
    );
    assert_equivalent(&inc, SQL_GRAMMAR, "sql recovery fixed");

    // Re-introduce a broken chunk at a different position.
    let select = find_subslice(inc.input(), b"SELECT").unwrap();
    inc.edit(select, select, b"???\n");
    assert_equivalent(&inc, SQL_GRAMMAR, "sql recovery reintroduced");
}

#[test]
fn append_char_by_char_matches_fresh_on_every_step() {
    // Streaming-LLM scenario: the classic case incremental parsing is
    // designed to accelerate. Equivalence must hold at every single
    // character boundary, not just at the end.
    let target = r#"{"a": 1, "b": [true, null]}"#;
    let mut inc = Parser::new(JSON_GRAMMAR).unwrap();
    for (i, ch) in target.char_indices() {
        inc.append(&target.as_bytes()[i..i + ch.len_utf8()]);
        assert_equivalent(&inc, JSON_GRAMMAR, &format!("json char-by-char step {i}"));
    }
}

#[test]
fn set_input_resets_cache_cleanly() {
    // Consecutive set_input calls on unrelated inputs must not leak
    // state between parses — every set_input rebuilds the cache from
    // scratch, so the equivalence property holds trivially after each
    // reset.
    let mut inc = Parser::new(JSON_GRAMMAR).unwrap();
    inc.set_input(br#"{"a": 1}"#.to_vec());
    assert_equivalent(&inc, JSON_GRAMMAR, "reset step 1");
    inc.set_input(br#"[1, 2, 3]"#.to_vec());
    assert_equivalent(&inc, JSON_GRAMMAR, "reset step 2");
    inc.set_input(b"null".to_vec());
    assert_equivalent(&inc, JSON_GRAMMAR, "reset step 3");
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}
