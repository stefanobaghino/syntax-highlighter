# Architecture

This is a PEG-based syntax highlighting engine. The single crate is split into four modules, layered bottom-up:

- `src/pegvm/` — bytecode VM, instruction set, memo cache, incremental-edit invalidation protocol. **Zero external dependencies** (only `std`). Knows nothing about grammar source syntax, ANSI, themes, or colors. This is the piece intended to eventually become an independent library crate. See [`src/pegvm/README.md`](src/pegvm/README.md) for the type-by-type walkthrough, the instruction set, bytecode invariants, and academic references.
- `src/pegc/` — PEG compiler: grammar source → `pegvm::Program`. `pegc::compile(source)` is the deep one-step entry; lower-level `parse`, `Grammar::compile`, and `compile_pattern` stay public for tests and composition. Depends on `pegvm`; `pegvm` has no reverse dependency. See [`src/pegc/README.md`](src/pegc/README.md) for the grammar source language spec — syntax, escapes, precedence, the `@name{pattern}` and `*^` / `+^` extensions, and error shapes.
- `src/pegb.rs` — binary serialization of `pegvm::Program`. `pegb::encode` / `pegb::decode` round-trip a compiled program through bytes for shipping pre-compiled grammars. Depends on `pegvm`; sibling of `pegc`. Format is **not** stable in v0 — bytecode artifacts are tied to the exact crate build that produced them. See the module rustdoc for the wire format.
- `src/parser.rs` — the deep incremental-parsing abstraction. One type, `Parser`, bundles "compile a grammar, feed input, parse it, carry a memo across edits" behind a small interface (`new`, `from_program`, `set_input`, `edit`, `append`, `input`, `captures`, `capture_kinds`, `is_complete`, `last_stats`). `from_program` accepts a pre-built `Program` (e.g. from `pegb::decode`) so callers can skip the source pipeline. Hides `pegvm` and `pegc`/`pegb` primitives from consumers who don't need them.
- `src/walk.rs` — capture-stream walker. `walk()` takes a parser's captures and emits a flat segment stream that tiles input bytes exactly. Pure-structural; knows nothing about ANSI, themes, or rendering. Consumed by the demo CLI's renderer. The structural coverage invariant is documented in the module rustdoc and asserted by hand-constructed tests in the same module.

## Binaries

Three `[[bin]]` targets, with different audiences:

- `demo` (`src/bin/demo/`) — quickstart demo CLI. Reads input, emits ANSI-coloured output. The Highlighter wrapper, theme constants, and demo-specific tests all live alongside `main.rs` in this directory; ANSI presentation is a demo-only concern. Tiny by design.
- `pegc` (`src/bin/pegc.rs`) — compile-time grammar inspection: `stats`. Operates on a grammar source; input-independent. See [`TOOLS.md`](TOOLS.md).
- `pegdb` (`src/bin/pegdb.rs`) — debug-time grammar inspection: `dump-captures`. Operates on a parse trace. JSONL output. See [`TOOLS.md`](TOOLS.md).

The module boundaries are load-bearing:

- `pegvm` stays dependency-free and reverse-dependency-free; don't leak grammar parsing, parsing-state orchestration, or highlighting concepts into it.
- `pegc` targets `pegvm::Program` one-way. A future non-PEG target would be a sibling module, not a detour through `pegc`.
- `pegc` and `pegb` import only from `pegvm`'s type-side modules (`instruction`, `program`) — never from execution-side (`vm`, `incremental`). The dep arrow is on the type language, not the runtime semantics; if a serialization or compilation concern needs to reach into VM execution, that's a sign the boundary is wrong.
- Both `pegc` and `parser` are deep modules in the sense John Ousterhout describes in *A Philosophy of Software Design*, Ch. 4 ("Modules Should Be Deep"): small interfaces hiding large implementations. `pegc::compile(source)` hides grammar parsing, AST handling, and bytecode emission behind one call; `Parser` hides compilation, memo threshold tuning, invalidation math, and VM wiring behind seven methods. Callers with a one-shot "give me captures" use case should talk to `Parser`, not the primitives.
- `walk` is a pure-structural consumer of `parser`. Adding presentation variants (HTML, JSON AST dump, terminal 24-bit color) means adding new presenters that consume `walk`'s segment stream alongside the demo's `Highlighter`, not extending `Parser`.

## Design ethos

Project-level design goals live in [`README.md`](README.md) under *Why this approach*. The operational rules below translate those goals into what-to-do / what-not-to-do guidance for anyone changing the code:

- **Don't introduce code that requires per-language changes to `src/`.** Adding a language means adding a grammar file, nothing else.
- **Preserve runtime loadability.** Architectural choices that foreclose loading a grammar at runtime are off the table even when the MVP doesn't exercise it.
- **FFI direction for `pegvm`.** The intended FFI future is a separate `pegvm-ffi` crate exposing `extern "C"` handles. Keep `pegvm`'s Rust-side abstractions thin enough that a C wrapper doesn't fight them.
