use syntax_highlighter::pegvm::CharSet;

/// Source position of a [`Pattern`] node, captured by the parser when
/// it enters the construct's parse function. One-based line and column,
/// matching [`crate::pegc::parser::ParseError`].
///
/// Used by [`crate::pegc::analysis::LintFinding`] (call-site span) and
/// [`crate::pegc::compiler::CompileError::CannotInferBoundary`] to give
/// fatal grammar diagnostics a navigable position. Editor tooling (#39)
/// can pick this up for hover / jump-to-definition without further AST
/// surgery.
///
/// Synthesized nodes — those produced by the resolver / lowering passes
/// from a FOLLOW set rather than directly from source — carry
/// [`Span::SYNTHETIC`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Ord, PartialOrd)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    /// Marker for nodes with no source position: the FOLLOW-set
    /// boundary materialization, the `@recovery ((!B .)*)` body
    /// synthesized in `lower_inferred_boundary_catch`, the
    /// left-recursion alternative materialization, and Pattern trees
    /// hand-built by tests.
    pub const SYNTHETIC: Span = Span { line: 0, col: 0 };
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

/// One integer argument to a parameterized rule call or an indent
/// combinator: either a literal column or a reference to an in-scope
/// local (a rule parameter or an `as`-bound indent width). Lowered to a
/// concrete [`syntax_highlighter::pegvm::ArgSrc`] by the compiler, which resolves
/// `Local` names to activation-slot indices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndentArg {
    /// A literal column — the `0` in `block(0)`.
    Lit(i32),
    /// A reference to an in-scope local by name (`outer`, `i`).
    Local(String),
}

/// The relation a declarative indent combinator asserts between the
/// indentation it measures at the current line and its argument column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentOp {
    /// `deeper(n)` — measured strictly greater than `n`.
    Deeper,
    /// `same(n)` — measured exactly equal to `n`.
    Same,
    /// `at_least(n)` — measured greater than or equal to `n`.
    AtLeast,
}

