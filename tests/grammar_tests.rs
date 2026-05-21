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

/// Builds the desugared AST that `p*^` lowers to: a `Repeat` over a
/// `Catch` whose recovery body is the supplied pattern wrapped in a
/// capture named `recovery`. The label defaults to `"recovery"` to
/// match bare `*^`; the `_with_label` variant covers `*^:lbl`.
/// Mirrors `build_recover_repeat` in `src/pegc/parser.rs`. Used by the
/// AST-shape assertions below.
fn desugared_recover_repeat(inner: Pattern, recovery_body: Pattern) -> Pattern {
    desugared_recover_repeat_with_label(inner, recovery_body, "recovery")
}

fn desugared_recover_repeat_with_label(
    inner: Pattern,
    recovery_body: Pattern,
    label: &str,
) -> Pattern {
    Pattern::Repeat(Box::new(Pattern::Catch {
        inner: Box::new(inner),
        label: label.into(),
        recovery: Box::new(Pattern::Capture("recovery".into(), Box::new(recovery_body))),
    }))
}

#[test]
fn recover_repeat_postfix_star_caret() {
    // `p*^` desugars to `(p ^recovery @recovery{.})*` at parse time.
    let g = parse("r <- 'x'*^");
    assert_eq!(
        g.rules["r"],
        desugared_recover_repeat(Pattern::literal("x"), Pattern::AnyChar)
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
            desugared_recover_repeat(Pattern::literal("x"), Pattern::AnyChar),
        ])
    );
}

#[test]
fn sync_set_postfix_star_caret_charset() {
    // `p*^[;]` desugars to `(p ^recovery @recovery{(![;] .)* [;]})*`.
    let semi = CharSet::from_bytes(b";");
    let g = parse("r <- 'x'*^[;]");
    let skip_loop = Pattern::Repeat(Box::new(Pattern::Sequence(vec![
        Pattern::NotPredicate(Box::new(Pattern::CharClass(semi))),
        Pattern::AnyChar,
    ])));
    let recovery_body = Pattern::Sequence(vec![skip_loop, Pattern::CharClass(semi)]);
    assert_eq!(
        g.rules["r"],
        desugared_recover_repeat(Pattern::literal("x"), recovery_body)
    );
}

#[test]
fn sync_set_postfix_plus_caret_charset() {
    // `p+^[;]` lowers to `p (p*^[;])`.
    let semi = CharSet::from_bytes(b";");
    let g = parse("r <- 'x'+^[;]");
    let skip_loop = Pattern::Repeat(Box::new(Pattern::Sequence(vec![
        Pattern::NotPredicate(Box::new(Pattern::CharClass(semi))),
        Pattern::AnyChar,
    ])));
    let recovery_body = Pattern::Sequence(vec![skip_loop, Pattern::CharClass(semi)]);
    assert_eq!(
        g.rules["r"],
        Pattern::Sequence(vec![
            Pattern::literal("x"),
            desugared_recover_repeat(Pattern::literal("x"), recovery_body),
        ])
    );
}

#[test]
fn sync_set_requires_no_whitespace_before_bracket() {
    // `*^ [;]` is `*^` (plain) followed by a separate atom `[;]` in
    // the enclosing sequence, NOT a sync set. The whitespace breaks
    // the postfix glue.
    let g = parse("r <- 'x'*^ [;]");
    let semi = CharSet::from_bytes(b";");
    assert_eq!(
        g.rules["r"],
        Pattern::Sequence(vec![
            desugared_recover_repeat(Pattern::literal("x"), Pattern::AnyChar),
            Pattern::CharClass(semi),
        ])
    );
}

#[test]
fn sync_set_accepts_negated_and_ranges() {
    // The sync set parses via the same `parse_charclass` path as a
    // normal `[...]` atom — ranges (`a-z`) and negation (`[^...]`)
    // are both supported.
    let g = parse("r <- 'x'*^[^a-z]");
    let mut alpha = CharSet::empty();
    alpha.add_range(b'a', b'z');
    let neg_alpha = alpha.negate();
    let skip_loop = Pattern::Repeat(Box::new(Pattern::Sequence(vec![
        Pattern::NotPredicate(Box::new(Pattern::CharClass(neg_alpha))),
        Pattern::AnyChar,
    ])));
    let recovery_body = Pattern::Sequence(vec![skip_loop, Pattern::CharClass(neg_alpha)]);
    assert_eq!(
        g.rules["r"],
        desugared_recover_repeat(Pattern::literal("x"), recovery_body)
    );
}

