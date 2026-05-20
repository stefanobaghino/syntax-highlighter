# `pegc` and `pegdb` — grammar developer tools

Two binaries split along the conceptual line between **compile-time
inspection** of a grammar source and **runtime debugging** of a parse:

- **`pegc`** — compiler-side toolchain. Subcommands today: `stats`.
  Operates on a `<grammar.peg>` source; input-independent. Future
  seats for serialization (`pegc compile -o foo.bc`) and disassembly
  (`pegc disasm`) live here.
- **`pegdb`** — debug surface for grammar authors. Subcommands today:
  `dump-captures`, `explain-recoveries`. Both operate on a parse — they
  need a grammar source (`-g <grammar.peg>`) and a fixture input.

Both are distinct from the `demo` CLI at `src/bin/demo/`, which is a
quickstart showcase of ANSI highlighting; reach for `pegc`/`pegdb`
when you're authoring or diagnosing a grammar.

Build and run:

```
cargo run --bin pegc  -- <subcommand> [options] [args]
cargo run --bin pegdb -- <subcommand> [options] [args]
```

`pegc stats` and `pegdb dump-captures` emit **JSONL**: one JSON object
per `\n`-delimited line. Streamable, `jq`-composable, and trivially
decodable by any standard JSON parser.

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
- **`pegdb dump-captures`** — *the post-mortem*. Per-capture spans +
  kinds, byte-precise. Use when a kind-mismatch test fails or when
  you want to see exactly which spans the grammar produced over a
  given input: "what kind did the grammar give byte N?" → one `jq`
  filter answers it. On rows where `kind == "recovery"`, an
  additional `farthest_reach` object reports the deepest position
  reached by the failed `*^` iteration and the rule-call stack at
  that point — see the schema table below.
- **`pegdb explain-recoveries`** — *cluster view*. Same data source
  as the `farthest_reach` field, but rolled up per rule-stack: one
  line per cluster instead of one per recovery byte. Use when
  `dump-captures` is too noisy to read directly — e.g. a fixture
  with tens of thousands of recoveries collapses into a handful of
  bug-class lines sorted by count.

Walker correctness — that the renderer's segment stream tiles input
bytes exactly, with no gaps or overlaps — is a structural property of
the `walk` abstraction in `src/walk.rs` and is asserted by unit
tests inside that module. It is grammar-independent and does not
need a CLI surface.

---

# `pegc`

## `stats <grammar.peg>`

Compile a PEG grammar file and print its bytecode size.

**Synopsis:** `pegc stats <grammar.peg>`

**Output:** one JSON object on a single line.

| Field                 | Meaning                                                  |
|-----------------------|----------------------------------------------------------|
| `path`                | The grammar path passed in (echoed as a label).          |
| `instructions`        | `Program::code.len()` — bytecode-instruction count.      |
| `rules`               | `Program::rule_count` — number of rules in the grammar.  |
| `capture_kinds_count` | Number of distinct capture kinds the grammar declares.   |
| `capture_kinds`       | Array of capture-kind names, in declaration order.       |

**Stdin:** not accepted — `stats` always takes a `<grammar.peg>` path.

**Exit codes:** 0 success, 2 usage error, 3 grammar-compile error.

**Examples:**

```
$ pegc stats grammars/json.peg
{"path":"grammars/json.peg","instructions":197,"rules":16,"capture_kinds_count":5,"capture_kinds":["punctuation","property","string","number","constant"]}

# Compare across all shipped grammars (each file emits one line; jq tabulates):
$ for g in grammars/*.peg; do pegc stats "$g"; done \
    | jq -r '[.path, .instructions, .rules, .capture_kinds_count] | @tsv' \
    | column -t
```

---

# `pegdb`

## `dump-captures -g <grammar.peg> [--max-literal=N] [<path>]`

Print one capture per line as a JSON object. The byte-precise
diagnostic for "what spans got which kind?"

**Synopsis:** `pegdb dump-captures -g <grammar.peg> [--max-literal=N] [<path>]`

**Grammar:** required via `-g` / `--grammar` / `--grammar=<path>`; no
fallback. The grammar is read from disk and compiled fresh on every
invocation.

**Stdin:** read for the fixture input when no `<path>` is given.

**Output fields (each line is a JSON object):**

| Field            | Meaning                                                                           |
|------------------|-----------------------------------------------------------------------------------|
| `start`          | Byte offset of the capture's first byte in the input (inclusive).                |
| `end`            | Byte offset just past the capture's last byte (exclusive).                       |
| `kind`           | Capture-kind name (one of the names listed in `pegc stats … capture_kinds`).     |
| `depth`          | Nesting level: `0` for an outermost capture, `1+` for one nested inside another. |
| `literal`        | The captured bytes as a JSON string. Control bytes (including `0x1b` ESC) are escaped. |
| `farthest_reach` | **Recovery rows only.** `{"pos":N,"rule_stack":[...]}` — see *Recovery diagnostics* below. |

