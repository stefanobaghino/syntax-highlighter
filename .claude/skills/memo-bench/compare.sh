#!/usr/bin/env bash
# Compare the bench harness output between two git refs.
#
# Usage:
#   ./compare.sh <ref-before> <ref-after> [bench-name...]
#
# Default bench list: memo, incremental. Pass a subset to scope the run.
# Example: ./compare.sh main HEAD incremental
#
# Strategy:
#   - Create temporary git worktrees for each ref.
#   - Run `cargo bench --bench <name>` in each, collecting the JSONL
#     artifacts at target/bench-results/<name>.jsonl.
#   - Join rows by their identifying key fields (grammar/fixture/edit_kind
#     for incremental, grammar/fixture/threshold for memo) and print a
#     delta table.
#
# Depends on: git, cargo, jq.

set -euo pipefail

if [ $# -lt 2 ]; then
    cat >&2 <<'USAGE'
usage: compare.sh <ref-before> <ref-after> [bench-name...]
  benches: memo (default), incremental (default)
USAGE
    exit 2
fi

REF_A="$1"
REF_B="$2"
shift 2
if [ $# -eq 0 ]; then
    BENCHES=("memo" "incremental")
else
    BENCHES=("$@")
fi

command -v jq >/dev/null || { echo "jq is required" >&2; exit 1; }
command -v git >/dev/null || { echo "git is required" >&2; exit 1; }
command -v cargo >/dev/null || { echo "cargo is required" >&2; exit 1; }

REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT"

TMP=$(mktemp -d)
trap 'git worktree remove --force "$TMP/a" 2>/dev/null; git worktree remove --force "$TMP/b" 2>/dev/null; rm -rf "$TMP"' EXIT

echo "[1/4] preparing worktrees: a=$REF_A, b=$REF_B"
git worktree add --detach "$TMP/a" "$REF_A" >/dev/null
git worktree add --detach "$TMP/b" "$REF_B" >/dev/null

for bench in "${BENCHES[@]}"; do
    echo
    echo "[2/4] running bench '$bench' on $REF_A"
    (cd "$TMP/a" && cargo bench --bench "$bench" --quiet) >/dev/null
    echo "[3/4] running bench '$bench' on $REF_B"
    (cd "$TMP/b" && cargo bench --bench "$bench" --quiet) >/dev/null

    out_a="$TMP/a/target/bench-results/$bench.jsonl"
    out_b="$TMP/b/target/bench-results/$bench.jsonl"
    if [ ! -s "$out_a" ] || [ ! -s "$out_b" ]; then
        echo "  missing jsonl output for '$bench', skipping" >&2
        continue
    fi

    # Pick the identifying key fields and the primary timing column
    # per bench. The key set must be stable across commits so the join
    # finds matching rows.
    case "$bench" in
        memo)
            key='[.grammar, .fixture, (.threshold|tostring)] | join("/")'
            metric='.time_us'
            metric_label='time_us'
            ;;
        incremental)
            key='[.grammar, .fixture, .edit_kind] | join("/")'
            metric='.warm_us'
            metric_label='warm_us'
            ;;
        *)
            echo "  unknown bench '$bench', skipping" >&2
            continue
            ;;
    esac

    echo
    echo "[4/4] delta for '$bench' ($metric_label, $REF_A → $REF_B):"
    echo
    printf "  %-40s %10s %10s %10s\n" "key" "$REF_A" "$REF_B" "Δ%"
    printf "  %-40s %10s %10s %10s\n" "----------------------------------------" "----------" "----------" "----------"

    # Emit (key, metric) rows per file, then join on key.
    jq -r "\"\($key)\t\($metric)\"" "$out_a" | sort > "$TMP/a.tsv"
    jq -r "\"\($key)\t\($metric)\"" "$out_b" | sort > "$TMP/b.tsv"
    join -t $'\t' -a 1 -a 2 -e 'N/A' -o '0,1.2,2.2' "$TMP/a.tsv" "$TMP/b.tsv" |
        awk -F'\t' '{
            a=$2; b=$3;
            delta = (a == "N/A" || b == "N/A") ? "N/A" \
                : (a == 0 ? "inf" : sprintf("%+.1f%%", (b - a) / a * 100));
            printf "  %-40s %10s %10s %10s\n", $1, a, b, delta
        }'
done

echo
echo "done."
