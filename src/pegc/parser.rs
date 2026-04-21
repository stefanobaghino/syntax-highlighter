use std::collections::HashMap;

use super::compiler::{compile_rules, CompileError};
use super::pattern::Pattern;
use crate::pegvm::{CharSet, Program};

#[derive(Debug, Clone)]
pub struct Grammar {
    pub rules: HashMap<String, Pattern>,
    pub start: String,
}

impl Grammar {
    /// Construct a grammar from a hand-built rule map. Useful for
    /// tests that build `Pattern` trees directly without going
    /// through [`parse`].
    pub fn new(rules: HashMap<String, Pattern>, start: impl Into<String>) -> Self {
        Grammar {
            rules,
            start: start.into(),
        }
    }

    /// Compile this grammar's rules into a runnable
    /// [`Program`](crate::pegvm::Program). For one-shot grammar source
    /// → [`Program`] callers, prefer the top-level
    /// [`compile`](super::compile) instead.
    pub fn compile(&self) -> Result<Program, CompileError> {
        compile_rules(&self.rules, &self.start)
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "grammar parse error at {}:{}: {}",
            self.line, self.col, self.message
        )
    }
}

impl std::error::Error for ParseError {}

pub fn parse(input: &str) -> Result<Grammar, ParseError> {
    let mut p = Parser::new(input);
    p.parse_grammar()
}

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser {
            src: input.as_bytes(),
            pos: 0,
        }
    }

    fn parse_grammar(&mut self) -> Result<Grammar, ParseError> {
        self.skip_ws();
        let mut rules = HashMap::new();
        let mut order = Vec::new();
        while self.pos < self.src.len() {
            let name = self.parse_ident()?;
            self.skip_ws();
            self.expect_str("<-")?;
            self.skip_ws();
            let pat = self.parse_choice()?;
            if rules.insert(name.clone(), pat).is_some() {
                return Err(self.err(format!("rule '{}' defined twice", name)));
            }
            order.push(name);
            self.skip_ws();
        }
        if order.is_empty() {
            return Err(self.err("grammar has no rules".into()));
        }
        let start = order[0].clone();
        Ok(Grammar { rules, start })
    }

    fn parse_choice(&mut self) -> Result<Pattern, ParseError> {
        let mut alts = vec![self.parse_sequence()?];
        loop {
            self.skip_ws();
            if self.peek() == Some(b'/') {
                self.pos += 1;
                self.skip_ws();
                alts.push(self.parse_sequence()?);
            } else {
                break;
            }
        }
        Ok(Pattern::choice(alts))
    }

    fn parse_sequence(&mut self) -> Result<Pattern, ParseError> {
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if !self.at_prefix_start() {
                break;
            }
            items.push(self.parse_prefix()?);
        }
        if items.is_empty() {
            return Err(self.err("expected pattern".into()));
        }
        Ok(Pattern::seq(items))
    }

    fn at_prefix_start(&self) -> bool {
        match self.peek() {
            None => false,
            Some(b'/') | Some(b')') | Some(b'}') => false,
            Some(b'<') => false, // start of next rule's "<-"
            Some(c) if is_ident_start(c) => !self.looks_like_rule_def(),
            Some(c) => is_atom_start(c),
        }
    }

    /// Look ahead from the current position to determine whether what follows is
    /// `ident <- ...` (a new rule definition) rather than a non-terminal reference
    /// in an expression. Does not consume input.
    fn looks_like_rule_def(&self) -> bool {
        let mut p = self.pos;
        // Skip identifier
        if p >= self.src.len() || !is_ident_start(self.src[p]) {
            return false;
        }
        p += 1;
        while p < self.src.len() && is_ident_cont(self.src[p]) {
            p += 1;
        }
        // Skip horizontal whitespace and comments and newlines
        loop {
            while p < self.src.len() && matches!(self.src[p], b' ' | b'\t' | b'\n' | b'\r') {
                p += 1;
            }
            if p < self.src.len() && self.src[p] == b'#' {
                while p < self.src.len() && self.src[p] != b'\n' {
                    p += 1;
                }
                continue;
            }
            break;
        }
        p + 1 < self.src.len() && &self.src[p..p + 2] == b"<-"
    }

    fn parse_prefix(&mut self) -> Result<Pattern, ParseError> {
        match self.peek() {
            Some(b'!') => {
                self.pos += 1;
                self.skip_ws();
                let inner = self.parse_postfix()?;
                Ok(Pattern::NotPredicate(Box::new(inner)))
            }
            Some(b'&') => {
                self.pos += 1;
                self.skip_ws();
                let inner = self.parse_postfix()?;
                Ok(Pattern::AndPredicate(Box::new(inner)))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Pattern, ParseError> {
        let mut atom = self.parse_atom()?;
        loop {
            match self.peek() {
                Some(b'*') => {
                    self.pos += 1;
                    atom = Pattern::Repeat(Box::new(atom));
                }
                Some(b'+') => {
                    self.pos += 1;
                    atom = Pattern::RepeatOne(Box::new(atom));
                }
                Some(b'?') => {
                    self.pos += 1;
                    atom = Pattern::Optional(Box::new(atom));
                }
                _ => break,
            }
        }
        Ok(atom)
    }

    fn parse_atom(&mut self) -> Result<Pattern, ParseError> {
        match self.peek() {
            Some(b'(') => {
                self.pos += 1;
                self.skip_ws();
                let inner = self.parse_choice()?;
                self.skip_ws();
                self.expect(b')')?;
                Ok(inner)
            }
            Some(b'"') | Some(b'\'') => self.parse_string(),
            Some(b'[') => self.parse_charclass(),
            Some(b'.') => {
                self.pos += 1;
                Ok(Pattern::AnyChar)
            }
            Some(b'@') => self.parse_capture(),
            Some(c) if is_ident_start(c) => {
                // Disambiguation against `name <- ...` is handled by at_prefix_start.
                let name = self.parse_ident()?;
                Ok(Pattern::NonTerminal(name))
            }
            Some(c) => Err(self.err(format!("unexpected '{}'", c as char))),
            None => Err(self.err("unexpected end of input".into())),
        }
    }

    fn parse_capture(&mut self) -> Result<Pattern, ParseError> {
        self.expect(b'@')?;
        let name = self.parse_ident()?;
        self.expect(b'{')?;
        self.skip_ws();
        let inner = self.parse_choice()?;
        self.skip_ws();
        self.expect(b'}')?;
        Ok(Pattern::Capture(name, Box::new(inner)))
    }

    fn parse_string(&mut self) -> Result<Pattern, ParseError> {
        let quote = self.peek().unwrap();
        self.pos += 1;
        let mut bytes = Vec::new();
        loop {
            match self.peek() {
                None => return Err(self.err("unterminated string literal".into())),
                Some(b'\\') => {
                    self.pos += 1;
                    bytes.push(self.parse_escape()?);
                }
                Some(c) if c == quote => {
                    self.pos += 1;
                    return Ok(Pattern::Literal(bytes));
                }
                Some(c) => {
                    bytes.push(c);
                    self.pos += 1;
                }
            }
        }
    }

    fn parse_charclass(&mut self) -> Result<Pattern, ParseError> {
        self.expect(b'[')?;
        let negate = if self.peek() == Some(b'^') {
            self.pos += 1;
            true
        } else {
            false
        };
        let mut set = CharSet::empty();
        while self.peek().is_some() && self.peek() != Some(b']') {
            let lo = self.parse_class_char()?;
            // Range form `lo-hi` (but `-` at end is literal; require non-`]` after `-`)
            if self.peek() == Some(b'-') && self.peek_at(1) != Some(b']') {
                self.pos += 1;
                let hi = self.parse_class_char()?;
                if hi < lo {
                    return Err(self.err(format!(
                        "char class range out of order: 0x{:02x}-0x{:02x}",
                        lo, hi
                    )));
                }
                set.add_range(lo, hi);
            } else {
                set.add(lo);
            }
        }
        self.expect(b']')?;
        let final_set = if negate { set.negate() } else { set };
        Ok(Pattern::CharClass(final_set))
    }

    fn parse_class_char(&mut self) -> Result<u8, ParseError> {
        match self.peek() {
            Some(b'\\') => {
                self.pos += 1;
                self.parse_escape()
            }
            Some(c) => {
                self.pos += 1;
                Ok(c)
            }
            None => Err(self.err("unterminated character class".into())),
        }
    }

    fn parse_escape(&mut self) -> Result<u8, ParseError> {
        match self.peek() {
            Some(b'n') => {
                self.pos += 1;
                Ok(b'\n')
            }
            Some(b'r') => {
                self.pos += 1;
                Ok(b'\r')
            }
            Some(b't') => {
                self.pos += 1;
                Ok(b'\t')
            }
            Some(b'0') => {
                self.pos += 1;
                Ok(0)
            }
            Some(b'\\') => {
                self.pos += 1;
                Ok(b'\\')
            }
            Some(b'\'') => {
                self.pos += 1;
                Ok(b'\'')
            }
            Some(b'"') => {
                self.pos += 1;
                Ok(b'"')
            }
            Some(b']') => {
                self.pos += 1;
                Ok(b']')
            }
            Some(b'[') => {
                self.pos += 1;
                Ok(b'[')
            }
            Some(b'-') => {
                self.pos += 1;
                Ok(b'-')
            }
            Some(b'/') => {
                self.pos += 1;
                Ok(b'/')
            }
            Some(c) => Err(self.err(format!("unknown escape '\\{}'", c as char))),
            None => Err(self.err("unterminated escape sequence".into())),
        }
    }

    fn parse_ident(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        match self.peek() {
            Some(c) if is_ident_start(c) => self.pos += 1,
            _ => return Err(self.err("expected identifier".into())),
        }
        while let Some(c) = self.peek() {
            if is_ident_cont(c) {
                self.pos += 1;
            } else {
                break;
            }
        }
        Ok(std::str::from_utf8(&self.src[start..self.pos])
            .expect("identifier is ASCII")
            .to_string())
    }

    fn skip_ws(&mut self) {
        loop {
            match self.peek() {
                Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') => self.pos += 1,
                Some(b'#') => {
                    while let Some(c) = self.peek() {
                        self.pos += 1;
                        if c == b'\n' {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        self.src.get(self.pos + offset).copied()
    }

    fn expect(&mut self, c: u8) -> Result<(), ParseError> {
        if self.peek() == Some(c) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.err(format!("expected '{}'", c as char)))
        }
    }

    fn expect_str(&mut self, s: &str) -> Result<(), ParseError> {
        let bytes = s.as_bytes();
        if self.pos + bytes.len() <= self.src.len()
            && &self.src[self.pos..self.pos + bytes.len()] == bytes
        {
            self.pos += bytes.len();
            Ok(())
        } else {
            Err(self.err(format!("expected '{}'", s)))
        }
    }

    fn err(&self, message: String) -> ParseError {
        let (line, col) = self.line_col();
        ParseError { message, line, col }
    }

    fn line_col(&self) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for &b in &self.src[..self.pos.min(self.src.len())] {
            if b == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }
}

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

fn is_ident_cont(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

fn is_atom_start(c: u8) -> bool {
    matches!(c, b'(' | b'"' | b'\'' | b'[' | b'.' | b'@' | b'!' | b'&') || is_ident_start(c)
}
