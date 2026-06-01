use syntax_highlighter::pegc::{parse as parse_src, Pattern, Span};
use syntax_highlighter::pegvm::CharSet;

/// Parse, then strip spans from every rule body. Tests in this file
/// assert structural shape via `assert_eq!` against hand-built
/// fixtures that carry [`Span::SYNTHETIC`]; the parsed pattern's real
/// positions would otherwise make the equality check diverge on the
/// span field. The dedicated positional-assertion tests for #114 use
/// `parse_src` directly to inspect real spans.
fn parse(src: &str) -> syntax_highlighter::pegc::Grammar {
    let mut g = parse_src(src).expect("parse failed");
    g.rules = g
        .rules
        .into_iter()
        .map(|(k, v)| (k, v.strip_spans()))
        .collect();
    g
}

#[test]
fn simple_rule() {
    let g = parse("foo <- \"hi\"");
    let order: Vec<&str> = g.rule_headers.iter().map(|h| h.name.as_str()).collect();
    assert_eq!(order, vec!["foo"]);
    assert_eq!(g.rules["foo"], Pattern::literal("hi"));
}

#[test]
fn rule_order_preserved() {
    let g = parse("first <- 'a'\nsecond <- 'b'");
    let order: Vec<&str> = g.rule_headers.iter().map(|h| h.name.as_str()).collect();
    assert_eq!(order, vec!["first", "second"]);
    assert_eq!(g.rules.len(), 2);
}

#[test]
fn ordered_choice_in_grammar() {
    let g = parse("r <- 'a' / 'b' / 'c'");
    assert_eq!(
        g.rules["r"],
        Pattern::choice(vec![
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
        Pattern::seq(vec![
            Pattern::literal("a"),
            Pattern::literal("b"),
            Pattern::literal("c"),
        ])
    );
}

#[test]
fn postfix_operators() {
    let g = parse("a <- 'x'*\nb <- 'x'+\nc <- 'x'?");
    assert_eq!(g.rules["a"], Pattern::repeat(Pattern::literal("x")));
    assert_eq!(g.rules["b"], Pattern::repeat_one(Pattern::literal("x")));
    assert_eq!(g.rules["c"], Pattern::optional(Pattern::literal("x")));
}

#[test]
fn postfix_repeat_count_single() {
    // `p{1}` is the bare atom — `Pattern::seq()` unwraps singletons.
    let g = parse("r <- 'x'{1}");
    assert_eq!(g.rules["r"], Pattern::literal("x"));
}

#[test]
fn postfix_repeat_count_multiple() {
    let g = parse("r <- 'x'{4}");
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![
            Pattern::literal("x"),
            Pattern::literal("x"),
            Pattern::literal("x"),
            Pattern::literal("x"),
        ])
    );
}

#[test]
fn postfix_repeat_count_on_backslash_atom() {
    let g = parse("r <- \\d{4}");
    let digit = Pattern::char_class(CharSet::from_ranges(&[('0', '9')]).unwrap());
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![digit.clone(), digit.clone(), digit.clone(), digit])
    );
}

#[test]
fn postfix_repeat_count_on_group() {
    let g = parse("r <- ('a' / 'b'){3}");
    let choice = Pattern::choice(vec![Pattern::literal("a"), Pattern::literal("b")]);
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![choice.clone(), choice.clone(), choice])
    );
}

#[test]
fn postfix_repeat_count_chains_with_star() {
    // `p{n}*` parses as `(p{n})*` — the postfix loop applies the `*`
    // to the already-desugared sequence.
    let g = parse("r <- 'x'{2}*");
    assert_eq!(
        g.rules["r"],
        Pattern::repeat(Pattern::seq(vec![
            Pattern::literal("x"),
            Pattern::literal("x"),
        ]))
    );
}

#[test]
fn postfix_repeat_count_at_maximum_accepted() {
    // Boundary case: 1024 is the cap, must be accepted.
    let g = parse("r <- 'x'{1024}");
    match &g.rules["r"] {
        Pattern::Sequence { items, .. } => assert_eq!(items.len(), 1024),
        other => panic!("expected Sequence of 1024 items, got: {other:?}"),
    }
}

#[test]
fn postfix_repeat_count_zero_rejected() {
    let err = parse_src("r <- 'x'{0}").unwrap_err();
    assert!(
        err.message.contains("positive"),
        "expected positive-required message, got: {}",
        err.message
    );
}

#[test]
fn postfix_repeat_count_empty_rejected() {
    let err = parse_src("r <- 'x'{}").unwrap_err();
    assert!(
        err.message.contains("expected positive integer"),
        "expected integer-required message, got: {}",
        err.message
    );
}

#[test]
fn postfix_repeat_count_unterminated_rejected() {
    let err = parse_src("r <- 'x'{4").unwrap_err();
    assert!(
        err.message.contains("'}'"),
        "expected closing-brace message, got: {}",
        err.message
    );
}

#[test]
fn postfix_repeat_count_non_decimal_rejected() {
    // The digit scan stops at `a`, leaving zero digits read — same error
    // path as `{}`.
    let err = parse_src("r <- 'x'{a}").unwrap_err();
    assert!(
        err.message.contains("expected positive integer"),
        "expected integer-required message, got: {}",
        err.message
    );
}

#[test]
fn postfix_repeat_count_exceeds_maximum_rejected() {
    let err = parse_src("r <- 'x'{1025}").unwrap_err();
    assert!(
        err.message.contains("exceeds maximum 1024"),
        "expected exceeds-maximum message, got: {}",
        err.message
    );
}

#[test]
fn postfix_repeat_count_whitespace_inside_braces_rejected() {
    // `{n}` is tight, matching the rest of the postfix tier — interior
    // whitespace makes the digit scan see zero digits.
    let err = parse_src("r <- 'x'{ 4 }").unwrap_err();
    assert!(
        err.message.contains("expected positive integer"),
        "expected integer-required message, got: {}",
        err.message
    );
}

