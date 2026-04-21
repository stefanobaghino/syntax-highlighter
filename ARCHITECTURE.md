# Architecture

This is a PEG-based syntax highlighting engine. The single crate is split into four modules, layered bottom-up:

- `src/pegvm/` — bytecode VM, instruction set, memo cache, incremental-edit invalidation protocol. **Zero external dependencies** (only `std`). Knows nothing about grammar source syntax, ANSI, themes, or colors. This is the piece intended to eventually become an independent library crate. See [`src/pegvm/README.md`](src/pegvm/README.md) for the type-by-type walkthrough, the instruction set, bytecode invariants, and academic references.
- `src/grammar/` — grammar source → `pegvm::Program` pipeline. Owns the grammar-text parser, the AST (`Pattern`, `Grammar`), and the bytecode compiler (`compile`, `compile_pattern`). Depends on `pegvm`; `pegvm` has no reverse dependency.
- `src/parser.rs` — the deep incremental-parsing abstraction. One type, `Parser`, bundles "compile a grammar, feed input, parse it, carry a memo across edits" behind a small interface (`new`, `set_input`, `edit`, `append`, `input`, `captures`, `capture_kinds`, `last_stats`). Hides `pegvm` and `grammar` primitives from consumers who don't need them.
- `src/highlight/` — ANSI-coloring consumer of `Parser`. `Highlighter` holds a `Parser` and renders its captures; every parsing method is a one-line delegation, `highlight()` is the only non-trivial body. The rendering strategy and the load-bearing `strip_ansi(highlight(x)) == x` invariant are documented as module-level rustdoc in `src/highlight/mod.rs` — that's where they live closest to the code they constrain.

The module boundaries are load-bearing:

- `pegvm` stays dependency-free and reverse-dependency-free; don't leak grammar parsing, parsing-state orchestration, or highlighting concepts into it.
- `grammar` targets `pegvm::Program` one-way. A future non-PEG target would be a sibling module, not a detour through `grammar`.
- `parser` is a deep module in the sense John Ousterhout describes in *A Philosophy of Software Design*, Ch. 4 ("Modules Should Be Deep"): a small interface hiding a large implementation (grammar source parsing, bytecode compilation, memo threshold tuning, invalidation math, VM wiring). Consumers with a one-shot "give me captures" use case should talk to `Parser`, not the primitives.
- `highlight` is a thin consumer of `parser`. Adding rendering variants (HTML, JSON AST dump, terminal 24-bit color) means adding siblings next to `Highlighter`, not extending `Parser`.

## Design ethos

- Grammars are **data, not generated code**. Adding a language means adding a grammar file, not recompiling the library. Don't introduce code that requires per-language changes to `src/`.
- The VM is the only compiled code; languages are pure data. Architectural choices should preserve runtime loadability even when the MVP doesn't exercise it.
- Highlight annotations live inline in the grammar. A single file defines both syntax and coloring — no external theme/scope mapping.
- **Zero dependencies** in the crate today. Adding one needs a real justification; for `pegvm` in particular, the intent is a lean, dependency-free API surface that a future FFI layer (likely a separate `pegvm-ffi` crate exposing `extern "C"` handles) can wrap without fighting Rust-only abstractions. `no_std` compatibility is *not* an active goal.
