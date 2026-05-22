use std::collections::HashMap;

use syntax_highlighter::pegc::analysis::{
    compute_follow, lint_partial_match, FollowElement, LintFinding, LintKind,
};
use syntax_highlighter::pegc::{compile_pattern, parse, Grammar, Pattern};
use syntax_highlighter::pegvm::{
    Capture, CaptureKind, CharSet, Instruction, Label, LabelId, MatchResult, MemoId, RuleKind, VM,
};

fn run_pattern(pat: &Pattern, input: &[u8]) -> MatchResult {
    let prog = compile_pattern(pat);
    VM::new(&prog.code, input).run()
}

/// Construct a `Capture` succinctly for test assertions.
fn cap(kind: u16, start: usize, end: usize) -> Capture {
    Capture {
        kind: CaptureKind(kind),
        start,
        end,
    }
}

#[test]
fn literal_pattern() {
    let p = Pattern::literal("hi");
    assert_eq!(
        run_pattern(&p, b"hi"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert!(!run_pattern(&p, b"ho").complete);
}

#[test]
fn char_class_pattern() {
    let p = Pattern::CharClass(CharSet::from_ranges(&[(b'0', b'9')]));
    assert_eq!(
        run_pattern(&p, b"5"),
        MatchResult {
            matched: 1,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert!(!run_pattern(&p, b"x").complete);
}

#[test]
fn any_char_pattern() {
    let p = Pattern::AnyChar;
    assert_eq!(
        run_pattern(&p, b"q"),
        MatchResult {
            matched: 1,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert!(!run_pattern(&p, b"").complete);
}

#[test]
fn sequence_pattern() {
    let p = Pattern::seq(vec![Pattern::literal("ab"), Pattern::literal("cd")]);
    assert_eq!(
        run_pattern(&p, b"abcd"),
        MatchResult {
            matched: 4,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert!(!run_pattern(&p, b"abce").complete);
}

#[test]
fn ordered_choice_two() {
    let p = Pattern::choice(vec![Pattern::literal("ab"), Pattern::literal("ax")]);
    assert_eq!(
        run_pattern(&p, b"ab"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run_pattern(&p, b"ax"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert!(!run_pattern(&p, b"ay").complete);
}

#[test]
fn ordered_choice_three() {
    let p = Pattern::choice(vec![
        Pattern::literal("foo"),
        Pattern::literal("bar"),
        Pattern::literal("baz"),
    ]);
    assert_eq!(
        run_pattern(&p, b"foo"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run_pattern(&p, b"bar"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run_pattern(&p, b"baz"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert!(!run_pattern(&p, b"qux").complete);
}

#[test]
fn repeat_zero_or_more() {
    let p = Pattern::Repeat(Box::new(Pattern::literal("a")));
    assert_eq!(
        run_pattern(&p, b""),
        MatchResult {
            matched: 0,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run_pattern(&p, b"aaa"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run_pattern(&p, b"aab"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
}

#[test]
fn repeat_one_or_more() {
    let p = Pattern::RepeatOne(Box::new(Pattern::literal("a")));
    assert!(!run_pattern(&p, b"").complete);
    assert_eq!(
        run_pattern(&p, b"a"),
        MatchResult {
            matched: 1,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run_pattern(&p, b"aaa"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
}

/// Builds the desugared AST that `p*^` lowers to: a `Repeat` over a
/// `Catch(inner, kind, @kind{.})`. Mirrors `build_recover_repeat` in
/// `src/pegc/parser.rs`. The `kind` argument controls both the
/// `Catch` label and the capture name — historically the parser
/// always picked `"recovery"`, and tests parameterize to exercise the
/// label/capture-interning pipeline.
fn recover(inner: Pattern, kind: &str) -> Pattern {
    Pattern::Repeat(Box::new(Pattern::Catch {
        inner: Box::new(inner),
        label: kind.into(),
        recovery: Box::new(Pattern::Capture(kind.into(), Box::new(Pattern::AnyChar))),
    }))
}

#[test]
fn recover_repeat_empty_input() {
    let p = recover(Pattern::literal("a"), "recovery");
    assert_eq!(
        run_pattern(&p, b""),
        MatchResult {
            matched: 0,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
}

#[test]
fn recover_repeat_all_inner_matches_emit_no_recovery_captures() {
    let p = recover(Pattern::literal("a"), "recovery");
    assert_eq!(
        run_pattern(&p, b"aaa"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
}

#[test]
fn recover_repeat_all_garbage_emits_one_recovery_capture_per_byte() {
    let p = recover(Pattern::literal("a"), "recovery");
    let r = run_pattern(&p, b"xyz");
    assert!(r.complete);
    assert_eq!(r.matched, 3);
    assert_eq!(
        r.captures,
        vec![cap(0, 0, 1), cap(0, 1, 2), cap(0, 2, 3)],
        "one recovery capture per skipped byte; complete parse at EOF",
    );
}

#[test]
fn recover_repeat_mixed_success_and_recovery() {
    let p = recover(Pattern::literal("a"), "recovery");
    let r = run_pattern(&p, b"axa");
    assert!(r.complete);
    assert_eq!(r.matched, 3);
    assert_eq!(
        r.captures,
        vec![cap(0, 1, 2)],
        "successful 'a' iterations emit no captures; the middle 'x' is the only recovery span",
    );
}

#[test]
fn recover_repeat_preserves_failed_inner_attempt_deepest_captures() {
    // inner = @open{"a"} "b" — the @open capture opens before the "b"
    // that may fail. Per issue #16, the deepest-progress captures of a
    // failed inner attempt are re-materialized by `RecoverToScopedMax`
    // before the recovery branch fires, and the recovery span covers
    // only the byte where parsing actually broke (not the iteration's
    // whole baseline-to-failure window).
    //
    // Kind interning order: `*^` desugars to `Repeat(Catch(inner,
    // "recovery", @recovery{.}))`; the `Catch` arm compiles inner
    // before the recovery body, so @open interns first (id 0) and
    // @recovery second (id 1).
    let inner = Pattern::seq(vec![
        Pattern::Capture("open".into(), Box::new(Pattern::literal("a"))),
        Pattern::literal("b"),
    ]);
    let p = recover(inner, "recovery");
    let r = run_pattern(&p, b"axab");
    assert!(r.complete);
    assert_eq!(r.matched, 4);
    assert_eq!(
        r.captures,
        vec![
            cap(0, 0, 1), // @open: re-materialized from the failed iteration's deepest reach
            cap(1, 1, 2), // recovery: 'x' (Any(1) consumes from scoped_max_sp=1, not baseline_sp=0)
            cap(0, 2, 3), // @open: the successful 'a' at sp=2
        ],
        "the failed inner attempt's deepest-progress @open capture survives into the result",
    );
}

#[test]
fn recover_repeat_failed_attempt_reaching_eof_does_not_leak_clean_match() {
    // inner = @open{"a"} "b" against input "a" — the inner @open opens
    // at sp=0, then Char 'a' consumes ('sp=1'), then Char 'b' tries at
    // sp=1 (EOF) and fails. The failed attempt's scoped_max_sp ==
    // input.len(); RecoverToScopedMax would move sp to EOF, and the
    // recovery branch's Any(1) would fail. Without the pre-
    // materialization inner Choice baseline, the materialized
    // @open(0,1) would leak into the result as a clean match — this
    // test pins down that the materialization is undone and the parse
    // reports an incomplete match instead. See the SQLite
    // assert_not_clean_parse cases (e.g. "SELECT SELECT") for the
    // grammar-level analogue.
    let inner = Pattern::seq(vec![
        Pattern::Capture("open".into(), Box::new(Pattern::literal("a"))),
        Pattern::literal("b"),
    ]);
    let p = recover(inner, "recovery");
    let r = run_pattern(&p, b"a");
    // The loop exits cleanly at sp=0, then `compile_pattern` appends
    // an `End` instruction — so `complete` is true at sp=0.
    // `matched < input.len()` is what flags the partial parse.
    assert!(r.matched < 1, "must not advance past failed attempt");
    assert!(
        r.captures.is_empty(),
        "materialized @open must not leak; got {:?}",
        r.captures
    );
}

#[test]
fn recover_repeat_nested_loops_do_not_panic() {
    // outer = ((@a{"a"} "X")*^) "Z"  (the outer is itself wrapped in *^)
    //
    // Two RecoverScope frames are simultaneously live whenever the
    // outer iteration is executing its inner `*^`. The
    // `maybe_snapshot` and `protect_max_captures` walks must handle
    // both frames without panicking — this regression-tests the
    // multi-scope spillover logic. Behaviour-level assertions are
    // deliberately loose: nested `*^` is a corner case, and the
    // per-scope watermark machinery still has known cosmetic edges
    // (an inner iteration that undoes via the EOF-Backtrack path may
    // leave its materialized captures referenced by an outer scope's
    // `saved_above` — see RecoverToScopedMax in src/pegvm/vm.rs).
    // What we lock in here is the operational invariant: no panic,
    // parse runs to completion.
    let inner = recover(
        Pattern::seq(vec![
            Pattern::Capture("a".into(), Box::new(Pattern::literal("a"))),
            Pattern::literal("X"),
        ]),
        "inner_rec",
    );
    let outer = recover(
        Pattern::seq(vec![inner, Pattern::literal("Z")]),
        "outer_rec",
    );
    // Several shapes that exercise different paths through the
    // double-scope state machine. Every input must reach `End` (i.e.
    // `complete` is true). The compiler-emitted `End` after the
    // top-level `*^` always succeeds, so completeness measures only
    // that the dispatcher didn't panic — sufficient to guard the
    // multi-scope spillover.
    for input in [b"".as_slice(), b"aX".as_slice(), b"!".as_slice(), b"aXaXZ"] {
        let r = run_pattern(&outer, input);
        assert!(r.complete, "nested *^ panicked or stalled on {:?}", input);
    }
}

#[test]
fn recover_repeat_empty_capture_in_failed_inner_attempt_is_dropped() {
    // inner = @x{!"x"} "z" — @x opens and closes at the same sp via
    // the NotPredicate succeeding without consuming. On input "by",
    // inner fails at "z"; the failed attempt's @x(0,0) sits at the
    // iteration's baseline sp and is dropped, NOT re-materialized.
    //
    // Why dropped: the per-iteration watermark mirrors the global
    // `max_*` design — `maybe_snapshot` only bumps `scoped_max_*`
    // when `sp` advances, so a capture that opens and closes without
    // any byte being consumed doesn't enter the watermark's
    // `(scoped_max_sp, scoped_max_captures_len)` snapshot. This
    // matches PEG-failure semantics for the global watermark
    // (`finalize_partial`) too, where a closed-but-empty capture at
    // `max_sp` would equally not appear in the partial result.
    //
    // The test pins this behaviour explicitly so a future refactor
    // that tries to "improve" the empty-capture case is forced to
    // re-baseline this test deliberately.
    let inner = Pattern::seq(vec![
        Pattern::Capture(
            "x".into(),
            Box::new(Pattern::NotPredicate(Box::new(Pattern::literal("x")))),
        ),
        Pattern::literal("z"),
    ]);
    let p = recover(inner, "recovery");
    let r = run_pattern(&p, b"by");
    assert!(r.complete);
    assert_eq!(r.matched, 2);
    // Kind interning order: "x" (id 0, inner compiled first), then
    // "recovery" (id 1, the desugared `@recovery{.}` body). No @x
    // captures survive — both iterations produced only empty ones at
    // their baseline sp, which don't enter the watermark.
    assert_eq!(
        r.captures,
        vec![
            cap(1, 0, 1), // recovery: 'b'
            cap(1, 1, 2), // recovery: 'y'
        ],
    );
}

#[test]
fn recover_repeat_drops_unclosed_capture_from_failed_inner_attempt() {
    // inner = @kw{ "abc" / "abd" } — the @kw capture opens BEFORE the
    // alternatives, and neither alternative reaches its CaptureEnd on
    // input "abx" (both consume "ab" then fail on the third byte).
    // The failed attempt's `scoped_max_sp` is at sp=2; the open @kw
    // OpenCapture sits in `scoped_saved_above` with `end: None`.
    //
    // Pre-fix, `RecoverToScopedMax` manufactured a close at
    // `scoped_max_sp`, leaking a phantom `@kw(0,2)="ab"` — the SQLite
    // dump-captures "stutter" (e.g. `keyword(56,59)="rep"` from
    // `replace_body` matching half of `repository`). The fix drops
    // every entry whose `end.is_none()` during the splice: a still-
    // open capture at the watermark belongs to a production that
    // didn't complete and must not become a token.
    let inner = Pattern::Capture(
        "kw".into(),
        Box::new(Pattern::choice(vec![
            Pattern::literal("abc"),
            Pattern::literal("abd"),
        ])),
    );
    let p = recover(inner, "recovery");
    let r = run_pattern(&p, b"abx");
    assert!(r.complete);
    assert_eq!(r.matched, 3);
    // Kind interning order: "kw" (id 0, inner compiled first), then
    // "recovery" (id 1, from the desugared `@recovery{.}` body).
    // Recovery byte is 'x' at sp=2 (Any(1) consumes from
    // `scoped_max_sp`, not the iteration baseline). No @kw capture
    // appears.
    assert_eq!(
        r.captures,
        vec![cap(1, 2, 3)],
        "an unclosed @kw from a failed inner attempt must not phantom-close at scoped_max_sp"
    );
}

#[test]
fn recover_repeat_inside_called_rule_returns_cleanly() {
    // start <- "PRE" loop
    // loop  <- "a"*^
    //
    // Against "PREaxa": the *^ runs to EOF, then start's Return must
    // pop a Return frame — not a Backtrack frame leaked from the loop.
    // This is the regression analogue of the PartialCommit hazard
    // documented in src/pegvm/README.md invariant 1.
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::seq(vec![
            Pattern::literal("PRE"),
            Pattern::NonTerminal("loop".into()),
        ]),
    );
    rules.insert("loop".into(), recover(Pattern::literal("a"), "recovery"));
    let prog = Grammar::new(rules, "start").compile().unwrap();
    let r = VM::new(&prog.code, b"PREaxa").run();
    assert!(
        r.complete,
        "Return after *^ loop must not panic on stack shape"
    );
    assert_eq!(r.matched, 6);
    // Recovery capture for the middle 'x'; the two 'a's matched cleanly.
    assert_eq!(r.captures, vec![cap(0, 4, 5)]);
}

fn catch(inner: Pattern, label: &str, recovery: Pattern) -> Pattern {
    Pattern::catch(inner, label, recovery)
}

#[test]
fn catch_inner_success_does_not_run_recovery() {
    // inner = @open{"ab"}, recovery = @err{(!';' .)*}
    // Input matches inner cleanly; recovery branch must not fire.
    let p = catch(
        Pattern::Capture("open".into(), Box::new(Pattern::literal("ab"))),
        "lbl",
        Pattern::Capture(
            "err".into(),
            Box::new(Pattern::Repeat(Box::new(Pattern::seq(vec![
                Pattern::NotPredicate(Box::new(Pattern::literal(";"))),
                Pattern::AnyChar,
            ])))),
        ),
    );
    let r = run_pattern(&p, b"ab");
    assert!(r.complete);
    assert_eq!(r.matched, 2);
    // Inner enters first → "open" interns to id 0, "err" to id 1.
    assert_eq!(
        r.captures,
        vec![cap(0, 0, 2)],
        "only the inner's @open capture; recovery's @err must not fire",
    );
}

#[test]
fn catch_inner_failure_runs_recovery() {
    // inner = @open{"ab"} fails at sp=0; recovery = @err{.} consumes one byte.
    let p = catch(
        Pattern::Capture("open".into(), Box::new(Pattern::literal("ab"))),
        "lbl",
        Pattern::Capture("err".into(), Box::new(Pattern::AnyChar)),
    );
    let r = run_pattern(&p, b"xy");
    assert!(r.complete);
    assert_eq!(r.matched, 1);
    assert_eq!(
        r.captures,
        vec![cap(1, 0, 1)],
        "@err captures the single byte the recovery consumed",
    );
}

#[test]
fn catch_preserves_failed_inner_attempt_deepest_captures() {
    // The whole point of `^` over `/`: when inner fails after partial
    // progress, the deepest-reach captures from the failed attempt are
    // re-materialized (via RecoverToScopedMax) and recovery runs from
    // that resync point — not from baseline sp.
    //
    // inner = @open{"a"} "b" — opens an @open over the leading 'a',
    // then requires 'b' which fails on input "ax". recovery = @err{.}
    // consumes one byte starting at the failed attempt's deepest sp
    // (sp=1, after the 'a'), not at baseline (sp=0).
    let p = catch(
        Pattern::seq(vec![
            Pattern::Capture("open".into(), Box::new(Pattern::literal("a"))),
            Pattern::literal("b"),
        ]),
        "lbl",
        Pattern::Capture("err".into(), Box::new(Pattern::AnyChar)),
    );
    let r = run_pattern(&p, b"ax");
    assert!(r.complete);
    assert_eq!(r.matched, 2);
    assert_eq!(
        r.captures,
        vec![
            cap(0, 0, 1), // @open: re-materialized from the failed attempt's deepest reach
            cap(1, 1, 2), // @err: starts at sp=1 (deepest reach), not at baseline sp=0
        ],
        "failed inner's @open survives; recovery's @err starts at scoped_max_sp",
    );
}

#[test]
fn catch_recovery_failure_propagates() {
    // Both branches fail: inner needs "ab", recovery needs "yz", input
    // is "ax". The catch as a whole must fail. Wrap it in an outer
    // OrderedChoice with a literal fallback to observe the failure
    // visibly via the fallback running.
    let p = Pattern::choice(vec![
        catch(Pattern::literal("ab"), "lbl", Pattern::literal("yz")),
        Pattern::literal("ax"),
    ]);
    let r = run_pattern(&p, b"ax");
    assert!(r.complete);
    assert_eq!(
        r.matched, 2,
        "catch failed (both branches), so the OrderedChoice fell through to the 'ax' literal",
    );
}

#[test]
fn catch_emits_recover_scope_skeleton() {
    let p = catch(Pattern::literal("a"), "lbl", Pattern::literal("b"));
    let prog = compile_pattern(&p);
    // 0:  RecoverScopeBegin(LabelId(0))
    // 1:  Choice rec(4)
    // 2:  Char 'a'                ; <inner>
    // 3:  Commit done(6)          ; pops outer Backtrack
    // 4:  rec: RecoverToScopedMax
    // 5:  Char 'b'                ; <recovery>
    // 6:  done: RecoverScopeEnd
    // 7:  End
    //
    // Smaller than RecoverRepeat's loop: no inner Choice (the recovery
    // body is author-written; if it fails, the VM's `fail()` cleans up
    // the RecoverScope frame via the same arm that handles `*^`
    // escapes — see src/pegvm/vm.rs). No synthetic recovery-byte
    // capture either.
    assert_eq!(
        prog.code,
        vec![
            Instruction::RecoverScopeBegin(LabelId(0)),
            Instruction::Choice(Label(4)),
            Instruction::Char(b'a'),
            Instruction::Commit(Label(6)),
            Instruction::RecoverToScopedMax,
            Instruction::Char(b'b'),
            Instruction::RecoverScopeEnd,
            Instruction::End,
        ]
    );
    assert_eq!(prog.capture_kinds, Vec::<String>::new());
    assert_eq!(prog.label_kinds, vec!["lbl".to_string()]);
}

#[test]
fn catch_nested_inside_recover_repeat() {
    // Nested RecoverScope frames: outer `*^` loop wraps a `^` catch.
    // The catch's frame is pushed and popped each iteration; the
    // outer's frame stays live across iterations. Regression-tests
    // that both frames stay balanced under nested capture
    // re-materialization. Like recover_repeat_nested_loops_do_not_panic
    // we lock in the operational invariant (no panic, complete parse)
    // rather than exact capture spans.
    let inner_catch = catch(
        Pattern::seq(vec![
            Pattern::Capture("open".into(), Box::new(Pattern::literal("a"))),
            Pattern::literal("b"),
        ]),
        "lbl",
        Pattern::Capture("err".into(), Box::new(Pattern::AnyChar)),
    );
    let p = recover(inner_catch, "recovery");
    let r = run_pattern(&p, b"abaxabZ");
    assert!(
        r.complete,
        "nested catch inside *^ must complete without panic"
    );
    assert_eq!(r.matched, 7);
}

#[test]
fn label_interning_dedups_across_catches() {
    // Two catches using the same label name share one `LabelId`,
    // with exactly one entry in `label_kinds`. Confirms
    // `intern_label` is idempotent.
    let p = catch(
        catch(Pattern::literal("a"), "missing", Pattern::literal("x")),
        "missing",
        Pattern::literal("y"),
    );
    let prog = compile_pattern(&p);
    let scope_begin_count = prog
        .code
        .iter()
        .filter(|i| matches!(i, Instruction::RecoverScopeBegin(_)))
        .count();
    assert_eq!(scope_begin_count, 2);
    assert_eq!(prog.label_kinds, vec!["missing".to_string()]);
    for ins in &prog.code {
        if let Instruction::RecoverScopeBegin(lid) = ins {
            assert_eq!(lid.0, 0, "both scopes resolve to LabelId(0)");
        }
    }
}

#[test]
fn label_intern_shared_with_recover_repeat_recovery_kind() {
    // `*^` desugars to a `Catch` labeled with its `recovery_kind`,
    // so a hand-written catch using the same string lands on the
    // same `LabelId`. Confirms the intern is by name, not by
    // emission site — useful so `pegdb explain-recoveries` can
    // cluster `*^` recoveries and matching `^lbl` catches under one
    // bucket when the author chose identical names.
    let p = Pattern::seq(vec![
        recover(Pattern::literal("a"), "shared"),
        catch(Pattern::literal("b"), "shared", Pattern::literal("c")),
    ]);
    let prog = compile_pattern(&p);
    assert_eq!(prog.label_kinds, vec!["shared".to_string()]);
}

#[test]
fn optional_pattern() {
    let p = Pattern::seq(vec![
        Pattern::Optional(Box::new(Pattern::literal("-"))),
        Pattern::literal("x"),
    ]);
    assert_eq!(
        run_pattern(&p, b"x"),
        MatchResult {
            matched: 1,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run_pattern(&p, b"-x"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert!(!run_pattern(&p, b"--x").complete);
}

#[test]
fn not_predicate_pattern() {
    let p = Pattern::seq(vec![
        Pattern::NotPredicate(Box::new(Pattern::literal("a"))),
        Pattern::AnyChar,
    ]);
    assert_eq!(
        run_pattern(&p, b"b"),
        MatchResult {
            matched: 1,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert!(!run_pattern(&p, b"a").complete);
}

#[test]
fn and_predicate_pattern() {
    // &"a" "ab"  -> matches "ab" only when first char is 'a' (always true here, but the
    // &-predicate consumes nothing)
    let p = Pattern::seq(vec![
        Pattern::AndPredicate(Box::new(Pattern::literal("a"))),
        Pattern::literal("ab"),
    ]);
    assert_eq!(
        run_pattern(&p, b"ab"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert!(!run_pattern(&p, b"bb").complete);
}

#[test]
fn capture_records_kind_and_span() {
    let p = Pattern::Capture("number".into(), Box::new(Pattern::literal("42")));
    let prog = compile_pattern(&p);
    assert_eq!(prog.capture_kinds, vec!["number".to_string()]);
    assert_eq!(
        VM::new(&prog.code, b"42").run(),
        MatchResult {
            matched: 2,
            captures: vec![cap(0, 0, 2)],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
}

#[test]
fn nested_captures_flow_through_compile() {
    // @outer{ @inner{"a"} @inner{"b"} }
    let p = Pattern::Capture(
        "outer".into(),
        Box::new(Pattern::seq(vec![
            Pattern::Capture("inner".into(), Box::new(Pattern::literal("a"))),
            Pattern::Capture("inner".into(), Box::new(Pattern::literal("b"))),
        ])),
    );
    let prog = compile_pattern(&p);
    // Kinds interned in the order they're first encountered during compile.
    assert_eq!(
        prog.capture_kinds,
        vec!["outer".to_string(), "inner".to_string()]
    );
    assert_eq!(
        VM::new(&prog.code, b"ab").run(),
        MatchResult {
            matched: 2,
            captures: vec![
                cap(0, 0, 2), // @outer
                cap(1, 0, 1), // @inner "a"
                cap(1, 1, 2), // @inner "b"
            ],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
}

#[test]
fn grammar_with_nonterminals() {
    // start <- digit+
    // digit <- [0-9]
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::RepeatOne(Box::new(Pattern::NonTerminal("digit".into()))),
    );
    rules.insert(
        "digit".into(),
        Pattern::CharClass(CharSet::from_ranges(&[(b'0', b'9')])),
    );
    let prog = Grammar::new(rules, "start").compile().unwrap();
    assert_eq!(
        VM::new(&prog.code, b"123abc").run(),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert!(!VM::new(&prog.code, b"abc").run().complete);
}

#[test]
fn grammar_undefined_rule_errors() {
    let mut rules = HashMap::new();
    rules.insert("start".into(), Pattern::NonTerminal("missing".into()));
    let err = Grammar::new(rules, "start").compile().unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("missing"), "got: {}", msg);
}

#[test]
fn ordered_choice_emits_expected_skeleton() {
    // p = "a" / "b"
    // 0: Choice 3
    // 1: Char a
    // 2: Commit 4
    // 3: Char b
    // 4: End
    let p = Pattern::choice(vec![Pattern::literal("a"), Pattern::literal("b")]);
    let prog = compile_pattern(&p);
    assert_eq!(
        prog.code,
        vec![
            Instruction::Choice(Label(3)),
            Instruction::Char(b'a'),
            Instruction::Commit(Label(4)),
            Instruction::Char(b'b'),
            Instruction::End,
        ]
    );
}

#[test]
fn repeat_emits_partial_commit() {
    let p = Pattern::Repeat(Box::new(Pattern::literal("a")));
    let prog = compile_pattern(&p);
    // Choice jumps over the loop on first failure; PartialCommit jumps to the
    // body (index 1), NOT back to Choice — the existing backtrack entry is reused.
    assert_eq!(
        prog.code,
        vec![
            Instruction::Choice(Label(3)),
            Instruction::Char(b'a'),
            Instruction::PartialCommit(Label(1)),
            Instruction::End,
        ]
    );
}

#[test]
fn recover_repeat_emits_choice_commit_skeleton() {
    // `*^` desugars to `(p ^recovery @recovery{.})*` at parse time —
    // the emit is the natural concatenation of the `Repeat` arm's
    // Choice/PartialCommit skeleton and the `Catch` arm's
    // RecoverScope/Choice/RecoverToScopedMax/RecoverScopeEnd shape.
    //
    //   Choice exit              ; Repeat outer Backtrack
    // body: RecoverScopeBegin     ; Catch start
    //       Choice rec            ; Catch inner Backtrack
    //       Char 'a'
    //       Commit done           ; on inner success
    // rec:  RecoverToScopedMax    ; on inner failure: splice + advance
    //       CaptureBegin recovery
    //       Any(1)
    //       CaptureEnd
    // done: RecoverScopeEnd
    //       PartialCommit body    ; loop edge — reuses outer Backtrack
    // exit: End
    //
    // The EOF clean-exit edge that the old `RecoverRepeat` shape
    // emitted explicitly is now implicit: when the recovery body's
    // `Any(1)` fails at EOF, `fail()` unwinds past the
    // `RecoverScope` frame and lands on the Repeat's `Choice exit`
    // Backtrack — the loop terminates cleanly.
    let p = recover(Pattern::literal("a"), "recovery");
    let prog = compile_pattern(&p);
    assert_eq!(
        prog.code,
        vec![
            Instruction::Choice(Label(11)),             // → exit
            Instruction::RecoverScopeBegin(LabelId(0)), // body: Catch start
            Instruction::Choice(Label(5)),              // → rec
            Instruction::Char(b'a'),
            Instruction::Commit(Label(9)),   // → done
            Instruction::RecoverToScopedMax, // rec:
            Instruction::CaptureBegin(CaptureKind(0)),
            Instruction::Any(1),
            Instruction::CaptureEnd,
            Instruction::RecoverScopeEnd,         // done:
            Instruction::PartialCommit(Label(1)), // → body
            Instruction::End,                     // exit:
        ]
    );
    assert_eq!(prog.capture_kinds, vec!["recovery".to_string()]);
    assert_eq!(prog.label_kinds, vec!["recovery".to_string()]);
}

#[test]
fn recover_repeat_labeled_interns_author_label() {
    // `*^:bad_thing` interns the author-supplied label while leaving
    // the capture kind at its hardcoded `"recovery"` — the catch
    // scope is renamed (so pegdb explain-recoveries clusters under
    // "bad_thing"), but theme styling is unaffected.
    let prog = syntax_highlighter::pegc::compile("r <- 'a'*^:bad_thing").unwrap();
    assert_eq!(prog.capture_kinds, vec!["recovery".to_string()]);
    assert_eq!(prog.label_kinds, vec!["bad_thing".to_string()]);
}

/// Builds the desugared AST that `p*^[cs]` lowers to: a `Repeat` over
/// `Catch(inner, "recovery", @recovery{(!cs .)* cs})`. Mirrors
/// `build_recover_repeat` in `src/pegc/parser.rs`.
fn sync_set_recover(inner: Pattern, charset: CharSet) -> Pattern {
    let skip_loop = Pattern::Repeat(Box::new(Pattern::Sequence(vec![
        Pattern::NotPredicate(Box::new(Pattern::CharClass(charset))),
        Pattern::AnyChar,
    ])));
    let recovery_body = Pattern::Sequence(vec![skip_loop, Pattern::CharClass(charset)]);
    Pattern::Repeat(Box::new(Pattern::Catch {
        inner: Box::new(inner),
        label: "recovery".into(),
        recovery: Box::new(Pattern::Capture("recovery".into(), Box::new(recovery_body))),
    }))
}

#[test]
fn sync_set_emits_skip_to_delim_loop() {
    // The recovery body of `*^[;]` is `@recovery{(![;] .)* [;]}` —
    // a skip-until-delim loop followed by a delimiter consume, both
    // wrapped in a single `recovery` capture. The skeleton verifies
    // the structural pieces are present rather than nailing exact
    // Label values (which would entangle the test with the host
    // Catch/Repeat emit shape).
    let semi = CharSet::from_bytes(b";");
    let p = sync_set_recover(Pattern::literal("a"), semi);
    let prog = compile_pattern(&p);
    let has = |needle: &Instruction| prog.code.iter().any(|i| i == needle);
    assert!(has(&Instruction::RecoverScopeBegin(LabelId(0))));
    assert!(has(&Instruction::RecoverToScopedMax));
    assert!(has(&Instruction::CaptureBegin(CaptureKind(0))));
    assert!(has(&Instruction::Set(semi)));
    assert!(prog
        .code
        .iter()
        .any(|i| matches!(i, Instruction::FailTwice)));
    assert_eq!(prog.capture_kinds, vec!["recovery".to_string()]);
    assert_eq!(prog.label_kinds, vec!["recovery".to_string()]);
}

#[test]
fn sync_set_one_recovery_capture_per_region() {
    // Input `aXY;c` with `('a'/'c')*^[;]`: iter 1 matches 'a', iter 2
    // fails on 'X', recovery skips "XY" then consumes ";" — one big
    // recovery capture covering [1, 4]. Iter 3 matches 'c'.
    let semi = CharSet::from_bytes(b";");
    let p = sync_set_recover(
        Pattern::choice(vec![Pattern::literal("a"), Pattern::literal("c")]),
        semi,
    );
    let r = run_pattern(&p, b"aXY;c");
    assert!(r.complete);
    assert_eq!(r.matched, 5);
    assert_eq!(
        r.captures,
        vec![cap(0, 1, 4)],
        "exactly one recovery capture spanning the skipped region plus the delimiter",
    );
}

#[test]
fn sync_set_terminates_cleanly_at_eof_without_delim() {
    // Input `aXY` (no `;`): iter 1 matches 'a', iter 2 fails on 'X',
    // recovery's `(!; .)* ;` skips XY then expects ';' at EOF → fails.
    // The catch fails, the outer `*` terminates. The parse is
    // structurally complete (the `End` after `*` always succeeds),
    // but `matched` reports only the bytes the loop committed: 'a'.
    let semi = CharSet::from_bytes(b";");
    let p = sync_set_recover(Pattern::literal("a"), semi);
    let r = run_pattern(&p, b"aXY");
    assert!(r.complete);
    assert_eq!(r.matched, 1, "must not advance past the unrecoverable tail");
    assert!(
        r.captures.is_empty(),
        "no recovery span emitted when the delimiter is missing"
    );
}

#[test]
fn grammar_rules_are_wrapped_in_memo_open_close() {
    // start <- "a"
    // other <- "b"
    // Layout:
    //   0: Call(start)
    //   1: End
    //   2: RuleEnter(0, Memo, L5)   ; start's Return is at 5
    //   3: Char 'a'
    //   4: MemoClose(0)
    //   5: Return
    //   6: RuleEnter(1, Memo, L9)   ; other's Return is at 9
    //   7: Char 'b'
    //   8: MemoClose(1)
    //   9: Return
    let mut rules = HashMap::new();
    rules.insert("start".into(), Pattern::literal("a"));
    rules.insert("other".into(), Pattern::literal("b"));
    let prog = Grammar::new(rules, "start").compile().unwrap();
    assert_eq!(prog.rule_count, 2);
    assert_eq!(
        prog.code,
        vec![
            Instruction::Call(Label(2)),
            Instruction::End,
            Instruction::RuleEnter(MemoId(0), RuleKind::Memo, Label(5)),
            Instruction::Char(b'a'),
            Instruction::MemoClose(MemoId(0)),
            Instruction::Return,
            Instruction::RuleEnter(MemoId(1), RuleKind::Memo, Label(9)),
            Instruction::Char(b'b'),
            Instruction::MemoClose(MemoId(1)),
            Instruction::Return,
        ]
    );
}

#[test]
fn direct_lr_rule_emits_lrbody_lrtail_skeleton() {
    // start <- start "+" "n" / "n"
    // Layout (no MemoClose for an LR rule — LRTail closes the body):
    //   0: Call(2)
    //   1: End
    //   2: RuleEnter(0, Lr, <ret>)
    //   3 (body_start): Choice(7)
    //   4: Call(2)
    //   5: Char '+'
    //   6: Char 'n'
    //   7: Commit(8)        ; first-alt commit lands at the leaf 'n'
    //   ...
    // Exact target labels are validated by running the program; here we
    // only assert the prologue/epilogue shape.
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::choice(vec![
            Pattern::seq(vec![
                Pattern::NonTerminal("start".into()),
                Pattern::literal("+"),
                Pattern::literal("n"),
            ]),
            Pattern::literal("n"),
        ]),
    );
    let prog = Grammar::new(rules, "start").compile().unwrap();
    assert_eq!(prog.rule_count, 1);
    // Prologue is at code[2]; the bootstrap is the usual Call/End pair.
    assert!(matches!(prog.code[0], Instruction::Call(Label(2))));
    assert!(matches!(prog.code[1], Instruction::End));
    assert!(matches!(
        prog.code[2],
        Instruction::RuleEnter(MemoId(0), RuleKind::Lr, _)
    ));
    // Last three instructions: LRTail, then the final Return for the rule.
    let n = prog.code.len();
    assert!(matches!(
        prog.code[n - 2],
        Instruction::LRTail(MemoId(0), _)
    ));
    assert!(matches!(prog.code[n - 1], Instruction::Return));
    // No Memo-kind RuleEnter / MemoClose anywhere — LR rules use the
    // LR prologue/epilogue exclusively.
    for ins in &prog.code {
        assert!(
            !matches!(
                ins,
                Instruction::RuleEnter(_, RuleKind::Memo, _) | Instruction::MemoClose(..)
            ),
            "LR rule must not emit Memo-kind RuleEnter/MemoClose: {:?}",
            ins
        );
    }
    // RuleEnter's return label points at the rule's Return (last instruction).
    if let Instruction::RuleEnter(_, RuleKind::Lr, Label(ret)) = prog.code[2] {
        assert_eq!(ret as usize, n - 1);
    }
    // LRTail's body-start label points at the instruction after RuleEnter.
    if let Instruction::LRTail(_, Label(body_start)) = prog.code[n - 2] {
        assert_eq!(body_start, 3);
    }
}

#[test]
fn right_recursive_rule_is_not_marked_lr() {
    // start <- "n" "+" start / "n"
    // The recursive call is not in first-call position — "n" consumes
    // input first. Compile must use the standard Memo-kind RuleEnter
    // and MemoClose.
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::choice(vec![
            Pattern::seq(vec![
                Pattern::literal("n"),
                Pattern::literal("+"),
                Pattern::NonTerminal("start".into()),
            ]),
            Pattern::literal("n"),
        ]),
    );
    let prog = Grammar::new(rules, "start").compile().unwrap();
    let has_memo_open = prog
        .code
        .iter()
        .any(|i| matches!(i, Instruction::RuleEnter(_, RuleKind::Memo, _)));
    let has_lr = prog.code.iter().any(|i| {
        matches!(
            i,
            Instruction::RuleEnter(_, RuleKind::Lr, _) | Instruction::LRTail(..)
        )
    });
    assert!(
        has_memo_open,
        "right-recursive rule must use Memo-kind RuleEnter"
    );
    assert!(!has_lr, "right-recursive rule must not emit LR opcodes");
}

#[test]
fn indirect_lr_cycle_of_2_emits_lrbody_lrtail() {
    // a <- b "x" / "y"
    // b <- a "z" / "w"
    // First-call SCC {a, b}; both rules must be wrapped as LR.
    let mut rules = HashMap::new();
    rules.insert(
        "a".into(),
        Pattern::choice(vec![
            Pattern::seq(vec![
                Pattern::NonTerminal("b".into()),
                Pattern::literal("x"),
            ]),
            Pattern::literal("y"),
        ]),
    );
    rules.insert(
        "b".into(),
        Pattern::choice(vec![
            Pattern::seq(vec![
                Pattern::NonTerminal("a".into()),
                Pattern::literal("z"),
            ]),
            Pattern::literal("w"),
        ]),
    );
    let prog = Grammar::new(rules, "a").compile().unwrap();
    let lr_bodies = prog
        .code
        .iter()
        .filter(|i| matches!(i, Instruction::RuleEnter(_, RuleKind::Lr, _)))
        .count();
    let lr_tails = prog
        .code
        .iter()
        .filter(|i| matches!(i, Instruction::LRTail(..)))
        .count();
    assert_eq!(lr_bodies, 2, "both SCC members must emit Lr-kind RuleEnter");
    assert_eq!(lr_tails, 2, "both SCC members must emit LRTail");
    for ins in &prog.code {
        assert!(
            !matches!(
                ins,
                Instruction::RuleEnter(_, RuleKind::Memo, _) | Instruction::MemoClose(..)
            ),
            "indirect-LR rules must not emit Memo-kind RuleEnter/MemoClose: {:?}",
            ins
        );
    }
}

#[test]
fn indirect_lr_cycle_of_3_emits_lrbody_lrtail() {
    // a <- b "x" / "p"
    // b <- c "y" / "q"
    // c <- a "z" / "r"
    // First-call SCC {a, b, c}; all three rules must be wrapped as LR.
    let mut rules = HashMap::new();
    rules.insert(
        "a".into(),
        Pattern::choice(vec![
            Pattern::seq(vec![
                Pattern::NonTerminal("b".into()),
                Pattern::literal("x"),
            ]),
            Pattern::literal("p"),
        ]),
    );
    rules.insert(
        "b".into(),
        Pattern::choice(vec![
            Pattern::seq(vec![
                Pattern::NonTerminal("c".into()),
                Pattern::literal("y"),
            ]),
            Pattern::literal("q"),
        ]),
    );
    rules.insert(
        "c".into(),
        Pattern::choice(vec![
            Pattern::seq(vec![
                Pattern::NonTerminal("a".into()),
                Pattern::literal("z"),
            ]),
            Pattern::literal("r"),
        ]),
    );
    let prog = Grammar::new(rules, "a").compile().unwrap();
    let lr_bodies = prog
        .code
        .iter()
        .filter(|i| matches!(i, Instruction::RuleEnter(_, RuleKind::Lr, _)))
        .count();
    let lr_tails = prog
        .code
        .iter()
        .filter(|i| matches!(i, Instruction::LRTail(..)))
        .count();
    assert_eq!(
        lr_bodies, 3,
        "all three SCC members must emit Lr-kind RuleEnter"
    );
    assert_eq!(lr_tails, 3, "all three SCC members must emit LRTail");
    for ins in &prog.code {
        assert!(
            !matches!(
                ins,
                Instruction::RuleEnter(_, RuleKind::Memo, _) | Instruction::MemoClose(..)
            ),
            "indirect-LR rules must not emit Memo-kind RuleEnter/MemoClose: {:?}",
            ins
        );
    }
}

#[test]
fn right_recursive_two_rule_grammar_is_not_marked_lr() {
    // a <- "x" b / "y"
    // b <- "z" a / "w"
    // Each call site is preceded by a literal — no first-call edges, so
    // no SCC and no LR wrapping. Sanity check that the analysis isn't
    // over-eager about cross-rule recursion.
    let mut rules = HashMap::new();
    rules.insert(
        "a".into(),
        Pattern::choice(vec![
            Pattern::seq(vec![
                Pattern::literal("x"),
                Pattern::NonTerminal("b".into()),
            ]),
            Pattern::literal("y"),
        ]),
    );
    rules.insert(
        "b".into(),
        Pattern::choice(vec![
            Pattern::seq(vec![
                Pattern::literal("z"),
                Pattern::NonTerminal("a".into()),
            ]),
            Pattern::literal("w"),
        ]),
    );
    let prog = Grammar::new(rules, "a").compile().unwrap();
    let has_lr = prog.code.iter().any(|i| {
        matches!(
            i,
            Instruction::RuleEnter(_, RuleKind::Lr, _) | Instruction::LRTail(..)
        )
    });
    assert!(!has_lr, "non-first-call mutual recursion must not emit LR");
    let has_memo_open = prog
        .code
        .iter()
        .any(|i| matches!(i, Instruction::RuleEnter(_, RuleKind::Memo, _)));
    assert!(has_memo_open, "non-LR rules must use Memo-kind RuleEnter");
}

#[test]
fn lr_through_nullable_prefix_is_detected() {
    // start <- opt start "+" "n" / "n"
    // opt   <- "x"?
    // The recursive call is gated by an optional prefix; nullability
    // analysis must propagate the first-call through `opt`.
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::choice(vec![
            Pattern::seq(vec![
                Pattern::NonTerminal("opt".into()),
                Pattern::NonTerminal("start".into()),
                Pattern::literal("+"),
                Pattern::literal("n"),
            ]),
            Pattern::literal("n"),
        ]),
    );
    rules.insert(
        "opt".into(),
        Pattern::Optional(Box::new(Pattern::literal("x"))),
    );
    let prog = Grammar::new(rules, "start").compile().unwrap();
    assert!(prog
        .code
        .iter()
        .any(|i| matches!(i, Instruction::RuleEnter(MemoId(0), RuleKind::Lr, _))));
}

// -- FOLLOW analysis ------------------------------------------------------

fn rule(name: &str) -> FollowElement {
    FollowElement::Rule(name.into())
}

fn lit(s: &str) -> FollowElement {
    FollowElement::Literal(s.as_bytes().to_vec())
}

fn cap_lit(kind: &str, s: &str) -> FollowElement {
    FollowElement::Capture {
        kind: kind.into(),
        inner: Box::new(lit(s)),
    }
}

#[test]
fn follow_set_single_rule_tail() {
    // start <- a; a <- 'x'
    let mut rules = HashMap::new();
    rules.insert("start".into(), Pattern::NonTerminal("a".into()));
    rules.insert("a".into(), Pattern::literal("x"));
    let g = Grammar::new(rules, "start");
    let follow = compute_follow(&g);
    assert_eq!(follow["a"], BTreeSetOf::from([FollowElement::Eof]));
    assert_eq!(follow["start"], BTreeSetOf::from([FollowElement::Eof]));
}

#[test]
fn follow_set_sequence_after() {
    // start <- a 'y'; a <- 'x'
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::seq(vec![
            Pattern::NonTerminal("a".into()),
            Pattern::literal("y"),
        ]),
    );
    rules.insert("a".into(), Pattern::literal("x"));
    let g = Grammar::new(rules, "start");
    let follow = compute_follow(&g);
    assert_eq!(follow["a"], BTreeSetOf::from([lit("y")]));
}

#[test]
fn follow_set_choice_tail() {
    // start <- a / b; a <- 'x'; b <- 'y'
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::choice(vec![
            Pattern::NonTerminal("a".into()),
            Pattern::NonTerminal("b".into()),
        ]),
    );
    rules.insert("a".into(), Pattern::literal("x"));
    rules.insert("b".into(), Pattern::literal("y"));
    let g = Grammar::new(rules, "start");
    let follow = compute_follow(&g);
    // Each alt sits at the tail of `start`; FOLLOW(start) = {Eof}.
    assert_eq!(follow["a"], BTreeSetOf::from([FollowElement::Eof]));
    assert_eq!(follow["b"], BTreeSetOf::from([FollowElement::Eof]));
}

#[test]
fn follow_set_repeat_self() {
    // start <- a*; a <- 'x'
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::Repeat(Box::new(Pattern::NonTerminal("a".into()))),
    );
    rules.insert("a".into(), Pattern::literal("x"));
    let g = Grammar::new(rules, "start");
    let follow = compute_follow(&g);
    // Body can iterate (next iteration's FIRST = FIRST(a) = {Rule("a")}),
    // and at termination FOLLOW(start) = {Eof} applies.
    let f_a = &follow["a"];
    assert!(
        f_a.contains(&rule("a")),
        "FOLLOW(a) should include Rule(\"a\") (iterating body), got {f_a:?}"
    );
    assert!(
        f_a.contains(&FollowElement::Eof),
        "FOLLOW(a) should include Eof (after the loop terminates), got {f_a:?}"
    );
}

#[test]
fn follow_set_nullable_skip() {
    // start <- a b 'z'; a <- 'x'; b <- 'y'?
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::seq(vec![
            Pattern::NonTerminal("a".into()),
            Pattern::NonTerminal("b".into()),
            Pattern::literal("z"),
        ]),
    );
    rules.insert("a".into(), Pattern::literal("x"));
    rules.insert(
        "b".into(),
        Pattern::Optional(Box::new(Pattern::literal("y"))),
    );
    let g = Grammar::new(rules, "start");
    let follow = compute_follow(&g);
    let f_a = &follow["a"];
    // After `a`: FIRST(b) = {Rule("b")} (opaque, one-level), plus — since b
    // is nullable — FIRST of what follows b, the literal 'z'.
    assert!(
        f_a.contains(&rule("b")),
        "expected Rule(\"b\") in FOLLOW(a): {f_a:?}"
    );
    assert!(
        f_a.contains(&lit("z")),
        "expected 'z' in FOLLOW(a): {f_a:?}"
    );
}

#[test]
fn follow_set_recursive() {
    // list <- 'x' (',' list)?
    let mut rules = HashMap::new();
    rules.insert(
        "list".into(),
        Pattern::seq(vec![
            Pattern::literal("x"),
            Pattern::Optional(Box::new(Pattern::seq(vec![
                Pattern::literal(","),
                Pattern::NonTerminal("list".into()),
            ]))),
        ]),
    );
    let g = Grammar::new(rules, "list");
    let follow = compute_follow(&g);
    // The recursive `list` call is at the tail of an Optional, which is at
    // the tail of `list` itself — so FOLLOW(list) propagates to itself,
    // and the seed Eof reaches all the way down.
    assert_eq!(follow["list"], BTreeSetOf::from([FollowElement::Eof]));
}

#[test]
fn follow_set_capture_preserved() {
    // start <- a @punctuation{','}; a <- 'x'
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::seq(vec![
            Pattern::NonTerminal("a".into()),
            Pattern::Capture("punctuation".into(), Box::new(Pattern::literal(","))),
        ]),
    );
    rules.insert("a".into(), Pattern::literal("x"));
    let g = Grammar::new(rules, "start");
    let follow = compute_follow(&g);
    let f_a = &follow["a"];
    assert!(
        f_a.contains(&cap_lit("punctuation", ",")),
        "FOLLOW(a) should preserve the capture wrapper: {f_a:?}"
    );
}

#[test]
fn follow_set_predicate_lookahead() {
    // start <- a &'y' 'z'; a <- 'x'
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::seq(vec![
            Pattern::NonTerminal("a".into()),
            Pattern::AndPredicate(Box::new(Pattern::literal("y"))),
            Pattern::literal("z"),
        ]),
    );
    rules.insert("a".into(), Pattern::literal("x"));
    let g = Grammar::new(rules, "start");
    let follow = compute_follow(&g);
    let f_a = &follow["a"];
    // The lookahead &'y' contributes FIRST('y') = {'y'} (predicate is
    // nullable for sequence flow, so FIRST of 'z' also reaches FOLLOW(a)).
    assert!(
        f_a.contains(&lit("y")),
        "expected 'y' in FOLLOW(a): {f_a:?}"
    );
    assert!(
        f_a.contains(&lit("z")),
        "expected 'z' in FOLLOW(a): {f_a:?}"
    );
}

#[test]
fn follow_set_real_sqlite_grammar() {
    let src = include_str!("../grammars/sqlite.peg");
    let grammar = parse(src).expect("sqlite.peg parses");
    let follow = compute_follow(&grammar);

    let f_result_column = follow.get("result_column").expect("result_column defined");
    assert!(
        f_result_column.contains(&cap_lit("punctuation", ",")),
        "FOLLOW(result_column) should include @punctuation{{','}}: {f_result_column:?}"
    );
    // One-level analysis: the rule called right after `result_list` in
    // `select_core` is `from_clause` (whose FIRST is `kw_from`). Authors
    // chasing the missing-keyword question follow Rule chains one step.
    assert!(
        f_result_column.contains(&rule("from_clause")),
        "FOLLOW(result_column) should include Rule(\"from_clause\"): {f_result_column:?}"
    );

    // Load-bearing acceptance criterion: PR #101 hand-missed `kw_returning`
    // in `where_clause_boundary`. With the FOLLOW analysis surfacing
    // `returning_clause` as an immediate follower of `where_clause`
    // (through `update_stmt` / `delete_stmt`), the author would have a
    // direct lead to the missing keyword via `pegc follow-set
    // returning_clause` → kw_returning.
    let f_where_clause = follow.get("where_clause").expect("where_clause defined");
    assert!(
        f_where_clause.contains(&rule("returning_clause")),
        "FOLLOW(where_clause) should include Rule(\"returning_clause\"): {f_where_clause:?}"
    );
}

/// Helper for `BTreeSet::from([...])` since `std::collections::BTreeSet`
/// doesn't have a const `from` for arrays in stable Rust without a
/// `<const N: usize>` import. Defined locally to keep tests readable.
type BTreeSetOf<T> = std::collections::BTreeSet<T>;

// -- partial-match leniency lint ----------------------------------------

fn finding(rule: &str, caller: &str) -> LintFinding {
    LintFinding {
        kind: LintKind::PartialMatchLeniency,
        rule: rule.into(),
        caller: caller.into(),
    }
}

#[test]
fn lint_partial_match_trailing_optional_with_eof_validator() {
    // start <- a; a <- 'x' 'y'?
    // a is called from the start rule, whose only continuation is Eof.
    // Eof rejects any non-empty leftover bytes — a validator. No flag.
    let mut rules = HashMap::new();
    rules.insert("start".into(), Pattern::NonTerminal("a".into()));
    rules.insert(
        "a".into(),
        Pattern::seq(vec![
            Pattern::literal("x"),
            Pattern::Optional(Box::new(Pattern::literal("y"))),
        ]),
    );
    let g = Grammar::new(rules, "start");
    assert!(lint_partial_match(&g).is_empty());
}

#[test]
fn lint_partial_match_no_trailing_nullable_skipped() {
    // start <- a; a <- 'x' 'y' — no trailing optional/nullable.
    let mut rules = HashMap::new();
    rules.insert("start".into(), Pattern::NonTerminal("a".into()));
    rules.insert(
        "a".into(),
        Pattern::seq(vec![Pattern::literal("x"), Pattern::literal("y")]),
    );
    let g = Grammar::new(rules, "start");
    assert!(lint_partial_match(&g).is_empty());
}

#[test]
fn lint_partial_match_anchored_via_andpredicate() {
    // start <- a &'y' 'y'; a <- 'x' 'y'?
    // The AndPredicate anchors the call to a even though a's trailing
    // overlaps with what follows.
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::seq(vec![
            Pattern::NonTerminal("a".into()),
            Pattern::AndPredicate(Box::new(Pattern::literal("y"))),
            Pattern::literal("y"),
        ]),
    );
    rules.insert(
        "a".into(),
        Pattern::seq(vec![
            Pattern::literal("x"),
            Pattern::Optional(Box::new(Pattern::literal("y"))),
        ]),
    );
    let g = Grammar::new(rules, "start");
    assert!(
        lint_partial_match(&g).is_empty(),
        "AndPredicate should anchor"
    );
}

#[test]
fn lint_partial_match_validated_by_disjoint_consumer() {
    // start <- a 'z'; a <- 'x' 'y'?
    // 'z' after a is a non-nullable consumer with FIRST={'z'} disjoint
    // from a's trailing {'y'}. The consumer validates.
    let mut rules = HashMap::new();
    rules.insert(
        "start".into(),
        Pattern::seq(vec![
            Pattern::NonTerminal("a".into()),
            Pattern::literal("z"),
        ]),
    );
    rules.insert(
        "a".into(),
        Pattern::seq(vec![
            Pattern::literal("x"),
            Pattern::Optional(Box::new(Pattern::literal("y"))),
        ]),
    );
    let g = Grammar::new(rules, "start");
    assert!(lint_partial_match(&g).is_empty());
}

#[test]
fn lint_partial_match_absorbed_by_outer_catch_flagged() {
    // start <- (a)*^[;]
    // a <- 'x' 'y'?
    // a's leniency at the call site is absorbed by the *^ recovery
    // wrapper — exactly the PR #101 shape.
    let mut rules = HashMap::new();
    let recovery_body = Pattern::Repeat(Box::new(Pattern::seq(vec![
        Pattern::NotPredicate(Box::new(Pattern::literal(";"))),
        Pattern::AnyChar,
    ])));
    let call_inside_catch = Pattern::Repeat(Box::new(Pattern::Catch {
        inner: Box::new(Pattern::NonTerminal("a".into())),
        label: "recovery".into(),
        recovery: Box::new(recovery_body),
    }));
    rules.insert("start".into(), call_inside_catch);
    rules.insert(
        "a".into(),
        Pattern::seq(vec![
            Pattern::literal("x"),
            Pattern::Optional(Box::new(Pattern::literal("y"))),
        ]),
    );
    let g = Grammar::new(rules, "start");
    let findings = lint_partial_match(&g);
    assert_eq!(findings, vec![finding("a", "start")]);
}

#[test]
fn lint_partial_match_real_sqlite_grammar_aliased_expr_anchored() {
    // The shipped grammar has the PR #101 fix: aliased_expr is anchored
    // via `&(ws result_column_boundary)` inside result_column. The lint
    // must not flag aliased_expr → result_column.
    let source =
        std::fs::read_to_string("grammars/sqlite.peg").expect("sqlite.peg fixture present");
    let g = parse(&source).expect("sqlite.peg parses");
    let findings = lint_partial_match(&g);
    let bad = findings
        .iter()
        .find(|f| f.rule == "aliased_expr" && f.caller == "result_column");
    assert!(
        bad.is_none(),
        "aliased_expr is anchored; should not be flagged"
    );
}

#[test]
fn lint_partial_match_real_sqlite_grammar_unanchored_aliased_expr_flagged() {
    // Synthesize the pre-PR-#101 shape inline: result_column's body is
    // just the choice with no anchor. Verify the lint catches the
    // load-bearing case.
    let source =
        std::fs::read_to_string("grammars/sqlite.peg").expect("sqlite.peg fixture present");
    let mut g = parse(&source).expect("sqlite.peg parses");
    // Replace result_column's body with an unanchored version.
    g.rules.insert(
        "result_column".into(),
        Pattern::OrderedChoice(vec![
            Pattern::NonTerminal("table_star".into()),
            Pattern::Capture("operator".into(), Box::new(Pattern::literal("*"))),
            Pattern::NonTerminal("aliased_expr".into()),
        ]),
    );
    let findings = lint_partial_match(&g);
    let hit = findings
        .iter()
        .find(|f| f.rule == "aliased_expr" && f.caller == "result_column");
    assert!(
        hit.is_some(),
        "load-bearing PR #101 case must be flagged when anchor is stripped; findings: {findings:?}"
    );
}

// ---- Definition-level lenient marker (~rule_name <- body) -----

#[test]
fn definition_lenient_marker_wraps_rule_body() {
    // `~rule <- body` parses the body with a top-level `Pattern::Lenient`
    // wrap, exposing the intent to the lint via the same AST node the
    // call-site `~p` form produces.
    let g = parse("~r <- 'a'?").expect("parse");
    assert_eq!(
        g.rules["r"],
        Pattern::Lenient(Box::new(Pattern::Optional(Box::new(Pattern::literal("a"))))),
    );
}

#[test]
fn definition_lenient_marker_requires_touching_name() {
    let err = parse("~ r <- 'a'?").expect_err("space between `~` and name must error");
    assert!(
        err.message.contains("expected identifier"),
        "unexpected error: {}",
        err.message
    );
}

#[test]
fn definition_lenient_suppresses_lint_at_every_call_site() {
    // Use the Catch-absorbed shape the lint reliably flags: a
    // top-level `*^[;]` recovery loop calling a trailing-Optional rule.
    let unmarked = parse("start <- (r)*^[;]\nr <- 'x' 'y'?").expect("parse unmarked");
    assert!(
        lint_partial_match(&unmarked)
            .iter()
            .any(|f| f.rule == "r" && f.caller == "start"),
        "baseline: unmarked grammar should flag r"
    );

    let marked = parse("start <- (r)*^[;]\n~r <- 'x' 'y'?").expect("parse marked");
    let findings = lint_partial_match(&marked);
    assert!(
        !findings.iter().any(|f| f.rule == "r"),
        "definition-level `~r` should suppress all r-related findings, got: {findings:?}"
    );
}

#[test]
fn definition_lenient_marker_is_runtime_transparent() {
    // The `~name <-` wrap compiles to the same bytecode as the bare form.
    let plain = parse("r <- 'x'+")
        .expect("parse plain")
        .compile()
        .expect("compile plain");
    let marked = parse("~r <- 'x'+")
        .expect("parse marked")
        .compile()
        .expect("compile marked");
    assert_eq!(plain.code, marked.code);
}

// ---- Compile-fatal lint integration ---------------------------

#[test]
fn compile_errors_on_partial_match_leniency() {
    // `*^[;]` Catch-absorbed shape — the lint reliably flags this.
    let g = parse("start <- (r)*^[;]\nr <- 'x' 'y'?").expect("parse");
    let err = g.compile().expect_err("compile should fail");
    match err {
        syntax_highlighter::pegc::CompileError::PartialMatchLeniency(findings) => {
            assert!(
                findings
                    .iter()
                    .any(|f| f.rule == "r" && f.caller == "start"),
                "expected r → start finding, got: {findings:?}"
            );
        }
        other => panic!("expected PartialMatchLeniency, got: {other:?}"),
    }
}

#[test]
fn compile_succeeds_with_definition_lenient_on_flagged_rule() {
    let g = parse("start <- (r)*^[;]\n~r <- 'x' 'y'?").expect("parse");
    g.compile()
        .expect("compile should succeed with definition-level `~r`");
}

#[test]
fn compile_succeeds_with_boundary_catch_anchor() {
    // `^^bad ';'` lowers to `Catch { Seq(a, &';'), bad, recovery }` —
    // anchoring `a` by lookahead so the lint sees no leniency.
    let g = parse("start <- (a ^^bad ';')*\na <- 'x' 'y'?").expect("parse");
    g.compile()
        .expect("compile should succeed with `^^bad ';'` anchor");
}

#[test]
fn compile_succeeds_with_bracketed_close_catch_sugar() {
    // `^^bad ..= '}'` lowers to a `Catch` whose inner is unanchored
    // and whose recovery is `Seq(@recovery{(!'}' .)*}, '}')`. The
    // inner ends in `'}'` (a hard terminator), so `lint_partial_match`
    // sees no trailing-nullable shape and compilation succeeds.
    let g = parse("start <- ('{' a '}' ^^bad ..= '}')*\na <- 'x'+").expect("parse");
    g.compile()
        .expect("compile should succeed with `^^bad ..= '}'` sugar");
}

#[test]
fn bracketed_close_catch_runs_recovery_path() {
    use syntax_highlighter::pegvm::VM;
    // Tiny grammar shaped like the corpus' `block` rule: opening `{`,
    // body that requires `'x'`, closing `}` with `^^bad ..= '}'` to
    // skip to the brace on body failure. Malformed input `{garbage}`
    // exercises the recovery: skip captures `garbage` and `}` is
    // captured as `@punctuation`.
    let g = parse(
        "start <- @punctuation{'{'} (body @punctuation{'}'} ^^bad ..= @punctuation{'}'})\nbody <- 'x'",
    )
    .expect("parse");
    let prog = g.compile().expect("compile");
    let r = VM::new(&prog.code, b"{garbage}").run();
    assert!(r.complete, "recovery should let the parse complete");
    assert_eq!(r.matched, 9);
    let kinds: Vec<&str> = r
        .captures
        .iter()
        .map(|c| prog.capture_kinds[c.kind.0 as usize].as_str())
        .collect();
    assert_eq!(
        kinds,
        vec!["punctuation", "recovery", "punctuation"],
        "expected open-punct, recovery span, close-punct (NOT recovery)"
    );
}

#[test]
fn compile_error_display_lists_findings() {
    let g = parse("start <- (r)*^[;]\nr <- 'x' 'y'?").expect("parse");
    let err = g.compile().expect_err("compile should fail");
    let msg = format!("{err}");
    assert!(msg.contains("partial-match leniency"));
    assert!(msg.contains("`r`"));
    assert!(msg.contains("`start`"));
}

#[test]
fn compile_errors_on_uninferable_boundary() {
    // The start rule has no FOLLOW context other than EOF — for a
    // rule with no callers at all, the inferred boundary would be
    // empty. Construct that case via a non-start unreachable rule.
    let g = parse("start <- 'x'\norphan <- 'a' ^^bad").expect("parse");
    let err = g.compile().expect_err("compile should fail");
    assert!(
        matches!(
            err,
            syntax_highlighter::pegc::CompileError::CannotInferBoundary { .. }
        ),
        "expected CannotInferBoundary, got: {err:?}"
    );
}

// ---- Trivia reserved-rule cascade --------------------------------

fn trivia_idx(program: &syntax_highlighter::pegvm::Program, name: &str) -> usize {
    program
        .rule_names
        .iter()
        .position(|n| n == name)
        .unwrap_or_else(|| {
            panic!(
                "rule `{name}` not indexed; rule_names = {:?}",
                program.rule_names
            )
        })
}

#[test]
fn trivia_root_marks_itself_and_no_root_leaves_bits_false() {
    // Without a `trivia` rule the bits are all false.
    let no_root = parse("start <- 'x'")
        .expect("parse")
        .compile()
        .expect("compile");
    assert!(no_root.rule_is_trivia.iter().all(|b| !b));

    // The trivia rule itself gets the bit when present.
    let with_root = parse("start <- trivia 'x'\ntrivia <- ' '*")
        .expect("parse")
        .compile()
        .expect("compile");
    assert!(with_root.rule_is_trivia[trivia_idx(&with_root, "trivia")]);
    assert!(!with_root.rule_is_trivia[trivia_idx(&with_root, "start")]);
}

#[test]
fn trivia_cascades_to_transitive_callees() {
    let program = parse(
        "start  <- trivia 'x'\n\
         trivia <- ws\n\
         ws     <- (comment / ' ')*\n\
         comment <- '#' (!'\\n' .)*",
    )
    .expect("parse")
    .compile()
    .expect("compile");
    assert!(program.rule_is_trivia[trivia_idx(&program, "trivia")]);
    assert!(program.rule_is_trivia[trivia_idx(&program, "ws")]);
    assert!(program.rule_is_trivia[trivia_idx(&program, "comment")]);
    assert!(!program.rule_is_trivia[trivia_idx(&program, "start")]);
}

#[test]
fn trivia_cascade_skips_catch_bearing_rules() {
    // `victim` is reached from the trivia subgraph but has a catch
    // in its body. The cascade pins it out so the catch's frame stays
    // visible in `pegdb explain-recoveries`.
    let program = parse(
        "start  <- trivia 'x'\n\
         trivia <- (victim / ' ')*\n\
         victim <- 'a' ^bad 'b'",
    )
    .expect("parse")
    .compile()
    .expect("compile");
    assert!(program.rule_is_trivia[trivia_idx(&program, "trivia")]);
    assert!(
        !program.rule_is_trivia[trivia_idx(&program, "victim")],
        "rules containing a catch must not cascade to trivia"
    );
}

#[test]
fn trivia_cascade_handles_recursion() {
    // Trivia subgraph with a self-cycle; fixpoint should terminate
    // and mark every reachable rule.
    let program = parse(
        "start  <- trivia 'x'\n\
         trivia <- ws\n\
         ws     <- (ws / ' ')*",
    )
    .expect("parse")
    .compile()
    .expect("compile");
    assert!(program.rule_is_trivia[trivia_idx(&program, "trivia")]);
    assert!(program.rule_is_trivia[trivia_idx(&program, "ws")]);
}

#[test]
fn backslash_cap_r_matches_crlf_atomically() {
    // `\R` lowers to `'\r\n' / '\n' / '\r'` — CRLF is matched as one
    // two-byte unit, not two separate line breaks.
    let g = parse("r <- \\R").expect("parse");
    let prog = g.compile().expect("compile");
    let r = VM::new(&prog.code, b"\r\n").run();
    assert!(r.complete, "\\R should match CRLF");
    assert_eq!(r.matched, 2, "\\R should consume both CR and LF atomically");

    let r = VM::new(&prog.code, b"\n").run();
    assert!(r.complete, "\\R should match bare LF");
    assert_eq!(r.matched, 1);

    let r = VM::new(&prog.code, b"\r").run();
    assert!(r.complete, "\\R should match bare CR");
    assert_eq!(r.matched, 1);
}

#[test]
fn backslash_z_matches_eof() {
    // `\z` lowers to `!.` — succeeds at end of input, fails otherwise.
    let g = parse("r <- \\z").expect("parse");
    let prog = g.compile().expect("compile");
    let r = VM::new(&prog.code, b"").run();
    assert!(r.complete, "\\z should match empty input");
    assert_eq!(r.matched, 0);

    let r = VM::new(&prog.code, b"x").run();
    assert!(!r.complete, "\\z should fail when bytes remain");
}

#[test]
fn all_shipped_grammars_compile_clean() {
    // Load-bearing: every grammar in `grammars/*.peg` must compile
    // cleanly via `pegc::compile`. Re-introducing partial-match
    // leniency without a `~` marker or `^^lbl` anchor breaks this.
    for entry in std::fs::read_dir("grammars").expect("read grammars dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("peg") {
            continue;
        }
        let src = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("read {path:?}"));
        let g = parse(&src).unwrap_or_else(|e| panic!("parse {path:?}: {e}"));
        g.compile()
            .unwrap_or_else(|e| panic!("compile {path:?}: {e}"));
    }
}