/// Surface-level indentation operator — the `%root` / `%align` / `%indent`
/// the grammar author writes. A transient kind: the parser produces a
/// [`Pattern::IndentOp`] carrying one of these, and `desugar_indent`
/// rewrites the whole thing into the lower-level [`IndentCombinator`] +
/// parameterized [`Pattern::NonTerminal`] threading before any other pass
/// runs. It never reaches analysis, lowering, or the VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentOpKind {
    /// `%root X` — seed a fresh anchor at the line's own column and run
    /// `X` under it. Desugars to `at_least(0) as <anchor>` over `X`.
    Root,
    /// `%align` — assert the current line starts at the ambient anchor
    /// column. Standalone (no operand). Desugars to `same(<anchor>)`.
    Align,
    /// `%indent X` — open a level strictly deeper than the ambient anchor,
    /// rebind the anchor to it, and run `X`. Desugars to
    /// `deeper(<anchor>) as <inner>` over `X`.
    Indent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    Literal {
        bytes: Vec<u8>,
        span: Span,
    },
    /// Character class: a set of Unicode scalar values matched against
    /// one UTF-8-decoded code point at the input pointer.
    CharClass {
        set: CharSet,
        span: Span,
    },
    AnyChar {
        span: Span,
    },
    Sequence {
        items: Vec<Pattern>,
        span: Span,
    },
    OrderedChoice {
        alts: Vec<Pattern>,
        span: Span,
    },
    Repeat {
        inner: Box<Pattern>,
        span: Span,
    },
    RepeatOne {
        inner: Box<Pattern>,
        span: Span,
    },
    Optional {
        inner: Box<Pattern>,
        span: Span,
    },
    NotPredicate {
        inner: Box<Pattern>,
        span: Span,
    },
    AndPredicate {
        inner: Box<Pattern>,
        span: Span,
    },
    NonTerminal {
        name: String,
        /// Indentation arguments for a parameterized call (`suite(i)`,
        /// `block(0)`). Empty for an ordinary reference — every call in
        /// every grammar that predates the indentation surface. A
        /// parameterized call is still fundamentally a rule reference, so
        /// the analyses treat it like any other `NonTerminal`; only the
        /// compiler reads `args` (to emit the `ArgPush` run before the
        /// `Call`).
        args: Vec<IndentArg>,
        span: Span,
    },
    Capture {
        kind: String,
        inner: Box<Pattern>,
        span: Span,
    },
    /// Try `inner`; on failure, materialize the failed attempt's
    /// deepest-reach captures (via `RecoverToScopedMax`) and run
    /// `recovery` from that resync point. If `recovery` also fails,
    /// the catch fails to its enclosing backtrack.
    ///
    /// The `label` is mandatory and serves as a diagnostic tag:
    /// `pegdb recoveries explain` clusters firings by it so a grammar
    /// author can see which catch is recovering on which input. It
    /// has no effect on failure propagation — every catch fires on
    /// any anonymous failure of `inner`. Future overlays (`^!label`
    /// throws, `^_` or similar anonymous catch) are reserved
    /// syntactic slots; see `src/pegc/README.md`.
    ///
    /// The `*^` and `*^[charset]` postfix operators desugar to
    /// `Repeat(Catch(inner, "recovery", @recovery <body>))` at parse
    /// time — see `build_recover_repeat` in
    /// [`crate::pegc::parser`]. There is no dedicated `RecoverRepeat`
    /// AST variant any more.
    Catch {
        inner: Box<Pattern>,
        label: String,
        recovery: Box<Pattern>,
        span: Span,
    },
    /// Author-local marker that a pattern is intentionally lenient — the
    /// `lint_partial_match` walker treats this as an opaque barrier and
    /// does not descend into it for call-site detection. At runtime the
    /// wrapper is transparent: the compiler emits exactly the inner
    /// pattern's bytecode. Surface syntax: the `name: partial` ascription.
    Lenient {
        inner: Box<Pattern>,
        span: Span,
    },
    /// Boundary-anchored catch with the boundary inferred from the
    /// call site's FOLLOW set. Placeholder produced at parse time by
    /// the `^^lbl` surface form; resolved before bytecode emission by
    /// `analysis::resolve_inferred_boundaries`, which rewrites it to
    /// the same shape the explicit `^^lbl B` form lowers to (a
    /// `Catch` whose inner is `Sequence([inner, AndPredicate(B)])`
    /// and whose recovery is `@recovery ((!B .)*)`).
    ///
    /// An `InferBoundaryCatch` should never reach the compiler or any
    /// downstream analysis — the resolver runs first and replaces it.
    ///
    /// When `anchor_only` is set (surface `^&lbl`), the resolver lowers
    /// it to a bare `Sequence([inner, AndPredicate(B)])` with **no**
    /// recovery `Catch`: the rule is anchored to its FOLLOW boundary so
    /// a prefix-only match fails and backtracks, but no skip-to-boundary
    /// recovery scan is synthesized. This is the cheap, speculative-safe
    /// anchor used to discharge the leniency lint without the
    /// recovery-on-backtrack cost a `Catch` incurs.
    InferBoundaryCatch {
        inner: Box<Pattern>,
        label: String,
        anchor_only: bool,
        span: Span,
    },
    /// A declarative indentation combinator — `deeper(outer) as i`,
    /// `same(i)`, `at_least(n) as i`. Measures the indentation run at the
    /// current line (which must be a line start; the grammar places these
    /// right after a newline), asserts [`IndentOp`] against `arg`, and
    /// optionally binds the measured width to `bind` so later `same(bind)`
    /// lines in the same suite can re-check alignment.
    ///
    /// The compiler lowers it to `IndentMeasure` + `IndentCmp` over the
    /// current activation's local slots (see `src/pegvm/instruction.rs`).
    /// It is **non-nullable and consuming** (it eats the leading
    /// `[ \t]*`) and contributes **no FIRST/FOLLOW non-terminal**, so the
    /// static analyses treat it as an opaque consuming leaf.
    IndentCombinator {
        op: IndentOp,
        arg: IndentArg,
        bind: Option<String>,
        span: Span,
    },
    /// Transient surface indentation operator — `%root X`, `%align`,
    /// `%indent X`. The parser emits this; `desugar_indent` (run first in
    /// `Grammar::compile`) rewrites it into [`IndentCombinator`] plus the
    /// implicit anchor parameter threaded through [`Pattern::NonTerminal`]
    /// `args`. `%align` carries no operand; `%root` / `%indent` carry
    /// exactly one. Because desugaring happens before every other pass,
    /// all downstream `Pattern` matches treat this variant as
    /// `unreachable!`.
    IndentOp {
        kind: IndentOpKind,
        operand: Option<Box<Pattern>>,
        span: Span,
    },
}

