---
name: grammar-refactor
description: Audit the shipped grammars (`grammars/*.peg`) for inlining and dead-code cleanup opportunities, in the spirit of PRs #123 and #136. Surfaces and classifies candidates — thin aliases, single-use rules, name-longer-than-body rules, character-class re-aliases, duplicate-body rules — without applying changes, because the keep-or-inline call is judgment-heavy. Invoke explicitly.
---

# grammar-refactor

A periodic cleanup sweep over `grammars/*.peg`. Each sweep tends to harvest a small, finite number of trivial-alias / thin-wrapper rules and then runs dry until the grammars accumulate more boilerplate. PRs #123 (the big sweep) and #136 (the small follow-up) are the reference shape. A sweep that finds zero candidates is a normal outcome.

## What it does NOT do

- It does not apply changes. The "evaluate" step is judgment-heavy (spec terminology, parallel structure, in-file rationale, atomicity constraints) and doesn't compress into a rule. The skill produces a classified candidate list and stops there. Applying is a follow-up.
- It does not consider semantic refactors (rule restructuring, FOLLOW boundary changes, capture-kind retagging). Pure surface-level inlining only.

## The four signals

Each candidate falls into one of four categories. The first two are mechanical, the next two are judgment calls.

1. **Char-class re-aliases (mechanical).** A rule whose body is just `[...]` (one or two character classes in sequence), and the rule name doesn't carry documentary intent beyond "this char class." Inline the class at use sites unless the name appears at many sites — a high-ref alias *is* the documentation.
2. **Duplicate-body rules (mechanical).** Two atomic rules with byte-identical bodies under different names — e.g. a grammar carrying both `ident_cont: atomic = [a-zA-Z\d_]` and a separate `ident_char: atomic = [a-zA-Z\d_]` boundary class. Merge to one name unless the dual naming documents a future divergence.
3. **Refs=1 single-use rules (judgment).** Many rules are referenced once. Most are NOT inline candidates — they exist to name a multi-line cascade entry, an alternation, or a spec-significant phrase. Only inline when the body is a single `name+` / `name*` / `name` / `[...]` / literal — i.e. a textbook thin wrapper.
4. **Name ≥ body (judgment).** When the rule name is at least as long as the body string, the indirection isn't paying its way. Same evaluation as refs=1: only inline trivial wrappers.

## Workflow

### 1. Generate stats

`pegc stats <grammar.peg>` prints a one-line JSON header plus NDJSON, one row per rule with `references` and `body_chars`. Loop over `grammars/*.peg`:

```bash
mkdir -p /tmp/grammar-refactor
for g in grammars/*.peg; do
  cargo run --bin pegc --quiet -- stats "$g" \
    > "/tmp/grammar-refactor/$(basename "$g" .peg).ndjson"
done
```

The first line of each file is the header (with `instructions` and `rules` totals); the rest are per-rule rows. `tail -n +2` skips the header for per-rule queries.

### 2. List candidates per category

**Char-class bodies (single class as the entire rule body):**

```bash
grep -n -E '^\s*\*?[a-z_]+\s*=\s*\[[^]]+\]\s*$' grammars/*.peg
```

**Duplicate atomic-rule bodies (same body, different names):**

Walk each grammar, normalize the body source (strip the name, any `: ascription` clause, and the `=`), group by body string, flag groups of size > 1.

**Refs=1 (single-use) and name ≥ body:**

```bash
for f in /tmp/grammar-refactor/*.ndjson; do
  g=$(basename "$f" .ndjson)
  echo "=== $g ==="
  echo "-- single-use, excluding reserved --"
  tail -n +2 "$f" | jq -r 'select(.references==1 and .rule!="trivia" and .rule!="root" and .rule!="wb") | "\(.rule) (\(.body_chars) chars)"'
  echo "-- name length >= body chars, excluding reserved --"
  tail -n +2 "$f" | jq -r 'select((.rule|length) >= .body_chars and .rule!="trivia" and .rule!="root" and .rule!="wb") | "\(.rule) (name=\(.rule|length), body=\(.body_chars), refs=\(.references))"'
done
```

`root`, `trivia`, and `wb` are reserved (`src/pegc/parser.rs` enforces) — never candidates. `reserved` and `preferred` are compiler-*synthesized* (from the `reserved` / `preferred` rules) and can't be defined at all. A `reserved`- or `preferred`-ascribed rule is also off-limits to inlining: see the *Atomicity check* below.

### 3. For each candidate, evaluate

Open the grammar, read the rule, read its callers, and read any comments above the rule or the call sites. Classify into one of:

