use std::collections::HashMap;

use syntax_highlighter::pegvm::{
    compile_grammar, compile_pattern, Capture, CaptureKind, CharSet, Instruction, Label,
    MatchResult, Pattern, VM,
};

fn run_pattern(pat: &Pattern, input: &[u8]) -> Option<MatchResult> {
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
        Some(MatchResult {
            matched: 2,
            captures: vec![]
        })
    );
    assert_eq!(run_pattern(&p, b"ho"), None);
}

#[test]
fn char_class_pattern() {
    let p = Pattern::CharClass(CharSet::from_ranges(&[(b'0', b'9')]));
    assert_eq!(
        run_pattern(&p, b"5"),
        Some(MatchResult {
            matched: 1,
            captures: vec![]
        })
    );
    assert_eq!(run_pattern(&p, b"x"), None);
}

#[test]
fn any_char_pattern() {
    let p = Pattern::AnyChar;
    assert_eq!(
        run_pattern(&p, b"q"),
        Some(MatchResult {
            matched: 1,
            captures: vec![]
        })
    );
    assert_eq!(run_pattern(&p, b""), None);
}

#[test]
fn sequence_pattern() {
    let p = Pattern::seq(vec![Pattern::literal("ab"), Pattern::literal("cd")]);
    assert_eq!(
        run_pattern(&p, b"abcd"),
        Some(MatchResult {
            matched: 4,
            captures: vec![]
        })
    );
    assert_eq!(run_pattern(&p, b"abce"), None);
}

#[test]
fn ordered_choice_two() {
    let p = Pattern::choice(vec![Pattern::literal("ab"), Pattern::literal("ax")]);
    assert_eq!(
        run_pattern(&p, b"ab"),
        Some(MatchResult {
            matched: 2,
            captures: vec![]
        })
    );
    assert_eq!(
        run_pattern(&p, b"ax"),
        Some(MatchResult {
            matched: 2,
            captures: vec![]
        })
    );
    assert_eq!(run_pattern(&p, b"ay"), None);
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
        Some(MatchResult {
            matched: 3,
            captures: vec![]
        })
    );
    assert_eq!(
        run_pattern(&p, b"bar"),
        Some(MatchResult {
            matched: 3,
            captures: vec![]
        })
    );
    assert_eq!(
        run_pattern(&p, b"baz"),
        Some(MatchResult {
            matched: 3,
            captures: vec![]
        })
    );
    assert_eq!(run_pattern(&p, b"qux"), None);
}

#[test]
fn repeat_zero_or_more() {
    let p = Pattern::Repeat(Box::new(Pattern::literal("a")));
    assert_eq!(
        run_pattern(&p, b""),
        Some(MatchResult {
            matched: 0,
            captures: vec![]
        })
    );
    assert_eq!(
        run_pattern(&p, b"aaa"),
        Some(MatchResult {
            matched: 3,
            captures: vec![]
        })
    );
    assert_eq!(
        run_pattern(&p, b"aab"),
        Some(MatchResult {
            matched: 2,
            captures: vec![]
        })
    );
}

#[test]
fn repeat_one_or_more() {
    let p = Pattern::RepeatOne(Box::new(Pattern::literal("a")));
    assert_eq!(run_pattern(&p, b""), None);
    assert_eq!(
        run_pattern(&p, b"a"),
        Some(MatchResult {
            matched: 1,
            captures: vec![]
        })
    );
    assert_eq!(
        run_pattern(&p, b"aaa"),
        Some(MatchResult {
            matched: 3,
            captures: vec![]
        })
    );
}

#[test]
fn optional_pattern() {
    let p = Pattern::seq(vec![
        Pattern::Optional(Box::new(Pattern::literal("-"))),
        Pattern::literal("x"),
    ]);
    assert_eq!(
        run_pattern(&p, b"x"),
        Some(MatchResult {
            matched: 1,
            captures: vec![]
        })
    );
    assert_eq!(
        run_pattern(&p, b"-x"),
        Some(MatchResult {
            matched: 2,
            captures: vec![]
        })
    );
    assert_eq!(run_pattern(&p, b"--x"), None);
}

#[test]
fn not_predicate_pattern() {
    let p = Pattern::seq(vec![
        Pattern::NotPredicate(Box::new(Pattern::literal("a"))),
        Pattern::AnyChar,
    ]);
    assert_eq!(
        run_pattern(&p, b"b"),
        Some(MatchResult {
            matched: 1,
            captures: vec![]
        })
    );
    assert_eq!(run_pattern(&p, b"a"), None);
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
        Some(MatchResult {
            matched: 2,
            captures: vec![]
        })
    );
    assert_eq!(run_pattern(&p, b"bb"), None);
}

#[test]
fn capture_records_kind_and_span() {
    let p = Pattern::Capture("number".into(), Box::new(Pattern::literal("42")));
    let prog = compile_pattern(&p);
    assert_eq!(prog.capture_kinds, vec!["number".to_string()]);
    assert_eq!(
        VM::new(&prog.code, b"42").run(),
        Some(MatchResult {
            matched: 2,
            captures: vec![cap(0, 0, 2)],
        })
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
        Some(MatchResult {
            matched: 2,
            captures: vec![
                cap(0, 0, 2), // @outer
                cap(1, 0, 1), // @inner "a"
                cap(1, 1, 2), // @inner "b"
            ],
        })
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
    let prog = compile_grammar(&rules, "start").unwrap();
    assert_eq!(
        VM::new(&prog.code, b"123abc").run(),
        Some(MatchResult {
            matched: 3,
            captures: vec![]
        })
    );
    assert_eq!(VM::new(&prog.code, b"abc").run(), None);
}

#[test]
fn grammar_undefined_rule_errors() {
    let mut rules = HashMap::new();
    rules.insert("start".into(), Pattern::NonTerminal("missing".into()));
    let err = compile_grammar(&rules, "start").unwrap_err();
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