#[test]
fn repeat_count_in_string_literal_is_literal() {
    // `'p{4}'` is the literal byte sequence `p{4}`, not a quantified
    // literal — string-literal escapes are not affected by this PR.
    let g = parse("r <- 'p{4}'");
    assert_eq!(g.rules["r"], Pattern::literal("p{4}"));
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
    Pattern::repeat(Pattern::Catch {
        inner: Box::new(inner),
        label: label.into(),
        recovery: Box::new(Pattern::capture("recovery", recovery_body)),
        span: Span::SYNTHETIC,
    })
}

#[test]
fn recover_repeat_postfix_star_caret() {
    // `p*^` desugars to `(p ^recovery @recovery{.})*` at parse time.
    let g = parse("r <- 'x'*^");
    assert_eq!(
        g.rules["r"],
        desugared_recover_repeat(Pattern::literal("x"), Pattern::any_char())
    );
}

#[test]
fn recover_repeat_postfix_plus_caret_lowers_to_seq() {
    // p+^  ≡  p (p*^)  — at least one inner success required.
    let g = parse("r <- 'x'+^");
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![
            Pattern::literal("x"),
            desugared_recover_repeat(Pattern::literal("x"), Pattern::any_char()),
        ])
    );
}

#[test]
fn sync_set_postfix_star_caret_charset() {
    // `p*^[;]` desugars to `(p ^recovery @recovery{(![;] .)* [;]})*`.
    let semi = CharSet::from_chars(&[';']);
    let g = parse("r <- 'x'*^[;]");
    let skip_loop = Pattern::repeat(Pattern::seq(vec![
        Pattern::not_predicate(Pattern::char_class(semi.clone())),
        Pattern::any_char(),
    ]));
    let recovery_body = Pattern::seq(vec![skip_loop, Pattern::char_class(semi)]);
    assert_eq!(
        g.rules["r"],
        desugared_recover_repeat(Pattern::literal("x"), recovery_body)
    );
}

#[test]
fn sync_set_postfix_plus_caret_charset() {
    // `p+^[;]` lowers to `p (p*^[;])`.
    let semi = CharSet::from_chars(&[';']);
    let g = parse("r <- 'x'+^[;]");
    let skip_loop = Pattern::repeat(Pattern::seq(vec![
        Pattern::not_predicate(Pattern::char_class(semi.clone())),
        Pattern::any_char(),
    ]));
    let recovery_body = Pattern::seq(vec![skip_loop, Pattern::char_class(semi)]);
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![
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
    let semi = CharSet::from_chars(&[';']);
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![
            desugared_recover_repeat(Pattern::literal("x"), Pattern::any_char()),
            Pattern::char_class(semi),
        ])
    );
}

#[test]
fn sync_set_accepts_negated_and_ranges() {
    // The sync set parses via the same `parse_charclass` path as a
    // normal `[...]` atom — ranges (`a-z`) and negation (`[^...]`)
    // are both supported.
    let g = parse("r <- 'x'*^[^a-z]");
    let alpha = CharSet::single_range('a', 'z').unwrap();
    let neg_alpha = alpha.negate();
    let skip_loop = Pattern::repeat(Pattern::seq(vec![
        Pattern::not_predicate(Pattern::char_class(neg_alpha.clone())),
        Pattern::any_char(),
    ]));
    let recovery_body = Pattern::seq(vec![skip_loop, Pattern::char_class(neg_alpha)]);
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
        desugared_recover_repeat_with_label(Pattern::literal("x"), Pattern::any_char(), "bad")
    );
}

#[test]
fn recover_repeat_postfix_plus_caret_with_label() {
    // `p+^:bad` lowers to `p (p*^:bad)` — the label flows through the
    // tail `*^` only; the head `p` is the unguarded one-iteration prefix.
    let g = parse("r <- 'x'+^:bad");
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![
            Pattern::literal("x"),
            desugared_recover_repeat_with_label(Pattern::literal("x"), Pattern::any_char(), "bad"),
        ])
    );
}

#[test]
fn sync_set_postfix_star_caret_charset_with_label() {
    // `p*^[;]:bad_stmt` interns label "bad_stmt"; recovery body
    // (sync-set skip) is unchanged.
    let semi = CharSet::from_chars(&[';']);
    let g = parse("r <- 'x'*^[;]:bad_stmt");
    let skip_loop = Pattern::repeat(Pattern::seq(vec![
        Pattern::not_predicate(Pattern::char_class(semi.clone())),
        Pattern::any_char(),
    ]));
    let recovery_body = Pattern::seq(vec![skip_loop, Pattern::char_class(semi)]);
    assert_eq!(
        g.rules["r"],
        desugared_recover_repeat_with_label(Pattern::literal("x"), recovery_body, "bad_stmt")
    );
}

#[test]
fn sync_set_postfix_plus_caret_charset_with_label() {
    // `p+^[;]:bad_stmt` lowers to `p (p*^[;]:bad_stmt)`.
    let semi = CharSet::from_chars(&[';']);
    let g = parse("r <- 'x'+^[;]:bad_stmt");
    let skip_loop = Pattern::repeat(Pattern::seq(vec![
        Pattern::not_predicate(Pattern::char_class(semi.clone())),
        Pattern::any_char(),
    ]));
    let recovery_body = Pattern::seq(vec![skip_loop, Pattern::char_class(semi)]);
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![
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
    let Pattern::Repeat { inner, .. } = &g.rules["r"] else {
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
        desugared_recover_repeat_with_label(Pattern::literal("x"), Pattern::any_char(), "_foo")
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
            span: Span::SYNTHETIC,
        }
    );
}

