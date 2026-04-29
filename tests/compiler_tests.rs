use std::collections::HashMap;

use syntax_highlighter::pegc::{compile_pattern, Grammar, Pattern};
use syntax_highlighter::pegvm::{
    Capture, CaptureKind, CharSet, Instruction, Label, MatchResult, MemoId, RuleKind, VM,
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
        }
    );
    assert_eq!(
        run_pattern(&p, b"ax"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
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
        }
    );
    assert_eq!(
        run_pattern(&p, b"bar"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
        }
    );
    assert_eq!(
        run_pattern(&p, b"baz"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
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
        }
    );
    assert_eq!(
        run_pattern(&p, b"aaa"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
        }
    );
    assert_eq!(
        run_pattern(&p, b"aab"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
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
        }
    );
    assert_eq!(
        run_pattern(&p, b"aaa"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
        }
    );
}

fn recover(inner: Pattern, kind: &str) -> Pattern {
    Pattern::RecoverRepeat {
        inner: Box::new(inner),
        recovery_kind: kind.into(),
    }
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
fn recover_repeat_truncates_failed_inner_attempt_captures() {
    // inner = @open{"a"} "b" — the @open capture opens before the "b"
    // that may fail. After a failed attempt, that partial @open must NOT
    // appear in the result; only successful attempts contribute to it.
    //
    // Kind interning order: RecoverRepeat enters before recursing into
    // inner, so "recovery" interns first (id 0), "open" second (id 1).
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
            cap(0, 0, 1), // recovery: 'a' (failed inner attempt's @open(0,1) is gone)
            cap(0, 1, 2), // recovery: 'x'
            cap(1, 2, 3), // @open: the successful 'a' at sp=2
        ],
        "the failed inner attempt's open capture must not leak into the result",
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
        }
    );
    assert_eq!(
        run_pattern(&p, b"-x"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
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
    let p = recover(Pattern::literal("a"), "recovery");
    let prog = compile_pattern(&p);
    // loop_top: Choice rec
    //           Char 'a'
    //           Commit loop_top
    // rec:      Choice exit
    //           CaptureBegin recovery
    //           Any(1)
    //           CaptureEnd
    //           Commit loop_top
    // exit:     End
    //
    // The recovery loop uses fresh Choice/Commit per iteration (not
    // PartialCommit) so each retry's backtrack baseline is at the
    // advanced sp. See src/pegvm/README.md invariant 1.
    assert_eq!(
        prog.code,
        vec![
            Instruction::Choice(Label(3)),             // → rec
            Instruction::Char(b'a'),                   // <inner>
            Instruction::Commit(Label(0)),             // → loop_top
            Instruction::Choice(Label(8)),             // rec: → exit
            Instruction::CaptureBegin(CaptureKind(0)), // recovery
            Instruction::Any(1),
            Instruction::CaptureEnd,
            Instruction::Commit(Label(0)), // → loop_top
            Instruction::End,              // exit
        ]
    );
    assert_eq!(prog.capture_kinds, vec!["recovery".to_string()]);
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
