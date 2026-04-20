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

## Status

Early MVP. Ships JSON, TOML, and SQLite-SELECT grammars and an
ANSI-coloring CLI. The properties described in *Why this approach*
below are **design goals that this MVP is trying to prove** — the
architecture supports them, and cross-language validation spans three
grammar shapes (JSON, TOML, and a backtracking-heavy SQL SELECT
subset). Expect breaking changes.

The CLI picks a grammar by file extension (`.json`, `.toml`, `.sql`)
or an explicit `-l json|toml|sql` flag; stdin without a flag defaults
to JSON.

Malformed or incomplete inputs render the longest valid prefix styled and
the rest plain (see `src/pegvm/vm.rs` for the farthest-failure tracking).

## Why this approach

The aim is a small, efficient runtime with compact grammars and
acceptable performance for the syntax-highlighting workload. The
intent is to minimize binary footprint — for both the runtime and the
grammars — while achieving performance adequate for tooling use.

The design goals:

- **Grammars compile to compact bytecode.** The MVP's JSON grammar fits in a
  few hundred instructions; the ambition is that most languages stay in the
  kilobyte range per grammar.
- **The VM is the only compiled code; languages are pure data.** Adding,
  swapping, or generating a language grammar — including at runtime, or by
  an LLM — should not require recompiling the library.
- **Highlight annotations live inline in the grammar.** A single file
  defines both the syntax and its coloring — one artifact per language, not
  a grammar plus a separate theme/scope mapping.

The MVP is the first implementation of this architecture and exercises it
against JSON. Demonstrating the goals across more languages — and
quantifying the resulting binary footprint — is the work ahead.

## Quick demo

```bash
echo '{"key": [1, true, "hello"], "nested": {"a": null}}' | cargo run
```

Or point it at a file (grammar picked by extension):

```bash
cargo run -- path/to/file.json
cargo run -- Cargo.toml
```

For stdin with a non-default language, pass `-l`:

```bash
printf '[package]\nname = "demo"\n' | cargo run -- -l toml
printf 'SELECT id FROM users WHERE active;\n' | cargo run -- -l sql
```

## Grammar format

See `grammars/json.peg` for a complete example. The format follows standard
PEG notation with a single extension: `@name{pattern}` declares a capture
with the given highlight name. Names are mapped to ANSI colors by the
highlighter's built-in theme (see `src/highlight/theme.rs`).

## Roadmap

The architecture is designed so that every item below can be added without
rewriting the core. The list is grouped by prerequisite, from smallest to
largest piece of work.

### Standalone additions

These need no new VM or compiler machinery.

- **More language grammars.** JSON, TOML, and a SQLite SELECT subset
  ship today. Adding a language means writing a grammar file.
  Expanding the SQLite grammar to the full dialect (INSERT/UPDATE/
  DELETE/DDL/triggers) is the intended vehicle for two roadmap
  deliverables it is uniquely positioned to support: a large-grammar
  bytecode-size datapoint (JSON and TOML are both compact formats) and
  a multi-statement error-recovery use case (single-SELECT does not
  exercise it). Remaining candidates beyond SQLite: Python, Rust, CSS.
  YAML is deferred — its context-sensitive indentation semantics make
  it a poor fit for PEG without additional machinery.
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

- **Threshold-based memoization.** Runtime filter on `MemoClose`: skip
  inserting an entry whose matched span is shorter than some byte
  threshold. Tiny leaf rules pay the lookup cost without the storage
  win, and empirically that is where most of the memo-table memory is
  wasted (Yedidia thesis §5.2.4 — gains flatten around 4096 bytes;
  GPeg defaults to 512 and benchmarks at 128). No grammar changes, one
  configurable constant. Needs a benchmark harness first to pick a
  sensible default. Listed before incremental parsing because it is
  what makes a persistent memo table memory-feasible — GPeg's
  rationale for having this filter at all.
- **Incremental parsing.** O(Δ) per input change instead of O(n), by
  invalidating only the memo entries whose spans cross the edit point
  (Yedidia's thesis, Ch. 4). Covers both append-only streaming (an LLM
  response arriving character-by-character in a TUI) and arbitrary-
  position edits (a cursor insertion or deletion anywhere in the input).
  Builds on the memoization layer — the memo table is the substrate;
  this task adds the public edit/diff API and the invalidation protocol.
  Depends on threshold-based memoization above for memory feasibility:
  without the threshold, every rule call at every position accumulates
  an entry in a table that must survive across edits.
- **Left recursion support.** Not needed for JSON, but natural for some
  grammar idioms (especially arithmetic-expression grammars). The
  intended reference is Medeiros, Mascarenhas & Ierusalimschy 2014 —
  §5 gives a parsing-machine extension matching ours, and its
  bounded-recursion L table is stack-structured and separate from the
  packrat memo, so it composes cleanly with threshold-based memoization
  above. Warth/Douglass/Millstein 2008 is the seminal paper and the
  most widely cited approach, but it couples seed-and-grow to the memo
  table itself — which would fight the threshold filter — and inherits
  the nullable-LR bugs Medeiros 2014 §6 documents.
- **Explicit memoization opt-out.** Maybe. Per-rule or per-expression
  annotation (e.g. `Rule <-! body` or `{{! expr }}`) disabling caching
  on named hot spots. Grammar authors should not have to reason about
  cache strategy — that is the library's job — so this is reserved as
  an escape hatch, not a default. Add it only if, after threshold-based
  memoization, profiling still points at specific rules where rematch
  beats lookup. Inverts GPeg's `{{ p }}` opt-in at this level, which
  maps cleanly if the need arises.
- **Bytecode serialization.** Pre-compile grammars once and ship the
  `Vec<Instruction>` plus capture-name table as data. Useful for embedded
  distributions and startup-time-sensitive consumers.
- **Error recovery / parsing past syntax errors.** PEG is strict by
  default — a parse either succeeds or fails. For syntax highlighting,
  the input is frequently incomplete or malformed (mid-edit buffers,
  streaming responses, in-progress code). The ability to recover past
  a syntax error and continue highlighting the rest of the input is
  important; this is a real extension to the VM and a known research
  area in PEG implementations. If left recursion lands first, the
  recovery protocol must unwind Medeiros' L table consistently
  alongside the VM stack — analogous to how `fail()` already handles
  `Memo` frames.

### Self-hosting — parsing PEG grammars using pegvm itself

The long-form project: replace the hand-written recursive-descent parser
in `src/pegvm/grammar.rs` with a PEG grammar that describes PEG grammar
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
4. **Replace the recursive-descent parser.** `parse_grammar(src)` becomes
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
