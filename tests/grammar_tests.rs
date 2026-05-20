use syntax_highlighter::pegc::{parse as parse_src, Pattern};
use syntax_highlighter::pegvm::CharSet;

fn parse(src: &str) -> syntax_highlighter::pegc::Grammar {
    parse_src(src).expect("parse failed")
}

#[test]
fn simple_rule() {
    let g = parse("foo <- \"hi\"");
    assert_eq!(g.start, "foo");
    assert_eq!(g.rules["foo"], Pattern::literal("hi"));
}

#[test]
fn first_rule_is_start() {
    let g = parse("first <- 'a'\nsecond <- 'b'");
    assert_eq!(g.start, "first");
    assert_eq!(g.rules.len(), 2);
}

#[test]
fn ordered_choice_in_grammar() {
    let g = parse("r <- 'a' / 'b' / 'c'");
    assert_eq!(
        g.rules["r"],
        Pattern::OrderedChoice(vec![
            Pattern::literal("a"),
            Pattern::literal("b"),
            Pattern::literal("c"),
        ])
    );
}

#[test]
fn sequence_in_grammar() {
    let g = parse("r <- 'a' 'b' 'c'");
    assert_eq!(
        g.rules["r"],
        Pattern::Sequence(vec![
            Pattern::literal("a"),
            Pattern::literal("b"),
            Pattern::literal("c"),
        ])
    );
}

#[test]
fn postfix_operators() {
    let g = parse("a <- 'x'*\nb <- 'x'+\nc <- 'x'?");
    assert_eq!(
        g.rules["a"],
        Pattern::Repeat(Box::new(Pattern::literal("x")))
    );
    assert_eq!(
        g.rules["b"],
        Pattern::RepeatOne(Box::new(Pattern::literal("x")))
    );
    assert_eq!(
        g.rules["c"],
        Pattern::Optional(Box::new(Pattern::literal("x")))
    );
}

#[test]
fn recover_repeat_postfix_star_caret() {
    let g = parse("r <- 'x'*^");
    assert_eq!(
        g.rules["r"],
        Pattern::RecoverRepeat {
            inner: Box::new(Pattern::literal("x")),
            recovery_kind: "recovery".into(),
        }
    );
}

#[test]
fn recover_repeat_postfix_plus_caret_lowers_to_seq() {
    // p+^  ≡  p (p*^)  — at least one inner success required.
    let g = parse("r <- 'x'+^");
    assert_eq!(
        g.rules["r"],
        Pattern::Sequence(vec![
            Pattern::literal("x"),
            Pattern::RecoverRepeat {
                inner: Box::new(Pattern::literal("x")),
                recovery_kind: "recovery".into(),
            },
        ])
    );
}

#[test]
fn catch_basic_parses_to_pattern_catch() {
    let g = parse("r <- 'a' ^ 'b'");
    assert_eq!(
        g.rules["r"],
        Pattern::Catch {
            inner: Box::new(Pattern::literal("a")),
            recovery: Box::new(Pattern::literal("b")),
        }
    );
}

#[test]
fn catch_binds_tighter_than_choice() {
    // 'a' ^ 'b' / 'c'   ≡   ('a' ^ 'b') / 'c'
    let g = parse("r <- 'a' ^ 'b' / 'c'");
    assert_eq!(
        g.rules["r"],
        Pattern::OrderedChoice(vec![
            Pattern::Catch {
                inner: Box::new(Pattern::literal("a")),
                recovery: Box::new(Pattern::literal("b")),
            },
            Pattern::literal("c"),
        ])
    );
}

#[test]
fn catch_binds_looser_than_sequence() {
    // 'a' 'b' ^ 'c' 'd'   ≡   ('a' 'b') ^ ('c' 'd')
    let g = parse("r <- 'a' 'b' ^ 'c' 'd'");
    assert_eq!(
        g.rules["r"],
        Pattern::Catch {
            inner: Box::new(Pattern::Sequence(vec![
                Pattern::literal("a"),
                Pattern::literal("b"),
            ])),
            recovery: Box::new(Pattern::Sequence(vec![
                Pattern::literal("c"),
                Pattern::literal("d"),
            ])),
        }
    );
}

