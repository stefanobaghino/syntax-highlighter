# pegvm — a PEG bytecode virtual machine

This module is a self-contained implementation of a parsing machine for parsing expression grammars (PEGs). Its role in the crate is to execute a compiled program against an input and emit a list of captured spans. Everything else — ANSI rendering, themes, the CLI — lives outside this module and treats captures as the abstract output. The module has no dependencies beyond `std`.

This document is a guided tour of the **types** that make up the module, extended upstream to cover the full `&str → Grammar → Program → MatchResult` pipeline for continuity. Types that surface grammar-source parsing and compilation (`Pattern`, `Grammar`, `ParseError`, `CompileError`) now live in [`crate::pegc`](../pegc/) after the split; see [`src/pegc/README.md`](../pegc/README.md) for the grammar-source language spec (syntax, escapes, precedence, extensions, errors). The sections below keep their names for clarity but the module path is `pegc`, not `pegvm`. Implementation details (bit layouts, dispatch tables) are only mentioned when the semantics force them.

## The pipeline in one picture

Three transformations connect the key types:

```
&str  ──pegc::parse──▶  Grammar  ──Grammar::compile──▶  Program  ──VM::run──▶  MatchResult
```

- `pegc::parse` is text-level: it consumes a grammar source and produces a `Grammar` value. `pegc::compile` folds this and the next step into one call.
- `Grammar::compile` is AST-level: it consumes a `Grammar` and produces a `Program` (bytecode plus metadata).
- `VM::run` is runtime-level: it consumes a `Program` and an input and produces a `MatchResult` whose `complete` flag distinguishes full matches from partial ones.

Every other type in the module exists to serve one of these three stages.

## Grammars as values

### `Pattern` — PEG as an algebraic data type

`Pattern` is the recursive AST of a parsing expression. Its variants correspond one-to-one to the PEG constructs introduced by Ford (2004):

| Variant | PEG notation | Meaning |
|---|---|---|
| `Literal(Vec<u8>)` | `"abc"` | Matches a specific sequence of bytes, in order. |
| `CharClass(CharSet)` | `[0-9a-f]` | Matches any single byte in the set. |
| `AnyChar` | `.` | Matches any single byte (fails only at end of input). |
| `Sequence(Vec<Pattern>)` | `p1 p2 p3` | Concatenation; every sub-pattern must match in order. |
| `OrderedChoice(Vec<Pattern>)` | `p1 / p2 / p3` | First-match; unlike regex `\|`, alternation is biased — `p2` is tried only if `p1` fails. |
| `Repeat(Box<Pattern>)` | `p*` | Greedy, possibly-empty repetition. |
| `RepeatOne(Box<Pattern>)` | `p+` | Greedy, at-least-once repetition. |
| `Optional(Box<Pattern>)` | `p?` | Match once if possible, succeed unchanged otherwise. |
| `NotPredicate(Box<Pattern>)` | `!p` | Zero-width: succeeds iff `p` would fail here. Consumes no input. |
| `AndPredicate(Box<Pattern>)` | `&p` | Zero-width: succeeds iff `p` would succeed here. Consumes no input. |
| `NonTerminal(String)` | `rule_name` | References another `Pattern` in the enclosing `Grammar`. |
| `Capture(String, Box<Pattern>)` | `@name{p}` | Matches `p`; additionally records the matched span under the tag `name`. |
| `Catch { inner, label, recovery }` | `p ^label q` | Try `inner`; on failure splice the failed attempt's deepest-reach captures (via `RecoverToScopedMax`) and run `recovery` from that resync point. `*^` / `*^[cs]` / `+^` / `+^[cs]` desugar to `Repeat(Catch(...))` at parse time — see `build_recover_repeat` in `src/pegc/parser.rs`. |

A `Pattern` is a plain value. It can be constructed programmatically (see the compiler tests) or produced by `pegc::parse` from text. A `Pattern` has no knowledge of its environment: a `NonTerminal("digit")` is a dangling reference until it's placed inside a `Grammar` that defines `digit`.

