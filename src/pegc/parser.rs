use std::collections::{HashMap, HashSet};

use super::analysis::{
    append_wb_check, inject_auto_trivia, lint_partial_match, resolve_inferred_boundaries,
    synthesize_reserved_preferred, wrap_root,
};
use super::compiler::{compile_rules, CompileError};
use super::pattern::{Pattern, Span};
use crate::pegvm::{CharSet, Program};

/// Append `c` as UTF-8 to `out`. ASCII chars produce one byte; non-ASCII
/// up to four. Used by the parser to fold every form of escape (byte
/// escapes like `\n`, the code-point escape `\u{HHHH}`, and bare source
/// chars) through one path into the literal byte buffer.
fn append_char_utf8(c: char, out: &mut Vec<u8>) {
    let mut buf = [0u8; 4];
    out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
}

/// Fold a single code point into the [`CharSet`] used by the `i"…"`
/// desugar. ASCII letters expand to the two-element set containing both
/// cases; every other code point becomes a singleton. Non-ASCII letters
/// are matched literally — the grammars we ship only need ASCII case
/// folding, and Unicode case mapping would pull in a full mapping table
/// for no current caller.
fn case_fold_codepoint(c: char) -> CharSet {
    if c.is_ascii_alphabetic() {
        CharSet::from_chars(&[c.to_ascii_lowercase(), c.to_ascii_uppercase()])
    } else {
        CharSet::singleton(c)
    }
}

/// Reserved rule name for the grammar's start rule. Every grammar must
/// define exactly one rule named [`ROOT_RULE`]; the compiler wraps its
/// body to assert end-of-input and to splice optional leading / trailing
/// `trivia` calls when an auto-insertion target exists.
pub const ROOT_RULE: &str = "root";

/// Reserved rule name for the (optional) auto-insertion target. When a
/// grammar defines a rule named [`TRIVIA_RULE`], the compiler injects a
/// call to it between consecutive `Sequence` items in every non-atomic
/// rule's body. The rule carries no qualifiers; auto-insertion is
/// disabled grammar-wide by omitting `trivia`, not by marking it.
pub const TRIVIA_RULE: &str = "trivia";

/// Reserved rule name for the (optional) word-boundary target. A rule
/// marked with the `%name <- …` sigil is compiled atomic and has a
/// trailing call to [`WB_RULE`] appended inside its terminal captures by
/// [`append_wb_check`](super::analysis::append_wb_check), so a keyword
/// can't fire on the prefix of a longer identifier. Defining `wb` is
/// only required when at least one `%` rule exists; like [`TRIVIA_RULE`]
/// it must sit in the reserved slots immediately after [`ROOT_RULE`].
pub const WB_RULE: &str = "wb";

/// Compiler-generated rule name for the reserved-word set. Synthesized
/// by [`synthesize_reserved_preferred`](super::analysis::synthesize_reserved_preferred)
/// from the literals of every `%` rule minus every `%?` rule, so a
/// grammar can write `!reserved` to bar keywords from identifier
/// position without hand-maintaining the list. Authors may *reference*
/// it but not *define* it — the parser rejects a definition.
pub const RESERVED_RULE: &str = "reserved";

/// Compiler-generated rule name for the preferred-word set: the union
/// of every `%?` rule's literals (identifier-eligible distinguished
/// tokens, e.g. Go predeclared `int` / `len`). Synthesized alongside
/// [`RESERVED_RULE`]; like it, authors may reference but not define it.
pub const PREFERRED_RULE: &str = "preferred";

/// Cap for the `p{n}` exact-count quantifier. Conservative typo guard:
/// the largest plausible corpus site is `hex{8}`. Above this the parser
/// rejects with a clear error instead of expanding a malformed grammar
/// into a pathologically large bytecode block.
const MAX_REPEAT_COUNT: usize = 1024;

#[derive(Debug, Clone)]
pub struct Grammar {
    pub rules: HashMap<String, Pattern>,
    /// Names of rules marked atomic with the `*name <- body` sigil. The
    /// auto-insertion rewriter skips these rules: `Sequence` items inside
    /// an atomic body don't get a synthesized `trivia` call spliced
    /// between them. The `trivia` rule itself never appears here — it
    /// carries no qualifiers; auto-insertion is disabled by omitting
    /// the rule, not by marking it.
    pub atomic_rules: HashSet<String>,
    /// Names of rules marked with the `%name <- body` reserved-word
    /// sigil (and the `%?name <- body` preferred-word sigil, which is a
    /// subset — every `%?` rule is also here so it still gets a
    /// boundary). Each such rule is also in `atomic_rules` (so trivia is
    /// not auto-inserted inside it); membership here additionally drives
    /// [`append_wb_check`](super::analysis::append_wb_check), which
    /// appends a [`WB_RULE`] call inside the rule's terminal captures.
    pub percent_rules: HashSet<String>,
    /// Names of rules marked with the `%?name <- body` preferred-word
    /// sigil — identifier-eligible distinguished tokens (Go predeclared
    /// `int` / `len`, JS contextual `async` / `let`). A strict subset of
    /// `percent_rules`: a `%?` rule still gets the [`WB_RULE`] boundary,
    /// but its literals are *excluded* from the synthesized
    /// [`RESERVED_RULE`] and instead collected into [`PREFERRED_RULE`] by
    /// [`synthesize_reserved_preferred`](super::analysis::synthesize_reserved_preferred).
    pub preferred_rules: HashSet<String>,
    /// Per-rule source ranges, in declaration order. Populated by
    /// [`parse`]; empty for grammars built via [`Grammar::new`] (which
    /// has no source text). `pegc stats` uses these to slice the
    /// grammar source for the `body_chars` field; the reserved-name
    /// position check in [`Grammar::compile`] also walks the names in
    /// order to verify `root` is the first rule.
    pub rule_headers: Vec<RuleHeader>,
}

/// Byte ranges of one rule's pieces in the grammar source. All offsets
/// are into the same source string passed to [`parse`].
#[derive(Debug, Clone)]
pub struct RuleHeader {
    pub name: String,
    /// Byte offset of the rule's identifier (after any leading `~` / `*`).
    pub name_byte: usize,
    /// Byte offset of the first non-whitespace byte after `<-`.
    pub body_byte_start: usize,
    /// Byte offset just past the last byte consumed for the body.
    pub body_byte_end: usize,
}

