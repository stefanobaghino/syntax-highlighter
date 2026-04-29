# `pegc` and `pegdb` — grammar developer tools

Two binaries split along the conceptual line between **compile-time
inspection** of a grammar source and **runtime debugging** of a parse:

- **`pegc`** — compiler-side toolchain. Subcommands today: `stats`.
  Operates on a `<grammar.peg>` source; input-independent. Future
  seats for serialization (`pegc compile -o foo.bc`) and disassembly
  (`pegc disasm`) live here.
- **`pegdb`** — debug surface for grammar authors. Subcommands today:
  `dump-captures`. Operates on a parse — needs both a grammar source
  (`-g <grammar.peg>`) and a fixture input.

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
  filter answers it.

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

| Field     | Meaning                                                                           |
|-----------|-----------------------------------------------------------------------------------|
| `start`   | Byte offset of the capture's first byte in the input (inclusive).                |
| `end`     | Byte offset just past the capture's last byte (exclusive).                       |
| `kind`    | Capture-kind name (one of the names listed in `pegc stats … capture_kinds`).     |
| `depth`   | Nesting level: `0` for an outermost capture, `1+` for one nested inside another. |
| `literal` | The captured bytes as a JSON string. Control bytes (including `0x1b` ESC) are escaped. |

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
```

---

## What lives in `--help` vs. this file

Each binary's `--help` strings (top-level and per-subcommand) are
intentionally one-liner usage summaries. The full contract — JSONL
field schemas, exit-code semantics, partial-match behavior — lives
here. Keeps the binary's help output short and the contract in one
durable place.
