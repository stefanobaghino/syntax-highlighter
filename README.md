# syntax-highlighter

A compact syntax highlighting engine.

A syntax highlighting library that compiles
[PEG](https://en.wikipedia.org/wiki/Parsing_expression_grammar) grammars
to bytecode and executes them on a small virtual machine. Grammars are
data, not generated code — adding a language means adding a grammar
file, not recompiling the library. Highlight annotations live directly
inside the grammar, so each grammar is a self-contained highlighting
specification. The goal is a highlighter with a tiny, fixed-size runtime
that can scale to many languages without binary bloat.

## Why this approach

The aim is a small, efficient runtime with compact grammars and
acceptable performance for the syntax-highlighting workload. The
intent is to minimize binary footprint — for both the runtime and the
grammars — while achieving performance adequate for tooling use.

The design goals:

- **Grammars compile to compact bytecode.** The goal is that most
  languages stay in the kilobyte range per grammar; see Status below
  for per-grammar counts.
- **The VM is the only compiled code; languages are pure data.** Adding,
  swapping, or generating a language grammar — including at runtime, or by
  an LLM — should not require recompiling the library.
- **Highlight annotations live inline in the grammar.** A single file
  defines both the syntax and its coloring — one artifact per language, not
  a grammar plus a separate theme/scope mapping.
- **A joyful grammar-authoring experience.** The surface language
  should be pleasant to write and easy to read; tooling should help
  grammar authors — humans and AI agents alike — draft new grammars
  and maintain existing ones without fighting the format.
- **Zero dependencies today.** The crate uses only `std`. Adding one
  needs a real justification; for `pegvm` in particular, the intent
  is a lean, dependency-free API that a future FFI layer can wrap
  without fighting Rust-only abstractions. `no_std` compatibility is
  not an active goal.

## Quick demo

Run against any of the shipped bench fixtures — the grammar is
picked by file extension. Complete grammars:

```bash
cargo run -- benches/fixtures/medium.json
cargo run -- benches/fixtures/medium.toml
cargo run -- benches/fixtures/medium.sql
```

Partial grammars (see the Status table below for the tracking issue
per language) cover most real-world code but have documented gaps —
expect the occasional miscoloring at the edges:

```bash
cargo run -- benches/fixtures/medium.css
cargo run -- benches/fixtures/medium.c
cargo run -- benches/fixtures/medium.js
cargo run -- benches/fixtures/medium.go
cargo run -- benches/fixtures/medium.rs
```

Stdin is also accepted (defaults to JSON; pass `-l <lang>` to
override).

## Status

Early MVP; expect breaking changes. Eight grammars ship today, plus
an ANSI-coloring CLI. The "kilobyte range per grammar" design goal
from *Why this approach* above holds across the shipped set:

| Language   | Status         | Rules | Instructions |
|------------|----------------|------:|-------------:|
| JSON       | complete       |    16 |          197 |
| TOML       | complete       |    59 |          629 |
| CSS        | partial (#34)  |    55 |          688 |
| C          | partial (#33)  |   102 |        2,483 |
| JavaScript | partial (#31)  |   137 |        2,916 |
| Go         | partial (#32)  |   145 |        3,118 |
| Rust       | partial (#30)  |   172 |        3,413 |
| SQLite     | complete       |   426 |        5,622 |

The CLI picks a grammar by file extension (`.json`, `.toml`, `.sql`,
`.rs`, `.js`/`.mjs`/`.cjs`, `.go`, `.c`/`.h`, `.css`) or an explicit
`-l json|toml|sql|rust|js|go|c|css` flag; stdin without a flag
defaults to JSON.

Malformed or incomplete inputs render the longest valid prefix styled and
the rest plain (see `src/pegvm/vm.rs` for the farthest-failure tracking).
Grammars that opt into the `*^` recovery operator on a top-level
repetition resync past broken regions instead — see the SQL grammar
(`grammars/sqlite.peg`) for the shipped example.

`Highlighter` owns its input and reuses the memo table across edits:
append-only streaming and arbitrary-position edits reparse only the
regions whose memo entries cross the edit point, not the whole buffer
(see `src/pegvm/incremental.rs` for the invalidation protocol).

## Grammar format

See `grammars/json.peg` for a complete example. The format follows standard
PEG notation with two extensions: `@name{pattern}` declares a capture with
the given highlight name (mapped to ANSI colors by the built-in theme in
`src/highlight/theme.rs`), and `p*^` / `p+^` mark a repetition for
skip-byte error recovery (see `Pattern::RecoverRepeat` in
`src/pegc/pattern.rs` and the emission pattern in `src/pegc/compiler.rs`).

## Roadmap

The architecture is designed so that every item below can be added without
rewriting the core. The list is grouped by prerequisite, from smallest to
largest piece of work.

### Standalone additions

These need no new VM or compiler machinery.

- **More language grammars.** JSON, TOML, the full SQLite dialect,
  a pragmatic Rust subset, an ES2020-ish JavaScript subset, a Go
  subset, a lex-level C subset, and a CSS grammar (including
  nesting) ship today. Adding a language means writing a grammar
  file. SQLite remains the large-grammar bytecode-size datapoint
  the design goals called for: 5,627 VM instructions across 426
  rules — an order of magnitude larger than JSON (165 instr), CSS
  (688 instr), or TOML (511 instr); Rust (3,413 instr / 172 rules),
  Go (3,118 instr / 145 rules), JavaScript (2,916 instr / 137
  rules), and C (2,483 instr / 102 rules) sit between, still
  comfortably inside the "kilobyte range per grammar" ambition.
  Remaining candidates: Python, TypeScript. YAML is deferred —
  its context-sensitive indentation semantics make it a poor fit
  for PEG without additional machinery.
- **User-configurable themes.** The capture-name → ANSI mapping is
  hard-coded today; lifting it to a theme file (TOML or similar) is
  orthogonal to the VM.
- **Non-ANSI output backends.** The renderer's contract is already
  "captures in, styled output out"; an HTML backend is a parallel
  implementation of `render()`.
- **Editor-tooling adapters.** A Language Server Protocol shim,
  editor-specific plugins, or a CST representation tuned for editor
  latency targets. Builds on the memoization work below. Significant
  scope and not currently a priority.

### VM and compiler extensions

Each item is a self-contained feature that unlocks further work.

- **Left recursion support.** Not needed for JSON, but natural for some
  grammar idioms (especially arithmetic-expression grammars). The
  intended reference is Medeiros, Mascarenhas & Ierusalimschy 2014 —
  §5 gives a parsing-machine extension matching ours, and its
  bounded-recursion L table is stack-structured and separate from the
  packrat memo, so it composes cleanly with the existing memo-threshold
  filter. Warth/Douglass/Millstein 2008 is the seminal paper and the
  most widely cited approach, but it couples seed-and-grow to the memo
  table itself — which would fight the threshold filter — and inherits
  the nullable-LR bugs Medeiros 2014 §6 documents. Integration note:
  the L table must be unwound through the same `fail()` path that
  already commits `Memo` frames and that the shipped `*^` recovery
  operator relies on.
- **Explicit memoization opt-out.** Maybe. Per-rule or per-expression
  annotation (e.g. `Rule <-! body` or `{{! expr }}`) disabling caching
  on named hot spots. Grammar authors should not have to reason about
  cache strategy — that is the library's job — so this is reserved as
  an escape hatch, not a default. Add it only if, on top of the
  memo-threshold filter, profiling still points at specific rules
  where rematch beats lookup. Inverts GPeg's `{{ p }}` opt-in at this
  level, which maps cleanly if the need arises.
- **Bytecode serialization.** Pre-compile grammars once and ship the
  `Vec<Instruction>` plus capture-name table as data. Useful for embedded
  distributions and startup-time-sensitive consumers. Landing this should
  also expose a `Parser::from_program(Program)` constructor so the deep
  `parser` module accepts pre-compiled bytecode without callers falling
  back to the `pegvm` primitives.

### Self-hosting — parsing PEG grammars using pegvm itself

The long-form project: replace the hand-written recursive-descent parser
in `src/pegc/parser.rs` with a PEG grammar that describes PEG grammar
syntax itself, executed by the VM. Prerequisites, in order:

1. **Semantic actions / typed captures.** Today's VM emits flat
   `Capture { kind, start, end }` records. To rebuild a typed AST from a
   parse, captures must be able to produce user values (e.g. a `Pattern`
   node) rather than spans. LPEG calls these *function captures*;
   Yedidia's thesis discusses them. This is a real VM feature, not a
   refactor. Interacts with memoization: a memo entry's `captures`
   field today stores `Vec<Capture>` (plain spans); with typed captures
   it becomes a vector of user values, which will need to be
   clone-able or reference-counted to replay from cache.
2. **Write `peg.peg`.** A PEG grammar that describes PEG grammar syntax.
   A few dozen rules; the language is small.
3. **Hand-compile `peg.peg` to bytecode.** The bootstrap artifact: a
   `const BOOTSTRAP_PROGRAM` baked into the source so the very first
   parse has something to run. From that point on, any change to the
   grammar language requires coordinated edits to both `peg.peg` and
   the baked-in bytecode.
4. **Replace the recursive-descent parser.** `pegc::parse(src)` becomes
   `VM::run(BOOTSTRAP_PROGRAM, src)` with semantic actions that build the
   `Pattern` AST.

Payoff: removes ~400 lines of hand-written parsing, stress-tests the VM
against a non-trivial grammar, and makes future grammar-language
extensions a grammar edit rather than a code edit.

## More documentation

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — how the codebase is organized:
  module split, boundaries, design ethos. Includes pointers to
  per-module deep documentation.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — how to make changes: commands,
  project-level rules, code conventions. Applies to anyone touching the
  code, human or AI agent.
- [`AGENTS.md`](AGENTS.md) — entry point for AI coding agents (mostly a
  pointer to the above).

## License

Dual-licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual-licensed as above, without any additional terms
or conditions.
