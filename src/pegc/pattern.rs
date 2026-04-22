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
}