impl Pattern {
    /// Span of this pattern: where the parser entered the construct.
    /// See the per-variant convention documented on [`Span`].
    pub fn span(&self) -> Span {
        match self {
            Pattern::Literal { span, .. }
            | Pattern::CharClass { span, .. }
            | Pattern::AnyChar { span }
            | Pattern::Sequence { span, .. }
            | Pattern::OrderedChoice { span, .. }
            | Pattern::Repeat { span, .. }
            | Pattern::RepeatOne { span, .. }
            | Pattern::Optional { span, .. }
            | Pattern::NotPredicate { span, .. }
            | Pattern::AndPredicate { span, .. }
            | Pattern::NonTerminal { span, .. }
            | Pattern::Capture { span, .. }
            | Pattern::Catch { span, .. }
            | Pattern::Lenient { span, .. }
            | Pattern::InferBoundaryCatch { span, .. }
            | Pattern::IndentCombinator { span, .. }
            | Pattern::IndentOp { span, .. } => *span,
        }
    }

    pub fn literal(s: &str) -> Pattern {
        Pattern::Literal {
            bytes: s.as_bytes().to_vec(),
            span: Span::SYNTHETIC,
        }
    }

    pub fn literal_bytes(bytes: Vec<u8>) -> Pattern {
        Pattern::Literal {
            bytes,
            span: Span::SYNTHETIC,
        }
    }

    pub fn char_class(set: CharSet) -> Pattern {
        Pattern::CharClass {
            set,
            span: Span::SYNTHETIC,
        }
    }

    pub fn any_char() -> Pattern {
        Pattern::AnyChar {
            span: Span::SYNTHETIC,
        }
    }

    /// Span of `Sequence` is the start of its first child — see the
    /// per-variant convention on [`Span`]. Synthetic if `items` is
    /// empty (only happens in tests; the parser rejects empty
    /// sequences).
    pub fn seq(items: Vec<Pattern>) -> Pattern {
        if items.len() == 1 {
            items.into_iter().next().unwrap()
        } else {
            let span = items.first().map(Pattern::span).unwrap_or(Span::SYNTHETIC);
            Pattern::Sequence { items, span }
        }
    }

    /// Span of `OrderedChoice` is the start of its first alternative.
    pub fn choice(items: Vec<Pattern>) -> Pattern {
        if items.len() == 1 {
            items.into_iter().next().unwrap()
        } else {
            let span = items.first().map(Pattern::span).unwrap_or(Span::SYNTHETIC);
            Pattern::OrderedChoice { alts: items, span }
        }
    }

    /// Span of `Repeat` is the start of its operand.
    pub fn repeat(inner: Pattern) -> Pattern {
        let span = inner.span();
        Pattern::Repeat {
            inner: Box::new(inner),
            span,
        }
    }

    /// Span of `RepeatOne` is the start of its operand.
    pub fn repeat_one(inner: Pattern) -> Pattern {
        let span = inner.span();
        Pattern::RepeatOne {
            inner: Box::new(inner),
            span,
        }
    }

    /// Span of `Optional` is the start of its operand.
    pub fn optional(inner: Pattern) -> Pattern {
        let span = inner.span();
        Pattern::Optional {
            inner: Box::new(inner),
            span,
        }
    }

    pub fn not_predicate(inner: Pattern) -> Pattern {
        Pattern::NotPredicate {
            inner: Box::new(inner),
            span: Span::SYNTHETIC,
        }
    }

    pub fn and_predicate(inner: Pattern) -> Pattern {
        Pattern::AndPredicate {
            inner: Box::new(inner),
            span: Span::SYNTHETIC,
        }
    }

    pub fn nt(name: impl Into<String>) -> Pattern {
        Pattern::NonTerminal {
            name: name.into(),
            args: Vec::new(),
            span: Span::SYNTHETIC,
        }
    }

    pub fn capture(kind: impl Into<String>, inner: Pattern) -> Pattern {
        Pattern::Capture {
            kind: kind.into(),
            inner: Box::new(inner),
            span: Span::SYNTHETIC,
        }
    }

    pub fn lenient(inner: Pattern) -> Pattern {
        Pattern::Lenient {
            inner: Box::new(inner),
            span: Span::SYNTHETIC,
        }
    }