The `Capture` variant is the bridge between parsing (which produces `Pattern`) and highlighting (which consumes tag names). It carries a string that is later interned to an integer id during compilation, and resolved back to a name by the highlighter. See `CaptureKind`.

### `CharSet` — a set of bytes

A `CharSet` is *a set of byte values*. Cardinality: the domain is the 256 possible `u8` values, so the value-space of `CharSet` is finite and small. The type appears in two places:

- In a `Pattern`, as the payload of `CharClass` — the parsed representation of `[a-z_]`, `[^"\\]`, etc.
- In an `Instruction`, as the payload of `Set` and `TestSet` — the compiled, directly-executable form.

The representation is a 256-bit bitmap (`[u8; 32]`); `contains(b)` is a constant-time bit test. Operations (`negate`, `union`, range construction) are closed over the type, so building a class like `[^"\\]` as `CharSet::from_bytes(b"\"\\").negate()` returns another `CharSet`. No allocation.

The compiler performs no transformation on a `CharSet` when going from `Pattern::CharClass(s)` to `Instruction::Set(s)` — the same value flows through. This is deliberate: the byte domain of a character class is already the most direct representation of "which input bytes does this class accept?" and there's no benefit to re-encoding it.

### `Grammar` — named patterns plus a start rule

```rust
pub struct Grammar {
    pub rules: HashMap<String, Pattern>,
    pub start: String,
}
```

A `Grammar` is the value-level form of an entire PEG document. It lifts a collection of `Pattern`s into a coherent whole by giving each a name (the map keys) and designating one as the entry point (`start`, guaranteed to be a key in `rules` when produced by `pegc::parse`). It is the unit of compilation: `Grammar::compile` takes a `Grammar` and produces a `Program`.

The distinction between `Pattern` and `Grammar` is exactly the distinction between an *expression* and a *set of named definitions*: `Pattern::NonTerminal("digit")` inside a rule body is a free reference that only makes sense in an environment that binds `digit` to another `Pattern`. A `Grammar` *is* that environment.

## Compilation: from `Pattern` to `Program`

The compiler turns the tree-shaped `Grammar` into a flat sequence of `Instruction`s. This is the canonical LPEG/GPeg-style compilation (Ierusalimschy 2009; Medeiros & Ierusalimschy 2008; Yedidia 2021, Ch. 3): execution becomes a linear walk through a `Vec<Instruction>` with branching, backtracking, and rule calls expressed as jumps and stack manipulations.

### `Instruction` — opcodes for the parsing machine

Each `Pattern` variant maps to a small fixed sequence of `Instruction`s. The instruction set is partitioned by role:

