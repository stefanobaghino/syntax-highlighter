use syntax_highlighter::pegvm::{
    Capture, CaptureKind, CharSet, Instruction, Label, MatchResult, MemoId, RuleKind, SetId, VM,
};

fn run(program: &[Instruction], input: &[u8]) -> MatchResult {
    VM::new(program, input).run()
}

fn run_with_classes(program: &[Instruction], classes: &[CharSet], input: &[u8]) -> MatchResult {
    VM::new(program, input).with_char_sets(classes).run()
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
fn match_literal_abc() {
    let prog = [
        Instruction::Byte(b'a'),
        Instruction::Byte(b'b'),
        Instruction::Byte(b'c'),
        Instruction::End,
    ];
    assert_eq!(
        run(&prog, b"abc"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run(&prog, b"abcd"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run(&prog, b"abx"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: false,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run(&prog, b"ab"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: false,
            recovery_diagnostics: vec![],
        }
    );
}

#[test]
fn match_charset() {
    let digits = CharSet::from_ranges(&[('0', '9')]).unwrap();
    let classes = [digits];
    let prog = [Instruction::CharSet(SetId(0)), Instruction::End];
    assert_eq!(
        run_with_classes(&prog, &classes, b"7"),
        MatchResult {
            matched: 1,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run_with_classes(&prog, &classes, b"a"),
        MatchResult {
            matched: 0,
            captures: vec![],
            complete: false,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run_with_classes(&prog, &classes, b""),
        MatchResult {
            matched: 0,
            captures: vec![],
            complete: false,
            recovery_diagnostics: vec![],
        }
    );
}

#[test]
fn any_consumes_one_codepoint() {
    let prog = [Instruction::Any, Instruction::Any, Instruction::End];
    // ASCII: two single-byte code points.
    assert_eq!(
        run(&prog, b"xy"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    // "éx": 'é' is 0xC3 0xA9 (two bytes), 'x' is one — three bytes total.
    assert_eq!(
        run(&prog, "éx".as_bytes()),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    // Single byte → second Any fails at EOF.
    let r = run(&prog, b"x");
    assert!(!r.complete);
}

#[test]
fn ordered_choice_first_alternative() {
    // p = "ab" / "ax"
    // Choice L1 ; Char a ; Char b ; Commit L2 ; L1: Char a ; Char x ; L2: End
    let prog = [
        Instruction::Choice(Label(4)),
        Instruction::Byte(b'a'),
        Instruction::Byte(b'b'),
        Instruction::Commit(Label(6)),
        Instruction::Byte(b'a'),
        Instruction::Byte(b'x'),
        Instruction::End,
    ];
    assert_eq!(
        run(&prog, b"ab"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run(&prog, b"ax"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    let r = run(&prog, b"ay");
    assert!(!r.complete);
}

#[test]
fn repetition_zero_or_more() {
    // p = [a]*
    // Choice L2 ; L_body: Char a ; PartialCommit L_body ; L2: End
    // PartialCommit re-uses the existing backtrack entry rather than pushing a new one.
    let prog = [
        Instruction::Choice(Label(3)),
        Instruction::Byte(b'a'),
        Instruction::PartialCommit(Label(1)),
        Instruction::End,
    ];
    assert_eq!(
        run(&prog, b""),
        MatchResult {
            matched: 0,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run(&prog, b"a"),
        MatchResult {
            matched: 1,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run(&prog, b"aaaa"),
        MatchResult {
            matched: 4,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    assert_eq!(
        run(&prog, b"aaab"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
}

#[test]
fn not_predicate() {
    // !'a' .  : matches one char that is not 'a'
    // Choice L1 ; Char a ; FailTwice ; L1: Any 1 ; End
    let prog = [
        Instruction::Choice(Label(3)),
        Instruction::Byte(b'a'),
        Instruction::FailTwice,
        Instruction::Any,
        Instruction::End,
    ];
    assert_eq!(
        run(&prog, b"b"),
        MatchResult {
            matched: 1,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    let r_a = run(&prog, b"a");
    assert!(!r_a.complete);
    let r_empty = run(&prog, b"");
    assert!(!r_empty.complete);
}

#[test]
fn call_and_return() {
    // main = sub sub  ; sub = 'a'
    //  0: Call 4
    //  1: Call 4
    //  2: End
    //  3: (unreachable Fail to separate)
    //  4: Char a
    //  5: Return
    let prog = [
        Instruction::Call(Label(4)),
        Instruction::Call(Label(4)),
        Instruction::End,
        Instruction::Fail,
        Instruction::Byte(b'a'),
        Instruction::Return,
    ];
    assert_eq!(
        run(&prog, b"aa"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
    let r = run(&prog, b"ab");
    assert!(!r.complete);
    assert_eq!(r.matched, 1);
}

#[test]
fn captures_recorded_on_success() {
    // capture("ab")
    // 0: CaptureBegin 7
    // 1: Char a
    // 2: Char b
    // 3: CaptureEnd
    // 4: End
    let prog = [
        Instruction::CaptureBegin(CaptureKind(7)),
        Instruction::Byte(b'a'),
        Instruction::Byte(b'b'),
        Instruction::CaptureEnd,
        Instruction::End,
    ];
    assert_eq!(
        run(&prog, b"ab"),
        MatchResult {
            matched: 2,
            captures: vec![cap(7, 0, 2)],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
}

#[test]
fn captures_truncated_on_backtrack() {
    // (capture("ab")) / "ax"
    // First alternative captures 'a' then fails on 'b'. The capture
    // recorded inside it must be discarded before the second alternative.
    //
    //  0: Choice L1 (=6)
    //  1: CaptureBegin 9
    //  2: Char a
    //  3: Char b
    //  4: CaptureEnd
    //  5: Commit L2 (=8)
    //  6: Char a
    //  7: Char x
    //  8: End
    let prog = [
        Instruction::Choice(Label(6)),
        Instruction::CaptureBegin(CaptureKind(9)),
        Instruction::Byte(b'a'),
        Instruction::Byte(b'b'),
        Instruction::CaptureEnd,
        Instruction::Commit(Label(8)),
        Instruction::Byte(b'a'),
        Instruction::Byte(b'x'),
        Instruction::End,
    ];
    assert_eq!(
        run(&prog, b"ax"),
        MatchResult {
            matched: 2,
            captures: vec![], // discarded during backtrack
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
}

#[test]
fn nested_captures_kept_in_order() {
    // outer( inner('a') inner('b') )
    //  0: CaptureBegin 1   (outer)
    //  1: CaptureBegin 2   (inner)
    //  2: Char a
    //  3: CaptureEnd
    //  4: CaptureBegin 2
    //  5: Char b
    //  6: CaptureEnd
    //  7: CaptureEnd       (outer)
    //  8: End
    let prog = [
        Instruction::CaptureBegin(CaptureKind(1)),
        Instruction::CaptureBegin(CaptureKind(2)),
        Instruction::Byte(b'a'),
        Instruction::CaptureEnd,
        Instruction::CaptureBegin(CaptureKind(2)),
        Instruction::Byte(b'b'),
        Instruction::CaptureEnd,
        Instruction::CaptureEnd,
        Instruction::End,
    ];
    assert_eq!(
        run(&prog, b"ab"),
        MatchResult {
            matched: 2,
            captures: vec![
                cap(1, 0, 2), // outer
                cap(2, 0, 1), // inner 1
                cap(2, 1, 2), // inner 2
            ],
            complete: true,
            recovery_diagnostics: vec![],
        }
    );
}

#[test]
fn charset_negate_and_union() {
    let vowels = CharSet::from_chars(&['a', 'e', 'i', 'o', 'u']);
    let consonants = vowels.negate();
    assert!(!consonants.contains_char('a'));
    assert!(consonants.contains_char('b'));
    let merged = vowels.union(&CharSet::from_chars(&['y']));
    assert!(merged.contains_char('y'));
    assert!(merged.contains_char('a'));
}

#[test]
fn partial_match_on_failure_returns_max_sp_and_open_captures() {
    // Grammar: "ab" @mark{ "cd" }  — match "ab", then a captured "cd".
    //  0: Char a
    //  1: Char b
    //  2: CaptureBegin 0
    //  3: Char c
    //  4: Char d
    //  5: CaptureEnd
    //  6: End
    //
    // Input "abcX" advances to sp=3 inside the @mark capture, then fails
    // on 'd' vs 'X'. Expect complete:false, matched=3, and one capture
    // still open at the failure point closed at sp=3 (start=2).
    let prog = [
        Instruction::Byte(b'a'),
        Instruction::Byte(b'b'),
        Instruction::CaptureBegin(CaptureKind(0)),
        Instruction::Byte(b'c'),
        Instruction::Byte(b'd'),
        Instruction::CaptureEnd,
        Instruction::End,
    ];
    let r = run(&prog, b"abcX");
    assert!(!r.complete);
    assert_eq!(r.matched, 3);
    assert_eq!(r.captures, vec![cap(0, 2, 3)]);
}

#[test]
fn partial_match_fail_before_any_progress() {
    let prog = [Instruction::Byte(b'a'), Instruction::End];
    let r = run(&prog, b"z");
    assert!(!r.complete);
    assert_eq!(r.matched, 0);
    assert_eq!(r.captures, vec![]);
}

#[test]
fn partial_match_prefers_deepest_point_across_backtracks() {
    // Grammar: "aaa" / "aab"
    // First alternative advances to sp=2 on "aaX" before failing on 'a'
    // vs 'X'; second alternative fails earlier. max_sp should stick at 2.
    //
    //  0: Choice L1 (=5)
    //  1: Char a
    //  2: Char a
    //  3: Char a
    //  4: Commit L2 (=8)
    //  5: Char a   (L1)
    //  6: Char a
    //  7: Char b
    //  8: End      (L2)
    let prog = [
        Instruction::Choice(Label(5)),
        Instruction::Byte(b'a'),
        Instruction::Byte(b'a'),
        Instruction::Byte(b'a'),
        Instruction::Commit(Label(8)),
        Instruction::Byte(b'a'),
        Instruction::Byte(b'a'),
        Instruction::Byte(b'b'),
        Instruction::End,
    ];
    let r = run(&prog, b"aaX");
    assert!(!r.complete);
    assert_eq!(r.matched, 2);
}

#[test]
fn partial_match_captures_survive_backtrack_below_watermark() {
    // Regression for the lazy-snapshot path: an open capture created
    // inside a failing first alternative must still appear in the
    // partial-match result even though `fail()` truncates the captures
    // vector below the `max_sp` watermark before the second alternative
    // runs. The lazy snapshot has to rescue those captures before the
    // truncate; otherwise the final result loses them.
    //
    // Grammar: `@mark{ "abc" } / "ab" "x"` against input "abdX".
    //  0: Choice L1 (=6)
    //  1: CaptureBegin 0
    //  2: Char a
    //  3: Char b
    //  4: Char c       <- fails here on 'd', sp was 2 → max_sp=2,
    //                     captures=[{kind:0,start:0,end:None}]
    //  5: CaptureEnd
    //  6: Commit L2 (=11) — unreached; Char c failure triggers Backtrack
    //                         unwind that truncates captures to 0.
    //  7: Char a     (L1, second alternative)
    //  8: Char b
    //  9: Char x      <- fails here on 'd'
    // 10: End        (L2, unreached)
    //
    // Expect: complete=false, matched=2, one capture (kind=0, 0..2).
    let prog = [
        Instruction::Choice(Label(7)),
        Instruction::CaptureBegin(CaptureKind(0)),
        Instruction::Byte(b'a'),
        Instruction::Byte(b'b'),
        Instruction::Byte(b'c'),
        Instruction::CaptureEnd,
        Instruction::Commit(Label(10)),
        Instruction::Byte(b'a'),
        Instruction::Byte(b'b'),
        Instruction::Byte(b'x'),
        Instruction::End,
    ];
    let r = run(&prog, b"abdX");
    assert!(!r.complete);
    assert_eq!(r.matched, 2);
    assert_eq!(r.captures, vec![cap(0, 0, 2)]);
}

// Hand-built bytecode for direct left-recursion. These tests pin the
// VM-level semantics on a raw program so VM changes can be validated
// without going through the compiler. Equivalent grammar:
//
//     expr <- expr "+" "n" / "n"
//
// Code layout:
//
//      0: Call(2)              ; bootstrap → expr
//      1: End
//      2: RuleEnter(0, Lr, 14) ; LR prologue (return_label = 14, the Return)
//      3: Choice(11)           ; body: try first alternative
//      4: Call(2)              ;   recursive call (RuleEnter hit replays seed
//                              ;   or fails when seed is None)
//      5: Char('+')
//      6: CaptureBegin(0)      ;   tag the right operand as kind=0
//      7: Char('n')
//      8: CaptureEnd
//      9: Commit(13)
//     10: -- (unused; padding)
//     11: CaptureBegin(0)      ; second alternative: leaf
//     12: Char('n')
//     13: -- end-of-body marker (LRTail follows)
//     14: -- this is the Return target; we lay LRTail at 13 below
//
// The actual instruction stream is simpler than the comment suggests; see
// the array literals below. The constants are: body_start=3, lrtail=13,
// return_addr=14.
fn lr_expr_program() -> Vec<Instruction> {
    let body_start = 3u32;
    vec![
        // 0: bootstrap
        Instruction::Call(Label(2)),
        Instruction::End,
        // 2: RuleEnter (return_addr = 14)
        Instruction::RuleEnter(MemoId(0), RuleKind::Lr, Label(14)),
        // 3 (body_start): Choice → 11 (second alternative)
        Instruction::Choice(Label(10)),
        // 4: Call(expr)  -- recursive
        Instruction::Call(Label(2)),
        // 5: '+'
        Instruction::Byte(b'+'),
        // 6: CaptureBegin
        Instruction::CaptureBegin(CaptureKind(0)),
        // 7: 'n'
        Instruction::Byte(b'n'),
        // 8: CaptureEnd
        Instruction::CaptureEnd,
        // 9: Commit → 13 (LRTail)
        Instruction::Commit(Label(13)),
        // 10: second alt — CaptureBegin
        Instruction::CaptureBegin(CaptureKind(0)),
        // 11: 'n'
        Instruction::Byte(b'n'),
        // 12: CaptureEnd
        Instruction::CaptureEnd,
        // 13: LRTail (body_start = 3)
        Instruction::LRTail(MemoId(0), Label(body_start)),
        // 14: Return
        Instruction::Return,
    ]
}

#[test]
fn lr_single_atom_no_growth_returns_seed() {
    // Input "n": first iteration matches via the second alternative. The
    // recursive call inside the first alternative fails (seed=None), so
    // the body falls through to the leaf, sp advances 0→1. LRTail sees
    // growth (0→1), updates seed, and re-iterates from sp=0. Second
    // iteration: recursive call now hits the seed (sp=1), returns to
    // LRTail's caller's continuation. The body tries to match '+', but
    // input ends — fails. fail() encounters the LFrame with seed=Some,
    // accepts the seed, returns true at return_addr (14). Final result:
    // matched=1, one capture for the leaf 'n'.
    let prog = lr_expr_program();
    let r = run(&prog, b"n");
    assert_eq!(r.matched, 1);
    assert!(r.complete);
    assert_eq!(r.captures, vec![cap(0, 0, 1)]);
}

#[test]
fn lr_left_associative_chain() {
    // Input "n+n+n": the seed-and-grow loop must produce the
    // left-associative parse. Captures should emit in input order:
    // leaf at 0..1, leaf at 2..3, leaf at 4..5.
    let prog = lr_expr_program();
    let r = run(&prog, b"n+n+n");
    assert_eq!(r.matched, 5);
    assert!(r.complete);
    assert_eq!(r.captures, vec![cap(0, 0, 1), cap(0, 2, 3), cap(0, 4, 5)]);
}

#[test]
fn lr_partial_match_when_input_ends_mid_chain() {
    // Input "n+n+": after consuming "n+n", the next iteration tries
    // "n+n + n" but the trailing 'n' is missing. fail() should rescue
    // with the prior seed (sp=3) and the parse succeeds at matched=3.
    let prog = lr_expr_program();
    let r = run(&prog, b"n+n+");
    assert_eq!(r.matched, 3);
    assert!(r.complete);
    assert_eq!(r.captures, vec![cap(0, 0, 1), cap(0, 2, 3)]);
}

#[test]
fn lr_no_match_returns_partial() {
    // Input "x": the leaf 'n' fails on first iteration with no seed.
    // The LR rule fails (no rescue possible), the parse is partial.
    let prog = lr_expr_program();
    let r = run(&prog, b"x");
    assert!(!r.complete);
    assert_eq!(r.matched, 0);
    assert_eq!(r.captures, vec![]);
}

#[test]
fn lr_empty_input_returns_partial() {
    let prog = lr_expr_program();
    let r = run(&prog, b"");
    assert!(!r.complete);
    assert_eq!(r.matched, 0);
    assert_eq!(r.captures, vec![]);
}
