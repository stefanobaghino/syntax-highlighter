use std::collections::HashMap;

use super::analysis::{lint_partial_match, resolve_inferred_boundaries};
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
    ///
    /// Pipeline:
    /// 1. Resolve `Pattern::InferBoundaryCatch` placeholders (`^^lbl`)
    ///    via [`resolve_inferred_boundaries`] — synthesizes each
    ///    boundary from FOLLOW.
    /// 2. Run [`lint_partial_match`]; any finding is a fatal
    ///    `CompileError::PartialMatchLeniency`. Anchor real bugs with
    ///    `^^lbl B` (or `^^lbl`); mark intentional leniency with `~`
    ///    at the call site or `~name <- body` at the rule definition.
    /// 3. Emit bytecode via `compile_rules`.
    pub fn compile(&self) -> Result<Program, CompileError> {
        let mut resolved = Grammar {
            rules: self.rules.clone(),
            start: self.start.clone(),
        };
        resolve_inferred_boundaries(&mut resolved)?;
        let findings = lint_partial_match(&resolved);
        if !findings.is_empty() {
            return Err(CompileError::PartialMatchLeniency(findings));
        }
        compile_rules(&resolved.rules, &resolved.start)
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
            // Definition-level lenient marker: `~name <- body` declares
            // that every call to `name` is intentionally lenient and
            // suppresses `lint_partial_match` findings at every call site
            // of `name`. The marker touches the name (no whitespace);
            // the rule's body is wrapped with `Pattern::Lenient` so the
            // lint walker sees the same shape as a per-call-site `~p`.
            let mut definition_lenient = false;
            if self.peek() == Some(b'~') {
                self.pos += 1;
                definition_lenient = true;
            }
            let name = self.parse_ident()?;
            self.skip_ws();
            self.expect_str("<-")?;
            self.skip_ws();
            let mut pat = self.parse_choice()?;
            if definition_lenient {
                pat = Pattern::Lenient(Box::new(pat));
            }
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
        let mut alts = vec![self.parse_catch()?];
        loop {
            self.skip_ws();
            if self.peek() == Some(b'/') {
                self.pos += 1;
                self.skip_ws();
                alts.push(self.parse_catch()?);
            } else {
                break;
            }
        }
        Ok(Pattern::choice(alts))
    }

    /// Two operators share the catch position, distinguished by a
    /// single byte after the first `^`:
    ///
    /// - `inner ^label recovery` — **bare catch.** Author writes the
    ///   recovery branch. Left-associative. See `src/pegc/README.md`.
    /// - `inner ^^label B` / `inner ^^label` — **boundary-anchored
    ///   catch.** The doubled caret marks a new operator family: the
    ///   recovery branch is auto-synthesized as `@recovery{(!B .)*}`,
    ///   and the inner is implicitly anchored with `&B`. The boundary
    ///   `B` is parsed as one atom-with-prefix-postfix; if absent
    ///   (no atom-start follows the label), the FOLLOW-inferred form
    ///   emits `Pattern::InferBoundaryCatch { inner, label }` and the
    ///   compile-time resolver synthesizes `B` from the call site's
    ///   FOLLOW set.
    ///
    /// The label identifier must touch the discriminator `^` (bare)
    /// or the second `^` of `^^` (anchored). `^_` / `^^_` are
    /// reserved for anonymous-catch; `^!lbl` / `^^!lbl` are reserved
    /// for the throw-atom slot.
    ///
    /// No collision with the `*^` / `+^` postfixes — those only fire
    /// when `^` directly follows `*` / `+` with no whitespace; an `^`
    /// at infix position is always reached after `parse_postfix`
    /// returns.
    fn parse_catch(&mut self) -> Result<Pattern, ParseError> {
        let mut lhs = self.parse_sequence()?;
        loop {
            self.skip_ws();
            if self.peek() != Some(b'^') {
                break;
            }
            self.pos += 1;
            // Doubled-caret discriminator: a second `^` immediately
            // after the first marks the boundary-anchored family.
            if self.peek() == Some(b'^') {
                self.pos += 1;
                let label = self.parse_catch_label("^^")?;
                self.skip_ws();
                let catch_pat = if self.at_prefix_start() {
                    let boundary = self.parse_prefix()?;
                    lower_boundary_catch(lhs, label, boundary)
                } else {
                    Pattern::InferBoundaryCatch {
                        inner: Box::new(lhs),
                        label,
                    }
                };
                // Trailing sequence atoms after `^^lbl B` (or
                // `^^lbl` inferred) belong to the enclosing sequence,
                // not to the catch. `parse_sequence` above already
                // returned when it hit the first `^`, so absorb the
                // continuation here and rejoin under a Sequence.
                let mut items = vec![catch_pat];
                loop {
                    self.skip_ws();
                    if !self.at_prefix_start() {
                        break;
                    }
                    items.push(self.parse_prefix()?);
                }
                lhs = Pattern::seq(items);
            } else {
                let label = self.parse_catch_label("^")?;
                self.skip_ws();
                let rhs = self.parse_sequence()?;
                lhs = Pattern::Catch {
                    inner: Box::new(lhs),
                    label,
                    recovery: Box::new(rhs),
                };
            }
        }
        Ok(lhs)
    }

    /// Parse a catch label that must touch its sigil prefix
    /// (no whitespace). `sigil` is the literal `^` or `^^` for the
    /// error message. Reuses the existing `_`-reserved rule and the
    /// `^!`-reserved-throw-atom rule.
    fn parse_catch_label(&mut self, sigil: &str) -> Result<String, ParseError> {
        let label = self.parse_ident().map_err(|_| {
            self.err(format!(
                "expected label identifier immediately after `{sigil}` (no whitespace); \
                 anonymous (`{sigil}_`) and throw (`{sigil}!lbl`) forms are reserved for future use"
            ))
        })?;
        if label == "_" {
            return Err(
                self.err("label name `_` is reserved for future use (anonymous catch)".into())
            );
        }
        Ok(label)
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
            // `^` is the catch operator at infix position — leave it
            // for `parse_catch`. Future overlays (e.g. `^!lbl` throw
            // atom, `^_` anonymous catch) live in the reserved
            // `^<non-ident-byte>` slot; when added, this function
            // grows a one-byte lookahead.
            Some(b'^') => false,
            // `~name <- body` is a definition-level lenient marker for
            // the NEXT rule, not a call-site `~p` in the current body.
            // Look past the `~ ident` to see if `<-` follows.
            Some(b'~') if self.looks_like_lenient_rule_def() => false,
            Some(c) if is_ident_start(c) => !self.looks_like_rule_def(),
            Some(c) => is_atom_start(c),
        }
    }

    /// Look past a leading `~` to see if it starts a `~ident <-` rule
    /// definition. Used by `at_prefix_start` to keep `~` at the
    /// boundary between rules from being consumed as a call-site
    /// lenient marker for the previous rule's body.
    fn looks_like_lenient_rule_def(&self) -> bool {
        let mut p = self.pos;
        if p >= self.src.len() || self.src[p] != b'~' {
            return false;
        }
        p += 1;
        if p >= self.src.len() || !is_ident_start(self.src[p]) {
            return false;
        }
        p += 1;
        while p < self.src.len() && is_ident_cont(self.src[p]) {
            p += 1;
        }
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
                // Recurse into `parse_prefix` so chained prefixes
                // compose: `!~p` parses as `NotPredicate(Lenient(p))`.
                // Existing `!atom` shapes are unchanged.
                let inner = self.parse_prefix_or_postfix_after_predicate()?;
                Ok(Pattern::NotPredicate(Box::new(inner)))
            }
            Some(b'&') => {
                self.pos += 1;
                let inner = self.parse_prefix_or_postfix_after_predicate()?;
                Ok(Pattern::AndPredicate(Box::new(inner)))
            }
            // `~p` — intentional-leniency marker. `~` must touch the
            // atom (no whitespace); the wrapper is opaque to the lint
            // and transparent at runtime. See `src/pegc/README.md`.
            Some(b'~') => {
                self.pos += 1;
                if matches!(
                    self.peek(),
                    Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r')
                ) {
                    return Err(
                        self.err("expected pattern immediately after `~` (no whitespace)".into())
                    );
                }
                let inner = self.parse_postfix()?;
                Ok(Pattern::Lenient(Box::new(inner)))
            }
            _ => self.parse_postfix(),
        }
    }

    /// After `!` / `&` consume their sigil, allow either a single
    /// `~p` lenient marker or a plain `parse_postfix`. Skipping
    /// whitespace before the inner is allowed (existing behavior);
    /// `~` itself enforces its own touching-atom rule.
    fn parse_prefix_or_postfix_after_predicate(&mut self) -> Result<Pattern, ParseError> {
        self.skip_ws();
        if self.peek() == Some(b'~') {
            self.pos += 1;
            if matches!(
                self.peek(),
                Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r')
            ) {
                return Err(
                    self.err("expected pattern immediately after `~` (no whitespace)".into())
                );
            }
            let inner = self.parse_postfix()?;
            Ok(Pattern::Lenient(Box::new(inner)))
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&mut self) -> Result<Pattern, ParseError> {
        let mut atom = self.parse_atom()?;
        loop {
            match self.peek() {
                Some(b'*') => {
                    self.pos += 1;
                    if self.peek() == Some(b'^') {
                        self.pos += 1;
                        let charset = self.parse_optional_sync_set()?;
                        let label = self.parse_optional_recovery_label()?;
                        atom = build_recover_repeat(atom, charset, label);
                    } else {
                        atom = Pattern::Repeat(Box::new(atom));
                    }
                }
                Some(b'+') => {
                    self.pos += 1;
                    if self.peek() == Some(b'^') {
                        self.pos += 1;
                        let charset = self.parse_optional_sync_set()?;
                        let label = self.parse_optional_recovery_label()?;
                        // p+^  ≡  p (p*^)  — at least one inner success required.
                        let head = atom.clone();
                        atom = Pattern::seq(vec![head, build_recover_repeat(atom, charset, label)]);
                    } else {
                        atom = Pattern::RepeatOne(Box::new(atom));
                    }
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

    /// If the byte following `*^` / `+^` is `[`, parse a delimiter set
    /// (the sync set) using the same charclass syntax as a normal
    /// `[abc]` atom. The bracket must touch `^` — `^ [...]` parses as
    /// `^` then a separate atom and is rejected by the surrounding
    /// sequence rules. Returns the parsed `CharSet`, or `None` if the
    /// next byte is not `[`.
    fn parse_optional_sync_set(&mut self) -> Result<Option<CharSet>, ParseError> {
        if self.peek() != Some(b'[') {
            return Ok(None);
        }
        match self.parse_charclass()? {
            Pattern::CharClass(set) => Ok(Some(set)),
            _ => unreachable!("parse_charclass always returns Pattern::CharClass"),
        }
    }

    /// If the byte following `*^` / `*^[cs]` / `+^` / `+^[cs]` is `:`,
    /// parse an identifier touching the colon and use it as the catch
    /// label. The `:` must touch the preceding `^` or `]` (no whitespace),
    /// and the identifier must touch `:` — same tight-binding rule as
    /// `parse_catch`'s `^label`. Returns `None` when the next byte isn't
    /// `:`, in which case the caller falls back to the default
    /// `"recovery"` label. Bare `_` is rejected as reserved for future
    /// anonymous-catch syntax, mirroring `parse_catch`.
    fn parse_optional_recovery_label(&mut self) -> Result<Option<String>, ParseError> {
        if self.peek() != Some(b':') {
            return Ok(None);
        }
        self.pos += 1;
        let label = self.parse_ident().map_err(|_| {
            self.err("expected label identifier immediately after `:` (no whitespace)".into())
        })?;
        if label == "_" {
            return Err(
                self.err("label name `_` is reserved for future use (anonymous catch)".into())
            );
        }
        Ok(Some(label))
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
    matches!(
        c,
        b'(' | b'"' | b'\'' | b'[' | b'.' | b'@' | b'!' | b'&' | b'~'
    ) || is_ident_start(c)
}

/// Lower `INNER ^^lbl B` to the explicit boundary-anchored catch
/// shape: `Catch { Sequence([INNER, &B]), lbl, @recovery{(!B .)*} }`.
/// Parse-time lowering — the compiler sees this as a regular `Catch`
/// with an `AndPredicate` sibling, so the existing lint walker
/// recognizes it as anchored without a new arm.
fn lower_boundary_catch(inner: Pattern, label: String, boundary: Pattern) -> Pattern {
    let lookahead = Pattern::AndPredicate(Box::new(boundary.clone()));
    let stop_loop = Pattern::Repeat(Box::new(Pattern::Sequence(vec![
        Pattern::NotPredicate(Box::new(boundary)),
        Pattern::AnyChar,
    ])));
    let recovery = Pattern::Capture("recovery".into(), Box::new(stop_loop));
    Pattern::Catch {
        inner: Box::new(Pattern::Sequence(vec![inner, lookahead])),
        label,
        recovery: Box::new(recovery),
    }
}

/// Desugar `p*^` and `p*^[cs]` to a labeled-catch loop. Both forms are
/// `(p ^<label> @recovery{<body>})*` where `<body>` is `.` for the
/// plain form and `(!cs .)* cs` for the sync-set form. The capture
/// name is always `"recovery"`; the label is the author-supplied
/// `:lbl` suffix when present and falls back to the literal
/// `"recovery"` otherwise.
///
/// One implementation, two surface forms — see `parse_postfix`. The
/// EOF clean-exit behavior of the old `RecoverRepeat` opcode shape
/// emerges naturally here: when the recovery body fails (e.g. `.` at
/// EOF, or `cs` missing before EOF), the failure propagates to the
/// enclosing `*`'s Backtrack frame and the loop terminates without
/// consuming further input.
fn build_recover_repeat(
    inner: Pattern,
    charset: Option<CharSet>,
    label: Option<String>,
) -> Pattern {
    let body = match charset {
        None => Pattern::AnyChar,
        Some(cs) => Pattern::Sequence(vec![
            Pattern::Repeat(Box::new(Pattern::Sequence(vec![
                Pattern::NotPredicate(Box::new(Pattern::CharClass(cs))),
                Pattern::AnyChar,
            ]))),
            Pattern::CharClass(cs),
        ]),
    };
    let recovery = Pattern::Capture("recovery".into(), Box::new(body));
    Pattern::Repeat(Box::new(Pattern::Catch {
        inner: Box::new(inner),
        label: label.unwrap_or_else(|| "recovery".into()),
        recovery: Box::new(recovery),
    }))
}
