# `pegc` and `pegdb` — grammar developer tools

Two binaries split along the conceptual line between **compile-time
inspection** of a grammar source and **runtime debugging** of a parse:

- **`pegc`** — compiler-side toolchain. Subcommands today: `stats`.
  Operates on a `<grammar.peg>` source; input-independent. Future
  seats for serialization (`pegc compile -o foo.bc`) and disassembly
  (`pegc disasm`) live here.
- **`pegdb`** — debug surface for grammar authors. Noun-verb
  subcommands today: `captures dump`, `recoveries dump`,
  `recoveries explain`. Each operates on a parse — it needs a grammar
  source (`-g <grammar.peg>`) and a fixture input.

Both are distinct from the `demo` CLI at `src/bin/demo/`, which is a
quickstart showcase of ANSI highlighting; reach for `pegc`/`pegdb`
when you're authoring or diagnosing a grammar.

Build and run:

```
cargo run --bin pegc  -- <subcommand> [options] [args]
cargo run --bin pegdb -- <noun> <verb> [options] [args]
```

`pegc stats`, `pegdb captures dump`, and `pegdb recoveries dump` emit
**JSONL**: one JSON object per `\n`-delimited line. Streamable,
`jq`-composable, and trivially decodable by any standard JSON parser.

`pegdb` is a debug tool for *any* grammar source: every fixture-taking
subcommand requires `-g <grammar.peg>` (or its long form `--grammar`,
also accepting `--grammar=<path>`). There is no language-name
shortcut and no extension inference — specify the path explicitly.
The bundled `grammars/*.peg` files are convenient targets for local
debugging but enjoy no special privilege; the `demo` CLI in
`src/bin/demo/` is the showcase for those.

---

## When to reach for which subcommand

- **`pegc stats`** — *static* shape report. Answers "how big is this
  grammar's bytecode?" Useful as a sanity check after non-trivial
  grammar edits. Input-independent.
- **`pegdb captures dump`** — *per-capture post-mortem*. Spans + kinds
  for every capture the parser produced, byte-precise. Use when a
  kind-mismatch test fails or when you want to see exactly which
  spans the grammar produced over a given input: "what kind did the
  grammar give byte N?" → one `jq` filter answers it. Recovery
  diagnostics (label, deepest reach, trimmed rule stack) live in the
  `recoveries` subcommands — `captures dump` is intentionally lean.
- **`pegdb recoveries dump`** — *per-span recovery detail*. One row
  per maximal contiguous recovery span, with the capture half (byte
  range, literal) and the diagnostic half (label, deepest position,
  trimmed rule stack) inline. The first place to look when a recovery
  fires unexpectedly or doesn't fire when expected. Surfaces
  capture⇄diagnostic accounting mismatches as `"sanity":"orphan_*"`
  rows when present — silent in well-formed runs.
- **`pegdb recoveries explain`** — *cluster summary*. Same underlying
  data as `recoveries dump`, rolled up per `(rule_stack, label)` with
  counts sorted descending. Use when `recoveries dump` is too noisy —
  e.g. a fixture with hundreds of recoveries collapses into a handful
  of bug-class lines. Surfaces accounting mismatches as
  `[sanity]`-prefixed lines when present.

Walker correctness — that the renderer's segment stream tiles input
bytes exactly, with no gaps or overlaps — is a structural property of
the `walk` abstraction in `src/walk.rs` and is asserted by unit
tests inside that module. It is grammar-independent and does not
need a CLI surface.

---

# `pegc`

## `stats <grammar.peg>`

Compile a PEG grammar file and print its bytecode size plus a per-rule
call-graph summary.

**Synopsis:** `pegc stats <grammar.peg>`

**Output:** a JSON header line followed by one NDJSON record per rule.

Header fields:

| Field                 | Meaning                                                  |
|-----------------------|----------------------------------------------------------|
| `path`                | The grammar path passed in (echoed as a label).          |
| `instructions`        | `Program::code.len()` — bytecode-instruction count.      |
| `rules`               | `Program::rule_count` — number of rules in the grammar.  |
| `capture_kinds_count` | Number of distinct capture kinds the grammar declares.   |
| `capture_kinds`       | Array of capture-kind names, in declaration order.       |