#[test]
fn catch_binds_tighter_than_choice() {
    // 'a' ^lbl 'b' / 'c'   ≡   ('a' ^lbl 'b') / 'c'
    let g = parse("r <- 'a' ^lbl 'b' / 'c'");
    assert_eq!(
        g.rules["r"],
        Pattern::choice(vec![
            Pattern::Catch {
                inner: Box::new(Pattern::literal("a")),
                label: "lbl".into(),
                recovery: Box::new(Pattern::literal("b")),
                span: Span::SYNTHETIC,
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
            inner: Box::new(Pattern::seq(vec![
                Pattern::literal("a"),
                Pattern::literal("b"),
            ])),
            label: "lbl".into(),
            recovery: Box::new(Pattern::seq(vec![
                Pattern::literal("c"),
                Pattern::literal("d"),
            ])),
            span: Span::SYNTHETIC,
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
                span: Span::SYNTHETIC,
            }),
            label: "l2".into(),
            recovery: Box::new(Pattern::literal("c")),
            span: Span::SYNTHETIC,
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
        desugared_recover_repeat(Pattern::literal("x"), Pattern::any_char())
    );
    assert_eq!(
        g.rules["b"],
        Pattern::Catch {
            inner: Box::new(Pattern::repeat(Pattern::literal("x"))),
            label: "lbl".into(),
            recovery: Box::new(Pattern::literal("y")),
            span: Span::SYNTHETIC,
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
            recovery: Box::new(Pattern::choice(vec![
                Pattern::literal("b"),
                Pattern::literal("c"),
            ])),
            span: Span::SYNTHETIC,
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
        span: Span::SYNTHETIC,
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

// ---- Boundary-anchored catch (`^^lbl B` / `^^lbl`) ------------

/// Builds the AST that `INNER ^^lbl B` lowers to: a `Catch` whose
/// inner is `Sequence([INNER, AndPredicate(B)])` and whose recovery
/// is `@recovery{(!B .)*}`. Mirrors `lower_boundary_catch` in the
/// parser.
fn lowered_boundary_catch(inner: Pattern, label: &str, boundary: Pattern) -> Pattern {
    let stop_loop = Pattern::repeat(Pattern::seq(vec![
        Pattern::not_predicate(boundary.clone()),
        Pattern::any_char(),
    ]));
    Pattern::Catch {
        inner: Box::new(Pattern::seq(vec![inner, Pattern::and_predicate(boundary)])),
        label: label.into(),
        recovery: Box::new(Pattern::capture("recovery", stop_loop)),
        span: Span::SYNTHETIC,
    }
}

#[test]
fn boundary_catch_lowers_to_anchored_catch_with_recovery_loop() {
    let g = parse("r <- 'a' ^^lbl 'b'");
    assert_eq!(
        g.rules["r"],
        lowered_boundary_catch(Pattern::literal("a"), "lbl", Pattern::literal("b")),
    );
}

#[test]
fn boundary_catch_with_rule_boundary() {
    let g = parse("r <- 'a' ^^lbl b\nb <- 'x'");
    assert_eq!(
        g.rules["r"],
        lowered_boundary_catch(Pattern::literal("a"), "lbl", Pattern::nt("b")),
    );
}

#[test]
fn boundary_catch_with_charset_boundary() {
    let g = parse("r <- 'a' ^^lbl [,;)]");
    let delim = CharSet::from_chars(&[',', ';', ')']);
    assert_eq!(
        g.rules["r"],
        lowered_boundary_catch(Pattern::literal("a"), "lbl", Pattern::char_class(delim)),
    );
}

#[test]
fn boundary_catch_with_grouped_boundary() {
    let g = parse("r <- 'a' ^^lbl (b c)\nb <- 'x'\nc <- 'y'");
    let boundary = Pattern::seq(vec![Pattern::nt("b"), Pattern::nt("c")]);
    assert_eq!(
        g.rules["r"],
        lowered_boundary_catch(Pattern::literal("a"), "lbl", boundary),
    );
}

#[test]
fn boundary_catch_label_touches_second_caret() {
    // `^^lbl B` with no whitespace between the carets and the label
    // is the canonical form; `^^ lbl B` (space) errors with the
    // "expected label identifier" message.
    let g = parse("r <- 'a' ^^lbl 'b'");
    assert_eq!(
        g.rules["r"],
        lowered_boundary_catch(Pattern::literal("a"), "lbl", Pattern::literal("b")),
    );
    let err = parse_src("r <- 'a' ^^ lbl 'b'").unwrap_err();
    assert!(
        err.message.contains("expected label identifier"),
        "expected a label-identifier error, got: {}",
        err.message
    );
}

#[test]
fn boundary_catch_does_not_swallow_trailing_atoms() {
    // The boundary is one atom-with-prefix-postfix; trailing atoms
    // belong to the enclosing sequence: `A ^^lbl B C` is `(A ^^lbl B) C`.
    let g = parse("r <- 'a' ^^lbl 'b' 'c'");
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![
            lowered_boundary_catch(Pattern::literal("a"), "lbl", Pattern::literal("b")),
            Pattern::literal("c"),
        ]),
    );
}

#[test]
fn bare_catch_with_capture_rhs_still_parses() {
    // Regression guard: single `^` is bare, `@kind{...}` is the RHS.
    // The doubled-caret discriminator must not consume `@`.
    let g = parse("r <- 'a' ^lbl @rec{'b'}");
    assert_eq!(
        g.rules["r"],
        Pattern::Catch {
            inner: Box::new(Pattern::literal("a")),
            label: "lbl".into(),
            recovery: Box::new(Pattern::capture("rec", Pattern::literal("b"))),
            span: Span::SYNTHETIC,
        },
    );
}

#[test]
fn boundary_catch_rejects_reserved_underscore_label() {
    let err = parse_src("r <- 'a' ^^_ 'b'").unwrap_err();
    assert!(
        err.message.contains("reserved"),
        "expected reserved-label error, got: {}",
        err.message
    );
}

#[test]
fn boundary_catch_rejects_throw_atom_shape() {
    let err = parse_src("r <- 'a' ^^!lbl 'b'").unwrap_err();
    assert!(
        err.message.contains("reserved") || err.message.contains("expected label identifier"),
        "expected reserved-or-label error, got: {}",
        err.message
    );
}

#[test]
fn inferred_catch_at_end_of_rule_parses_to_placeholder() {
    let g = parse("r <- 'a' ^^lbl");
    assert_eq!(
        g.rules["r"],
        Pattern::InferBoundaryCatch {
            inner: Box::new(Pattern::literal("a")),
            label: "lbl".into(),
            span: Span::SYNTHETIC,
        },
    );
}

#[test]
fn inferred_catch_before_choice_separator() {
    let g = parse("r <- 'a' ^^lbl / 'b'");
    assert_eq!(
        g.rules["r"],
        Pattern::choice(vec![
            Pattern::InferBoundaryCatch {
                inner: Box::new(Pattern::literal("a")),
                label: "lbl".into(),
                span: Span::SYNTHETIC,
            },
            Pattern::literal("b"),
        ]),
    );
}

#[test]
fn inferred_catch_before_paren_close() {
    let g = parse("r <- ('a' ^^lbl) 'c'");
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![
            Pattern::InferBoundaryCatch {
                inner: Box::new(Pattern::literal("a")),
                label: "lbl".into(),
                span: Span::SYNTHETIC,
            },
            Pattern::literal("c"),
        ]),
    );
}

