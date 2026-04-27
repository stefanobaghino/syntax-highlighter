# pegc — compiling grammar source to pegvm bytecode

This module is the source-language half of the crate's parsing
pipeline: it reads a PEG grammar written in `.peg` source and produces
a runnable [`Program`](../pegvm/program.rs) of bytecode that
[`pegvm`](../pegvm/README.md) executes.

```
&str  ──pegc::parse──▶  Grammar  ──Grammar::compile──▶  Program  ──VM::run──▶  MatchResult
```

`pegc::compile(source)` folds the first two steps into one call.

This document is the syntactic spec of the `.peg` source language —
what a grammar author writes. For the compiled-bytecode side
(instruction set, VM execution, capture protocol, invariants), see
[`src/pegvm/README.md`](../pegvm/README.md).

## Source structure

A `.peg` file is a sequence of rule definitions:

```peg
name  <-  body
name2 <-  body2
```

- The **first rule** is the start rule.
- **Identifiers** are ASCII `[A-Za-z_][A-Za-z0-9_]*`.
- **Whitespace** (space, tab, newline, carriage return) separates
  tokens but is otherwise ignored.
- **Comments** run from `#` to end of line.
- **Duplicate rule definitions** and **empty grammars** are parse
  errors.

Full shipped example: [`grammars/json.peg`](../../grammars/json.peg).

## Patterns

A rule body is a *pattern*. Patterns combine through operators with
the following precedence, tightest-binding first:

```
atom       "abc"   [a-z]   .   ident   (...)   @name{...}
postfix    p*      p+      p?  p*^     p+^
prefix     !p      &p
sequence   p1 p2 p3          (juxtaposition)
choice     p1 / p2 / p3
```

### Atoms

| Syntax | Meaning |
|---|---|
| `"abc"` / `'abc'` | Literal byte sequence. |
| `[a-z]` / `[^"\\]` | Character class — a set of bytes; leading `^` negates. |
| `.` | Any single byte (fails only at end of input). |
| `ident` | Reference to another rule. |
| `(...)` | Grouping — any pattern. |
| `@name{...}` | Named capture — see below. |

**String literals** use either `"..."` or `'...'`. Recognized escapes
(inside strings and character classes): `\n`, `\r`, `\t`, `\0`, `\\`,
`\'`, `\"`, `\]`, `\[`, `\-`, `\/`. Unknown escapes are a parse error.

**Character classes** use the standard `[lo-hi]` range syntax. A `-`
immediately before the closing `]` is a literal hyphen. Ranges with
`hi < lo` are a parse error.

The grammar is **byte-oriented** — no UTF-8 decoding happens at any
stage. `[a-z]` is the ASCII byte range, `.` is one byte, not one code
point.

### Postfix operators

| Syntax | Meaning |
|---|---|
| `p*` | Greedy, possibly-empty repetition. |
| `p+` | Greedy, at-least-once repetition. |
| `p?` | Optional. |
| `p*^` | Repetition with skip-byte error recovery (see below). |
| `p+^` | At-least-once recovery form — desugars to `p (p*^)`. |

### Prefix operators

Zero-width — consume no input, emit no captures:

| Syntax | Meaning |
|---|---|
| `!p` | Succeeds iff `p` would fail at the current position. |
| `&p` | Succeeds iff `p` would succeed at the current position. |

### Sequence and ordered choice

| Syntax | Meaning |
|---|---|
| `p1 p2 p3` | Sequence — every sub-pattern must match in order; adjacent patterns, whitespace-separated. |
| `p1 / p2 / p3` | Ordered choice — biased: `p2` is tried only if `p1` fails, `p3` only if `p2` fails. **Not regex alternation.** |

## Extensions beyond classical PEG

Two additions over Ford 2004 PEG syntax:

### `@name{pattern}` — named captures

Wraps a sub-pattern with a highlight tag. On a successful enclosing
match the VM emits a `Capture { kind, start, end }` record over the
matched bytes. `name` is interned to a small integer (`CaptureKind`)
at compile time; the highlighter resolves it back via
`Program::capture_kinds`.