- **Mechanical inline:** body is `name+` / `name*` / `name` / `[...]` / a single literal; refs=1 (or refs≤5 and the name carries no documentary weight beyond the body itself); no comment above the rule explaining its purpose; **atomicity matches at every callsite** (see *Atomicity check* below).
- **Judgment-call keep:** the rule name names a spec phrase (e.g. `compound_selector` = CSS Selectors L4 term; `decl_spec_list` = C99 standard naming; `type_constraint` = Go spec name; `expr_no_comma` = JS spec `AssignmentExpression`'s comma-free position) — the alias documents which production this is.
- **Documented keep:** there's an explanatory comment above the rule (e.g. `rust.peg`'s `type_expr_suffix_marker` — "Split from the cascade so the type_expr stays attached") that argues for the indirection. Don't inline these without explicit human review.
- **Parallel-structure keep:** the rule is one of a sibling group whose names line up at use sites (e.g. `toml.peg`'s `local_date`/`local_time`/`local_datetime`/`full_datetime` in the value alternation, or `*hex_digits`/`*oct_digits`/`*bin_digits` in number cascades). Inlining one breaks the visual cluster.

### 4. Atomicity check (correctness-critical)

Inlining changes which rule the auto-trivia rewriter is walking. From `src/pegc/analysis.rs::inject_auto_trivia`:

- Trivia is auto-inserted between consecutive Sequence elements *of non-atomic rules*. NonTerminal calls are leaves from the rewriter's POV.
- Inlining an **atomic** rule body into a **non-atomic** caller exposes the body to auto-injection — trivia will now appear between its elements where it didn't before.
- Inlining a **non-atomic** rule body into an **atomic** caller suppresses auto-injection where it used to fire.

Safe shapes regardless of atomicity:
- Body is a single char class, single literal, single NonTerminal, or single Repeat-of-NonTerminal — no internal Sequence, so no internal trivia injection to gain or lose.

When the body has internal Sequence elements (e.g. `'foo' bar baz`), inlining is **only safe when source and target rule atomicity match**. Verify by reading the `atomic` (or `reserved` / `preferred`) ascription on both rule headers before changing anything.

**`reserved` / `preferred` rules are never inline candidates.** A `name: reserved =` (or `name: preferred =`) rule carries an *implicit* trailing `wb` boundary that the compiler appends inside the rule's captures; the boundary is invisible in the rule body, so inlining the body silently drops it — the same class of hazard as the atomic case above, but worse because there's no `!ident_body` in the source to tip you off. The rule's literals are also collected into the synthesized `reserved` / `preferred` sets, so renaming or splitting it shifts what `!reserved` blocks. Leave `reserved` / `preferred` rules (and the `wb` rule itself) alone.

### 5. Report

Produce a table grouped by category:

| grammar | rule | body | refs | classification | reason |
|---|---|---|---|---|---|
| ... | ... | ... | ... | mechanical / spec-name / documented / parallel / reserved | one-line note |

Lead with the "mechanical" rows (the actionable ones); fold the "keep" rows into a `<details>` block when the report goes into a PR description.

### 6. Hand off

If the user wants to apply: spin up a worktree, edit each candidate, run `cargo check && cargo test && cargo fmt --check && cargo clippy --all-targets -- -D warnings`, re-run `pegc stats` to confirm bytecode shape (rules should drop by N, instructions should drop modestly), open a PR in the spirit of #136 with a pre/post bytecode table.

The applied edit is mechanical at this point — the judgment was the previous step. Don't conflate the two.

## Anti-patterns

- **Bulk-inlining all refs=1 rules.** Most refs=1 rules are spec-significant cascade entries (every binary-precedence level, every statement form, every type-position-specific entry). Inlining them produces an unreadable wall of alternation.
- **Inlining across atomicity boundaries without checking.** Will silently change parsing behavior — usually in a way that's not caught by the unit tests but shows up in the highlighter fixtures or recovery baselines.
- **Forgetting to update the grammar header comment** when removing a rule that's named there.
- **Touching `root`, `trivia`, `wb`, `reserved`, `preferred`, or any `reserved` / `preferred` rule.** `root` / `trivia` / `wb` are reserved (the parser rejects grammars that mangle their position); `reserved` / `preferred` are compiler-synthesized (defining them is a parse error); `reserved` / `preferred` rules carry an invisible trailing `wb` boundary that inlining would drop *and* feed the synthesized sets. Filter all of these out before evaluating.
- **Touching the `sqlite.peg` `kw_*` / `*_body` cluster.** Each `kw_*` rule has refs=1 by construction (one keyword, one statement-rule use), but the two-tier `kw_*` → `*_body` design is a deliberate convention. The `kw_*: reserved` literals are also the source of the synthesized `reserved` set (the no-keyword-as-identifier lookahead, `!reserved`); window-vocabulary keywords are `kw_*: preferred` so they stay usable as identifiers. Don't sweep these.

## Reference PRs

- #123 — the big sweep across all eight grammars (post-#86 / #87 cleanup; collapsed `trivia = ws`, inlined thin aliases, folded block-comment patterns into `..=`).
- #136 — the small follow-up (four trivial single-use aliases left after #123 and the implicit-`root`/`trivia` PR #132).
