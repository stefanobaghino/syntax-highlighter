---
name: memo-bench
description: Run and interpret the memoization-threshold sweep benchmark at `benches/memo.rs`. Use when the user wants to execute `cargo bench --bench memo`, decide on a threshold default for the VM's `with_memo_threshold`, read the sweep table, or diagnose unexpected output (non-monotone entries, correctness-guard failures, missing hits).
---

# memo-bench

The harness at `benches/memo.rs` sweeps `VM::with_memo_threshold`
(`src/pegvm/vm.rs`) across `(threshold × grammar × input)` and prints
one row per cell.

## Why the filter exists

Packrat memoization caches every successful rule body, including leaf
rules whose matched span is one or two bytes. Those entries pay the
`HashMap` insert-and-lookup cost on every invocation but rarely produce
a useful replay — a short rule is typically cheaper to re-execute than
to cache. Filtering them out is the classic memory-vs-time lever:
GPeg defaults to 512 bytes and benchmarks at 128; Yedidia's thesis
§5.2.4 finds the knee around 4096. The right value is workload- and
hardware-dependent, which is what this harness measures.

## Running it

```
cargo bench --bench memo
```

Output goes to stdout — it is not a file artifact. A full run takes a
few seconds on a modern laptop. The harness has no flags; to adjust
the threshold axis or runs-per-cell, edit the `THRESHOLDS` and
`RUNS_PER_CELL` constants at the top of `benches/memo.rs`.

The bench is deterministic in everything except wall time:
`(entries, hits, misses)` are stable across runs for a given
`(grammar, input, threshold)` cell, so a single execution is enough to
compare cells.

## Columns

| Column | Unit | Meaning |
|---|---|---|
| `input` | — | Fixture label (`small` / `medium` / `large`). Sources in `benches/fixtures/`. |
| `thresh` | bytes | The `memo_threshold` value in effect. `MemoClose` only writes back entries whose matched span is `≥ thresh`. |
| `time(us)` | µs | Median of `RUNS_PER_CELL` (default 11) wall-clock runs. |
| `entries` | count | `MemoStats.entries` — distinct `(memo_id, start_sp)` slots in the memo table at end of run. Includes failure entries (which are not threshold-filtered). |
| `hits` | count | `MemoStats.hits` — `MemoOpen` invocations that resolved via cache. |
| `misses` | count | `MemoStats.misses` — `MemoOpen` invocations that had to execute the rule body. `hits + misses` = total lookups. |
| `hit_rate` | % | `hits / (hits + misses)`. Use this, not `hits / entries` — they differ once `thresh > 0`. |

## Reading the sweep

Typical healthy shape for one grammar/input pair:

1. **`entries` decreases monotonically** as `thresh` grows. The filter
   is dropping successful spans shorter than the threshold — expected.
   Non-monotone is a bug: failure entries should be untouched, and the
   remaining successful spans can only shrink in count. Dig into
   `MemoClose` in `src/pegvm/vm.rs` if this is violated.
2. **`time(us)` drops sharply at the first non-zero threshold, then
   flattens.** The drop is the cost of storing-and-never-reading tiny
   leaf entries going away. The flat region *is* the range where the
   threshold is paying for itself without sacrificing useful hits.
3. **`hit_rate` usually *rises* from `thresh = 0` to `thresh = 32`** —
   the filtered entries weren't producing hits anyway, so removing
   them concentrates the rate on the entries that do. A drop in
   `hit_rate` past some threshold means you've started filtering out
   useful cached spans; that's the far side of the knee.
4. **Pick the threshold at the knee of the time curve** that still
   preserves `hit_rate`. The shipped default
   (`VM::DEFAULT_MEMO_THRESHOLD`, 128 bytes) was chosen from this
   harness to sit in the flat region of the time curve while matching
   GPeg's benchmark reference point. Reference values: GPeg ships 512;
   Yedidia's thesis §5.2.4 finds ≈4096. Hardware- and workload-dependent,
   which is why this harness exists — rerun it rather than trusting the
   literature numbers directly.

## When to re-run

- **After adding a new grammar.** A larger or differently-shaped grammar
  can push the knee upward; if the flat region no longer includes the
  shipping `DEFAULT_MEMO_THRESHOLD`, raise it. This is the main
  maintenance trigger — the roadmap entry for "More language grammars"
  (`README.md`) calls it out.
- **After significant VM changes** that alter memo-table access patterns
  (new instructions, changes to `MemoOpen`/`MemoClose`, memo replay
  semantics). The correctness guard catches breakage; the sweep shape
  tells you whether the tuning is still right.
- **Not after plain VM changes** that don't touch memoization — the
  bench is opinionated about what it measures, not a general
  regression check.

## Correctness guard

For every cell the harness asserts `(matched, complete)` is identical
to the `thresh = 0` baseline. The threshold filter must only change
*what is cached*, never *what is parsed*. If this assertion fires the
bench aborts — the `MemoClose` gate has corrupted semantics. The
capture buffer is the usual suspect (entries skipped by the gate still
need their live captures preserved — see the comment on the gate in
`src/pegvm/vm.rs`).

## What the output does not tell you

- **Bytes of memory used.** `entries` is a count, not a size. Scaling
  to bytes requires knowing `MemoEntry` width and the `HashMap`
  overhead — neither is worth the effort at this stage.
- **Statistical bounds on timing.** Median-of-11 is not a confidence
  interval. The threshold knee is an order-of-magnitude decision; if
  two adjacent thresholds look within a few percent of each other,
  treat them as tied.
- **Regression across commits.** The harness prints absolute numbers,
  not diffs. To compare two branches, run on both and diff by eye.