#[test]
fn inferred_vs_explicit_no_ambiguity() {
    // Disambiguation is purely by `at_prefix_start`: `^^lbl B C` is
    // explicit (B is the boundary, C is the trailing sequence atom);
    // `^^lbl / B` is inferred (`/` is not an atom-start).
    let explicit = parse("r <- 'a' ^^lbl 'b' 'c'");
    assert_eq!(
        explicit.rules["r"],
        Pattern::seq(vec![
            lowered_boundary_catch(Pattern::literal("a"), "lbl", Pattern::literal("b")),
            Pattern::literal("c"),
        ]),
    );
    let inferred = parse("r <- 'a' ^^lbl / 'b'");
    assert_eq!(
        inferred.rules["r"],
        Pattern::choice(vec![
            Pattern::InferBoundaryCatch {
                inner: Box::new(Pattern::literal("a")),
                label: "lbl".into(),
                span: Span::SYNTHETIC,
            },
            Pattern::literal("b"),
        ]),
    );
}

// ---- Lenient marker (definition-level only) -------------------
//
// Definition-level `~name <- body` parsing is verified end-to-end by
// the `definition_lenient_marker_*` tests in `tests/compiler_tests.rs`.
// The call-site `~p` form was considered during design and dropped
// before landing: empirically zero shipped grammars used it after the
// pivot to definition-level marking.

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
            span: Span::SYNTHETIC,
        }
    );
}

#[test]
fn predicate_operators() {
    let g = parse("a <- !'x' .\nb <- &'y' 'y'");
    assert_eq!(
        g.rules["a"],
        Pattern::seq(vec![
            Pattern::not_predicate(Pattern::literal("x")),
            Pattern::any_char(),
        ])
    );
    assert_eq!(
        g.rules["b"],
        Pattern::seq(vec![
            Pattern::and_predicate(Pattern::literal("y")),
            Pattern::literal("y"),
        ])
    );
}

#[test]
fn char_class_with_range() {
    let g = parse("d <- [0-9]");
    assert_eq!(
        g.rules["d"],
        Pattern::char_class(CharSet::from_ranges(&[('0', '9')]).unwrap())
    );
}

#[test]
fn char_class_negated() {
    let g = parse("nq <- [^\"\\\\]");
    let excluded = CharSet::from_chars(&['"', '\\']);
    assert_eq!(g.rules["nq"], Pattern::char_class(excluded.negate()));
}

#[test]
fn char_class_mixed_chars_and_ranges() {
    let g = parse("alnum <- [a-zA-Z0-9_]");
    let expected = CharSet::from_ranges(&[('a', 'z'), ('A', 'Z'), ('0', '9'), ('_', '_')]).unwrap();
    assert_eq!(g.rules["alnum"], Pattern::char_class(expected));
}

#[test]
fn capture_annotation() {
    let g = parse("r <- @keyword{'while'}");
    assert_eq!(
        g.rules["r"],
        Pattern::capture("keyword", Pattern::literal("while"))
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
        Pattern::seq(vec![
            Pattern::literal("a"),
            Pattern::choice(vec![Pattern::literal("b"), Pattern::literal("c")]),
            Pattern::literal("d"),
        ])
    );
}

#[test]
fn nonterminal_reference() {
    let g = parse("a <- b\nb <- 'x'");
    assert_eq!(g.rules["a"], Pattern::nt("b"));
}

#[test]
fn escape_sequences_in_string() {
    let g = parse("r <- '\\n\\t\\\\'");
    assert_eq!(
        g.rules["r"],
        Pattern::literal_bytes(vec![b'\n', b'\t', b'\\'])
    );
}

#[test]
fn dash_in_class_at_end_is_literal() {
    let g = parse("r <- [+\\-]");
    let s = CharSet::from_chars(&['+', '-']);
    assert_eq!(g.rules["r"], Pattern::char_class(s));
}

#[test]
fn backslash_d_atom() {
    let g = parse("r <- \\d");
    assert_eq!(
        g.rules["r"],
        Pattern::char_class(CharSet::from_ranges(&[('0', '9')]).unwrap())
    );
}