#[test]
fn recover_repeat_postfix_star_caret_with_label() {
    // `p*^:bad` interns label "bad" instead of the default "recovery";
    // the capture name is unchanged.
    let g = parse("r <- 'x'*^:bad");
    assert_eq!(
        g.rules["r"],
        desugared_recover_repeat_with_label(Pattern::literal("x"), Pattern::AnyChar, "bad")
    );
}

#[test]
fn recover_repeat_postfix_plus_caret_with_label() {
    // `p+^:bad` lowers to `p (p*^:bad)` — the label flows through the
    // tail `*^` only; the head `p` is the unguarded one-iteration prefix.
    let g = parse("r <- 'x'+^:bad");
    assert_eq!(
        g.rules["r"],
        Pattern::Sequence(vec![
            Pattern::literal("x"),
            desugared_recover_repeat_with_label(Pattern::literal("x"), Pattern::AnyChar, "bad"),
        ])
    );
}

#[test]
fn sync_set_postfix_star_caret_charset_with_label() {
    // `p*^[;]:bad_stmt` interns label "bad_stmt"; recovery body
    // (sync-set skip) is unchanged.
    let semi = CharSet::from_bytes(b";");
    let g = parse("r <- 'x'*^[;]:bad_stmt");
    let skip_loop = Pattern::Repeat(Box::new(Pattern::Sequence(vec![
        Pattern::NotPredicate(Box::new(Pattern::CharClass(semi))),
        Pattern::AnyChar,
    ])));
    let recovery_body = Pattern::Sequence(vec![skip_loop, Pattern::CharClass(semi)]);
    assert_eq!(
        g.rules["r"],
        desugared_recover_repeat_with_label(Pattern::literal("x"), recovery_body, "bad_stmt")
    );
}

#[test]
fn sync_set_postfix_plus_caret_charset_with_label() {
    // `p+^[;]:bad_stmt` lowers to `p (p*^[;]:bad_stmt)`.
    let semi = CharSet::from_bytes(b";");
    let g = parse("r <- 'x'+^[;]:bad_stmt");
    let skip_loop = Pattern::Repeat(Box::new(Pattern::Sequence(vec![
        Pattern::NotPredicate(Box::new(Pattern::CharClass(semi))),
        Pattern::AnyChar,
    ])));
    let recovery_body = Pattern::Sequence(vec![skip_loop, Pattern::CharClass(semi)]);
    assert_eq!(
        g.rules["r"],
        Pattern::Sequence(vec![
            Pattern::literal("x"),
            desugared_recover_repeat_with_label(Pattern::literal("x"), recovery_body, "bad_stmt"),
        ])
    );
}

#[test]
fn recovery_label_default_is_recovery() {
    // Back-compat: bare `*^` (no `:label`) lowers to label "recovery"
    // exactly as before — the helper's default already encodes this,
    // so the existing tests cover it; this is a self-documenting
    // sentinel that the default has not drifted.
    let g = parse("r <- 'x'*^");
    let Pattern::Repeat(inner) = &g.rules["r"] else {
        panic!("expected Repeat, got {:?}", g.rules["r"]);
    };
    let Pattern::Catch { label, .. } = inner.as_ref() else {
        panic!("expected Catch inside Repeat, got {:?}", inner);
    };
    assert_eq!(label, "recovery");
}

#[test]
fn recovery_label_rejects_whitespace_before_colon() {
    // `*^ :lbl` (whitespace between `^` and `:`) breaks the postfix
    // glue: the `*^` lowers without a label, the loose `:lbl` isn't a
    // valid sequence atom, and the parser falls through to the next
    // rule-start where `:` fails `parse_ident` with "expected
    // identifier".
    let err = parse_src("r <- 'x'*^ :lbl").unwrap_err();
    assert!(
        err.message.contains("expected identifier"),
        "expected an identifier-required error at top level, got: {}",
        err.message
    );
}

#[test]
fn recovery_label_rejects_whitespace_after_colon() {
    // `*^: lbl` — the identifier must touch `:`.
    let err = parse_src("r <- 'x'*^: lbl").unwrap_err();
    assert!(
        err.message.contains("expected label identifier"),
        "expected a label-identifier error, got: {}",
        err.message
    );
}

