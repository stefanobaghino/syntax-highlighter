use super::instruction::CharSet;

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
