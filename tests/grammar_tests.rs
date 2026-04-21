use syntax_highlighter::grammar::{parse as parse_src, Pattern};
use syntax_highlighter::pegvm::CharSet;

fn parse(src: &str) -> syntax_highlighter::grammar::Grammar {
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
    use syntax_highlighter::grammar::compile;
    use syntax_highlighter::pegvm::VM;
    let g = parse("number <- [0-9]+");
    let prog = compile(&g.rules, &g.start).unwrap();
    let r = VM::new(&prog.code, b"42abc").run();
    assert!(r.complete);
    assert_eq!(r.matched, 2);
}

#[test]
fn duplicate_rule_errors() {
    let err = parse_src("a <- 'x'\na <- 'y'").unwrap_err();
    assert!(err.message.contains("twice"), "got: {}", err.message);
}