```peg
string_lit <- @string{ '"' (!'"' .)* '"' }
```

Capture names may be any valid identifier. The built-in theme
(`src/highlight/theme.rs`) styles twelve names: `keyword`, `string`,
`number`, `comment`, `operator`, `punctuation`, `type`, `function`,
`constant`, `property`, `variable`, `recovery`. Names outside this
vocabulary compile and run — they just won't be styled by the default
theme.

### `p*^` / `p+^` — skip-byte error recovery

On a repetition, appending `^` turns each iteration into "try `p`; on
failure, skip one byte under a `recovery` capture and retry." The
loop terminates cleanly at end of input rather than aborting on the
first malformed sub-element — the mechanism behind multi-statement
resyncing after a syntax error.

```peg
sql_file <- ws (statement)*^ ws !.
```

`p+^` desugars to `p (p*^)` — one inner success is required, then
recover on the rest. The recovery capture kind is hard-coded as
`recovery`.

Implementation lives in `Pattern::RecoverRepeat`
(`src/pegc/pattern.rs`) and the emission in
`src/pegc/compiler.rs`. The compiler uses `Choice`/`Commit` per
iteration rather than `PartialCommit` — see the invariants section in
[`src/pegvm/README.md`](../pegvm/README.md).

**Empty-match caveat.** If `p` matches the empty string, `p*^` spins
forever — same hazard as plain `p*`. The compiler does not detect
this; grammar authors must ensure `p` consumes input on success.

## Semantics notes

- **Ordered choice is biased.** `p1 / p2` is not regex `p1|p2`; `p2`
  is tried only if `p1` fails.
- **Predicates consume no input.** `!p` and `&p` rewind any `sp`
  advance `p` would have made, and emit no captures.
- **Direct left recursion is supported.** `A <- A α / β` and similar
  shapes parse left-associatively via bounded LR (Medeiros et al.
  2014 §5; see [`src/pegvm/README.md`](../pegvm/README.md#left-recursion)).
  **Indirect** left recursion (`A <- B …; B <- A …`) is rejected
  with `CompileError::IndirectLeftRecursion`; tracked as a
  follow-up to #40.
- **Byte-oriented.** Character classes are sets of `u8`; UTF-8 is
  never decoded. Direction for Unicode support is under evaluation
  in #45.
- **Backtracking and memoization** are the VM's job — see
  [`src/pegvm/README.md`](../pegvm/README.md).

## Entry points

| Call | Returns | Use |
|---|---|---|
| `pegc::compile(source)` | `Result<Program, Error>` | One-step: source → runnable bytecode. |
| `pegc::parse(source)` | `Result<Grammar, ParseError>` | Stop at the AST (for inspection or tests). |
| `Grammar::compile()` | `Result<Program, CompileError>` | Follow-up to `parse`. |
| `Grammar::new(rules, start)` | `Grammar` | Build a grammar from a hand-built rule map. |
| `pegc::compile_pattern(&pat)` | `Program` | Compile a single `Pattern` as the start rule — for testing. |

## Errors

- **`ParseError`** — source is malformed. Carries line and column.
  Examples: unterminated string, missing `<-`, duplicate rule,
  character-class range out of order, unknown escape.
- **`CompileError`** — source is well-formed but semantically invalid.
  Examples: `NonTerminal("foo")` with no matching rule, a start rule
  that doesn't exist.
- **`Error`** — unified wrapper returned by `pegc::compile(source)`.
  `From<ParseError>` and `From<CompileError>` are provided.

Runtime mismatches (input doesn't conform to the grammar) are not
errors — the VM returns a `MatchResult` with `complete: false`. The
distinction is between *author bugs* (grammar) and *data bugs*
(input).

## Reference

- Bryan Ford, [*Parsing Expression Grammars: A Recognition-Based
  Syntactic Foundation*](https://bford.info/pub/lang/peg.pdf).
  POPL 2004 — foundational syntax and semantics.