#[test]
fn backslash_d_negated_atom() {
    let g = parse("r <- \\D");
    assert_eq!(
        g.rules["r"],
        Pattern::char_class(CharSet::from_ranges(&[('0', '9')]).unwrap().negate())
    );
}

#[test]
fn backslash_s_atom() {
    let g = parse("r <- \\s");
    assert_eq!(
        g.rules["r"],
        Pattern::char_class(CharSet::from_chars(&[' ', '\t', '\n', '\r']))
    );
}

#[test]
fn backslash_s_negated_atom() {
    let g = parse("r <- \\S");
    assert_eq!(
        g.rules["r"],
        Pattern::char_class(CharSet::from_chars(&[' ', '\t', '\n', '\r']).negate())
    );
}

#[test]
fn backslash_h_atom() {
    let g = parse("r <- \\h");
    assert_eq!(
        g.rules["r"],
        Pattern::char_class(CharSet::from_chars(&[' ', '\t']))
    );
}

#[test]
fn backslash_h_negated_atom() {
    let g = parse("r <- \\H");
    assert_eq!(
        g.rules["r"],
        Pattern::char_class(CharSet::from_chars(&[' ', '\t']).negate())
    );
}

#[test]
fn backslash_cap_r_linebreak_atom() {
    let g = parse("r <- \\R");
    assert_eq!(
        g.rules["r"],
        Pattern::choice(vec![
            Pattern::literal("\r\n"),
            Pattern::literal("\n"),
            Pattern::literal("\r"),
        ])
    );
}

#[test]
fn backslash_d_in_class_unions_digits() {
    let g = parse("r <- [\\d_]");
    let s = CharSet::from_ranges(&[('0', '9'), ('_', '_')]).unwrap();
    assert_eq!(g.rules["r"], Pattern::char_class(s));
}

#[test]
fn backslash_d_in_class_with_range_neighbor() {
    // Mixed class: `[\da-fA-F]` is the hex-digit set.
    let g = parse("r <- [\\da-fA-F]");
    let s = CharSet::from_ranges(&[('0', '9'), ('a', 'f'), ('A', 'F')]).unwrap();
    assert_eq!(g.rules["r"], Pattern::char_class(s));
}

#[test]
fn backslash_s_in_class_unions_whitespace() {
    let g = parse("r <- [\\sX]");
    let s = CharSet::from_chars(&[' ', '\t', '\n', '\r', 'X']);
    assert_eq!(g.rules["r"], Pattern::char_class(s));
}

#[test]
fn backslash_d_in_negated_class() {
    // `[^\d]` is the complement of `\d`, i.e. `\D`.
    let g = parse("r <- [^\\d]");
    assert_eq!(
        g.rules["r"],
        Pattern::char_class(CharSet::from_ranges(&[('0', '9')]).unwrap().negate())
    );
}

#[test]
fn backslash_d_in_string_still_errors() {
    // String literals keep their byte-only escape contract; `\d` is
    // not a single byte, so it's still an unknown string escape.
    let err = parse_src("r <- '\\d'").unwrap_err();
    assert!(
        err.message.contains("unknown escape"),
        "expected unknown-escape error in string, got: {}",
        err.message
    );
}

#[test]
fn backslash_cap_r_in_class_rejected() {
    let err = parse_src("r <- [\\R]").unwrap_err();
    assert!(
        err.message.contains("multi-byte") && err.message.contains("\\R"),
        "expected tailored multi-byte error for \\R in class, got: {}",
        err.message
    );
}

#[test]
fn backslash_d_range_start_rejected() {
    // Shortcuts are sets, not bounds — `[\d-z]` is meaningless.
    let err = parse_src("r <- [\\d-z]").unwrap_err();
    assert!(
        err.message.contains("range can't start with a shortcut"),
        "expected shortcut-in-range error, got: {}",
        err.message
    );
}

#[test]
fn unknown_backslash_atom_errors() {
    let err = parse_src("r <- \\q").unwrap_err();
    assert!(
        err.message.contains("unknown atom") && err.message.contains("\\q"),
        "expected unknown-atom error, got: {}",
        err.message
    );
}

#[test]
fn end_to_end_grammar_compile_run() {
    use syntax_highlighter::pegvm::VM;
    let g = parse("root <- number\nnumber <- [0-9]+");
    let prog = g.compile().unwrap();
    let r = VM::new_from_program(&prog, b"42abc").run();
    // `root`'s implicit `!.` rejects the trailing 'a'; the parse no
    // longer succeeds on the prefix.
    assert!(!r.complete);
    assert!(r.matched >= 2, "got matched={}", r.matched);
}

