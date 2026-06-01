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

- Every grammar must define a **`root` rule** — that's the entry
  point. The compiler wraps its body as `trivia? root_body trivia? !.`
  so end-of-input is implicit (any trailing junk fails the parse).
- An optional **`trivia` rule** acts as the auto-insertion target:
  when present, the compiler splices a `trivia` call between every
  pair of consecutive items in non-atomic rule bodies, including
  between iterations of `*` / `+`. Without a `trivia` rule, no
  auto-insertion happens.
- An optional **`wb` rule** is the word-boundary target consumed by
  `%` rules (see below); it is typically `wb <- !ident_body`. Defining
  it is only required when the grammar has at least one `%` rule.
- Rules may carry **prefix sigils**: `~name <-` for the intentional
  leniency marker, `*name <-` for the atomic marker (no `trivia`
  injected inside this rule's body), `%name <-` for the reserved-word
  marker (atomic *and* appends a trailing `wb` call inside the rule's
  terminal captures), and `%?name <-` for the preferred-word marker (a
  sibling of `%` for identifier-eligible distinguished tokens). `~`
  composes with `*` / `%` / `%?` (`~*name`, `%~name`, …); `*` and
  `%` / `%?` are mutually exclusive — both make a rule atomic. The
  `trivia` rule carries no qualifiers; auto-insertion is disabled by
  omitting `trivia`, not by marking it.
- Two special rules, **`reserved`** and **`preferred`**, are
  *synthesized* by the compiler from the `%` / `%?` rules (see below) —
  a grammar references them (`!reserved`) but never defines them.
- **Position constraint:** `root` is always the first rule. The
  optional special rules `trivia` and `wb` occupy the contiguous slots
  immediately after `root` (positions 1..2, in either order). Other
  rules follow in any order.
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
postfix    p*      p+      p?      p{n}    p*^     p+^     p*^[cs]   p+^[cs]
prefix     !p      &p
sequence   p1 p2 p3          (juxtaposition)
catch      p1 ^label p2      p1 ^^label B      p1 ^^label
choice     p1 / p2 / p3
```

Rule definitions may carry a top-level `~name <-` decorator to mark
the whole rule as intentionally lenient — see
[`~name <-`](#name----intentional-leniency-marker) below.

### Atoms

| Syntax | Meaning |
|---|---|
| `"abc"` / `'abc'` | Literal byte sequence. |
| `i"abc"` / `i'abc'` | Case-insensitive literal — ASCII letters fold, other code points stay literal. |
| `[a-z]` / `[^"\\]` | Character class — a set of bytes; leading `^` negates. |
| `.` | Any single byte (fails only at end of input). |
| `\d \D \s \S \h \H \R` | Built-in character classes — see below. |
| `ident` | Reference to another rule. |
| `(...)` | Grouping — any pattern. |
| `@name{...}` | Named capture — see below. |

**String literals** use either `"..."` or `'...'`. Recognized escapes
(inside strings and character classes): `\n`, `\r`, `\t`, `\0`, `\\`,
`\'`, `\"`, `\]`, `\[`, `\-`, `\/`. Unknown escapes are a parse error.
The backslash-letter atoms below (`\d`, `\R`, etc.) are *not* string
escapes — `'\d'` keeps the `unknown escape` error.

**Case-insensitive literals** add an `i` prefix immediately before the
opening quote: `i"select"` matches `select`, `SELECT`, `SeLeCt`, etc.
The `i` is only recognized when followed by `"` or `'` — identifiers
that happen to start with `i` (`ident`, `int_body`) still parse as rule
references. ASCII letters in the body fold to `{lower, upper}`; every
other code point is matched literally (no Unicode case folding —
grammars that need it can hand-roll the character class). Parse-time
sugar: `i"select"` desugars to the same shape as
`[sS][eE][lL][eE][cC][tT]` and produces byte-identical bytecode.

**Character classes** use the standard `[lo-hi]` range syntax. A `-`
immediately before the closing `]` is a literal hyphen. Ranges with
`hi < lo` are a parse error.

The grammar is **byte-oriented** — no UTF-8 decoding happens at any
stage. `[a-z]` is the ASCII byte range, `.` is one byte, not one code
point.

### Built-in character classes

Eight regex-style one-letter escapes are atom-level shortcuts for the
byte sets and idioms that recur across the corpus. ASCII semantics are
fixed:

| Escape | Equivalent | Notes |
|---|---|---|
| `\d` / `\D` | `[0-9]` / `[^0-9]` | Decimal digit and its complement. |
| `\s` / `\S` | `[ \t\n\r]` / complement | ASCII whitespace and its complement. |
| `\h` / `\H` | `[ \t]` / complement | Horizontal whitespace and its complement. |
| `\R` | `'\r\n' / '\n' / '\r'` | Linebreak — CRLF matched atomically. |

The six byte-set escapes (`\d`, `\D`, `\s`, `\S`, `\h`, `\H`) are also
recognized **inside `[...]`** and union into the surrounding class:
`[\da-fA-F]` is the hex-digit set, `[\d_]` is digit-or-underscore. A
class shortcut cannot be a range bound — `[\d-z]` is a parse error
because the shortcut is a set, not a single byte. A leading `^` still
negates the whole class: `[^\d]` ≡ `\D`.

`\R` is atom-only: it matches a multi-byte sequence (CRLF atomic) and
has no meaningful shape inside `[...]`, so it's rejected there with a
tailored error pointing back at the top-level form.

Deliberately excluded: `\w` / `\W` (word character varies meaningfully
per language — Rust raw idents, SQL `$`, CSS hyphens), `\v` / `\V`
(PCRE vertical-whitespace splits `\r\n`; `\R` is the right primitive),
and any hex-digit shortcut (would collide with `\h`; `[\da-fA-F]`
covers the case in one extra range).

### Postfix operators

| Syntax | Meaning |
|---|---|
| `p*` | Greedy, possibly-empty repetition. |
| `p+` | Greedy, at-least-once repetition. |
| `p?` | Optional. |
| `p{n}` | Exactly `n` repetitions, `1 ≤ n ≤ 1024`. Parse-time sugar for `p p … p` (`n` copies). |
| `p*^` | Repetition with skip-byte error recovery (see below). |
| `p+^` | At-least-once recovery form — desugars to `p (p*^)`. |
| `p*^[charset]` | Repetition with delimiter-scoped recovery: on inner failure, skip to and consume the next byte in `charset`. |
| `p+^[charset]` | At-least-once delimiter-scoped recovery — desugars to `p (p*^[charset])`. |
| `p*^:lbl` / `p*^[charset]:lbl` / `p+^:lbl` / `p+^[charset]:lbl` | Optional `:label` suffix on any of the above, naming the catch scope for `pegdb recoveries explain` clustering. Default label is `"recovery"`. |

**Exact-count `p{n}`.** `p{4}` desugars at parse time to four
copies of `p` concatenated — identical bytecode to writing `p p p p`
by hand. `n` is a positive decimal integer with `1 ≤ n ≤ 1024`. The
upper bound is a typo guard; the largest plausible site in the shipped
corpus is `hex{8}`. The braces are tight — no whitespace inside `{n}`
or between the atom and `{` — matching the rest of the postfix tier.
`p{1}` is equivalent to bare `p`; `{0}` is a parse error (write `''`
if you want the always-succeeding empty pattern). Inside string
literals, `'p{4}'` is the literal byte sequence `p{4}` — `{n}` is a
postfix atom-quantifier, not a string escape.

The bounded form `p{n,m}` and lower-bound form `p{n,}` are
deliberately **not** included: `p{n,}` is strictly redundant with `+`
(and `{0,}` with `*`), and no shipped grammar has a bounded-range
need. The `{n,m}` syntax remains an unambiguous future extension slot
should a real use case appear — see issue #87.

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

An optional `:lbl` suffix names the catch scope: `p*^:bad_doc`
interns label `"bad_doc"` instead of the default `"recovery"`. The
`:` must touch the preceding `^` and the identifier must touch `:`
— same tight-binding rule as `^label`. The capture kind is
unaffected (still `recovery`); only the label changes. `pegdb
recoveries explain` clusters by this label, so per-site naming
lets distinct call sites surface as their own buckets.

**Lowering.** `p*^` is syntactic sugar for a labeled catch wrapped in
a `Repeat`. The parser produces an AST equivalent to:

```peg
(p ^recovery @recovery{.})*
```

There is no dedicated `RecoverRepeat` AST node — `build_recover_repeat`
in `src/pegc/parser.rs` emits the desugared `Repeat(Catch(...))` tree
directly. The runtime behavior (one `recovery` capture per skipped
byte, clean exit at EOF) is unchanged; what was once a bespoke opcode
sequence is now the composition of the existing `Repeat` and `Catch`
compiler arms in `src/pegc/compiler.rs`.

**Empty-match caveat.** If `p` matches the empty string, `p*^` spins
forever — same hazard as plain `p*`. The compiler does not detect
this; grammar authors must ensure `p` consumes input on success.

### `p*^[charset]` / `p+^[charset]` — delimiter-scoped recovery

The sync-set form replaces the skip-one-byte recovery with a
skip-until-delimiter recovery: on inner failure, consume bytes that
aren't in `charset` and then consume one byte that is. One `recovery`
capture is emitted per resync region — covering the skipped bytes plus
the delimiter — instead of one capture per skipped byte.

```peg
sql_file <- ws (statement)*^[;] ws !.
```

On input like `INSERT INTO @@@ garbage @@@; SELECT 1;` the `*^[;]` form
emits a single `recovery` span covering `@@@ garbage @@@;` rather than
17 single-byte recovery spans. Same compile shape as `*^`, different
recovery body:

```peg
(p ^recovery @recovery{(![charset] .)* [charset]})*
```

`p+^[charset]` desugars to `p (p*^[charset])`, mirroring `p+^`.

The `[charset]` token uses the standard character-class syntax — same
ranges, escapes, and negation as a top-level `[...]` atom. It must
touch `^` (no whitespace between them) so the postfix glue isn't
broken by an intervening atom.

The optional `:lbl` suffix described above applies here too:
`p*^[;]:bad_stmt` interns label `"bad_stmt"`. The `:` must touch the
closing `]`.

**EOF semantics.** If the delimiter is missing before EOF, the
recovery body fails: the catch fails, the outer `*` terminates, and
the parse stops at the last successful inner match. With plain `*^`
the loop would have skipped past the missing delimiter byte-by-byte
to EOF; with sync sets the loop stops at the first unrecoverable
region. Pick `*^` when "skip what you can't parse" is the right
default and sync sets when you want recovery anchored to a specific
delimiter.

### `.. S` and `..= S` — skip until delimiter

The "repeat-until-delimiter" idiom `(!S .)*` recurs across comment
bodies, multiline strings, attribute payloads, and recovery scopes.
The two shorthands name the consume-or-not distinction directly:

| Syntax | Meaning | Lowers to |
|---|---|---|
| `.. S` | Skip bytes up to (but not including) `S`. The stop is a negative lookahead; `S` is left for the outer rule. | `(!S .)*` |
| `..= S` | Skip bytes up to and including `S`. The stop is matched and consumed after the skip; its capture kind (if any) is preserved on the trailing consume. | `(!S .)* S` |

The `..=` convention mirrors Rust's inclusive-range operator. Read
`..` as "up to" and `..=` as "up to and inclusive".

Two worked examples:

```peg
# Non-consuming: the newline is left for outer whitespace handling.
line_comment <- '//' .. '\n'

# Consuming: the closing `*/` is part of the comment.
block_comment <- '/*' ..= '*/'
```

Both operators are unary today — the LHS of the skip is always `.`
(any byte). Every site in the shipped corpus follows this shape; a
binary `p .. S` / `p ..= S` could be added later if structured
non-`.` bodies surface in a future grammar.

The two dots must be immediately adjacent, and the `=` of `..=`
must also touch (`. .` and `.. =` parse as separate atoms,
preserving today's behavior). Whitespace around the operator and
between the operator and the stop pattern is allowed.

### `inner ^label recovery` — labeled catch with recovery

Tries `inner`; on failure, splices the failed attempt's deepest-reach
captures back into the live buffer (via `RecoverToScopedMax`) and runs
`recovery` from that resync point. The recovery branch only fires when
`inner` fails; on success the catch behaves exactly like its inner.

```peg
stmt <- (assign / call) ^bad_stmt @err{ (!';' .)* } ';'
```

The `label` is mandatory: it tags this scope so `pegdb
recoveries explain` can cluster firings — the author's name for "what
went wrong here." Labels intern into `Program::label_kinds` (a
separate namespace from `capture_kinds`); `*^` interns its
`recovery_kind` string as a label the same way, so a `^foo` catch and
a `*^foo` resync at the same site end up in the same cluster.

The label identifier must touch `^` — no whitespace between them.
Anything else (`^ lbl`, `^_`, `^!lbl`) is a parse error today and
reserved for future overlays (anonymous catches, throw atoms — see
"Reserved syntax" below).

If both branches fail the catch fails to its enclosing backtrack.

Precedence: tighter than `/`, looser than sequence. So `a b ^lbl c d`
parses as `(a b) ^lbl (c d)`, and `a ^lbl b / c` parses as
`(a ^lbl b) / c`. Chained `^` is left-associative: `a ^l1 b ^l2 c`
≡ `(a ^l1 b) ^l2 c`.

**Difference from `/`.** Ordered choice rewinds `sp` and discards
inner's partial work before trying the alternative; `^label` preserves
the partial work (the failed attempt's deepest-reach captures and
position) and runs recovery from there. Subsumes Yacc-style error
productions:

```peg
stmt <- alt1 / alt2 / error ';'              # error-production style
stmt <- (alt1 / alt2) ^bad_stmt (!';' .)* ';'  # same idea with `^`
```

**Difference from `*^`.** No loop and no synthetic single-byte
`recovery` capture — the recovery body is author-written and emits
whatever captures the author put in it. Wrap recovery in
`@recovery{...}` (or any other capture name) if you want one. The
natural two-tier composition is `^label` inside a rule for known
failure points and `*^` outside as the loop-level backstop.

#### Anchoring catches at boundaries

PEG's prioritized choice and possessive `*` mean rules routinely
succeed by matching a prefix and leaving the rest as leftover at the
outer level — invisibly, with no failure. The bare `^label` catch
only helps when the rule actually *fails*, so anchoring the inner
on a boundary is what turns a partial-match success into a
catch-able failure. The
[`INNER ^^lbl B`](#inner-lbl-b---boundary-anchored-catch) operator
collapses the three concerns (anchor + catch + recovery body) into
one form. The bare `^label` form below is still useful for catches
whose recovery isn't a "skip until boundary" loop — see PR `#101`'s
`^block_close` sites.

### `INNER ^^lbl B` — boundary-anchored catch

The boundary-anchored catch is a syntactic sugar that takes one
inner pattern and one boundary `B` (any pattern) and lowers to the
three-piece idiom that recurs across `grammars/sqlite.peg`:

```peg
INNER ^^lbl B
# lowers to
(INNER &B) ^lbl @recovery{(!B .)*}
```

`&B` requires `INNER` to consume up to a position where `B` matches
(turning silent prefix-match success into a `^lbl` catch fire);
`(!B .)*` resyncs by skipping bytes until `B` is reachable without
consuming it (so the outer rule still sees the structural
delimiter). The `@recovery{...}` wrap is part of the lowering — the
operator owns the capture-kind choice so authors don't have to type
it.

**FOLLOW-inferred form `INNER ^^lbl`.** Omitting `B` synthesizes the
boundary from `INNER`'s call-site FOLLOW set at compile time. Use it
when the call site has a single, unambiguous FOLLOW and you don't
need leading-`ws` permissiveness in the lookahead; the resolver
synthesizes a boundary pattern from the rule's FOLLOW elements and
applies the same lowering. The two `grammars/sqlite.peg` POC sites
use the explicit form because their boundary rules include `ws` and
that wouldn't fall out of FOLLOW.

**Scoping footgun.** The operator must sit *inside* any required
prefix the rule needs to commit to. Writing the operator around the
whole rule body — e.g.

```peg
where_clause <- kw_where ws expr ^^bad_where (ws boundary)
```

— would let the operator fire even when `kw_where` itself fails (no
WHERE keyword present): inner fails → recovery `(!boundary .)*`
matches zero bytes and succeeds → every clean `SELECT 1;` emits a
spurious empty `recovery` capture. Push the operator past the
unconditional prefix:

```peg
where_clause <- kw_where ws (expr ^^bad_where (ws boundary))
```

so a missing prefix fails the rule cleanly with no catch involved.

**Operator-family discriminator.** The doubled caret `^^` marks the
boundary-anchored family; the single caret `^lbl recovery` remains
the bare catch with author-written recovery. Disambiguation is
positional — a single byte peek after the first `^`.

Implementation lives in `Pattern::Catch` (`src/pegc/pattern.rs`) and
the emission in `src/pegc/compiler.rs`. The FOLLOW-inferred form
uses a placeholder `Pattern::InferBoundaryCatch { inner, label }`
resolved by `analysis::resolve_inferred_boundaries` before bytecode
emission. No new VM machinery.

### `INNER ^^lbl ..= B` — bracketed-close catch sugar

A variant of the boundary-anchored catch for the case where the
boundary is consumed *by the catch itself* (and re-captured with its
own kind), not left for the outer rule. The five `^block_close`
sites across the C / CSS / Go / JavaScript / Rust grammars are the
motivating shape: a `block` rule whose happy path ends in
`@punctuation{'}'}` and whose recovery skips bytes until the next
`}` and then matches it, with the closing brace tagged as
`@punctuation` in both paths.

```peg
INNER ^^lbl ..= B
# lowers to
INNER ^lbl @recovery{(!B .)*} B
```

Two ways it differs from `^^lbl B`:

- **No `&B` lookahead anchor on INNER.** Every corpus site has INNER
  already ending in `B` (the structural delimiter); adding `&B`
  would require a second `B` to follow. Authors who need a leniency
  anchor on a non-self-terminating INNER write it explicitly.
- **B is consumed inside the recovery, not by the outer rule.** The
  recovery body is a `Sequence` of `@recovery{(!B .)*}` then `B` —
  the skip is captured as `recovery`; `B` keeps whatever capture
  kind its pattern carries, *outside* the `@recovery` wrap.

Worked example (the `block` rule from `grammars/rust.peg`):

```peg
block <- @punctuation{'{'} ws (block_body ws @punctuation{'}'} ^^block_close ..= @punctuation{'}'})
```

The catch fires when `block_body ws @punctuation{'}'}` fails (a
malformed block). The recovery skips up to the next `}` and consumes
it as `@punctuation` — so themes that style recoveries differently
still render the closing brace as a brace, not as a recovery span.

The `..=` spelling matches the standalone consuming semantics from
the `.. S` / `..= S` operators above. The catch position accepts
only `..=`, not `..` — the catch necessarily consumes its boundary,
and `^^lbl .. B` would diverge from the standalone `..` non-consuming
meaning. The parser rejects it with a hint pointing at `..=`.

### `~name <-` — intentional-leniency marker

The static `lint_partial_match` check flags trailing-nullable rules
called unanchored — but on shipped grammars almost every flag is a
"partial-match leniency intentionally absorbed by outer `*^`-style
recovery" that the static analysis can't statically prove safe. The
`~name <-` marker is the author's intent signal: "yes, this rule's
leniency is known, don't flag any call to it." Wraps the rule's body
in `Pattern::Lenient`, which the lint walker treats as an opaque
barrier and the compiler treats as transparent (emits the inner's
bytecode unchanged).

```peg
~opt_semi <- (@punctuation{';'} ws)?
```

The marker must touch the name (no whitespace between `~` and the
identifier). Whether the runtime invariant the marker assumes —
typically an outer `*^` / `*^[;]` / `^block_close` recovery scope
absorbing the leniency — actually holds is currently a convention
recorded in adjacent comments and unenforced by the lint; see
`#113` for the planned tightening.

### Reserved rule names

Three rule names get compile-time treatment:

- **`root`** — the start rule (mandatory). The compiler wraps its
  body as `trivia? root_body trivia? !.` so end-of-input is always
  asserted, and a `trivia` rule (when present) pads the leading and
  trailing whitespace. The wrap means a grammar
  source like `root <- value` parses whole inputs, not just longest
  prefixes — the implicit `!.` rejects trailing junk.

- **`trivia`** — the optional auto-insertion target. When defined,
  the compiler injects a call to `trivia` between
  every pair of consecutive items in every non-atomic rule's
  `Sequence`, plus prepends one to each iteration of `*` / `+`.
  This replaces the explicit `ws` / `spacing` calls that used to
  appear between tokens.

  The rule also seeds the diagnostic *trivia cascade*: every rule
  transitively reachable from `trivia` through the call graph gets
  `Program::rule_is_trivia = true`, and `pegdb recoveries explain`
  pops trailing trivia frames from the rule_stack when picking the
  displayed leaf of each cluster. Rules whose body contains a
  recovery catch (`^lbl`, `^^lbl`, `*^`) are pinned out of the
  cascade so the catch's diagnostic frame stays visible.

  Indentation-sensitive grammars (`#43`) keep significant-whitespace
  rules outside the `trivia` subgraph; only ignorable bytes go in.

  ```peg
  trivia        <- (comment / \s)*
  comment       <- @comment{'//' .. '\n' / '/*' ..= '*/'}
  ```

  Grammars without a `trivia` rule (e.g. indent-sensitive shapes)
  get no auto-insertion and no trivia-padding wrap on `root`; the
  EOF assertion is still applied. To keep a callable whitespace rule
  while leaving auto-insertion off, name it anything other than
  `trivia` and thread it by hand — the `trivia` rule carries no
  qualifiers, so its mere presence turns auto-insertion on. Such a
  hand-threaded rule sits outside the diagnostic cascade, which is
  keyed on the name `trivia`.

  *Why the per-iteration prepend matters.* The auto-insertion
  splices `trivia` at every inter-item boundary of a `Sequence`, but
  `Repeat` iterations have no parent `Sequence` to splice into —
  inserting between items of the body covers one iteration's
  interior, not the boundary between iteration N and iteration
  N+1. So the rewriter also prepends a `trivia` call to the body of
  every `Repeat` / `RepeatOne`. Consider `pair (',' pair)*` parsing
  `pair, pair, pair` with spaces between every token. Without the
  prepend, the Repeat body is `',' trivia pair`: the first
  iteration's `,` matches at the position just after the outer
  Sequence's `pair trivia` — but the second iteration starts on a
  space, so its `,` rejects and the loop ends with one pair
  missing. With the prepend the body becomes `trivia ',' trivia
  pair`, and each iteration begins by consuming whatever
  inter-iteration whitespace is sitting in front of it.

- **`wb`** — the optional word-boundary target for `%` / `%?` rules.
  Its body is a bare boundary predicate (typically `wb <- !ident_body`,
  or `!ident_cont` for a Unicode-aware continuation class). The
  compiler appends a `wb` call inside the terminal captures of every
  `%` / `%?` rule (see [`%name <-`](#name----reserved-word-marker)); it
  is required only when the grammar has at least one such rule. Like
  `trivia`, `wb` is exempt from trivia auto-insertion and must sit in
  the reserved slots immediately after `root`. It is **not** part of
  the trivia diagnostic cascade.

  ```peg
  wb            <- !ident_body
  ```

### `*name <-` — atomic-rule marker

A `*` prefix on the rule name opts the rule out of `trivia`
auto-insertion: the rewriter walks the body but does not splice
`trivia` between `Sequence` items or prepend one to `Repeat`
iterations. The two prefix sigils compose: `~*name` and `*~name`
are both valid.

```peg
*string_lit   <- '"' (str_escape / !'"' .)* '"'
*ident        <- [A-Za-z_] [A-Za-z0-9_]*
```

Used on token-shape rules — string / number / char literals,
identifiers, multi-byte keyword spellings — where injecting trivia
between adjacent bytes would let whitespace appear inside the
token. The atomic boundary stops at `NonTerminal` calls: a
non-atomic rule called from inside an atomic body still gets
auto-insertion in its own body.

For a keyword rule that also needs a trailing word boundary, prefer
the [`%name <-`](#name----reserved-word-marker) sibling below — it is
atomic *and* supplies the boundary, so you don't hand-write `!ident_body`.

### `%name <-` — reserved-word marker

A `%` prefix marks a rule as a **reserved word**: it is compiled
atomic (like `*`) *and* the compiler appends a call to the `wb` rule
inside the rule's terminal captures, so the match must be followed by
a word boundary. This keeps a keyword from firing on the prefix of a
longer identifier — `if` must not match the start of `ifx`.

```peg
wb            <- !ident_body
%kw_if        <- @keyword{'if'}
%storage_spec <- @keyword{'typedef'} / @keyword{'extern'} / @keyword{'static'}
```

The `wb` call is pushed inside each leaf capture (and distributed
across choice branches), not appended after the rule body. That
placement matters: it means the `@keyword` capture only commits when
the boundary holds, so a rejected keyword leaves no stray capture for
top-level `*^` recovery to surface. Because the rule is atomic, no
`trivia` is spliced between the literal and the appended `wb` call.

When a `%` / `%?` rule's body is a **capture-less alternation of plain
literals**, the compiler prefix-factors it into a longest-match **trie**
(with `wb` at each accepting leaf) instead of a flat alternation. The
trie is order-independent, so a shorter keyword can't shadow a longer
one that extends it — `do` / `double`, `go` / `goto`, `int` / `int8`
all match maximally regardless of source order. (Case-insensitive
`i"…"` keyword sets lower to char-class sequences, not plain literals,
so they keep the flat form — the synthesizer below emits those
longest-first so maximal munch still holds.)

- Requires a `wb` rule; with none defined, the synthesized
  `NonTerminal("wb")` surfaces as `CompileError::UndefinedRule("wb")`.
- Composes with `~` (`%~name` / `~%name`); mutually exclusive with `*`
  (combining them is a parse error).
- Rejected on the reserved rules `root` / `trivia` / `wb`.

### `%?name <-` — preferred-word marker

A `%?` prefix (the `?` touches the `%`) is the sibling of `%` for
**preferred words**: identifier-eligible distinguished tokens such as
Go's predeclared `int` / `len` / `true` or JavaScript's contextual
`async` / `let`. It behaves exactly like `%` — atomic, with the same
`wb` boundary and the same trie treatment — and differs only in *which
synthesized set the rule's literals feed*: a `%?` rule's words go into
`preferred` and are **excluded** from `reserved`, so they stay usable as
identifiers.

```peg
%?predeclared_type <- 'int8' / 'int16' / 'int'   # Go: shadowable
%?kw_async         <- @keyword{'async'}          # JS: contextual
```

- Same requirements / composition / rejections as `%`.
- A word reachable from both a `%` and a `%?` rule is a contradiction
  (it can't be both barred from and allowed in identifier position) and
  raises `CompileError::ReservedPreferredConflict`.

### `reserved` / `preferred` — synthesized word sets

The compiler synthesizes two rules from the `%` / `%?` rules above:

- **`reserved`** — every literal of a `%` rule that is **not** `%?`.
  Reference it as `!reserved` in an identifier rule to bar keywords from
  identifier position without hand-maintaining a list:

  ```peg
  *ident <- !reserved [A-Za-z_] [A-Za-z0-9_]*
  ```

- **`preferred`** — every literal of a `%?` rule. Materialized for
  symmetry / inspection (e.g. `pegdb`); a grammar usually doesn't
  reference it.

Both are emitted as `%` rules (trie + `wb`), so `!reserved` does correct
maximal-munch: `int` is reserved, but `integer` — a longer word that
merely starts with it — is not. A rule whose body isn't a fixed keyword
shape (it has a quantifier / wildcard / predicate, e.g. a number body
like `'0x' [\da-fA-F]+`) contributes nothing; keep such boundary-only
helpers as `*name <- … wb` rather than `%`. Authors **reference** these
two names but never **define** them (a definition is a parse error).

### Reserved syntax

`^<non-ident-byte>` is a parse error today and reserved for future
overlays. Two extensions have been sketched and dropped from the v1
catch operator pending real-grammar evidence that they're needed:

- **Anonymous catch (`^_` / `^^_`).** A catch with no label.
  Always emits a placeholder diagnostic. Today's mandatory-label
  rule keeps `pegdb recoveries explain` clusters meaningful by
  forcing every recovery point to be named.
- **Throw atom (`^!label` / `^^!label`).** A zero-byte pattern that
  always fails with `label`, with Maidl-style cross-rule routing
  semantics. Useful for compiler frontends that want commitment
  semantics and targeted "expected X" diagnostics; less load-bearing
  for syntax highlighters where `*^` covers the common resync case.

If you hit a grammar that needs either, open an issue with the
concrete use case.

## Semantics notes

- **Ordered choice is biased.** `p1 / p2` is not regex `p1|p2`; `p2`
  is tried only if `p1` fails.
- **Predicates consume no input.** `!p` and `&p` rewind any `sp`
  advance `p` would have made, and emit no captures.
- **Left recursion is supported.** Both direct (`A <- A α / β`) and
  indirect (`A <- B …; B <- A …`) shapes parse left-associatively via
  bounded LR (Medeiros et al. 2014 §5; see
  [`src/pegvm/README.md`](../pegvm/README.md#left-recursion)). The
  compiler wraps every member of any non-trivial first-call SCC with
  the LR prologue/epilogue, and the VM's stack-based L-table handles
  cross-rule cycles transparently.
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
  that doesn't exist, partial-match leniency on a call site that's
  neither anchored (`^^lbl B`) nor explicitly marked intentional
  (`~name <-`), an `^^lbl` whose call-site FOLLOW set is empty
  so no boundary can be inferred.
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
