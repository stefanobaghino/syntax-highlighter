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