Per-rule record fields (one object per rule, sorted alphabetically by `rule`;
every rule appears, including unreferenced ones):

| Field        | Meaning                                                                      |
|--------------|------------------------------------------------------------------------------|
| `rule`       | Rule name.                                                                   |
| `references` | Total occurrences of `<rule>` as a `NonTerminal` across every rule's body.   |
| `body_chars` | Source character count of the rule's body (after `=`), trimmed.             |

`references` counts **author-written** `NonTerminal` calls in rule bodies. The auto-injected `trivia` calls produced by `src/pegc/analysis.rs::inject_auto_trivia` are not counted, so the reserved `trivia` rule typically reports `references: 0` even though it is the most-invoked rule at runtime. Use the count to find dead or single-use rules in *authored* grammar source; do not read it as runtime call frequency.

**Stdin:** not accepted — `stats` always takes a `<grammar.peg>` path.

**Exit codes:** 0 success, 2 usage error, 3 grammar-compile error.

**Examples:**

```
$ pegc stats grammars/json.peg
{"path":"grammars/json.peg","instructions":205,"rules":12,"capture_kinds_count":5,"capture_kinds":["punctuation","property","string","number","constant"]}
{"rule":"array","references":1,"body_chars":117}
{"rule":"exp","references":1,"body_chars":15}
...

# Compare bytecode size across all shipped grammars (use jq to drop the
# per-rule lines; the header is always the first line):
$ for g in grammars/*.peg; do pegc stats "$g" | head -1; done \
    | jq -r '[.path, .instructions, .rules, .capture_kinds_count] | @tsv' \
    | column -t

# Inlining-audit recipe: list rules referenced exactly once (the per-rule
# stream lives after the header line, so skip line 1):
$ pegc stats grammars/sqlite.peg | tail -n +2 \
    | jq -r 'select(.references==1) | .rule'

# Dead-code recipe: rules that nothing calls (excluding the start rule,
# which is invoked by the runtime entry point rather than a NonTerminal):
$ pegc stats grammars/sqlite.peg | tail -n +2 \
    | jq -r 'select(.references==0) | .rule'
```

---

# `pegdb`

`pegdb` uses a noun-verb subcommand layout. Each noun is a kind of
parse-event observation; each verb is a presentation mode.

```
pegdb captures dump        # JSONL, one row per capture
pegdb recoveries dump      # JSONL, one row per recovery span
pegdb recoveries explain   # plain text, recovery clusters by count
```

---

## `captures dump -g <grammar.peg> [--max-literal=N] [<path>]`

Print one capture per line as a JSON object. The byte-precise
diagnostic for "what spans got which kind?"

**Synopsis:** `pegdb captures dump -g <grammar.peg> [--max-literal=N] [<path>]`

**Grammar:** required via `-g` / `--grammar` / `--grammar=<path>`; no
fallback. The grammar is read from disk and compiled fresh on every
invocation.

**Stdin:** read for the fixture input when no `<path>` is given.

**Output fields (each line is a JSON object):**

| Field     | Meaning                                                                           |
|-----------|-----------------------------------------------------------------------------------|
| `start`   | Byte offset of the capture's first byte in the input (inclusive).                |
| `end`     | Byte offset just past the capture's last byte (exclusive).                       |
| `kind`    | Capture-kind name (one of the names listed in `pegc stats … capture_kinds`).     |
| `depth`   | Nesting level: `0` for an outermost capture, `1+` for one nested inside another. |
| `literal` | The captured bytes as a JSON string. Control bytes (including `0x1b` ESC) are escaped. |

Recovery diagnostics (label, deepest position, rule stack) are *not*
emitted on `captures dump` rows — that information lives in
`recoveries dump` and `recoveries explain`, where one row per span
collapses the per-byte recovery captures into a single record with
the full diagnostic context inline.