impl Grammar {
    /// Construct a grammar from a hand-built rule map. Useful for
    /// tests that build `Pattern` trees directly without going
    /// through [`parse`].
    ///
    /// The start rule is always [`ROOT_RULE`]; the rule map must
    /// contain a `"root"` entry. No rules are atomic; `rule_headers`
    /// is left empty (hand-built grammars have no source text to
    /// range over).
    pub fn new(rules: HashMap<String, Pattern>) -> Self {
        Grammar {
            rules,
            atomic_rules: HashSet::new(),
            percent_rules: HashSet::new(),
            preferred_rules: HashSet::new(),
            rule_headers: Vec::new(),
        }
    }

    /// Construct a grammar with a pre-built atomic-rule set. Test-only
    /// shortcut for exercising the `*name` sigil's behavior without
    /// routing through grammar source.
    pub fn with_atomic(rules: HashMap<String, Pattern>, atomic_rules: HashSet<String>) -> Self {
        Grammar {
            rules,
            atomic_rules,
            percent_rules: HashSet::new(),
            preferred_rules: HashSet::new(),
            rule_headers: Vec::new(),
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
    /// 3. Inject auto-trivia calls into every non-atomic rule's body
    ///    (no-op when the grammar has no `trivia` rule).
    /// 4. Wrap `root`'s body with `trivia? body trivia? !.` (the
    ///    `trivia?` calls are present iff auto-insertion is active).
    /// 5. Emit bytecode via `compile_rules`.
    ///
    /// The lint and FOLLOW-driven boundary resolver see the author's
    /// original body — auto-insertion and the `root` wrap run after
    /// both, so synthesized `trivia` calls and the EOF assertion can't
    /// confuse the lint walker or the FOLLOW set.
    pub fn compile(&self) -> Result<Program, CompileError> {
        if !self.rules.contains_key(ROOT_RULE) {
            return Err(CompileError::MissingRootRule);
        }
        // When the grammar was parsed from source, enforce that `root`
        // is the first rule and `trivia` — when present — is the
        // second. Hand-built grammars from `Grammar::new` skip this —
        // `rule_headers` is empty for them.
        if !self.rule_headers.is_empty() {
            let root_pos = self
                .rule_headers
                .iter()
                .position(|h| h.name == ROOT_RULE)
                .expect("ROOT_RULE presence checked above");
            if root_pos != 0 {
                return Err(CompileError::RootRulePosition {
                    expected_pos: 0,
                    has_trivia: self.rules.contains_key(TRIVIA_RULE),
                });
            }
        }
        let mut resolved = Grammar {
            rules: self.rules.clone(),
            atomic_rules: self.atomic_rules.clone(),
            percent_rules: self.percent_rules.clone(),
            preferred_rules: self.preferred_rules.clone(),
            rule_headers: self.rule_headers.clone(),
        };
        resolve_inferred_boundaries(&mut resolved)?;
        let findings = lint_partial_match(&resolved);
        if !findings.is_empty() {
            return Err(CompileError::PartialMatchLeniency(findings));
        }
        inject_auto_trivia(&mut resolved);
        // Synthesize the `reserved` / `preferred` rules from the literals
        // of the `%` / `%?` rules. Runs after trivia injection (so the
        // synthesized atomic rules aren't walked) and before
        // `append_wb_check` (so the synthesized rules pick up the trie +
        // `wb` like any other `%` rule, and exist before `compile_rules`'
        // reference check).
        synthesize_reserved_preferred(&mut resolved)?;
        // After trivia injection (which skips the atomic `%` rules) and
        // before the root wrap: append the `wb` boundary call inside each
        // `%` rule's terminal captures. An undefined `wb` surfaces as
        // `CompileError::UndefinedRule("wb")` from `compile_rules`.
        append_wb_check(&mut resolved);
        wrap_root(&mut resolved);
        compile_rules(&resolved.rules)
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
    /// Byte offset of the first byte of each line (1-indexed lines, so
    /// `line_starts[0]` is the start of line 1 = 0). Precomputed once
    /// in `Parser::new` so that `span_at(offset)` is O(log lines)
    /// rather than O(offset). Every Pattern node carries a [`Span`];
    /// the per-node lookup dominates parser cost without this.
    line_starts: Vec<usize>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        let src = input.as_bytes();
        let mut line_starts = Vec::with_capacity(src.len() / 32 + 1);
        line_starts.push(0);
        for (i, &b) in src.iter().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Parser {
            src,
            pos: 0,
            line_starts,
        }
    }

    /// Convert a byte offset into a 1-based (line, col). O(log lines)
    /// via binary search over `line_starts`.
    fn span_at(&self, offset: usize) -> Span {
        let offset = offset.min(self.src.len());
        let line_idx = self
            .line_starts
            .partition_point(|&s| s <= offset)
            .saturating_sub(1);
        let line_start = self.line_starts[line_idx];
        Span {
            line: line_idx + 1,
            col: offset - line_start + 1,
        }
    }

    /// Span at the current parse position. Snapshot before consuming
    /// the bytes that start the construct being built.
    fn span(&self) -> Span {
        self.span_at(self.pos)
    }

    fn parse_grammar(&mut self) -> Result<Grammar, ParseError> {
        self.skip_ws();
        let mut rules = HashMap::new();
        let mut atomic_rules: HashSet<String> = HashSet::new();
        let mut percent_rules: HashSet<String> = HashSet::new();
        let mut preferred_rules: HashSet<String> = HashSet::new();
        let mut rule_headers: Vec<RuleHeader> = Vec::new();
        while self.pos < self.src.len() {
            // Definition-level lenient marker: `~name <- body` declares
            // that every call to `name` is intentionally lenient and
            // suppresses `lint_partial_match` findings at every call site
            // of `name`. The marker touches the name (no whitespace);
            // the rule's body is wrapped with `Pattern::Lenient` so the
            // lint walker sees the same shape as a per-call-site `~p`.
            //
            // The atomic marker `*name <- body` (sibling of `~`) opts
            // the rule out of trivia auto-insertion: the rewriter walks
            // the body but does not splice `trivia` between `Sequence`
            // items. The `trivia` rule carries no qualifiers; omitting it
            // (not marking it) is what disables auto-insertion.
            //
            // The reserved-word marker `%name <- body` implies atomic and
            // additionally appends a `wb` boundary call inside the rule's
            // terminal captures (see `append_wb_check`). `%` composes with
            // `~` but is mutually exclusive with `*` — both make a rule
            // atomic, so combining them is always a mistake.
            //
            // The preferred-word marker `%?name <- body` (the `?` touches
            // the `%`) is a sibling of `%`: same atomic + `wb` behavior,
            // but the rule's literals are *excluded* from the synthesized
            // `reserved` set and collected into `preferred` instead, so a
            // predeclared / contextual name (`int`, `async`) stays usable
            // as an identifier. `%?` is also mutually exclusive with `*`.
            let mut lenient_span: Option<Span> = None;
            let mut definition_atomic = false;
            let mut definition_percent = false;
            let mut definition_preferred = false;
            loop {
                match self.peek() {
                    Some(b'~') if lenient_span.is_none() => {
                        lenient_span = Some(self.span());
                        self.pos += 1;
                    }
                    Some(b'*') if !definition_atomic && !definition_percent => {
                        self.pos += 1;
                        definition_atomic = true;
                    }
                    Some(b'%') if !definition_percent && !definition_atomic => {
                        self.pos += 1;
                        definition_percent = true;
                        // `%?` — the optional preferred-word discriminator.
                        // The `?` must touch the `%` (no whitespace).
                        if self.peek() == Some(b'?') {
                            self.pos += 1;
                            definition_preferred = true;
                        }
                    }
                    _ => break,
                }
            }
            // A leftover `*`/`%` here means the two were combined
            // (`*%name`, `%*name`) or doubled — reject with a clear error
            // instead of a confusing "expected rule name".
            if matches!(self.peek(), Some(b'*') | Some(b'%')) {
                return Err(self.err(
                    "the `*` (atomic) and `%` (reserved-word) rule qualifiers cannot be combined"
                        .into(),
                ));
            }
            let definition_lenient = lenient_span.is_some();
            let name_byte = self.pos;
            let name = self.parse_ident()?;
            self.skip_ws();
            self.expect_str("<-")?;
            self.skip_ws();
            let body_byte_start = self.pos;
            let mut pat = self.parse_choice()?;
            let body_byte_end = self.pos;
            if let Some(span) = lenient_span {
                pat = Pattern::Lenient {
                    inner: Box::new(pat),
                    span,
                };
            }
            // `reserved` and `preferred` are compiler-generated (see
            // `synthesize_reserved_preferred`); an author definition would
            // be silently overwritten, so reject it outright. Grammars may
            // still *reference* them (`!reserved`).
            if name == RESERVED_RULE || name == PREFERRED_RULE {
                return Err(self.err(format!(
                    "`{name}` is a compiler-generated rule (synthesized from the `%` / `%?` rules) \
                     and cannot be defined explicitly; reference it (e.g. `!{name}`) instead"
                )));
            }
            // The three special rules — the start rule `root` and the two
            // auto-insertion targets `trivia` (whitespace) and `wb` (word
            // boundary) — are structural slots wrapped or injected by the
            // compiler, not lexable tokens, so every qualifier (`*`, `~`,
            // `%`, `%?`) is meaningless on them and rejected. Disabling
            // whitespace auto-insertion is done by omitting `trivia`, not
            // by qualifying it.
            if (definition_atomic
                || definition_percent
                || definition_preferred
                || definition_lenient)
                && (name == ROOT_RULE || name == TRIVIA_RULE || name == WB_RULE)
            {
                let disable_hint = if name == TRIVIA_RULE {
                    "; to disable auto-insertion, omit `trivia` and thread \
                     whitespace through a plainly-named rule"
                } else {
                    ""
                };
                return Err(self.err(format!(
                    "the `{name}` rule cannot carry a qualifier (`*`, `~`, `%`, `%?`){disable_hint}"
                )));
            }
            if definition_percent {
                percent_rules.insert(name.clone());
            }
            if definition_preferred {
                preferred_rules.insert(name.clone());
            }
            if definition_atomic || definition_percent {
                atomic_rules.insert(name.clone());
            }
            if rules.insert(name.clone(), pat).is_some() {
                return Err(self.err(format!("rule '{}' defined twice", name)));
            }
            rule_headers.push(RuleHeader {
                name,
                name_byte,
                body_byte_start,
                body_byte_end,
            });
            self.skip_ws();
        }
        if rule_headers.is_empty() {
            return Err(self.err("grammar has no rules".into()));
        }
        // Reserved-name enforcement for the special rules' positions is
        // parse-time so the error points at the offending source
        // location. `root`'s presence (and the requirement that `root`
        // itself sits at the top) is enforced by `Grammar::compile` —
        // that lets AST-only parser tests build fixtures with arbitrary
        // rule names without inventing a `root` placeholder.
        //
        // `trivia` and `wb`, when present, occupy the contiguous slots
        // immediately after `root` (positions 1..=n in either order),
        // where n is how many of them the grammar defines.
        let specials = [TRIVIA_RULE, WB_RULE];
        let n_special = rule_headers
            .iter()
            .filter(|h| specials.contains(&h.name.as_str()))
            .count();
        for (pos, h) in rule_headers.iter().enumerate() {
            if specials.contains(&h.name.as_str()) && !(1..=n_special).contains(&pos) {
                return Err(self.err(format!(
                    "`{}` must appear in the reserved slots immediately after `{ROOT_RULE}` (before any other rule)",
                    h.name
                )));
            }
        }
        // `wb`, like `trivia`, is exempt from trivia auto-insertion: its
        // body is a bare boundary predicate that must stay adjacent to
        // the keyword it follows.
        if rules.contains_key(WB_RULE) {
            atomic_rules.insert(WB_RULE.to_string());
        }
        Ok(Grammar {
            rules,
            atomic_rules,
            percent_rules,
            preferred_rules,
            rule_headers,
        })
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
            // Span of the catch / boundary-catch construct is the
            // position of the leading `^` (or `^^`). Snapshot before
            // consuming the operator bytes.
            let catch_span = self.span();
            self.pos += 1;
            // Doubled-caret discriminator: a second `^` immediately
            // after the first marks the boundary-anchored family.
            if self.peek() == Some(b'^') {
                self.pos += 1;
                let label = self.parse_catch_label("^^")?;
                self.skip_ws();
                let catch_pat = if self.at_until_inclusive_marker() {
                    // `INNER ^^lbl ..= B` — bracketed-close catch sugar.
                    self.pos += 3;
                    self.skip_ws();
                    let boundary = self.parse_prefix()?;
                    lower_bracketed_close_catch(lhs, label, boundary, catch_span)
                } else if self.peek() == Some(b'.') && self.peek_at(1) == Some(b'.') {
                    // `INNER ^^lbl .. B` — rejected. The catch necessarily
                    // consumes B; the non-consuming form has no use here.
                    return Err(self.err(
                        "`^^lbl .. B` is not a valid form; the catch consumes its boundary. \
                         Use `^^lbl ..= B` to skip-and-consume, or `^^lbl B` to leave B for the outer rule"
                            .into(),
                    ));
                } else if self.at_prefix_start() {
                    let boundary = self.parse_prefix()?;
                    lower_boundary_catch(lhs, label, boundary, catch_span)
                } else {
                    Pattern::InferBoundaryCatch {
                        inner: Box::new(lhs),
                        label,
                        span: catch_span,
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
                    span: catch_span,
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
            // `~` is a definition-level marker on the NEXT rule. There
            // is no call-site form, so `~` is never a valid in-body
            // atom-start: fall through and let the sequence terminate
            // here, letting `parse_grammar` consume `~name <- ...`.
            Some(b'~') => false,
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
                let span = self.span();
                self.pos += 1;
                self.skip_ws();
                let inner = self.parse_postfix()?;
                Ok(Pattern::NotPredicate {
                    inner: Box::new(inner),
                    span,
                })
            }
            Some(b'&') => {
                let span = self.span();
                self.pos += 1;
                self.skip_ws();
                let inner = self.parse_postfix()?;
                Ok(Pattern::AndPredicate {
                    inner: Box::new(inner),
                    span,
                })
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
                    if self.peek() == Some(b'^') {
                        self.pos += 1;
                        let charset = self.parse_optional_sync_set()?;
                        let label = self.parse_optional_recovery_label()?;
                        atom = build_recover_repeat(atom, charset, label);
                    } else {
                        // Span of `Repeat` inherits its operand's span.
                        atom = Pattern::repeat(atom);
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
                        atom = Pattern::repeat_one(atom);
                    }
                }
                Some(b'?') => {
                    self.pos += 1;
                    atom = Pattern::optional(atom);
                }
                Some(b'{') => {
                    self.pos += 1;
                    let n = self.parse_repeat_count()?;
                    self.expect(b'}')?;
                    atom = Pattern::seq(vec![atom; n]);
                }
                _ => break,
            }
        }
        Ok(atom)
    }

    /// Parse the body of an exact-count quantifier `{n}`. The opening
    /// `{` has already been consumed; this reads decimal digits, checks
    /// `1 ≤ n ≤ MAX_REPEAT_COUNT`, and leaves the cursor at the byte
    /// following the digits (the closing `}` is consumed by the caller).
    /// The cap is a typo guard — the largest plausible site in the
    /// shipped corpus is `hex{8}`.
    fn parse_repeat_count(&mut self) -> Result<usize, ParseError> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(self.err("expected positive integer in `{n}`".into()));
        }
        let digits =
            std::str::from_utf8(&self.src[start..self.pos]).expect("ASCII digits are valid UTF-8");
        let n: usize = digits.parse().map_err(|_| {
            self.err(format!(
                "repeat count `{}` exceeds maximum {}",
                digits, MAX_REPEAT_COUNT
            ))
        })?;
        if n == 0 {
            return Err(self.err("repeat count must be positive (got `{0}`)".into()));
        }
        if n > MAX_REPEAT_COUNT {
            return Err(self.err(format!(
                "repeat count {} exceeds maximum {}",
                n, MAX_REPEAT_COUNT
            )));
        }
        Ok(n)
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
            Pattern::CharClass { set, .. } => Ok(Some(set)),
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
        let span = self.span();
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
            // `i"…"` / `i'…'` — case-insensitive literal sugar. The `i`
            // is only consumed when the next byte is a quote, so plain
            // identifiers starting with `i` (e.g. `ident_char`) still
            // parse as `NonTerminal` through the `is_ident_start` arm.
            Some(b'i') if matches!(self.peek_at(1), Some(b'"') | Some(b'\'')) => {
                self.pos += 1;
                self.parse_string_case_insensitive()
            }
            Some(b'[') => self.parse_charclass(),
            Some(b'.') => {
                // Three-byte lookahead disambiguates the dot family:
                //   `..=` — skip-until inclusive (consumes the stop)
                //   `..`  — skip-until exclusive (leaves the stop)
                //   `.`   — AnyChar (single byte)
                // The two dots must be immediately adjacent; the `=` of
                // `..=` must also touch. Whitespace between the dots
                // would parse as two separate `AnyChar` atoms.
                if self.peek_at(1) == Some(b'.') {
                    let inclusive = self.peek_at(2) == Some(b'=');
                    self.pos += if inclusive { 3 } else { 2 };
                    self.skip_ws();
                    let stop = self.parse_prefix()?;
                    return Ok(if inclusive {
                        lower_until_inclusive(stop, span)
                    } else {
                        lower_until_exclusive(stop, span)
                    });
                }
                self.pos += 1;
                Ok(Pattern::AnyChar { span })
            }
            Some(b'@') => self.parse_capture(),
            Some(b'\\') => self.parse_backslash_atom(),
            Some(c) if is_ident_start(c) => {
                // Disambiguation against `name <- ...` is handled by at_prefix_start.
                let name = self.parse_ident()?;
                Ok(Pattern::NonTerminal { name, span })
            }
            Some(c) => Err(self.err(format!("unexpected '{}'", c as char))),
            None => Err(self.err("unexpected end of input".into())),
        }
    }

    /// Parse a backslash-letter atom. The seven one-letter escapes are
    /// the regex-style character-class family:
    ///
    /// - `\d` / `\D`: `[0-9]` / its complement.
    /// - `\s` / `\S`: `[ \t\n\r]` / its complement.
    /// - `\h` / `\H`: `[ \t]` / its complement.
    /// - `\R`: ordered choice `'\r\n' / '\n' / '\r'` (CRLF atomic).
    ///
    /// ASCII-only and byte-oriented, matching the rest of the engine.
    /// These are *atom-level* escapes; they are not recognized inside
    /// string literals (`'\d'` keeps today's `unknown escape` error).
    /// The six byte-set escapes are *also* recognized inside `[...]`
    /// by `parse_charclass`; `\R` is rejected there with a pointer
    /// back to this form.
    fn parse_backslash_atom(&mut self) -> Result<Pattern, ParseError> {
        debug_assert_eq!(self.peek(), Some(b'\\'));
        // Span of the synthesized atom is the position of the leading
        // `\`. The synthesized inner patterns (literals, AnyChar) are
        // not directly authored — they all share the backslash atom's
        // span so a diagnostic firing inside the desugar still points
        // at the author's `\R` site.
        let span = self.span();
        self.pos += 1;
        match self.peek() {
            Some(c) if class_for_shortcut(c).is_some() => {
                self.pos += 1;
                Ok(Pattern::CharClass {
                    set: class_for_shortcut(c).unwrap(),
                    span,
                })
            }
            Some(b'R') => {
                self.pos += 1;
                Ok(Pattern::OrderedChoice {
                    alts: vec![
                        Pattern::Literal {
                            bytes: b"\r\n".to_vec(),
                            span,
                        },
                        Pattern::Literal {
                            bytes: b"\n".to_vec(),
                            span,
                        },
                        Pattern::Literal {
                            bytes: b"\r".to_vec(),
                            span,
                        },
                    ],
                    span,
                })
            }
            Some(b'p') if self.peek_at(1) == Some(b'{') => {
                self.pos += 2;
                let set = self.parse_braced_property_escape(false)?;
                Ok(Pattern::CharClass { set, span })
            }
            Some(b'P') if self.peek_at(1) == Some(b'{') => {
                self.pos += 2;
                let set = self.parse_braced_property_escape(true)?;
                Ok(Pattern::CharClass { set, span })
            }
            Some(b'u') if self.peek_at(1) == Some(b'{') => {
                // `\u{HHHH}` at atom position: emit a literal byte
                // sequence (the UTF-8 encoding of the code point). For
                // matching against well-formed input this is byte-by-
                // byte identical to a `CharSet` match; the `CharSet`
                // opcode is reserved for character classes.
                self.pos += 2;
                let c = self.parse_braced_codepoint_escape()?;
                let mut bytes = Vec::new();
                append_char_utf8(c, &mut bytes);
                Ok(Pattern::Literal { bytes, span })
            }
            Some(c) => Err(self.err(format!(
                "unknown atom '\\{}' (valid: \\d \\D \\s \\S \\h \\H \\R \\p{{NAME}} \\P{{NAME}} \\u{{HHHH}})",
                c as char
            ))),
            None => Err(self.err("unterminated escape at top level".into())),
        }
    }

    fn parse_capture(&mut self) -> Result<Pattern, ParseError> {
        // Span of `Capture` is the position of the `@` sigil.
        let span = self.span();
        self.expect(b'@')?;
        let name = self.parse_ident()?;
        self.expect(b'{')?;
        self.skip_ws();
        let inner = self.parse_choice()?;
        self.skip_ws();
        self.expect(b'}')?;
        Ok(Pattern::Capture {
            kind: name,
            inner: Box::new(inner),
            span,
        })
    }

    fn parse_string(&mut self) -> Result<Pattern, ParseError> {
        // Span of the literal is the position of the opening quote.
        let span = self.span();
        let quote = self.peek().unwrap();
        self.pos += 1;
        let mut bytes = Vec::new();
        loop {
            match self.peek() {
                None => return Err(self.err("unterminated string literal".into())),
                Some(b'\\') => {
                    self.pos += 1;
                    let c = self.parse_escape()?;
                    append_char_utf8(c, &mut bytes);
                }
                Some(c) if c == quote => {
                    self.pos += 1;
                    return Ok(Pattern::Literal { bytes, span });
                }
                Some(c) => {
                    bytes.push(c);
                    self.pos += 1;
                }
            }
        }
    }

    /// Parse the body of an `i"…"` / `i'…'` literal — the case-
    /// insensitive sugar. The caller has consumed the `i` prefix; this
    /// reads the quoted body with the same escape handling as
    /// [`parse_string`] and lowers it to a `Sequence` of `CharClass`
    /// nodes, one per code point. ASCII letters fold to the two-element
    /// set `{lower, upper}`; every other code point becomes a singleton
    /// set, matching only itself. The result is byte-equivalent to the
    /// hand-written `[xX][yY]…` form that previously had to be spelled
    /// out per keyword.
    ///
    /// Synthesized child nodes share the outer literal's span so any
    /// diagnostic firing inside the desugar still points at the `i"…"`
    /// site — same convention as `parse_backslash_atom`.
    fn parse_string_case_insensitive(&mut self) -> Result<Pattern, ParseError> {
        let span = self.span();
        let quote = self.peek().unwrap();
        self.pos += 1;
        let mut chars: Vec<char> = Vec::new();
        let mut buf = [0u8; 4];
        loop {
            match self.peek() {
                None => return Err(self.err("unterminated string literal".into())),
                Some(b'\\') => {
                    self.pos += 1;
                    let c = self.parse_escape()?;
                    chars.push(c);
                }
                Some(c) if c == quote => {
                    self.pos += 1;
                    return Ok(match chars.len() {
                        // Empty literal: matches the empty string,
                        // same as `parse_string`'s treatment of `""`.
                        0 => Pattern::Literal {
                            bytes: Vec::new(),
                            span,
                        },
                        // Single code point: emit the bare `CharClass`
                        // so `i"a"*` composes the same as `[aA]*`
                        // (`Pattern::seq` unwraps single-item sequences
                        // for the same reason).
                        1 => Pattern::CharClass {
                            set: case_fold_codepoint(chars[0]),
                            span,
                        },
                        _ => {
                            let items = chars
                                .into_iter()
                                .map(|c| Pattern::CharClass {
                                    set: case_fold_codepoint(c),
                                    span,
                                })
                                .collect();
                            Pattern::Sequence { items, span }
                        }
                    });
                }
                Some(_) => {
                    // Decode one UTF-8 code point from the input. The
                    // grammar source is validated UTF-8 elsewhere; this
                    // walks `char_indices` of the remainder to find the
                    // next scalar value.
                    let rest = &self.src[self.pos..];
                    let s = std::str::from_utf8(rest)
                        .map_err(|_| self.err("invalid UTF-8 inside i\"…\" literal".into()))?;
                    let ch = s
                        .chars()
                        .next()
                        .expect("non-empty after peek matched a byte");
                    chars.push(ch);
                    self.pos += ch.encode_utf8(&mut buf).len();
                }
            }
        }
    }

    fn parse_charclass(&mut self) -> Result<Pattern, ParseError> {
        // Span of the char class is the position of the opening `[`.
        let span = self.span();
        self.expect(b'[')?;
        let negate = if self.peek() == Some(b'^') {
            self.pos += 1;
            true
        } else {
            false
        };
        // Collect entries as code-point ranges. Every class lowers to
        // `Pattern::CharClass` carrying a `CharSet` (interval list over
        // `(char, char)`); the legacy ASCII-byte-bitmap fast path was
        // removed when the runtime collapsed to a single class opcode.
        let mut entries: Vec<(u32, u32)> = Vec::new();
        let mut shortcut_set = CharSet::empty();
        let mut has_shortcut = false;
        let mut property_set = CharSet::empty();
        let mut has_property = false;
        while self.peek().is_some() && self.peek() != Some(b']') {
            // Class-shortcut path: `\d`, `\D`, `\s`, `\S`, `\h`, `\H`
            // expand into the surrounding set instead of contributing
            // a single byte. `\R` is multi-byte and doesn't fit inside
            // `[...]` — reject with a pointer back to the top-level
            // form. Other escapes fall through to the per-codepoint
            // `parse_class_codepoint`.
            if self.peek() == Some(b'\\') {
                if let Some(c) = self.peek_at(1) {
                    if let Some(shortcut) = class_for_shortcut(c) {
                        self.pos += 2;
                        // Shortcuts can't form ranges: `[\d-z]` is
                        // ambiguous (the shortcut is a set, not a
                        // bound) and rejected explicitly. A trailing
                        // `-]` is still a literal dash, same as the
                        // single-byte path.
                        if self.peek() == Some(b'-') && self.peek_at(1) != Some(b']') {
                            return Err(self.err(format!(
                                "character-class range can't start with a shortcut '\\{}' (shortcuts are sets, not bounds)",
                                c as char
                            )));
                        }
                        shortcut_set = shortcut_set.union(&shortcut);
                        has_shortcut = true;
                        continue;
                    }
                    if c == b'R' {
                        return Err(self.err(
                            "'\\R' is a multi-byte sequence and can't appear in a character class — use \\R as a standalone atom".into(),
                        ));
                    }
                    if (c == b'p' || c == b'P') && self.peek_at(2) == Some(b'{') {
                        let negate = c == b'P';
                        self.pos += 3;
                        let class = self.parse_braced_property_escape(negate)?;
                        // Property class can't form a range, same
                        // rationale as the byte-shortcut path.
                        if self.peek() == Some(b'-') && self.peek_at(1) != Some(b']') {
                            return Err(self.err(format!(
                                "character-class range can't start with a property class '\\{}{{...}}'",
                                c as char
                            )));
                        }
                        property_set = property_set.union(&class);
                        has_property = true;
                        continue;
                    }
                }
            }
            let lo = self.parse_class_codepoint()?;
            // Range form `lo-hi` (but `-` at end is literal; require non-`]` after `-`)
            if self.peek() == Some(b'-') && self.peek_at(1) != Some(b']') {
                self.pos += 1;
                let hi = self.parse_class_codepoint()?;
                if hi < lo {
                    return Err(self.err(format!(
                        "char class range out of order: U+{:04X}-U+{:04X}",
                        lo, hi
                    )));
                }
                entries.push((lo, hi));
            } else {
                entries.push((lo, lo));
            }
        }
        self.expect(b']')?;
        // Build a `CharSet`: union of explicit `(lo, hi)` ranges, the
        // shortcut set, the property set, then negate if `[^...]`.
        // Every `(lo, hi)` already passed `\u{...}` validation, so
        // `char::from_u32` can't fail here.
        let mut char_ranges: Vec<(char, char)> = Vec::new();
        for &(lo, hi) in &entries {
            let lo_c = char::from_u32(lo)
                .ok_or_else(|| self.err(format!("invalid scalar value U+{:04X}", lo)))?;
            let hi_c = char::from_u32(hi)
                .ok_or_else(|| self.err(format!("invalid scalar value U+{:04X}", hi)))?;
            char_ranges.push((lo_c, hi_c));
        }
        let explicit = CharSet::from_ranges(&char_ranges)
            .map_err(|e| self.err(format!("invalid character class: {}", e)))?;
        let _ = has_shortcut; // shortcut_set always unioned in; flag retained for symmetry
        let _ = has_property;
        let combined = explicit.union(&shortcut_set).union(&property_set);
        let final_set = if negate { combined.negate() } else { combined };
        Ok(Pattern::CharClass {
            set: final_set,
            span,
        })
    }

    /// Parse one entry of a character class as a code point. Handles
    /// three shapes:
    ///
    /// 1. A `\\`-prefixed escape (delegates to [`parse_escape`]).
    /// 2. A plain ASCII byte from source.
    /// 3. A non-ASCII byte from source — UTF-8-decoded into one code
    ///    point. Flows into the unified `Pattern::CharClass`.
    fn parse_class_codepoint(&mut self) -> Result<u32, ParseError> {
        match self.peek() {
            Some(b'\\') => {
                self.pos += 1;
                Ok(self.parse_escape()? as u32)
            }
            Some(c) if c < 0x80 => {
                self.pos += 1;
                Ok(c as u32)
            }
            Some(_) => {
                // Non-ASCII source byte: decode the UTF-8 sequence
                // starting here and treat it as a single code-point
                // class entry.
                match crate::pegvm::utf8::decode_at(self.src, self.pos) {
                    crate::pegvm::utf8::Utf8DecodeResult::Valid { scalar, bytes } => {
                        self.pos += bytes;
                        Ok(scalar as u32)
                    }
                    crate::pegvm::utf8::Utf8DecodeResult::Invalid { .. } => Err(self.err(
                        "invalid UTF-8 byte sequence in grammar source character class".into(),
                    )),
                    crate::pegvm::utf8::Utf8DecodeResult::Eof => {
                        Err(self.err("unterminated character class".into()))
                    }
                }
            }
            None => Err(self.err("unterminated character class".into())),
        }
    }

    fn parse_escape(&mut self) -> Result<char, ParseError> {
        let c = match self.peek() {
            Some(b'n') => '\n',
            Some(b'r') => '\r',
            Some(b't') => '\t',
            Some(b'0') => '\0',
            Some(b'\\') => '\\',
            Some(b'\'') => '\'',
            Some(b'"') => '"',
            Some(b']') => ']',
            Some(b'[') => '[',
            Some(b'-') => '-',
            Some(b'/') => '/',
            Some(b'u') if self.peek_at(1) == Some(b'{') => {
                self.pos += 2;
                return self.parse_braced_codepoint_escape();
            }
            Some(c) => return Err(self.err(format!("unknown escape '\\{}'", c as char))),
            None => return Err(self.err("unterminated escape sequence".into())),
        };
        self.pos += 1;
        Ok(c)
    }

    /// Parse the body of a `\p{NAME}` or `\P{NAME}` escape. The caller
    /// has already consumed the opening `\p{` or `\P{`; this consumes
    /// the property name and the closing `}`, looks the name up in
    /// [`crate::pegc::unicode_properties`], and returns the resulting
    /// [`CharSet`] (negated for `\P{...}`).
    fn parse_braced_property_escape(&mut self, negate: bool) -> Result<CharSet, ParseError> {
        let start_pos = self.pos;
        loop {
            match self.peek() {
                Some(b'}') => {
                    if self.pos == start_pos {
                        return Err(self.err("\\p{} is empty: expected a property name".into()));
                    }
                    let name = std::str::from_utf8(&self.src[start_pos..self.pos])
                        .map_err(|_| self.err("invalid UTF-8 in \\p{...} property name".into()))?;
                    self.pos += 1;
                    let set = crate::pegc::unicode_properties::lookup(name)
                        .ok_or_else(|| self.err(format!("unknown Unicode property '{}'", name)))?;
                    return Ok(if negate { set.negate() } else { set.clone() });
                }
                Some(c) if c.is_ascii_alphanumeric() || c == b'_' || c == b'=' || c == b'-' => {
                    self.pos += 1;
                }
                Some(c) => {
                    return Err(self.err(format!(
                        "unexpected '{}' in \\p{{...}} property name",
                        c as char
                    )));
                }
                None => return Err(self.err("unterminated \\p{...} escape".into())),
            }
        }
    }

    /// Parse the body of a `\u{...}` escape: 1–6 hex digits followed by
    /// `}`. The caller has already consumed `\u{`. Rejects values >
    /// U+10FFFF, surrogates (U+D800..=U+DFFF), and empty braces.
    fn parse_braced_codepoint_escape(&mut self) -> Result<char, ParseError> {
        let start_pos = self.pos;
        let mut value: u32 = 0;
        let mut digits = 0usize;
        loop {
            match self.peek() {
                Some(c) if c.is_ascii_hexdigit() => {
                    if digits == 6 {
                        return Err(self.err("\\u{...} accepts at most 6 hex digits".into()));
                    }
                    let nibble = match c {
                        b'0'..=b'9' => c - b'0',
                        b'a'..=b'f' => c - b'a' + 10,
                        b'A'..=b'F' => c - b'A' + 10,
                        _ => unreachable!(),
                    };
                    value = (value << 4) | (nibble as u32);
                    digits += 1;
                    self.pos += 1;
                }
                Some(b'}') => {
                    self.pos += 1;
                    if digits == 0 {
                        return Err(self.err("\\u{} is empty: expected 1-6 hex digits".into()));
                    }
                    if value > 0x10_FFFF {
                        return Err(self.err(format!("\\u{{{:X}}} exceeds U+10FFFF", value)));
                    }
                    if (0xD800..=0xDFFF).contains(&value) {
                        return Err(self.err(format!(
                            "\\u{{{:04X}}} is a surrogate (U+D800..=U+DFFF) and not a Unicode scalar value",
                            value
                        )));
                    }
                    return Ok(char::from_u32(value).expect("validated bounds"));
                }
                Some(c) => {
                    return Err(self.err(format!(
                        "expected hex digit or '}}' in \\u{{...}}, found '{}'",
                        c as char
                    )));
                }
                None => {
                    // Restore for diagnostic-position purposes.
                    self.pos = start_pos;
                    return Err(self.err("unterminated \\u{...} escape".into()));
                }
            }
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

    /// True iff the next three bytes are `..=` — the bracketed-close
    /// catch-sugar marker after `^^lbl`. Adjacent-only; whitespace
    /// between the dots or before the `=` makes them parse as separate
    /// tokens.
    fn at_until_inclusive_marker(&self) -> bool {
        self.peek() == Some(b'.') && self.peek_at(1) == Some(b'.') && self.peek_at(2) == Some(b'=')
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
        let Span { line, col } = self.span();
        ParseError { message, line, col }
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
        b'(' | b'"' | b'\'' | b'[' | b'.' | b'@' | b'!' | b'&' | b'\\'
    ) || is_ident_start(c)
}

/// Map a backslash-letter shortcut to its character set. Returns `None`
/// for any byte that isn't one of the character-set shortcuts (`d`, `D`,
/// `s`, `S`, `h`, `H`). The multi-byte (`R`) and zero-width (`z`)
/// shortcuts are handled separately by their respective parse arms.
fn class_for_shortcut(c: u8) -> Option<CharSet> {
    match c {
        b'd' => Some(CharSet::single_range('0', '9').unwrap()),
        b'D' => Some(CharSet::single_range('0', '9').unwrap().negate()),
        b's' => Some(CharSet::from_chars(&[' ', '\t', '\n', '\r'])),
        b'S' => Some(CharSet::from_chars(&[' ', '\t', '\n', '\r']).negate()),
        b'h' => Some(CharSet::from_chars(&[' ', '\t'])),
        b'H' => Some(CharSet::from_chars(&[' ', '\t']).negate()),
        _ => None,
    }
}

/// Lower `.. S` (skip-until exclusive) to `(!S .)*`. The stop pattern
/// is a negative lookahead; `S` is not consumed. All synthesized nodes
/// inherit the leading `..`'s span so a diagnostic firing inside the
/// desugar still points at the author's site.
fn lower_until_exclusive(stop: Pattern, span: Span) -> Pattern {
    Pattern::Repeat {
        inner: Box::new(Pattern::Sequence {
            items: vec![
                Pattern::NotPredicate {
                    inner: Box::new(stop),
                    span,
                },
                Pattern::AnyChar { span },
            ],
            span,
        }),
        span,
    }
}

/// Lower `..= S` (skip-until inclusive) to `(!S .)* S`. The stop is
/// matched and consumed after the skip; its capture kind (if any) is
/// preserved on the trailing consume.
fn lower_until_inclusive(stop: Pattern, span: Span) -> Pattern {
    Pattern::Sequence {
        items: vec![lower_until_exclusive(stop.clone(), span), stop],
        span,
    }
}

/// Lower `INNER ^^lbl B` to the explicit boundary-anchored catch
/// shape: `Catch { Sequence([INNER, &B]), lbl, @recovery{(!B .)*} }`.
/// Parse-time lowering — the compiler sees this as a regular `Catch`
/// with an `AndPredicate` sibling, so the existing lint walker
/// recognizes it as anchored without a new arm. All synthesized nodes
/// inherit the catch operator's span.
fn lower_boundary_catch(inner: Pattern, label: String, boundary: Pattern, span: Span) -> Pattern {
    let lookahead = Pattern::AndPredicate {
        inner: Box::new(boundary.clone()),
        span,
    };
    let stop_loop = Pattern::Repeat {
        inner: Box::new(Pattern::Sequence {
            items: vec![
                Pattern::NotPredicate {
                    inner: Box::new(boundary),
                    span,
                },
                Pattern::AnyChar { span },
            ],
            span,
        }),
        span,
    };
    let recovery = Pattern::Capture {
        kind: "recovery".into(),
        inner: Box::new(stop_loop),
        span,
    };
    Pattern::Catch {
        inner: Box::new(Pattern::Sequence {
            items: vec![inner, lookahead],
            span,
        }),
        label,
        recovery: Box::new(recovery),
        span,
    }
}

/// Lower `INNER ^^lbl ..= B` (bracketed-close catch sugar) to:
/// `Catch { INNER, lbl, Sequence([@recovery{(!B .)*}, B]) }`.
///
/// Distinct from `lower_boundary_catch` in two ways: (1) the inner is
/// *not* anchored with `&B` — every corpus site has `INNER` already
/// ending in `B`, so the anchor would require another `B` to follow;
/// (2) the recovery body consumes `B` after the skip, with `B`
/// captured per its own kind (outside the `@recovery` wrap). This is
/// the shape the `^block_close` sites across the shipped grammars
/// need: the closing `}` keeps its `@punctuation` kind in both happy
/// and recovery paths.
fn lower_bracketed_close_catch(
    inner: Pattern,
    label: String,
    boundary: Pattern,
    span: Span,
) -> Pattern {
    let stop_loop = Pattern::Repeat {
        inner: Box::new(Pattern::Sequence {
            items: vec![
                Pattern::NotPredicate {
                    inner: Box::new(boundary.clone()),
                    span,
                },
                Pattern::AnyChar { span },
            ],
            span,
        }),
        span,
    };
    let recovery = Pattern::Sequence {
        items: vec![
            Pattern::Capture {
                kind: "recovery".into(),
                inner: Box::new(stop_loop),
                span,
            },
            boundary,
        ],
        span,
    };
    Pattern::Catch {
        inner: Box::new(inner),
        label,
        recovery: Box::new(recovery),
        span,
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
///
/// The synthesized recover-repeat tree inherits its operand's span,
/// matching the per-variant convention for `Repeat` ("start of
/// operand").
fn build_recover_repeat(
    inner: Pattern,
    charset: Option<CharSet>,
    label: Option<String>,
) -> Pattern {
    let span = inner.span();
    let body = match charset {
        None => Pattern::AnyChar { span },
        Some(cs) => Pattern::Sequence {
            items: vec![
                Pattern::Repeat {
                    inner: Box::new(Pattern::Sequence {
                        items: vec![
                            Pattern::NotPredicate {
                                inner: Box::new(Pattern::CharClass {
                                    set: cs.clone(),
                                    span,
                                }),
                                span,
                            },
                            Pattern::AnyChar { span },
                        ],
                        span,
                    }),
                    span,
                },
                Pattern::CharClass { set: cs, span },
            ],
            span,
        },
    };
    let recovery = Pattern::Capture {
        kind: "recovery".into(),
        inner: Box::new(body),
        span,
    };
    Pattern::Repeat {
        inner: Box::new(Pattern::Catch {
            inner: Box::new(inner),
            label: label.unwrap_or_else(|| "recovery".into()),
            recovery: Box::new(recovery),
            span,
        }),
        span,
    }
}
