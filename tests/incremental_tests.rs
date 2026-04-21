//! Integration tests for the incremental-parsing public surface.
//!
//! Exercises the full loop: run a VM, reclaim the `MemoCache`, seed a
//! fresh VM with it, and confirm the second run hits the cache and
//! produces identical output. Parse-edit-reparse equivalence tests over
//! real grammars arrive alongside the `IncrementalHighlighter` API in
//! a follow-up commit.

use std::collections::HashMap;

use syntax_highlighter::pegvm::{compile_grammar, CharSet, MatchResult, MemoCache, Pattern, VM};

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
            Pattern::NonTerminal("word".into()),
            Pattern::literal(" "),
            Pattern::NonTerminal("word".into()),
        ]),
    );
    rules.insert(
        "word".into(),
        Pattern::RepeatOne(Box::new(Pattern::CharClass(CharSet::from_ranges(&[(
            b'a', b'z',
        )])))),
    );
    compile_grammar(&rules, "start").unwrap()
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

    // Warm parse with the same input: every MemoOpen for a rule
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