**Nesting:** PEG grammars can wrap a captured rule around another that
itself contains capture annotations — the inner capture sits inside
the outer in both range and emission order. Two of the eight shipped
grammars exercise this today: C `string_lit` wrapping `comment` (the
inter-piece whitespace between concatenated string literals can match
a comment), and Go `qualified_ident` (a `@type ...` wrapping a
`@punctuation '.'`). `depth` makes the relationship explicit: filter
`select(.depth == 0)` for outermost-only, or
`select(.start <= N and .end > N)` for everything covering byte `N`
regardless of nesting.

The `literal` field is a plain JSON string. Control bytes below `0x20`
are escaped as `\n`, `\t`, `\u00XX`, etc.; the byte `0x1b` (ESC)
falls in that range, so a stray escape sequence captured from the
input cannot recolor the consumer's terminal. Round-trippable for
arbitrary bytes via any JSON parser.

**`--max-literal=N`** truncates the literal at or before byte `N` on a
UTF-8 char boundary, appending a `…` ellipsis before JSON-encoding. No
truncation by default; agent consumers want exact bytes.

**Partial-match handling:** when the parser doesn't reach `End` (the
input doesn't fully match the grammar), `captures dump` still emits
all captures the VM produced over `input[..matched]` — that's the
diagnostic surface a grammar author needs when their grammar is broken.
A final stderr line of the form `partial-match <path-or-stdin>: matched M of L bytes`
follows, and the exit code is 1. Stdout stays a clean JSONL stream —
no trailing sentinel object.

**Exit codes:** 0 on full parse, 1 on partial parse, 2 on usage, 3 on
grammar-compile error.

**Examples:**

```
$ pegdb captures dump -g grammars/json.peg benches/fixtures/small.json | head -3
{"start":0,"end":1,"kind":"punctuation","depth":0,"literal":"{"}
{"start":1,"end":5,"kind":"property","depth":0,"literal":"\"id\""}
{"start":5,"end":6,"kind":"punctuation","depth":0,"literal":":"}

# Find the capture covering byte 1234 in a Rust fixture:
$ pegdb captures dump -g grammars/rust.peg benches/fixtures/medium.rs \
    | jq 'select(.start <= 1234 and .end > 1234)'

# Pipeline-friendly: cap literals to 40 bytes for tabular previews:
$ pegdb captures dump -g grammars/rust.peg --max-literal=40 benches/fixtures/medium.rs \
    | jq -r '[.start, .end, .kind, .literal] | @tsv' | column -t

# Summarise distinct capture kinds emitted on a fixture:
$ pegdb captures dump -g grammars/rust.peg benches/fixtures/medium.rs \
    | jq -r '.kind' | sort | uniq -c
```

---

## `recoveries dump -g <grammar.peg> [--max-literal=N] [<path>]`

Print one JSON object per surviving recovery span. The detail view
for "what did the parser actually recover from, and where?"

**Synopsis:** `pegdb recoveries dump -g <grammar.peg> [--max-literal=N] [<path>]`

**Grammar / stdin:** same conventions as `captures dump`.

**Output fields:**

| Field        | Meaning                                                                              |
|--------------|--------------------------------------------------------------------------------------|
| `start`      | First byte of the span (inclusive).                                                  |
| `end`        | One past the last byte of the span (exclusive).                                      |
| `kind`       | Capture-kind name — always `"recovery"`. Emitted for symmetry with `captures dump`. |
| `label`      | The catch's label name — author-supplied for `^label`, or `recovery_kind`'s intern for bare `*^`. |
| `pos`        | Deepest byte offset reached by the failed iterations that contributed to this span. May sit anywhere relative to `end` — it's where the deepest dive happened, not where resync succeeded. |
| `rule_stack` | The full trivia-trimmed call stack at the moment `pos` was set, root-to-leaf. Trailing frames reached from the reserved `trivia` rule are popped; the leaf is the deepest semantically interesting rule. |
| `literal`    | The span's bytes as a JSON string (subject to `--max-literal=N`).                   |

A *recovery span* is a maximal contiguous run of `kind == "recovery"`
captures touching end-to-start (`cap[i].end == cap[i+1].start`). The
`*^` loop emits one single-byte recovery capture per failed iteration;
`recoveries dump` collapses adjacent ones into a single row carrying
the argmax-`pos` diagnostic across the span.

