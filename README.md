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

Early MVP. Ships a single JSON grammar and an ANSI-coloring CLI. The
properties described in *Why this approach* below are **design goals that
this MVP is trying to prove** — the architecture supports them, but none are
yet demonstrated across more than one grammar. Expect breaking changes.

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

Or point it at a file:

```bash
cargo run -- path/to/file.json
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

- **More language grammars.** Adding a language means writing a grammar
  file. Candidates: Python, Rust, TOML, YAML, a SQL dialect or two.
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

- **Partial-match rendering.** Small VM change: on failure, surface the
  deepest `sp` reached and the captures valid at that point. Lets the
  highlighter render `input[..matched]` styled and `input[matched..]`
  plain, eliminating the "plain → styled → plain → styled" flicker when
  rendering incomplete input.
- **Memoization (packrat parsing).** Caches rule results per input
  position. Primary payoff: *incremental parsing* — O(Δ) per input
  change instead of O(n). Covers both the easy case (append-only
  streaming — an LLM response arriving character-by-character in a TUI)
  and the general case (arbitrary-position edits — a cursor insertion
  or deletion anywhere in the input), because both reduce to
  "invalidate the memo entries whose spans cross the edit point and
  re-parse from there" per Yedidia's thesis, Ch. 4. Also a prerequisite
  for left-recursion support.
- **Left recursion support.** Not needed for JSON, but natural for some
  grammar idioms (especially arithmetic-expression grammars). Most modern
  PEG implementations support it via Warth et al.'s algorithm; most of
  those require memoization.
- **Bytecode serialization.** Pre-compile grammars once and ship the
  `Vec<Instruction>` plus capture-name table as data. Useful for embedded
  distributions and startup-time-sensitive consumers.
- **Error recovery / parsing past syntax errors.** PEG is strict by
  default — a parse either succeeds or fails. For syntax highlighting,
  the input is frequently incomplete or malformed (mid-edit buffers,
  streaming responses, in-progress code). The ability to recover past
  a syntax error and continue highlighting the rest of the input is
  important; this is a real extension to the VM and a known research
  area in PEG implementations.

### Self-hosting — parsing PEG grammars using pegvm itself

The long-form project: replace the hand-written recursive-descent parser
in `src/pegvm/grammar.rs` with a PEG grammar that describes PEG grammar
syntax itself, executed by the VM. Prerequisites, in order:

1. **Semantic actions / typed captures.** Today's VM emits flat
   `Capture { kind, start, end }` records. To rebuild a typed AST from a
   parse, captures must be able to produce user values (e.g. a `Pattern`
   node) rather than spans. LPEG calls these *function captures*;
   Yedidia's thesis discusses them. This is a real VM feature, not a
   refactor.
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