#[test]
fn catch_is_left_associative() {
    // 'a' ^ 'b' ^ 'c'   ≡   ('a' ^ 'b') ^ 'c'
    let g = parse("r <- 'a' ^ 'b' ^ 'c'");
    assert_eq!(
        g.rules["r"],
        Pattern::Catch {
            inner: Box::new(Pattern::Catch {
                inner: Box::new(Pattern::literal("a")),
                recovery: Box::new(Pattern::literal("b")),
            }),
            recovery: Box::new(Pattern::literal("c")),
        }
    );
}

#[test]
fn catch_does_not_collide_with_star_caret() {
    // `'x'*^` stays as RecoverRepeat (postfix with no whitespace
    // between `*` and `^`). `'x'* ^ 'y'` is a Catch of Repeat over a
    // separate recovery branch — the whitespace breaks the `*^`
    // postfix form.
    let g = parse("a <- 'x'*^\nb <- 'x'* ^ 'y'");
    assert_eq!(
        g.rules["a"],
        Pattern::RecoverRepeat {
            inner: Box::new(Pattern::literal("x")),
            recovery_kind: "recovery".into(),
        }
    );
    assert_eq!(
        g.rules["b"],
        Pattern::Catch {
            inner: Box::new(Pattern::Repeat(Box::new(Pattern::literal("x")))),
            recovery: Box::new(Pattern::literal("y")),
        }
    );
}

#[test]
fn catch_parens_force_grouping_on_recovery() {
    // Default `'a' ^ 'b' / 'c'` is `('a' ^ 'b') / 'c'` (catch tighter
    // than choice); to put a choice in the recovery branch the author
    // must parenthesize.
    let g = parse("r <- 'a' ^ ('b' / 'c')");
    assert_eq!(
        g.rules["r"],
        Pattern::Catch {
            inner: Box::new(Pattern::literal("a")),
            recovery: Box::new(Pattern::OrderedChoice(vec![
                Pattern::literal("b"),
                Pattern::literal("c"),
            ])),
        }
    );
}

#[test]
fn predicate_operators() {
    let g = parse("a <- !'x' .\nb <- &'y' 'y'");
    assert_eq!(
        g.rules["a"],
        Pattern::Sequence(vec![
            Pattern::NotPredicate(Box::new(Pattern::literal("x"))),
            Pattern::AnyChar,
        ])
    );
    assert_eq!(
        g.rules["b"],
        Pattern::Sequence(vec![
            Pattern::AndPredicate(Box::new(Pattern::literal("y"))),
            Pattern::literal("y"),
        ])
    );
}

#[test]
fn char_class_with_range() {
    let g = parse("d <- [0-9]");
    assert_eq!(
        g.rules["d"],
        Pattern::CharClass(CharSet::from_ranges(&[(b'0', b'9')]))
    );
}

#[test]
fn char_class_negated() {
    let g = parse("nq <- [^\"\\\\]");
    let mut excluded = CharSet::empty();
    excluded.add(b'"');
    excluded.add(b'\\');
    assert_eq!(g.rules["nq"], Pattern::CharClass(excluded.negate()));
}

#[test]
fn char_class_mixed_chars_and_ranges() {
    let g = parse("alnum <- [a-zA-Z0-9_]");
    let expected = {
        let mut s = CharSet::empty();
        s.add_range(b'a', b'z');
        s.add_range(b'A', b'Z');
        s.add_range(b'0', b'9');
        s.add(b'_');
        s
    };
    assert_eq!(g.rules["alnum"], Pattern::CharClass(expected));
}

#[test]
fn capture_annotation() {
    let g = parse("r <- @keyword{'while'}");
    assert_eq!(
        g.rules["r"],
        Pattern::Capture("keyword".into(), Box::new(Pattern::literal("while")))
    );
}

#[test]
fn comments_and_blank_lines() {
    let src = "
        # this is a top-level comment
        first <- 'a'   # trailing comment
        # another comment
        second <- 'b'
    ";
    let g = parse(src);
    assert_eq!(g.rules.len(), 2);
    assert_eq!(g.rules["first"], Pattern::literal("a"));
    assert_eq!(g.rules["second"], Pattern::literal("b"));
}