| Role | Instructions | What they do |
|---|---|---|
| Matching | `Char`, `Set`, `Any`, `TestChar`, `TestSet` | Consume input bytes, or fall through to a label if the current byte doesn't match. |
| Control flow | `Jump`, `Choice`, `Commit`, `PartialCommit`, `BackCommit`, `FailTwice`, `Fail` | Express ordered choice, repetition, and predicates in terms of pushing and popping backtrack records. |
| Calls | `Call`, `Return` | Invoke a named rule and return from it — the `NonTerminal` variant compiles to `Call(address)`. |
| Rule entry | `RuleEnter` | Carries a `RuleKind` discriminant (`Memo` or `Lr`). The shared cache-hit prologue probes the packrat slot at `(memo_id, sp)`: on a success hit it replays cached captures and jumps to the rule's `Return`; on a cached failure it enters `fail()`. On a miss the kind branches: `Memo` pushes a `StackEntry::Memo` frame (committed by `MemoClose` on success or `fail()` on escape); `Lr` walks the stack for an in-flight `LFrame` (recursive entry replays the seed — `None` ⇒ fail, `Some` ⇒ jump to Return) and pushes a fresh `LFrame` if no match. |
| Rule exit | `MemoClose`, `LRTail` | Close a rule body. `MemoClose` pops the `Memo` frame and commits a success entry. `LRTail` is the seed-and-grow controller — growth re-iterates from `start_sp`; no growth commits the converged seed and writes a memo entry whose `examined_max` is the high-water mark across every iteration. Both route through `cache_success`, which applies the threshold filter to `Memo` kind and always caches `Lr` kind (issue #55). LR-rule **failure** caching is not yet implemented. |
| Captures | `CaptureBegin`, `CaptureEnd` | Bracket a matched span with a kind tag so the VM can record it. |
| Termination | `End` | Mark the end of a successful match. |

An `Instruction` is a pure value; it carries no pointers or state, only the data (`u8`, `CharSet`, `Label`, `CaptureKind`) its semantics need. The entire compiled grammar is a `Vec<Instruction>` — a blob of data that could in principle be serialized, sent over a wire, or generated by something other than the provided compiler.

**Memo-threshold filter.** Successful `RuleKind::Memo` rule bodies are not cached unconditionally: entries whose matched span is shorter than `VM::DEFAULT_MEMO_THRESHOLD` are discarded. Tiny leaf rules pay a lookup cost without a meaningful replay win, so filtering them out is the classic memory-vs-time lever. The policy lives in `VM::cache_success`. `RuleKind::Lr` seed commits at `LRTail` ignore the filter and always cache — the seed-and-grow loop relies on the cache to short-circuit subsequent visits at the same `sp`, and filtering short LR seeds out causes O(2^N) re-descent in deep LR cascades (issue #55). Failure entries in `fail()` are also not filtered (their value is short-circuiting future re-executions). The default can be overridden per-VM via `VM::with_memo_threshold(bytes)`; `0` restores pure packrat behavior across both kinds.

### `Label` — opaque code addresses

`Label` is a newtype over `u32`: the index of an instruction in a compiled program's code array.

```rust
#[repr(transparent)]
pub struct Label(pub u32);
```

Its purpose is *disambiguation*: a `Jump(label)` instruction can only target something that was deliberately constructed as a code address — a stray `sp` or `len` cannot silently be used where the grammar's structure expects a label. The inner width is `u32` rather than `usize` because every `Instruction` variant carrying a Label pays for that width via enum padding; trimming Label from 8 to 4 bytes shrinks the largest variant (`TestSet(CharSet, Label)`) from 40 to 36 payload bytes and the whole enum from 48 to 40 bytes, ~17% less memory across every Program. The cap of 4 G instructions is well beyond what `pegc` produces — sqlite, the largest grammar in this repo, compiles to ~6 K instructions.

Labels appear only in `Instruction` payloads. The VM widens to `usize` at the boundary where a Label flows into the instruction pointer or onto the backtrack stack via `Label::as_index` (`self.ip = label.as_index()`); the newtype's job is to keep the *data* of the program well-typed and compact.

### `CaptureKind` — interned capture-name tags

`CaptureKind` is a newtype over `u16`: the compiler-assigned integer id of a capture name.

```rust
#[repr(transparent)]
pub struct CaptureKind(pub u16);
```

During compilation, each distinct `Capture(name, …)` encountered in the grammar is interned: the first time the compiler sees `@property{…}` it assigns a fresh `CaptureKind(n)` and records the string in a table. Subsequent occurrences reuse the same id. The bytecode carries only the integer — strings never enter the VM — and the consumer (the highlighter) looks the id back up in `Program::capture_kinds`.

This is the only interaction between the VM and the rest of the crate that isn't purely through captures. The VM does not interpret the kind in any way; it only stores it and emits it.

### `Program` — the compiled artifact

```rust
pub struct Program {
    pub code: Vec<Instruction>,
    pub capture_kinds: Vec<String>,
}
```

A `Program` is a `Grammar` after compilation. Two things are bundled:

- `code` is the executable instruction sequence. It has a fixed prologue: `code[0] = Call(start_rule_address)`, `code[1] = End`. Rule bodies follow, each ending with `Return`. This layout is what allows the VM to execute the start rule by invoking its dispatch loop at ip = 0 and recognizing success when it reaches `End`.
- `capture_kinds` is the inverse of the compiler's interning table: `capture_kinds[k.0 as usize]` yields the original string name for a `CaptureKind` the VM emits. This is the only place strings survive past compilation.

A `Program` is the VM's input contract. Given one, the VM's behavior is fully determined by the bytecode and the input; no other state from the `Grammar` or `Pattern` world is needed.

## Execution: from `Program` to `Captures`

### `VM` — parsing machine state

`VM<'p, 'i>` holds transient state for one in-flight match: a borrow of a program, a borrow of an input, an instruction pointer, a subject pointer, a backtrack-and-call stack, and a capture buffer.

```rust
pub struct VM<'p, 'i> { /* program, input, ip, sp, stack, captures */ }
```

The `'p` / `'i` lifetimes encode the fact that a `VM` owns no data — it only reads from its program and input. This is why `VM::run` consumes `self`: a VM instance is a one-shot transducer from `(&Program, &[u8])` to `Option<MatchResult>`.

The stack is the machinery behind PEG's ordered choice, predicates, and calls. Its entries are one of two shapes:

- **Backtrack frames** (pushed by `Choice`, read by `Fail` and the `*Commit` family) capture enough state to rewind: a fallback `ip`, the `sp` at the time of the choice, and the capture count at the time of the choice.
- **Return frames** (pushed by `Call`, popped by `Return`) save the caller's `ip`.

Both kinds share one stack because, from the VM's perspective, a PEG rule invocation *is* a structured subexpression whose failure must backtrack through any `Choice`s made inside it. Yedidia Ch. 3 derives this unification formally.

Conceptually the dispatch loop is a total function from `(program, ip, input, sp, stack, captures)` to one of three outcomes: advance (move to the next `ip` or jump), fail (enter the backtrack protocol), or terminate (return a `MatchResult`).

### `Capture` and `MatchResult`

```rust
pub struct Capture {
    pub kind: CaptureKind,
    pub start: usize,
    pub end: usize,
}

pub struct MatchResult {
    pub matched: usize,
    pub captures: Vec<Capture>,
    pub complete: bool,
}
```

A `Capture` is a closed span over the input paired with a kind tag: "these bytes matched under this name." Its `kind` field is the same `CaptureKind` the grammar author (transitively) wrote as `@name{…}`, intern-translated through compilation and emitted unchanged by the VM.

Captures returned in a `MatchResult` have two non-obvious guarantees:

1. **They form a properly-nested forest over the input.** Because PEG syntactic structure nests, captures can only stand in a sibling-or-parent relationship — never partially overlap. This is what lets the highlighter use a stack-based renderer rather than an interval tree.
2. **They reflect only alternatives that actually succeeded up to `matched`.** Captures begun inside a failed choice are discarded during backtracking (see `VM::fail`). The caller sees a history consistent with the winning parse, not with everything the VM tried.

A `MatchResult` is always returned. The `complete` flag distinguishes the two cases:

- `complete == true`: the VM reached `End`. `matched` is the `sp` at that point (potentially less than `input.len()` if the grammar is designed to stop early — no trailing `!.`).
- `complete == false`: the VM exhausted its backtrack stack without reaching `End`. `matched` is the farthest input position the VM ever reached before retreating — the "farthest failure position" heuristic of Ford 2004, used for error reporting by LPegLabel and the `lpeg-ffp` fork — and `captures` are the captures valid at that point, with any still-open captures closed at `matched`.

This partial result is what lets the highlighter render a styled prefix and a plain tail for malformed input, without the VM knowing anything about highlighting.

A grammar that opts into the `*^` recovery operator on a top-level repetition can flip a parse that would otherwise be partial into `complete: true`: the loop emits `recovery_kind`-tagged captures over the bytes it skipped past inner failures and exits cleanly at end of input. Captures returned in that case interleave the successful `inner` captures with the recovery captures in input order.

## Left recursion

The compiler detects left-recursive rules — both direct (`A <- A α / β`) and indirect (`A <- B …; B <- A …`) — by finding strongly connected components in the first-call graph (see `analyze_left_recursion` in `src/pegc/analysis.rs`). Every member of any non-trivial SCC, plus any size-1 SCC with a self-edge, emits its `RuleEnter` with `RuleKind::Lr` and closes its body with `LRTail` instead of `MemoClose`. Cross-rule cycles need no extra runtime support: `RuleEnter`'s LR miss-path lookup walks the stack and finds the right `LFrame` regardless of how many other rules sit between the call site and the frame.

Bytecode shape for an LR rule:

```
rule_addr:    RuleEnter(memo_id, RuleKind::Lr, return_addr)
body_start:   <body>
              LRTail(memo_id, body_start)
return_addr:  Return
```

The runtime algorithm is **bounded left recursion** (Medeiros, Mascarenhas & Ierusalimschy 2014, §3.2 / §5):

0. **Cache check**: `RuleEnter`'s shared prologue probes `self.memo.get(&(memo_id, sp))`. A hit replays the cached captures and jumps to `return_addr` (success) or enters `fail()` (failure). The kind-specific miss path below is only reached on a cache miss.
1. **First entry at `sp`** (LR miss): the kind-`Lr` branch pushes `StackEntry::LFrame { memo_id, start_sp, capture_start_len, return_addr, seed: None }`. Body executes.
2. **Recursive entry at the same `sp`**: the LR miss path finds the existing `LFrame` on the stack. `seed: None` ⇒ `fail()`. `seed: Some(end_sp, captures)` ⇒ replay captures, set `sp = end_sp`, jump to `return_addr` so the recursive call appears to return the seed.
3. **Body succeeds**: `LRTail` decides. Growth (`sp > seed.end_sp`, or first success with `seed: None`) updates the seed and re-iterates from `start_sp`. No growth commits the seed, writes a memo entry (always, regardless of `memo_threshold` — see issue #55 and the threshold-filter note above), and falls through to `Return`.
4. **Body fails**: `fail()`'s `LFrame` arm rescues with the prior seed when `seed.is_some()` (returns `true`, resumes at `return_addr`); when `seed.is_none()` it continues unwinding (the LR rule failed without ever growing). LR-rule failure caching is not yet implemented — symmetric with `fail()`'s `Memo` arm but deferred until profiling motivates it.

The L table is stack-structured and lives on `self.stack`; converged seeds also flow into `self.memo` so subsequent runs (or subsequent calls within the same run) at the same `sp` short-circuit. The `examined_max` recorded with the entry is the value popped from `memo_examined` at `LRTail`'s commit branch — the high-water mark across every iteration of the seed-and-grow loop, since `RuleEnter`'s LR miss path pushes one watermark slot at frame entry and only the commit branch (or `fail()`'s `LFrame` arm) pops it. That bound feeds `MemoCache::apply_edit`'s invalidation predicate verbatim.

## Error types

Grammar-source errors (`ParseError`, `CompileError`, unified `Error`) belong to the compiler — see [`src/pegc/README.md`](../pegc/README.md#errors). The VM's contract is simpler: `VM::run` always returns a `MatchResult`; its `complete` flag distinguishes a successful match from a non-match. A non-match is not an error — the distinction is between *author bugs* (grammar) and *data bugs* (input).

## Invariants the types alone can't express

Some properties the module relies on can't be encoded in the Rust type system. They are documented here and enforced by review:

1. **`PartialCommit` must target the body of a repetition, not the `Choice` that starts it.** `PartialCommit` updates the existing top backtrack frame rather than pushing a new one; jumping back to the `Choice` would push a fresh frame every iteration, exhausting memory and — worse — corrupting the stack so that an enclosing `Return` pops the wrong kind of entry. This is the first thing to check when extending the compiler.
2. **Captures are truncated on backtrack.** Any instruction that enters the fail protocol routes through `VM::fail`, which restores the capture buffer length from the backtrack frame. New backtracking instructions must reuse this helper rather than re-implement it.
3. **`VM::fail` and `BackCommit` are the only sites where `sp` retreats.** Both must route through `VM::maybe_snapshot` so the farthest-failure bookkeeping sees every retreat. Between retreats `sp` is monotone non-decreasing, which is why snapshots at those two sites capture the true deepest point reached. Any new instruction that rewinds `sp` must update this invariant.
4. **Byte-oriented.** `CharSet` is a set of `u8` values; UTF-8 is never decoded. This is a deliberate simplification appropriate for the MVP and will need to be revisited before serious Unicode-aware grammars.
5. **Memo replay is absolute-sp.** Cached captures carry the original `start`/`end` byte offsets; a `RuleEnter` hit only fires when `sp == start_sp`, so the stored values are replayed verbatim. Entries are inserted as already-closed `OpenCapture`s so the enclosing `CaptureEnd`'s innermost-still-open search (`rposition(c.end.is_none())`) binds to the caller's open capture rather than a replayed one.
6. **`VM::fail` records every `Memo` frame it traverses.** When unwinding to find a `Backtrack`, each `Memo` frame encountered is committed as a failure entry before being discarded. Silently dropping a frame would leak re-executions on future calls at the same sp. The loop is an exhaustive `match` over `StackEntry` specifically to make omissions a compiler error.
7. **`RuleEnter` calls `maybe_snapshot` after applying a success hit.** The hit advances `sp` past code that didn't execute; without the snapshot call the farthest-failure bookkeeping would miss the advance and `MatchResult.complete == false` would report a stale deepest point.
8. **`Catch` emits a fresh `Choice`/`Commit` pair around its `inner`, never `PartialCommit`.** A catch's retry baseline must sit at the iteration baseline `sp`; reusing `PartialCommit` (which mutates an existing Backtrack frame in place) would corrupt that baseline and re-trigger invariant 1's hazard. This matters most for the `*^` / `*^[cs]` desugar (`Repeat(Catch(...))`) where each iteration enters a fresh `Catch`. See `compile_pat`'s `Catch` arm in `src/pegc/compiler.rs`.
9. **`LRTail`'s `Label` payload targets the instruction after `RuleEnter`, not `RuleEnter` itself.** The seed-and-grow loop must re-execute the body, not re-push the `LFrame` — re-pushing would lose the prior seed and turn growth into an infinite loop. See `compile_rules`' LR-rule branch in `src/pegc/compiler.rs`.
10. **`fail()`'s `LFrame` arm pops `memo_examined` before any rescue branch.** The watermark stack and `LFrame` push/pop in lockstep; if the rescue path took the early `return true` without popping, the next `MemoClose` / `LRTail` would underflow `memo_examined`. See `VM::fail`'s `StackEntry::LFrame` arm.
11. **`DiagState::current_rule_stack` push/pop pairs with rule-frame push/pop on the backtrack stack, *when the diagnostic is enabled*.** Pushed in `RuleEnter`'s `Memo`-kind miss path *and* the first-entry case of the `Lr`-kind miss path; popped in `MemoClose`, `LRTail`'s commit branch, `fail()`'s `Memo` arm, and `fail()`'s `LFrame` arm. The mirror feeds the `RecoveryDiagnostic.rule_stack` field that `pegdb dump-captures` emits — drift between this mirror and the live `StackEntry::Memo` / `StackEntry::LFrame` frames silently corrupts the diagnostic. Every site is gated on `self.diag.is_some()` so the highlighter hot path (knob off, the default) does nothing extra; the gate must be set in lockstep across all six sites or the mirror desyncs. Like `memo_examined`, the symmetry is invariant under the gate: when on, every rule-frame push twins with one mirror push, every pop with one mirror pop.
12. **`recovery_diagnostics` is finalize-filtered.** `RecoverToScopedMax` pushes one diagnostic per call, *before* the recovery body's `CaptureBegin` / body / `CaptureEnd` triple runs. If the recovery body later fails (e.g. the EOF-exit iteration of a `*^` loop where the recovery `Any(1)` can't advance, or any `^label` catch whose recovery body fails wholesale), the enclosing `Choice` / `Backtrack` pair rewinds the capture buffer to before the recovery's first `CaptureBegin` — leaving a phantom diagnostic with no surviving capture. `finalize_recovery_diagnostics` (in `src/pegvm/vm.rs`) drops phantoms by checking that the capture at each diagnostic's `capture_index` starts at the diagnostic's `pos`. The check is position-only (not width-also) because recovery bodies post-#92 / #97 emit multi-byte captures (`*^[cs]`'s `(![cs] .)* [cs]` body, any `^label` catch with author-written multi-byte recovery), and the earlier width=1 form was tuned for plain `*^`'s `Any(1)` body and silently dropped diagnostics for the newer surfaces. Future changes to any recovery lowering must keep the "diagnostic position == surviving capture's start" invariant or update the filter.

## References

### Papers

- Bryan Ford, [*Parsing Expression Grammars: A Recognition-Based Syntactic Foundation*](https://bford.info/pub/lang/peg.pdf). POPL 2004.
- Sérgio Medeiros and Roberto Ierusalimschy, [*A Parsing Machine for PEGs*](https://www.inf.puc-rio.br/~roberto/docs/ry08-4.pdf). DLS 2008.
- Roberto Ierusalimschy, [*A Text Pattern-Matching Tool based on Parsing Expression Grammars*](https://www.inf.puc-rio.br/~roberto/docs/peg.pdf). Software: Practice and Experience, 39(3):221–258, 2009.
- Zachary Yedidia, [*Incremental PEG Parsing*](https://zyedidia.github.io/notes/yedidia_thesis.pdf). Ph.D. thesis, 2021. Chapter 3 ("A PEG Parsing Machine") is the clearest modern presentation of the instruction set used here and the reference to read first.
- Sérgio Medeiros, Fabio Mascarenhas, and Roberto Ierusalimschy, [*Left recursion in parsing expression grammars*](https://arxiv.org/pdf/1207.0443.pdf). Science of Computer Programming 96:177–190, 2014. "Bounded left recursion" semantics with §5 giving the parsing-machine extension implemented here as `RuleEnter`'s LR miss path / `LRTail`; L table stack-structured and separate from the packrat memo. Fixes nullable-LR bugs that Warth 2008's algorithm exhibits.

Left recursion (historical context; alternative algorithms not implemented):

- Alessandro Warth, James R. Douglass, and Todd Millstein, [*Packrat Parsers Can Support Left Recursion*](https://web.cs.ucla.edu/~todd/research/pepm08.pdf). PEPM 2008. Seminal seed-and-grow algorithm; couples left-recursion handling to the packrat memo table.
- Laurence Tratt, [*Direct Left-Recursive Parsing Expression Grammars*](http://tratt.net/laurie/research/pubs/papers/tratt__direct_left_recursive_parsing_expression_grammars.pdf). Middlesex University Technical Report EIS-10-01, 2010. Adapts Warth's idea to PEGs without packrat memoization; direct left recursion only.

### Reference implementations

- [LPEG](http://www.inf.puc-rio.br/~roberto/lpeg/) — Roberto Ierusalimschy's original parsing-machine implementation, in Lua.
- [GPeg](https://github.com/zyedidia/gpeg) — Zachary Yedidia's Go implementation with incremental-parsing extensions; the most readable working reference for this codebase.