#[test]
fn end_to_end_recover_repeat_compile_run() {
    use syntax_highlighter::pegvm::VM;
    // Top-level `*^` resyncs past garbage one byte at a time; the parse
    // completes at EOF, with one "recovery"-tagged capture per skipped byte.
    let g = parse("root <- doc\ndoc <- @kw{\"foo\"}*^");
    let prog = g.compile().unwrap();
    let r = VM::new_from_program(&prog, b"fooXXfoo").run();
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
fn end_to_end_inferred_catch_matches_explicit_behavior() {
    // `^^lbl` (FOLLOW-inferred) should produce a program with the
    // same observable behavior as the explicit `^^lbl B` form when
    // `B` mirrors the call site's FOLLOW set. Byte-identical
    // bytecode isn't expected (label numbering shifts), but the
    // MatchResult on the same input must agree.
    use syntax_highlighter::pegvm::VM;
    let inferred = parse("root <- list\nlist <- aliased (',' aliased)*\naliased <- 'x' ^^bad")
        .compile()
        .unwrap();
    let explicit =
        parse("root <- list\nlist <- aliased (',' aliased)*\naliased <- 'x' ^^bad (',' / !.)")
            .compile()
            .unwrap();
    for input in [&b"x,x"[..], b"xy,x", b"x"].iter() {
        let r_inferred = VM::new(&inferred.code, input).run();
        let r_explicit = VM::new(&explicit.code, input).run();
        assert_eq!(
            r_inferred.complete, r_explicit.complete,
            "completion differs for input {input:?}"
        );
        assert_eq!(
            r_inferred.matched, r_explicit.matched,
            "matched length differs for input {input:?}"
        );
    }
}

#[test]
fn end_to_end_inferred_catch_fires_on_partial_match() {
    use syntax_highlighter::pegvm::VM;
    // `aliased <- 'x' ^^bad` lowers to a catch that anchors `x` to
    // FOLLOW(aliased). FOLLOW contains `,` from the call site in
    // `list`. Input `xy,x` — the first `x` succeeds but the next
    // byte `y` is not in FOLLOW; the catch fires, recovery skips
    // `y` until it sees `,`, and parsing resumes with the second `x`.
    let prog = parse("root <- list\nlist <- aliased (',' aliased)*\naliased <- 'x' ^^bad")
        .compile()
        .unwrap();
    let r = VM::new_from_program(&prog, b"xy,x").run();
    assert!(r.complete, "catch should let parse complete; got {:?}", r);
}

#[test]
fn end_to_end_inferred_catch_clean_input_no_recovery() {
    use syntax_highlighter::pegvm::VM;
    let prog = parse("root <- list\nlist <- aliased (',' aliased)*\naliased <- 'x' ^^bad")
        .compile()
        .unwrap();
    let r = VM::new_from_program(&prog, b"x,x").run();
    assert!(r.complete);
    let recovery_captures = r
        .captures
        .iter()
        .filter(|c| prog.capture_kinds[c.kind.0 as usize] == "recovery")
        .count();
    assert_eq!(recovery_captures, 0, "clean input should emit no recovery");
}

#[test]
fn end_to_end_lenient_marker_is_runtime_transparent() {
    use syntax_highlighter::pegvm::VM;
    // The marker rides a non-special helper rule — `root` (like
    // `trivia` / `wb`) rejects every qualifier.
    let plain = parse("root <- r\nr <- 'x'+").compile().unwrap();
    let lenient = parse("root <- r\n~r <- 'x'+").compile().unwrap();
    assert_eq!(plain.code, lenient.code, "`~` is runtime-transparent");
    let r = VM::new(&lenient.code, b"xxx").run();
    assert!(r.complete);
    assert_eq!(r.matched, 3);
}

#[test]
fn end_to_end_recover_repeat_with_label_intern() {
    use syntax_highlighter::pegvm::VM;
    // `*^:bad_doc` interns the author-supplied label in
    // `Program::label_kinds`; the runtime behavior is otherwise
    // identical to the unlabeled form (one recovery capture per
    // skipped byte, clean exit at EOF). pegdb recoveries explain
    // clusters by this label.
    let g = parse("root <- doc\ndoc <- @kw{\"foo\"}*^:bad_doc");
    let prog = g.compile().unwrap();
    assert_eq!(prog.label_kinds, vec!["bad_doc"]);
    let r = VM::new_from_program(&prog, b"fooXXfoo").run();
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
        "root <- stmt\nstmt <- (@kw{'SELECT'} ' ' @kw{'FROM'} ' ' 'x' ';') ^bad_select @err{(!';' .)*} ';'",
    );
    let prog = g.compile().unwrap();
    let r = VM::new_from_program(&prog, b"SELECT bogus;").run();
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

/// Builds the desugared AST that `.. S` lowers to:
/// `Repeat(Seq(NotPredicate(S), AnyChar))`. Mirrors
/// `lower_until_exclusive` in `src/pegc/parser.rs`.
fn desugared_until_exclusive(stop: Pattern) -> Pattern {
    Pattern::repeat(Pattern::seq(vec![
        Pattern::not_predicate(stop),
        Pattern::any_char(),
    ]))
}

/// Builds the desugared AST that `..= S` lowers to:
/// `Seq(.. S, S)`. Mirrors `lower_until_inclusive` in
/// `src/pegc/parser.rs`.
fn desugared_until_inclusive(stop: Pattern) -> Pattern {
    Pattern::seq(vec![desugared_until_exclusive(stop.clone()), stop])
}

#[test]
fn skip_until_exclusive_lowers_to_repeat() {
    let g = parse("r <- .. 'b'");
    assert_eq!(
        g.rules["r"],
        desugared_until_exclusive(Pattern::literal("b"))
    );
}

#[test]
fn skip_until_inclusive_lowers_with_trailing_consume() {
    let g = parse("r <- ..= 'b'");
    assert_eq!(
        g.rules["r"],
        desugared_until_inclusive(Pattern::literal("b"))
    );
}

#[test]
fn skip_until_inclusive_vs_exclusive_distinction() {
    // Same stop pattern; ASTs differ exactly in the trailing consume.
    let excl = parse("r <- .. 'b'").rules["r"].clone();
    let incl = parse("r <- ..= 'b'").rules["r"].clone();
    assert_eq!(incl, Pattern::seq(vec![excl, Pattern::literal("b")]));
}

#[test]
fn skip_until_inside_sequence() {
    let g = parse("r <- 'x' .. 'b' 'y'");
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![
            Pattern::literal("x"),
            desugared_until_exclusive(Pattern::literal("b")),
            Pattern::literal("y"),
        ])
    );
}

#[test]
fn skip_until_requires_adjacent_dots() {
    // `. . 'b'` (space between dots) is three atoms, not `..`.
    let g = parse("r <- . . 'b'");
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![
            Pattern::any_char(),
            Pattern::any_char(),
            Pattern::literal("b"),
        ])
    );
}