**`--max-literal=N`** truncates `literal` like `captures dump`.

**Accounting cross-check.** When the capture-half and diagnostic-half
of a recovery event disagree (a class of bug that previously hid in
the gap between `captures dump`'s per-byte rows and the cluster
summary), `recoveries dump` appends extra rows with a discriminating
`sanity` key:

| Sanity row                  | Meaning                                                                   |
|-----------------------------|---------------------------------------------------------------------------|
| `{"sanity":"orphan_capture",…}`    | A `kind == "recovery"` capture survived but had no matching diagnostic. |
| `{"sanity":"orphan_diagnostic",…}` | A diagnostic survived but its `capture_index` doesn't point at any span. |

Silent in well-formed runs — both lists are empty under correct VM
state. Consumers that want only the data rows can filter with
`jq 'select(has("sanity") | not)'`.

**Partial-match handling and exit codes:** same as `captures dump`.

**Examples:**

```
$ pegdb recoveries dump -g grammars/rust.peg /tmp/broken.rs
{"start":42,"end":58,"kind":"recovery","label":"bad_column","pos":53,"rule_stack":["sql_file","sql_stmt","result_column"],"literal":"IN leaf AS leaf"}
{"start":120,"end":121,"kind":"recovery","label":"recovery","pos":120,"rule_stack":["rust_file"],"literal":"@"}

# Just the labels and where they fired:
$ pegdb recoveries dump -g grammars/rust.peg /tmp/broken.rs \
    | jq -r 'select(has("sanity") | not) | [.start, .end, .label, .rule_stack[-1]] | @tsv'

# Catch any accounting drift in CI:
$ pegdb recoveries dump -g grammars/rust.peg testdata/regression.rs \
    | jq -e 'select(has("sanity"))' && echo "DRIFT" || echo "clean"
```

---

## `recoveries explain -g <grammar.peg> [<path>]`

Cluster `*^` recoveries by the rule-call stack reached during the
failed iteration, sorted by count descending. Use to collapse
thousands of recovery rows into a handful of bug-class lines.

**Synopsis:** `pegdb recoveries explain -g <grammar.peg> [<path>]`

**Grammar / stdin:** same conventions as `captures dump`.

**Output:** one cluster per line on stdout in the form
`<count> recoveries — farthest reach ends at <rule> (label: <name>)`,
where `<rule>` is the leaf of the trivia-trimmed rule stack the failed
iterations reached deepest, and `<name>` is the catch label.
Clusters are sorted by `<count>` descending. When the parse produces
no recoveries and no orphan rows, the single line `no recoveries` is
emitted.

**Accounting cross-check.** Same span-aggregation drift `recoveries
dump` surfaces as JSONL `sanity` rows shows up here as
`[sanity]`-prefixed lines appended after the cluster output:

```
[sanity] 2 orphan recovery captures (no diagnostic): bytes 42..43, 88..89
[sanity] orphan diagnostic (no surviving capture): pos=57 rule=expression label=block_close
```

Silent in well-formed runs.

**Exit codes:** 0 on full parse, 1 on partial parse, 2 on usage, 3 on
grammar-compile error.

**Examples:**

```
$ pegdb recoveries explain -g grammars/rust.peg /tmp/broken.rs
5 recoveries — farthest reach ends at rust_file (label: recovery)
2 recoveries — farthest reach ends at block (label: block_close)

# Pipe to `head` to see the top bug class only:
$ pegdb recoveries explain -g grammars/go.peg benches/fixtures/xlarge.go | head -1
```

The cluster key is the full trivia-trimmed rule stack plus the catch
label — two recoveries are in the same cluster iff they reached the
same leaf via the same intermediate frames under the same label.

---

## What lives in `--help` vs. this file

Each binary's `--help` strings (top-level, per-noun, per-verb) are
intentionally one-liner usage summaries. The full contract — JSONL
field schemas, exit-code semantics, partial-match behavior — lives
here. Keeps the binary's help output short and the contract in one
durable place.
