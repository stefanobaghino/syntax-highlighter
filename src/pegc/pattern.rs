use crate::pegvm::CharSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    Literal(Vec<u8>),
    CharClass(CharSet),
    AnyChar,
    Sequence(Vec<Pattern>),
    OrderedChoice(Vec<Pattern>),
    Repeat(Box<Pattern>),
    RepeatOne(Box<Pattern>),
    Optional(Box<Pattern>),
    NotPredicate(Box<Pattern>),
    AndPredicate(Box<Pattern>),
    NonTerminal(String),
    Capture(String, Box<Pattern>),
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
    /// `Repeat(Catch(inner, "recovery", @recovery{<body>}))` at parse
    /// time — see `build_recover_repeat` in
    /// [`crate::pegc::parser`]. There is no dedicated `RecoverRepeat`
    /// AST variant any more.
    Catch {
        inner: Box<Pattern>,
        label: String,
        recovery: Box<Pattern>,
    },
    /// Author-local marker that a pattern is intentionally lenient — the
    /// `lint_partial_match` walker treats this as an opaque barrier and
    /// does not descend into it for call-site detection. At runtime the
    /// wrapper is transparent: the compiler emits exactly the inner
    /// pattern's bytecode. Surface syntax: postfix `~p`.
    Lenient(Box<Pattern>),
    /// Boundary-anchored catch with the boundary inferred from the
    /// call site's FOLLOW set. Placeholder produced at parse time by
    /// the `^^lbl` surface form; resolved before bytecode emission by
    /// `analysis::resolve_inferred_boundaries`, which rewrites it to
    /// the same shape the explicit `^^lbl B` form lowers to (a
    /// `Catch` whose inner is `Sequence([inner, AndPredicate(B)])`
    /// and whose recovery is `@recovery{(!B .)*}`).
    ///
    /// An `InferBoundaryCatch` should never reach the compiler or any
    /// downstream analysis — the resolver runs first and replaces it.
    InferBoundaryCatch {
        inner: Box<Pattern>,
        label: String,
    },
}

impl Pattern {
    pub fn literal(s: &str) -> Pattern {
        Pattern::Literal(s.as_bytes().to_vec())
    }

    pub fn seq(items: Vec<Pattern>) -> Pattern {
        if items.len() == 1 {
            items.into_iter().next().unwrap()
        } else {
            Pattern::Sequence(items)
        }
    }

    pub fn choice(items: Vec<Pattern>) -> Pattern {
        if items.len() == 1 {
            items.into_iter().next().unwrap()
        } else {
            Pattern::OrderedChoice(items)
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
        }
    }
}