#[test]
fn skip_until_inclusive_requires_adjacent_equals() {
    // `.. = 'b'` (space between `..` and `=`) is `..` whose stop is
    // `=`, then `'b'` as a trailing atom. Since `=` isn't a valid
    // atom-starter, the parse should fail at `=`.
    let err = parse_src("r <- .. = 'b'").unwrap_err();
    assert!(
        err.message.contains("unexpected") || err.message.contains("expected"),
        "expected parse error on `=`, got: {}",
        err.message
    );
}

#[test]
fn skip_until_whitespace_after_operator_ok() {
    let a = parse("r <- ..'b'").rules["r"].clone();
    let b = parse("r <- ..  'b'").rules["r"].clone();
    assert_eq!(a, b);
    let c = parse("r <- ..='b'").rules["r"].clone();
    let d = parse("r <- ..=  'b'").rules["r"].clone();
    assert_eq!(c, d);
}

#[test]
fn skip_until_with_charclass_stop() {
    let g = parse("r <- .. [;]");
    let stop = Pattern::char_class(CharSet::from_chars(&[';']));
    assert_eq!(g.rules["r"], desugared_until_exclusive(stop));
}

#[test]
fn skip_until_inclusive_with_grouped_stop() {
    // Mirrors the sqlite `bracket_id` shape: stop is an OrderedChoice.
    let g = parse("r <- ..= (']' / 'x')");
    let stop = Pattern::choice(vec![Pattern::literal("]"), Pattern::literal("x")]);
    assert_eq!(g.rules["r"], desugared_until_inclusive(stop));
}

#[test]
fn skip_until_requires_right_operand() {
    let err = parse_src("r <- ..").unwrap_err();
    assert!(
        err.message.contains("unexpected") || err.message.contains("expected"),
        "expected parse error at end of `..`, got: {}",
        err.message
    );
}

#[test]
fn skip_until_inclusive_with_capture_stop() {
    // The stop pattern's capture survives lowering: the trailing
    // consume in `..= S` retains the same capture kind so themes
    // render the consumed delimiter consistently.
    let g = parse("r <- ..= @punctuation{'}'}");
    let stop = Pattern::capture("punctuation", Pattern::literal("}"));
    assert_eq!(g.rules["r"], desugared_until_inclusive(stop));
}

#[test]
fn skip_until_with_capture_stop_exclusive() {
    let g = parse("r <- .. @comment{'#'}");
    let stop = Pattern::capture("comment", Pattern::literal("#"));
    assert_eq!(g.rules["r"], desugared_until_exclusive(stop));
}

/// Builds the desugared AST that `INNER ^^lbl ..= B` lowers to:
/// `Catch { INNER, lbl, Seq(@recovery{(!B .)*}, B) }`. No `&B`
/// anchor on the inner. Mirrors `lower_bracketed_close_catch` in
/// `src/pegc/parser.rs`.
fn desugared_bracketed_close_catch(inner: Pattern, label: &str, boundary: Pattern) -> Pattern {
    let stop_loop = Pattern::repeat(Pattern::seq(vec![
        Pattern::not_predicate(boundary.clone()),
        Pattern::any_char(),
    ]));
    let recovery = Pattern::seq(vec![Pattern::capture("recovery", stop_loop), boundary]);
    Pattern::Catch {
        inner: Box::new(inner),
        label: label.into(),
        recovery: Box::new(recovery),
        span: Span::SYNTHETIC,
    }
}

#[test]
fn bracketed_close_catch_lowers_without_inner_anchor() {
    let g = parse("r <- 'a' ^^lbl ..= '}'");
    assert_eq!(
        g.rules["r"],
        desugared_bracketed_close_catch(Pattern::literal("a"), "lbl", Pattern::literal("}"))
    );
}

#[test]
fn bracketed_close_catch_with_capture_boundary() {
    // The boundary's capture kind is preserved on the trailing
    // consume — `}` is captured as `@punctuation` in both happy and
    // recovery paths.
    let g = parse("r <- 'a' ^^lbl ..= @punctuation{'}'}");
    let boundary = Pattern::capture("punctuation", Pattern::literal("}"));
    assert_eq!(
        g.rules["r"],
        desugared_bracketed_close_catch(Pattern::literal("a"), "lbl", boundary)
    );
}

#[test]
fn bracketed_close_catch_vs_boundary_catch_distinction() {
    // `^^lbl B`   — anchors INNER with `&B`; recovery is just `@recovery{(!B .)*}` (B left for outer).
    // `^^lbl ..= B` — no inner anchor; recovery is `Seq(@recovery{(!B .)*}, B)` (B consumed by recovery).
    let anchored = parse("r <- 'a' ^^lbl '}'").rules["r"].clone();
    let bracketed = parse("r <- 'a' ^^lbl ..= '}'").rules["r"].clone();
    match (&anchored, &bracketed) {
        (
            Pattern::Catch {
                inner: i_anch,
                recovery: r_anch,
                ..
            },
            Pattern::Catch {
                inner: i_brkt,
                recovery: r_brkt,
                ..
            },
        ) => {
            // Anchored inner is a Sequence ending in AndPredicate; bracketed inner is the bare INNER.
            assert!(
                matches!(i_anch.as_ref(), Pattern::Sequence { items, .. } if matches!(items.last(), Some(Pattern::AndPredicate { .. }))),
                "anchored form should have &B on inner: {i_anch:?}"
            );
            assert_eq!(
                i_brkt.as_ref(),
                &Pattern::literal("a"),
                "bracketed form should NOT anchor inner: {i_brkt:?}"
            );
            // Anchored recovery is just the capture; bracketed recovery is Seq(capture, B).
            assert!(
                matches!(r_anch.as_ref(), Pattern::Capture { .. }),
                "anchored recovery should be a capture, got: {r_anch:?}"
            );
            assert!(
                matches!(r_brkt.as_ref(), Pattern::Sequence { .. }),
                "bracketed recovery should be a Sequence, got: {r_brkt:?}"
            );
        }
        other => panic!("expected two Catch nodes, got: {other:?}"),
    }
}