    /// Labeled catch — equivalent to `inner ^label recovery` in source.
    /// The label is a diagnostic tag flowed into `RecoveryDiagnostic` so
    /// `pegdb recoveries explain` can cluster firings by it.
    pub fn catch(inner: Pattern, label: impl Into<String>, recovery: Pattern) -> Pattern {
        Pattern::Catch {
            inner: Box::new(inner),
            label: label.into(),
            recovery: Box::new(recovery),
            span: Span::SYNTHETIC,
        }
    }

    /// Return a clone with every node's span replaced by [`Span::SYNTHETIC`].
    /// Test ergonomics: parsed grammars carry real positions, hand-built
    /// fixtures use synthetic spans, and `assert_eq!` would diverge on
    /// the span field even when the structural shape matches. Compare
    /// the stripped form to assert "same shape, ignore positions".
    pub fn strip_spans(&self) -> Pattern {
        match self {
            Pattern::Literal { bytes, .. } => Pattern::Literal {
                bytes: bytes.clone(),
                span: Span::SYNTHETIC,
            },
            Pattern::CharClass { set, .. } => Pattern::CharClass {
                set: set.clone(),
                span: Span::SYNTHETIC,
            },
            Pattern::AnyChar { .. } => Pattern::AnyChar {
                span: Span::SYNTHETIC,
            },
            Pattern::Sequence { items, .. } => Pattern::Sequence {
                items: items.iter().map(Pattern::strip_spans).collect(),
                span: Span::SYNTHETIC,
            },
            Pattern::OrderedChoice { alts, .. } => Pattern::OrderedChoice {
                alts: alts.iter().map(Pattern::strip_spans).collect(),
                span: Span::SYNTHETIC,
            },
            Pattern::Repeat { inner, .. } => Pattern::Repeat {
                inner: Box::new(inner.strip_spans()),
                span: Span::SYNTHETIC,
            },
            Pattern::RepeatOne { inner, .. } => Pattern::RepeatOne {
                inner: Box::new(inner.strip_spans()),
                span: Span::SYNTHETIC,
            },
            Pattern::Optional { inner, .. } => Pattern::Optional {
                inner: Box::new(inner.strip_spans()),
                span: Span::SYNTHETIC,
            },
            Pattern::NotPredicate { inner, .. } => Pattern::NotPredicate {
                inner: Box::new(inner.strip_spans()),
                span: Span::SYNTHETIC,
            },
            Pattern::AndPredicate { inner, .. } => Pattern::AndPredicate {
                inner: Box::new(inner.strip_spans()),
                span: Span::SYNTHETIC,
            },
            Pattern::NonTerminal { name, args, .. } => Pattern::NonTerminal {
                name: name.clone(),
                args: args.clone(),
                span: Span::SYNTHETIC,
            },
            Pattern::Capture { kind, inner, .. } => Pattern::Capture {
                kind: kind.clone(),
                inner: Box::new(inner.strip_spans()),
                span: Span::SYNTHETIC,
            },
            Pattern::Catch {
                inner,
                label,
                recovery,
                ..
            } => Pattern::Catch {
                inner: Box::new(inner.strip_spans()),
                label: label.clone(),
                recovery: Box::new(recovery.strip_spans()),
                span: Span::SYNTHETIC,
            },
            Pattern::Lenient { inner, .. } => Pattern::Lenient {
                inner: Box::new(inner.strip_spans()),
                span: Span::SYNTHETIC,
            },
            Pattern::InferBoundaryCatch {
                inner,
                label,
                anchor_only,
                ..
            } => Pattern::InferBoundaryCatch {
                inner: Box::new(inner.strip_spans()),
                label: label.clone(),
                anchor_only: *anchor_only,
                span: Span::SYNTHETIC,
            },
            Pattern::IndentCombinator { op, arg, bind, .. } => Pattern::IndentCombinator {
                op: *op,
                arg: arg.clone(),
                bind: bind.clone(),
                span: Span::SYNTHETIC,
            },
            Pattern::IndentOp { kind, operand, .. } => Pattern::IndentOp {
                kind: *kind,
                operand: operand.as_ref().map(|p| Box::new(p.strip_spans())),
                span: Span::SYNTHETIC,
            },
        }
    }
}