#[test]
fn recovery_label_rejects_underscore() {
    // Bare `_` mirrors `^_`'s reservation for future anonymous-catch
    // syntax.
    let err = parse_src("r <- 'x'*^:_").unwrap_err();
    assert!(
        err.message.contains("reserved"),
        "expected a reserved-label error, got: {}",
        err.message
    );
}

#[test]
fn recovery_label_after_sync_set_requires_no_space_before_colon() {
    // `*^[;] :lbl` (whitespace between `]` and `:`) breaks the postfix
    // glue: `*^[;]` lowers without a label, the loose `:lbl` isn't a
    // valid sequence atom, and the parser falls through to the next
    // rule-start where `:` fails `parse_ident` with "expected
    // identifier".
    let err = parse_src("r <- 'x'*^[;] :lbl").unwrap_err();
    assert!(
        err.message.contains("expected identifier"),
        "expected an identifier-required error at top level, got: {}",
        err.message
    );
}

#[test]
fn recovery_label_accepts_underscore_prefixed_identifier() {
    // `_foo` (underscore prefix, not bare `_`) stays a valid label,
    // mirroring `parse_catch`.
    let g = parse("r <- 'x'*^:_foo");
    assert_eq!(
        g.rules["r"],
        desugared_recover_repeat_with_label(Pattern::literal("x"), Pattern::AnyChar, "_foo")
    );
}

#[test]
fn catch_basic_parses_to_pattern_catch() {
    let g = parse("r <- 'a' ^lbl 'b'");
    assert_eq!(
        g.rules["r"],
        Pattern::Catch {
            inner: Box::new(Pattern::literal("a")),
            label: "lbl".into(),
            recovery: Box::new(Pattern::literal("b")),
        }
    );
}

#[test]
fn catch_binds_tighter_than_choice() {
    // 'a' ^lbl 'b' / 'c'   ≡   ('a' ^lbl 'b') / 'c'
    let g = parse("r <- 'a' ^lbl 'b' / 'c'");
    assert_eq!(
        g.rules["r"],
        Pattern::OrderedChoice(vec![
            Pattern::Catch {
                inner: Box::new(Pattern::literal("a")),
                label: "lbl".into(),
                recovery: Box::new(Pattern::literal("b")),
            },
            Pattern::literal("c"),
        ])
    );
}

#[test]
fn catch_binds_looser_than_sequence() {
    // 'a' 'b' ^lbl 'c' 'd'   ≡   ('a' 'b') ^lbl ('c' 'd')
    let g = parse("r <- 'a' 'b' ^lbl 'c' 'd'");
    assert_eq!(
        g.rules["r"],
        Pattern::Catch {
            inner: Box::new(Pattern::Sequence(vec![
                Pattern::literal("a"),
                Pattern::literal("b"),
            ])),
            label: "lbl".into(),
            recovery: Box::new(Pattern::Sequence(vec![
                Pattern::literal("c"),
                Pattern::literal("d"),
            ])),
        }
    );
}

#[test]
fn catch_is_left_associative() {
    // 'a' ^l1 'b' ^l2 'c'   ≡   ('a' ^l1 'b') ^l2 'c'
    let g = parse("r <- 'a' ^l1 'b' ^l2 'c'");
    assert_eq!(
        g.rules["r"],
        Pattern::Catch {
            inner: Box::new(Pattern::Catch {
                inner: Box::new(Pattern::literal("a")),
                label: "l1".into(),
                recovery: Box::new(Pattern::literal("b")),
            }),
            label: "l2".into(),
            recovery: Box::new(Pattern::literal("c")),
        }
    );
}

#[test]
fn catch_does_not_collide_with_star_caret() {
    // `'x'*^` desugars to the recovery-loop AST (postfix with no
    // whitespace between `*` and `^`). `'x'* ^lbl 'y'` is a Catch of
    // Repeat over a separate recovery branch — whitespace before `^`
    // breaks the postfix glue.
    let g = parse("a <- 'x'*^\nb <- 'x'* ^lbl 'y'");
    assert_eq!(
        g.rules["a"],
        desugared_recover_repeat(Pattern::literal("x"), Pattern::AnyChar)
    );
    assert_eq!(
        g.rules["b"],
        Pattern::Catch {
            inner: Box::new(Pattern::Repeat(Box::new(Pattern::literal("x")))),
            label: "lbl".into(),
            recovery: Box::new(Pattern::literal("y")),
        }
    );
}

