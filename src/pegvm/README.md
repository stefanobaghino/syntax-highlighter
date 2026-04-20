# pegvm — a PEG bytecode virtual machine

This module is a self-contained implementation of a parsing machine for parsing expression grammars (PEGs). Its role in the crate is to turn a textual grammar into a runnable program and to execute that program against an input, emitting a list of captured spans. Everything else — ANSI rendering, themes, the CLI — lives outside this module and treats captures as the abstract output. The module has no dependencies beyond `std`.

This document is a guided tour of the **types** that make up the module: what each one represents in the PEG model, and how values of one type are produced from or consumed by another. Implementation details (bit layouts, dispatch tables) are only mentioned when the semantics force them.

## The pipeline in one picture

Three transformations connect the key types:

```
&str  ──parse_grammar──▶  Grammar  ──compile_grammar──▶  Program  ──VM::run──▶  Option<MatchResult>
```

- `parse_grammar` is text-level: it consumes a grammar source and produces a `Grammar` value.
- `compile_grammar` is AST-level: it consumes a `Grammar` and produces a `Program` (bytecode plus metadata).
- `VM::run` is runtime-level: it consumes a `Program` and an input and produces (if the input matches) a `MatchResult`.

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

A `Pattern` is a plain value. It can be constructed programmatically (see the compiler tests) or produced by `parse_grammar` from text. A `Pattern` has no knowledge of its environment: a `NonTerminal("digit")` is a dangling reference until it's placed inside a `Grammar` that defines `digit`.

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

A `Grammar` is the value-level form of an entire PEG document. It lifts a collection of `Pattern`s into a coherent whole by giving each a name (the map keys) and designating one as the entry point (`start`, guaranteed to be a key in `rules` when produced by `parse_grammar`). It is the unit of compilation: `compile_grammar` takes a `Grammar` and produces a `Program`.

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
| Captures | `CaptureBegin`, `CaptureEnd` | Bracket a matched span with a kind tag so the VM can record it. |
| Termination | `End` | Mark the end of a successful match. |

An `Instruction` is a pure value; it carries no pointers or state, only the data (`u8`, `CharSet`, `Label`, `CaptureKind`) its semantics need. The entire compiled grammar is a `Vec<Instruction>` — a blob of data that could in principle be serialized, sent over a wire, or generated by something other than the provided compiler.

### `Label` — opaque code addresses

`Label` is a newtype over `usize`: the index of an instruction in a compiled program's code array.

```rust
#[repr(transparent)]
pub struct Label(pub usize);
```

Its purpose is *disambiguation*, not just bit-packing. Within the VM, `usize` is also used for the instruction pointer (`ip`), the subject pointer (`sp`), the capture stack length, and various lengths and indices. Making `Label` a distinct type means that a `Jump(label)` instruction can only target something that was deliberately constructed as a code address — a stray `sp` or `len` cannot silently be used where the grammar's structure expects a label.

Labels appear only in `Instruction` payloads. Once the VM is actually executing, it works with raw `usize` values (`self.ip = label.0`); the newtype's job is to keep the *data* of the program well-typed.

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
    pub matched: usize,          // bytes consumed
    pub captures: Vec<Capture>,
}
```

A `Capture` is a closed span over the input paired with a kind tag: "these bytes matched under this name." Its `kind` field is the same `CaptureKind` the grammar author (transitively) wrote as `@name{…}`, intern-translated through compilation and emitted unchanged by the VM.

Captures returned in a `MatchResult` have two non-obvious guarantees:

1. **They form a properly-nested forest over the input.** Because PEG syntactic structure nests, captures can only stand in a sibling-or-parent relationship — never partially overlap. This is what lets the highlighter use a stack-based renderer rather than an interval tree.
2. **They reflect only alternatives that actually succeeded.** Captures begun inside a failed choice are discarded during backtracking (see `VM::fail`). The caller sees a history consistent with the winning parse, not with everything the VM tried.

A `MatchResult` is emitted only when the VM reaches `End` — partial matches (grammar fits a prefix but not the whole input) are expressed by `matched < input.len()`, not by a partial result. A full non-match returns `None`.

## Error types

The module has two error types, one per transformation in the pipeline that can fail ahead of time:

- `ParseError` is returned by `parse_grammar` when the grammar source is malformed (unterminated string, missing `<-`, duplicate rule, etc.). It carries line and column information.
- `CompileError` is returned by `compile_grammar` when the grammar is well-formed text but semantically invalid — a `NonTerminal("foo")` with no matching rule, or a start rule that doesn't exist.

Runtime mismatch (the input doesn't match the grammar) is not an error type: `VM::run` returns `Option<MatchResult>` and a failed parse is `None`. The distinction is deliberate — a grammar error is an author bug (the grammar needs fixing), a runtime non-match is a data bug (the input didn't conform).

## Invariants the types alone can't express

Some properties the module relies on can't be encoded in the Rust type system. They are documented here and enforced by review:

1. **`PartialCommit` must target the body of a repetition, not the `Choice` that starts it.** `PartialCommit` updates the existing top backtrack frame rather than pushing a new one; jumping back to the `Choice` would push a fresh frame every iteration, exhausting memory and — worse — corrupting the stack so that an enclosing `Return` pops the wrong kind of entry. This is the first thing to check when extending the compiler.
2. **Captures are truncated on backtrack.** Any instruction that enters the fail protocol routes through `VM::fail`, which restores the capture buffer length from the backtrack frame. New backtracking instructions must reuse this helper rather than re-implement it.
3. **Byte-oriented.** `CharSet` is a set of `u8` values; UTF-8 is never decoded. This is a deliberate simplification appropriate for the MVP and will need to be revisited before serious Unicode-aware grammars.

## References

### Papers

- Bryan Ford, [*Parsing Expression Grammars: A Recognition-Based Syntactic Foundation*](https://bford.info/pub/lang/peg.pdf). POPL 2004.
- Sérgio Medeiros and Roberto Ierusalimschy, [*A Parsing Machine for PEGs*](https://www.inf.puc-rio.br/~roberto/docs/ry08-4.pdf). DLS 2008.
- Roberto Ierusalimschy, [*A Text Pattern-Matching Tool based on Parsing Expression Grammars*](https://www.inf.puc-rio.br/~roberto/docs/peg.pdf). Software: Practice and Experience, 39(3):221–258, 2009.
- Zachary Yedidia, [*Incremental PEG Parsing*](https://zyedidia.github.io/notes/yedidia_thesis.pdf). Ph.D. thesis, 2021. Chapter 3 ("A PEG Parsing Machine") is the clearest modern presentation of the instruction set used here and the reference to read first.

### Reference implementations

- [LPEG](http://www.inf.puc-rio.br/~roberto/lpeg/) — Roberto Ierusalimschy's original parsing-machine implementation, in Lua.
- [GPeg](https://github.com/zyedidia/gpeg) — Zachary Yedidia's Go implementation with incremental-parsing extensions; the most readable working reference for this codebase.
