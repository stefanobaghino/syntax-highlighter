# Contributing

This guide applies to anyone changing the code, whether human or AI agent.

## Commands

This is a standard Cargo crate — `cargo build`, `test`, `run`, `fmt`, and `clippy` work as expected. The exact set of local checks that mirror CI (and the flags they use) lives in `.github/workflows/ci.yml`. Run those locally before pushing.

Requires Rust 1.78 or newer (the MSRV declared in `Cargo.toml`).

## Benchmarks

`cargo bench --bench memo` sweeps the memoization threshold across the
shipped grammars. Re-run it after adding a new grammar, or after any
change to `RuleEnter` / `MemoClose` / memo-replay semantics — a new
grammar's shape can push the knee past the shipping
`VM::DEFAULT_MEMO_THRESHOLD`, and VM changes that alter memo-table
access patterns can invalidate the current tuning. The `memo-bench`
skill (`.claude/skills/memo-bench/SKILL.md`) documents how to read the
output and when to raise the default.

## Developer tools

`pegc` (compile-time inspection) and `pegdb` (debug surface) cover the
grammar developer's workflow: bytecode stats, per-capture dumps. Full
contract in [`TOOLS.md`](TOOLS.md).

For a quick sanity check on a grammar's bytecode size after
non-trivial edits, run `cargo run --bin pegc -- stats grammars/<lang>.peg`.
Walker correctness is asserted by unit tests in `src/walk.rs` and
runs as part of `cargo test`.

## Project-level rules

- **Warnings fail CI.** `RUSTFLAGS: -D warnings` is set globally in the workflow. Don't introduce code that produces rustc or clippy warnings.
- **Lockfile-pinned builds.** CI uses `--locked` everywhere. Treat `Cargo.lock` as authoritative; don't run `cargo update` casually.
- **MSRV is enforced.** The MSRV is declared in `Cargo.toml` (`rust-version`) and verified by a CI job. Before using a feature stabilized after that version, either bump the MSRV deliberately (update `rust-version` and the CI workflow together) or pick a compatible alternative. Hard constraint: MSRV must be ≥ the Rust version that introduced the committed `Cargo.lock` format.

## Code conventions

### Newtype wrappers, not type aliases

For any value whose meaning is *more* than its underlying primitive — code addresses, opaque tags, identifiers, units, indices into a specific table — use a `#[repr(transparent)]` newtype struct, not a `type X = Y` alias.

```rust
// Don't:
pub type Label = usize;            // silently interchangeable with any usize

// Do:
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Label(pub usize);
```

**Why:** type aliases are transparent to the compiler — `Label` and `usize` are the *same type*, so `self.ip = some_unrelated_index` compiles silently. Newtypes are distinct types: assigning across them requires an explicit `Label(n)` (construction) or `.0` (unwrap), which is exactly the friction needed to catch mix-ups at boundaries (e.g. confusing the instruction pointer `ip` with the subject pointer `sp`, both `usize`).

**Cost:** zero. `#[repr(transparent)]` guarantees the same memory layout as the inner type, and wrap/unwrap operations compile away.

**Construction style:** `Label(n)` and `.0` directly. Do **not** add blanket `From<usize>` / `Into<usize>` impls — the implicit conversion they enable defeats the purpose. If a specific named conversion is genuinely useful (e.g. `Label::next(self) -> Label`), add it as a method.

**Lint:** there is no clippy lint that forbids `type X = Y` aliases. Reviewers and future agent sessions enforce this by reading this section.

### `debug_assert!` for preconditions and invariants the type system can't express

When a function has a precondition or internal invariant that's known at design time but can't be encoded in Rust's type system, document it with a `debug_assert!`. The check fires in tests and compiles away in release builds, so it surfaces bugs without runtime cost.

```rust
// Public API with a precondition no caller in the codebase
// violates today, but which a future caller might:
pub fn add_range(&mut self, lo: u8, hi: u8) {
    debug_assert!(
        lo <= hi,
        "CharSet::add_range: inverted range lo=0x{:02x} hi=0x{:02x}",
        lo, hi
    );
    // ...
}

// Internal helper whose contract is enforced by convention; the
// assertion catches contract violations close to the bug rather
// than surfacing them later as an opaque VM error:
fn patch_jump(&mut self, idx: usize, target: usize) {
    debug_assert!(idx < self.code.len(), "...");
    debug_assert!(target <= self.code.len(), "...");
    // ...
}
```

**Use it when:** a precondition (`lo <= hi`, indices in range), an internal invariant (stack shape at a point in the dispatch loop), or a documented contract on an internal helper would otherwise either silently corrupt state or fail later with a less-informative message (e.g. raw "index out of bounds").

**Do not use it when:**
- The condition involves *external* input (user input, file contents, network data) — that needs real validation via `Result<T, E>` so callers can recover and the check fires in production.
- The case is genuinely impossible — prefer `unreachable!()` (always panics, even in release) so an actual occurrence in production isn't silent. `debug_assert!` is for "should not happen but I want to know if I'm wrong"; `unreachable!()` is for "cannot happen, and if it did the program is in an undefined state."
- The type system can express the constraint instead — design out the impossibility (newtype, `NonZero`, sealed enum, …) rather than guarding it dynamically.

**Style:** the message is the first thing a developer sees when an assertion fires. Name the violated condition and include the offending values: `"CharSet::add_range: inverted range lo=0x.. hi=0x.."` beats `"lo <= hi"`.