#[test]
fn catch_parens_force_grouping_on_recovery() {
    // Default `'a' ^lbl 'b' / 'c'` is `('a' ^lbl 'b') / 'c'` (catch
    // tighter than choice); to put a choice in the recovery branch
    // the author must parenthesize.
    let g = parse("r <- 'a' ^lbl ('b' / 'c')");
    assert_eq!(
        g.rules["r"],
        Pattern::Catch {
            inner: Box::new(Pattern::literal("a")),
            label: "lbl".into(),
            recovery: Box::new(Pattern::OrderedChoice(vec![
                Pattern::literal("b"),
                Pattern::literal("c"),
            ])),
        }
    );
}

#[test]
fn catch_label_touches_caret_whitespace_insensitive_on_left() {
    // `foo ^lbl bar` and `foo^lbl bar` both parse identically — `^`
    // is whitespace-insensitive on its *left* (same as today's other
    // infix operators).
    let with_space = parse("r <- 'a' ^lbl 'b'");
    let glued = parse("r <- 'a'^lbl 'b'");
    let expected = Pattern::Catch {
        inner: Box::new(Pattern::literal("a")),
        label: "lbl".into(),
        recovery: Box::new(Pattern::literal("b")),
    };
    assert_eq!(with_space.rules["r"], expected);
    assert_eq!(glued.rules["r"], expected);
}

#[test]
fn catch_requires_label_without_whitespace() {
    // `foo ^ bar` (whitespace between `^` and the label) is rejected
    // — `^<non-ident-byte>` is a reserved syntactic slot for future
    // overlays.
    let err = parse_src("r <- 'a' ^ 'b'").unwrap_err();
    assert!(
        err.message.contains("label identifier"),
        "expected a label-identifier error, got: {}",
        err.message
    );
}

#[test]
fn catch_rejects_reserved_underscore_label() {
    // Bare `_` as a label name is reserved for future use (anonymous
    // catch).
    let err = parse_src("r <- 'a' ^_ 'b'").unwrap_err();
    assert!(
        err.message.contains("reserved"),
        "expected a reserved-label error, got: {}",
        err.message
    );
}

#[test]
fn catch_accepts_underscore_prefixed_labels() {
    // `_foo` (underscore prefix, not bare `_`) stays a valid label.
    let g = parse("r <- 'a' ^_foo 'b'");
    assert_eq!(
        g.rules["r"],
        Pattern::Catch {
            inner: Box::new(Pattern::literal("a")),
            label: "_foo".into(),
            recovery: Box::new(Pattern::literal("b")),
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
    // Capture-kind interning order in the bytecode: "kw" first (the
    // desugared `Catch`'s inner is compiled before the recovery body),
    // "recovery" second.
    assert_eq!(prog.capture_kinds, vec!["kw", "recovery"]);
    let kw = 0u16;
    let recovery = 1u16;
    let kinds: Vec<u16> = r.captures.iter().map(|c| c.kind.0).collect();
    let spans: Vec<(usize, usize)> = r.captures.iter().map(|c| (c.start, c.end)).collect();
    assert_eq!(kinds, vec![kw, recovery, recovery, kw]);
    assert_eq!(spans, vec![(0, 3), (3, 4), (4, 5), (5, 8)]);
}

#[test]
fn end_to_end_recover_repeat_with_label_intern() {
    use syntax_highlighter::pegvm::VM;
    // `*^:bad_doc` interns the author-supplied label in
    // `Program::label_kinds`; the runtime behavior is otherwise
    // identical to the unlabeled form (one recovery capture per
    // skipped byte, clean exit at EOF). pegdb explain-recoveries
    // clusters by this label.
    let g = parse("doc <- @kw{\"foo\"}*^:bad_doc");
    let prog = g.compile().unwrap();
    assert_eq!(prog.label_kinds, vec!["bad_doc"]);
    let r = VM::new(&prog.code, b"fooXXfoo").run();
    assert!(r.complete);
    assert_eq!(r.matched, 8);
    assert_eq!(prog.capture_kinds, vec!["kw", "recovery"]);
}

#[test]
fn end_to_end_catch_compile_run() {
    use syntax_highlighter::pegvm::VM;
    // Inner is a `SELECT … FROM` sequence; recovery scoops up the rest
    // of the statement under an @err capture and consumes the closing
    // semicolon. Input is malformed (no FROM), so the inner fails and
    // the recovery branch fires.
    let g = parse(
        "stmt <- (@kw{'SELECT'} ' ' @kw{'FROM'} ' ' 'x' ';') ^bad_select @err{(!';' .)*} ';'",
    );
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