**Nesting:** PEG grammars can wrap a captured rule around another that
itself contains capture annotations — the inner capture sits inside
the outer in both range and emission order. Two of the eight shipped
grammars exercise this today: C `string_lit` wrapping `comment` (the
inter-piece whitespace between concatenated string literals can match
a comment), and Go `qualified_ident` (a `@type{...}` wrapping a
`@punctuation{'.'}`). `depth` makes the relationship explicit: filter
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

**Recovery diagnostics (`farthest_reach`).** On every row whose `kind`
is `"recovery"`, a `farthest_reach` object surfaces *why* the parser
fell into the `*^` byte-eater at that position:

| Sub-field    | Meaning                                                                              |
|--------------|--------------------------------------------------------------------------------------|
| `pos`        | Deepest byte offset reached by the failed `*^` iterations contributing to this contiguous recovery span. May sit anywhere relative to `end`; it's where the deepest dive happened, not where resync succeeded. |
| `rule_stack` | Array of rule names (root-to-leaf) at the moment `pos` was set. The leaf is the most actionable signal — "the deepest reach was inside rule X." |

A *recovery span* is a maximal contiguous run of `recovery`-kind
captures (`cap[i].end == cap[i+1].start`). Every row in the same span
carries identical `farthest_reach` data; consumers that want one
record per span can `jq 'del(.farthest_reach) | unique'` or use
`pegdb explain-recoveries` instead. The field is additive — older
consumers that filter on `kind` and ignore unknown fields keep
working.

**Partial-match handling:** when the parser doesn't reach `End` (the
input doesn't fully match the grammar), `dump-captures` still emits
all captures the VM produced over `input[..matched]` — that's the
diagnostic surface a grammar author needs when their grammar is broken.
A final stderr line of the form `partial-match <path-or-stdin>: matched M of L bytes`
follows, and the exit code is 1. Stdout stays a clean JSONL stream —
no trailing sentinel object.

**Exit codes:** 0 on full parse, 1 on partial parse, 2 on usage, 3 on
grammar-compile error.

**Examples:**

```
$ pegdb dump-captures -g grammars/json.peg benches/fixtures/small.json | head -3
{"start":0,"end":1,"kind":"punctuation","depth":0,"literal":"{"}
{"start":1,"end":5,"kind":"property","depth":0,"literal":"\"id\""}
{"start":5,"end":6,"kind":"punctuation","depth":0,"literal":":"}

# Find the capture covering byte 1234 in a Rust fixture:
$ pegdb dump-captures -g grammars/rust.peg benches/fixtures/medium.rs \
    | jq 'select(.start <= 1234 and .end > 1234)'

# Pipeline-friendly: cap literals to 40 bytes for tabular previews:
$ pegdb dump-captures -g grammars/rust.peg --max-literal=40 benches/fixtures/medium.rs \
    | jq -r '[.start, .end, .kind, .literal] | @tsv' | column -t

# Summarise distinct capture kinds emitted on a fixture:
$ pegdb dump-captures -g grammars/rust.peg benches/fixtures/medium.rs \
    | jq -r '.kind' | sort | uniq -c

# Inspect the deepest rule reached at each recovery span:
$ pegdb dump-captures -g grammars/rust.peg /tmp/broken.rs \
    | jq 'select(.kind == "recovery") | .farthest_reach'
```

---

## `explain-recoveries -g <grammar.peg> [<path>]`

Cluster `*^` recoveries by the rule-call stack reached during the
failed iteration, sorted by count descending. Use to collapse
thousands of `dump-captures` recovery rows into a handful of
bug-class lines.

**Synopsis:** `pegdb explain-recoveries -g <grammar.peg> [<path>]`

**Grammar / stdin:** same conventions as `dump-captures` —
`-g`/`--grammar`/`--grammar=<path>` required, stdin used when no
fixture path is given.

**Output:** one cluster per line on stdout in the form
`<count> recoveries — farthest reach ends at <rule>`, where `<rule>`
is the leaf of the rule stack the failed iterations reached deepest.
Clusters are sorted by `<count>` descending. When the parse produces
no recoveries the single line `no recoveries` is emitted.

**Exit codes:** 0 on full parse, 1 on partial parse, 2 on usage, 3 on
grammar-compile error.

**Examples:**

```
$ pegdb explain-recoveries -g grammars/rust.peg /tmp/broken.rs
5 recoveries — farthest reach ends at rust_file
4 recoveries — farthest reach ends at line_comment
1 recoveries — farthest reach ends at ws

# Pipe to `head` to see the top bug class only:
$ pegdb explain-recoveries -g grammars/go.peg benches/fixtures/xlarge.go | head -1
```

The cluster key is the full rule stack — two recoveries are in the
same cluster iff they have identical `farthest_reach.rule_stack`
values. Future work may add suffix-clustering (`--suffix=N` keeps
only the last N frames as the key); v1 is full-stack only.

---

## What lives in `--help` vs. this file

Each binary's `--help` strings (top-level and per-subcommand) are
intentionally one-liner usage summaries. The full contract — JSONL
field schemas, exit-code semantics, partial-match behavior — lives
here. Keeps the binary's help output short and the contract in one
durable place.