#[test]
fn bracketed_close_catch_rejects_non_consuming_form() {
    // `^^lbl .. B` is inconsistent (catch necessarily consumes B); the
    // parser rejects it with a hint at `..=`.
    let err = parse_src("r <- 'a' ^^lbl .. '}'").unwrap_err();
    assert!(
        err.message.contains("..=") || err.message.contains("consume"),
        "expected hint about `..=`, got: {}",
        err.message
    );
}

#[test]
fn bracketed_close_catch_does_not_swallow_trailing_atoms() {
    // The catch sugar consumes its boundary in the recovery; any atoms
    // after the boundary belong to the enclosing sequence.
    let g = parse("r <- 'a' ^^lbl ..= '}' 'c'");
    match &g.rules["r"] {
        Pattern::Sequence { items, .. } => {
            assert_eq!(items.len(), 2, "expected Seq(catch, 'c'), got: {items:?}");
            assert!(matches!(&items[0], Pattern::Catch { .. }));
            assert_eq!(items[1], Pattern::literal("c"));
        }
        other => panic!("expected Sequence, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------
// `i"…"` / `i'…'` — case-insensitive literal sugar.
//
// The desugar emits a `Sequence` of `CharClass` nodes, one per code
// point: ASCII letters fold to `{lower, upper}`, everything else stays
// a singleton. Each test below asserts the desugar shape directly
// against a hand-built fixture; the per-letter `[xX][yY]…` form parses
// to the same shape, which is what makes the sugar a drop-in for the
// hand-rolled keyword bodies in `grammars/sqlite.peg`.

fn ci_class(lower: char, upper: char) -> Pattern {
    Pattern::char_class(CharSet::from_chars(&[lower, upper]))
}

#[test]
fn case_insensitive_literal_basic() {
    let g = parse("r <- i\"select\"");
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![
            ci_class('s', 'S'),
            ci_class('e', 'E'),
            ci_class('l', 'L'),
            ci_class('e', 'E'),
            ci_class('c', 'C'),
            ci_class('t', 'T'),
        ])
    );
}

#[test]
fn case_insensitive_literal_equivalent_to_manual_form() {
    // The drop-in invariant: `i"select"` and `[sS][eE][lL][eE][cC][tT]`
    // parse to byte-identical AST shapes (modulo spans, stripped here).
    let g_sugared = parse("r <- i\"select\"");
    let g_manual = parse("r <- [sS][eE][lL][eE][cC][tT]");
    assert_eq!(g_sugared.rules["r"], g_manual.rules["r"]);
}

#[test]
fn case_insensitive_literal_single_quoted() {
    // Both quote flavours work, mirroring `parse_string`.
    let g_double = parse("r <- i\"abc\"");
    let g_single = parse("r <- i'abc'");
    assert_eq!(g_double.rules["r"], g_single.rules["r"]);
}

#[test]
fn case_insensitive_literal_non_letter_codepoints_stay_singletons() {
    let g = parse("r <- i\"a1_z\"");
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![
            ci_class('a', 'A'),
            Pattern::char_class(CharSet::singleton('1')),
            Pattern::char_class(CharSet::singleton('_')),
            ci_class('z', 'Z'),
        ])
    );
}

#[test]
fn case_insensitive_literal_empty_is_empty_literal() {
    // Matches `parse_string`'s treatment of `""`: no atoms to fold, so
    // the result is the empty `Literal` (matches the empty string).
    let g = parse("r <- i\"\"");
    assert_eq!(g.rules["r"], Pattern::literal(""));
}

#[test]
fn case_insensitive_literal_no_letters_is_singletons() {
    let g = parse("r <- i\"123\"");
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![
            Pattern::char_class(CharSet::singleton('1')),
            Pattern::char_class(CharSet::singleton('2')),
            Pattern::char_class(CharSet::singleton('3')),
        ])
    );
}

#[test]
fn case_insensitive_literal_handles_escapes() {
    // Same escape handling as `parse_string`: `\n` is the literal byte,
    // not a letter, so it stays a singleton.
    let g = parse("r <- i\"a\\nb\"");
    assert_eq!(
        g.rules["r"],
        Pattern::seq(vec![
            ci_class('a', 'A'),
            Pattern::char_class(CharSet::singleton('\n')),
            ci_class('b', 'B'),
        ])
    );
}

#[test]
fn case_insensitive_literal_disambiguates_against_identifier() {
    // `i` followed by a non-quote byte must still parse as the start of
    // a `NonTerminal` — otherwise grammars with identifiers like
    // `ident_char` would suddenly become parse errors.
    let g = parse("r <- ident_char");
    assert_eq!(g.rules["r"], Pattern::nt("ident_char"));
}

#[test]
fn case_insensitive_literal_inside_capture() {
    let g = parse("r <- @keyword{i\"select\"}");
    let body = Pattern::seq(vec![
        ci_class('s', 'S'),
        ci_class('e', 'E'),
        ci_class('l', 'L'),
        ci_class('e', 'E'),
        ci_class('c', 'C'),
        ci_class('t', 'T'),
    ]);
    assert_eq!(g.rules["r"], Pattern::capture("keyword", body));
}

#[test]
fn case_insensitive_literal_composes_with_postfix_and_lookahead() {
    let g = parse("r <- i\"a\"*\ns <- !i\"a\"");
    let body = ci_class('a', 'A');
    assert_eq!(g.rules["r"], Pattern::repeat(body.clone()));
    assert_eq!(g.rules["s"], Pattern::not_predicate(body));
}

#[test]
fn case_insensitive_literal_unterminated_errors() {
    let err = parse_src("r <- i\"select").unwrap_err();
    assert!(
        err.message.contains("unterminated"),
        "expected unterminated-literal error, got: {}",
        err.message
    );
}
