# Architecture

This is a PEG-based syntax highlighting engine. The single crate has two cleanly separated halves:

- `src/pegvm/` — grammar parser, AST, bytecode compiler, and VM. **Zero external dependencies** (only `std`). This half knows nothing about ANSI, themes, or colors. See [`src/pegvm/README.md`](src/pegvm/README.md) for the type-by-type walkthrough, the instruction set, bytecode invariants, and academic references.
- `src/highlight/` — consumes the VM's captures and renders ANSI-colored output. Depends on `pegvm` only through re-exports in `src/lib.rs`. The rendering strategy and the load-bearing `strip_ansi(highlight(x)) == x` invariant are documented as module-level rustdoc in `src/highlight/mod.rs` — that's where they live closest to the code they constrain.

The module boundary is load-bearing: `pegvm` is the piece intended to eventually become an independent library crate. Do not introduce reverse dependencies or leak highlighting concepts into it.

## Design ethos

- Grammars are **data, not generated code**. Adding a language means adding a grammar file, not recompiling the library. Don't introduce code that requires per-language changes to `src/`.
- The VM is the only compiled code; languages are pure data. Architectural choices should preserve runtime loadability even when the MVP doesn't exercise it.
- Highlight annotations live inline in the grammar. A single file defines both syntax and coloring — no external theme/scope mapping.
- **Zero dependencies** in the crate today. Adding one needs a real justification; for `pegvm` in particular, the intent is a lean, dependency-free API surface that a future FFI layer (likely a separate `pegvm-ffi` crate exposing `extern "C"` handles) can wrap without fighting Rust-only abstractions. `no_std` compatibility is *not* an active goal.