#[test]
fn parens_and_precedence() {
    // 'a' ('b' / 'c') 'd'
    let g = parse("r <- 'a' ('b' / 'c') 'd'");
    assert_eq!(
        g.rules["r"],
        Pattern::Sequence(vec![
            Pattern::literal("a"),
            Pattern::OrderedChoice(vec![Pattern::literal("b"), Pattern::literal("c")]),
            Pattern::literal("d"),
        ])
    );
}

#[test]
fn nonterminal_reference() {
    let g = parse("a <- b\nb <- 'x'");
    assert_eq!(g.rules["a"], Pattern::NonTerminal("b".into()));
}

#[test]
fn escape_sequences_in_string() {
    let g = parse("r <- '\\n\\t\\\\'");
    assert_eq!(g.rules["r"], Pattern::Literal(vec![b'\n', b'\t', b'\\']));
}

#[test]
fn dash_in_class_at_end_is_literal() {
    let g = parse("r <- [+\\-]");
    let mut s = CharSet::empty();
    s.add(b'+');
    s.add(b'-');
    assert_eq!(g.rules["r"], Pattern::CharClass(s));
}

#[test]
fn end_to_end_grammar_compile_run() {
    use syntax_highlighter::pegvm::VM;
    let g = parse("number <- [0-9]+");
    let prog = g.compile().unwrap();
    let r = VM::new(&prog.code, b"42abc").run();
    assert!(r.complete);
    assert_eq!(r.matched, 2);
}

#[test]
fn end_to_end_recover_repeat_compile_run() {
    use syntax_highlighter::pegvm::VM;
    // Top-level `*^` resyncs past garbage one byte at a time; the parse
    // completes at EOF, with one "recovery"-tagged capture per skipped byte.
    let g = parse("doc <- @kw{\"foo\"}*^");
    let prog = g.compile().unwrap();
    let r = VM::new(&prog.code, b"fooXXfoo").run();
    assert!(r.complete);
    assert_eq!(r.matched, 8);
    // Capture-kind interning order in the bytecode: "recovery" first
    // (RecoverRepeat enters before its inner), "kw" second.
    assert_eq!(prog.capture_kinds, vec!["recovery", "kw"]);
    let kinds: Vec<u16> = r.captures.iter().map(|c| c.kind.0).collect();
    let spans: Vec<(usize, usize)> = r.captures.iter().map(|c| (c.start, c.end)).collect();
    assert_eq!(kinds, vec![1, 0, 0, 1]);
    assert_eq!(spans, vec![(0, 3), (3, 4), (4, 5), (5, 8)]);
}

#[test]
fn end_to_end_catch_compile_run() {
    use syntax_highlighter::pegvm::VM;
    // Inner is a `SELECT … FROM` sequence; recovery scoops up the rest
    // of the statement under an @err capture and consumes the closing
    // semicolon. Input is malformed (no FROM), so the inner fails and
    // the recovery branch fires.
    let g = parse("stmt <- (@kw{'SELECT'} ' ' @kw{'FROM'} ' ' 'x' ';') ^ @err{(!';' .)*} ';'");
    let prog = g.compile().unwrap();
    let r = VM::new(&prog.code, b"SELECT bogus;").run();
    assert!(
        r.complete,
        "catch should let parse complete on malformed input"
    );
    assert_eq!(r.matched, 13);
    // "kw" was interned first by the inner branch's first capture, "err" second.
    assert_eq!(prog.capture_kinds, vec!["kw", "err"]);
    let kinds: Vec<u16> = r.captures.iter().map(|c| c.kind.0).collect();
    let spans: Vec<(usize, usize)> = r.captures.iter().map(|c| (c.start, c.end)).collect();
    // Failed inner's deepest reach: the @kw{'SELECT'} survives via
    // RecoverToScopedMax. Recovery then captures the byte range past
    // "SELECT " up to the `;`.
    assert_eq!(kinds, vec![0, 1]);
    assert_eq!(spans, vec![(0, 6), (7, 12)]);
}

#[test]
fn duplicate_rule_errors() {
    let err = parse_src("a <- 'x'\na <- 'y'").unwrap_err();
    assert!(err.message.contains("twice"), "got: {}", err.message);
}
