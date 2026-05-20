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
    /// Like `Repeat`, but resync past failures inside the loop by
    /// consuming one byte under a capture tagged with `recovery_kind`
    /// and retrying. At end of input the loop terminates cleanly.
    ///
    /// Inherits the empty-match livelock of `Repeat`: if `inner` ever
    /// matches the empty string successfully, the enclosing loop spins
    /// forever — same hazard as plain `p*`.
    RecoverRepeat {
        inner: Box<Pattern>,
        recovery_kind: String,
    },
    /// Try `inner`; on failure, materialize the failed attempt's
    /// deepest-reach captures (via `RecoverToScopedMax`) and run
    /// `recovery` from that resync point. Reuses the `RecoverScope`
    /// machinery from #16, minus the loop wrapper of `RecoverRepeat`.
    /// If `recovery` also fails, the catch fails to its enclosing
    /// backtrack.
    ///
    /// The `label` is mandatory and serves as a diagnostic tag:
    /// `pegdb explain-recoveries` clusters firings by it so a grammar
    /// author can see which catch is recovering on which input. It
    /// has no effect on failure propagation — every catch fires on
    /// any anonymous failure of `inner`. Future overlays (`^!label`
    /// throws, `^_` or similar anonymous catch) are reserved
    /// syntactic slots; see `src/pegc/README.md`.
    Catch {
        inner: Box<Pattern>,
        label: String,
        recovery: Box<Pattern>,
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
    /// `pegdb explain-recoveries` can cluster firings by it.
    pub fn catch(inner: Pattern, label: impl Into<String>, recovery: Pattern) -> Pattern {
        Pattern::Catch {
            inner: Box::new(inner),
            label: label.into(),
            recovery: Box::new(recovery),
        }
    }
}
