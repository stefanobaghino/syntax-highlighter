use syntax_highlighter::pegvm::{
    Capture, CaptureKind, CharSet, Instruction, Label, MatchResult, VM,
};

fn run(program: &[Instruction], input: &[u8]) -> MatchResult {
    VM::new(program, input).run()
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
        Instruction::Char(b'a'),
        Instruction::Char(b'b'),
        Instruction::Char(b'c'),
        Instruction::End,
    ];
    assert_eq!(
        run(&prog, b"abc"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
        }
    );
    assert_eq!(
        run(&prog, b"abcd"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
        }
    );
    assert_eq!(
        run(&prog, b"abx"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: false,
        }
    );
    assert_eq!(
        run(&prog, b"ab"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: false,
        }
    );
}

#[test]
fn match_charset() {
    let digits = CharSet::from_ranges(&[(b'0', b'9')]);
    let prog = [Instruction::Set(digits), Instruction::End];
    assert_eq!(
        run(&prog, b"7"),
        MatchResult {
            matched: 1,
            captures: vec![],
            complete: true,
        }
    );
    assert_eq!(
        run(&prog, b"a"),
        MatchResult {
            matched: 0,
            captures: vec![],
            complete: false,
        }
    );
    assert_eq!(
        run(&prog, b""),
        MatchResult {
            matched: 0,
            captures: vec![],
            complete: false,
        }
    );
}

#[test]
fn any_skips_n_bytes() {
    let prog = [Instruction::Any(3), Instruction::End];
    assert_eq!(
        run(&prog, b"xyz"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
        }
    );
    assert_eq!(
        run(&prog, b"xy"),
        MatchResult {
            matched: 0,
            captures: vec![],
            complete: false,
        }
    );
}

#[test]
fn ordered_choice_first_alternative() {
    // p = "ab" / "ax"
    // Choice L1 ; Char a ; Char b ; Commit L2 ; L1: Char a ; Char x ; L2: End
    let prog = [
        Instruction::Choice(Label(4)),
        Instruction::Char(b'a'),
        Instruction::Char(b'b'),
        Instruction::Commit(Label(6)),
        Instruction::Char(b'a'),
        Instruction::Char(b'x'),
        Instruction::End,
    ];
    assert_eq!(
        run(&prog, b"ab"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
        }
    );
    assert_eq!(
        run(&prog, b"ax"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
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
        Instruction::Char(b'a'),
        Instruction::PartialCommit(Label(1)),
        Instruction::End,
    ];
    assert_eq!(
        run(&prog, b""),
        MatchResult {
            matched: 0,
            captures: vec![],
            complete: true,
        }
    );
    assert_eq!(
        run(&prog, b"a"),
        MatchResult {
            matched: 1,
            captures: vec![],
            complete: true,
        }
    );
    assert_eq!(
        run(&prog, b"aaaa"),
        MatchResult {
            matched: 4,
            captures: vec![],
            complete: true,
        }
    );
    assert_eq!(
        run(&prog, b"aaab"),
        MatchResult {
            matched: 3,
            captures: vec![],
            complete: true,
        }
    );
}

#[test]
fn not_predicate() {
    // !'a' .  : matches one char that is not 'a'
    // Choice L1 ; Char a ; FailTwice ; L1: Any 1 ; End
    let prog = [
        Instruction::Choice(Label(3)),
        Instruction::Char(b'a'),
        Instruction::FailTwice,
        Instruction::Any(1),
        Instruction::End,
    ];
    assert_eq!(
        run(&prog, b"b"),
        MatchResult {
            matched: 1,
            captures: vec![],
            complete: true,
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
        Instruction::Char(b'a'),
        Instruction::Return,
    ];
    assert_eq!(
        run(&prog, b"aa"),
        MatchResult {
            matched: 2,
            captures: vec![],
            complete: true,
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
        Instruction::Char(b'a'),
        Instruction::Char(b'b'),
        Instruction::CaptureEnd,
        Instruction::End,
    ];
    assert_eq!(
        run(&prog, b"ab"),
        MatchResult {
            matched: 2,
            captures: vec![cap(7, 0, 2)],
            complete: true,
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
        Instruction::Char(b'a'),
        Instruction::Char(b'b'),
        Instruction::CaptureEnd,
        Instruction::Commit(Label(8)),
        Instruction::Char(b'a'),
        Instruction::Char(b'x'),
        Instruction::End,
    ];
    assert_eq!(
        run(&prog, b"ax"),
        MatchResult {
            matched: 2,
            captures: vec![], // discarded during backtrack
            complete: true,
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
        Instruction::Char(b'a'),
        Instruction::CaptureEnd,
        Instruction::CaptureBegin(CaptureKind(2)),
        Instruction::Char(b'b'),
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
        }
    );
}

#[test]
fn charset_negate_and_union() {
    let vowels = CharSet::from_bytes(b"aeiou");
    let consonants = vowels.negate();
    assert!(!consonants.contains(b'a'));
    assert!(consonants.contains(b'b'));
    let merged = vowels.union(&CharSet::from_bytes(b"y"));
    assert!(merged.contains(b'y'));
    assert!(merged.contains(b'a'));
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
        Instruction::Char(b'a'),
        Instruction::Char(b'b'),
        Instruction::CaptureBegin(CaptureKind(0)),
        Instruction::Char(b'c'),
        Instruction::Char(b'd'),
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
    let prog = [Instruction::Char(b'a'), Instruction::End];
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
        Instruction::Char(b'a'),
        Instruction::Char(b'a'),
        Instruction::Char(b'a'),
        Instruction::Commit(Label(8)),
        Instruction::Char(b'a'),
        Instruction::Char(b'a'),
        Instruction::Char(b'b'),
        Instruction::End,
    ];
    let r = run(&prog, b"aaX");
    assert!(!r.complete);
    assert_eq!(r.matched, 2);
}
