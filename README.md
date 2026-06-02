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
  for the shipped example.
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

Run the `demo` binary against any of the shipped bench fixtures —
the grammar is picked by file extension. Complete grammars:

```bash
cargo run --bin demo -- crates/compiler/benches/fixtures/medium.json
cargo run --bin demo -- crates/compiler/benches/fixtures/medium.toml
cargo run --bin demo -- crates/compiler/benches/fixtures/medium.sql
```

Partial grammars cover most real-world code but have documented gaps —
expect the occasional miscoloring at the edges:

```bash
cargo run --bin demo -- crates/compiler/benches/fixtures/medium.css
cargo run --bin demo -- crates/compiler/benches/fixtures/medium.c
cargo run --bin demo -- crates/compiler/benches/fixtures/medium.js
cargo run --bin demo -- crates/compiler/benches/fixtures/medium.go
cargo run --bin demo -- crates/compiler/benches/fixtures/medium.rs
```

Stdin is also accepted (defaults to JSON; pass `-l <lang>` to
override).

## Status

Early MVP; expect breaking changes. Ten grammars ship today — eight
free-form, plus deliberately-pruned Starlark and YAML subsets that
validate the implicit indentation operators (issue #43; see
[`crates/compiler/src/pegc/README.md`](crates/compiler/src/pegc/README.md)) — plus an ANSI-coloring CLI.
The "kilobyte range per grammar" design goal holds across the shipped
set: as a representative data point, the largest grammar (SQLite)
compiles to roughly 5,600 instructions across 426 rules — comfortably
within the kilobyte-per-grammar target.

For current per-grammar numbers, run `cargo run --bin pegc -- stats grammars/<lang>.peg`; see [`TOOLS.md`](TOOLS.md) for the developer tools' full contract.

The CLI picks a grammar by file extension (`.json`, `.toml`, `.sql`,
`.rs`, `.js`/`.mjs`/`.cjs`, `.go`, `.c`/`.h`, `.css`, `.star`/`.bzl`,
`.yaml`/`.yml`) or an explicit
`-l json|toml|sql|rust|js|go|c|css|starlark|yaml` flag; stdin without a
flag defaults to JSON.

Malformed or incomplete inputs render the longest valid prefix styled and
the rest plain (see `crates/runtime/src/pegvm/vm.rs` for the farthest-failure tracking).
Grammars that opt into the `*^` recovery operator on a top-level
repetition resync past broken regions instead — see the SQL grammar
(`grammars/sqlite.peg`) for the shipped example.

`Parser` owns its input and reuses the memo table across edits:
append-only streaming and arbitrary-position edits reparse only the
regions whose memo entries cross the edit point, not the whole buffer
(see `crates/runtime/src/pegvm/incremental.rs` for the invalidation protocol).

## More documentation

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — how the codebase is organized:
  module split, boundaries, design ethos. Includes pointers to
  per-module deep documentation.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — how to make changes: commands,
  project-level rules, code conventions. Applies to anyone touching the
  code, human or AI agent.
- [`TOOLS.md`](TOOLS.md) — the `pegc` and `pegdb` grammar developer
  tools: bytecode stats (compile-time) and per-capture dumps
  (debug-time). Distinct from the demo CLI; reach for them when
  authoring or diagnosing a grammar.
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
