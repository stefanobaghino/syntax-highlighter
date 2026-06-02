//! `pegdb` — grammar debug tool.
//!
//! Noun-verb subcommands. Today: `captures dump`, `recoveries dump`,
//! `recoveries explain`. JSONL output (one JSON object per
//! `\n`-delimited line) for the `dump` verbs; plain-text clustered
//! summary for `recoveries explain`. See `TOOLS.md` at the repo root
//! for the full contract. Compile-time grammar inspection (`stats`,
//! etc.) lives in `pegc`.

use std::borrow::Cow;
use std::fmt::Write as _;
use std::io::{Read, Write};
use std::process::ExitCode;

use syntax_highlighter::pegvm::{Capture, MemoId, RecoveryDiagnostic, RecoveryOrigin};
use syntax_highlighter_compiler::parser::Parser;

const TOP_HELP: &str = "\
pegdb — grammar-developer debug surface for syntax-highlighter

Usage:
    pegdb <noun> <verb> [options] [args]

Nouns:
    captures      Per-capture detail of a parse.
    recoveries    `*^` recovery firings — detail or summary.

Options:
    -h, --help    Show this help.

For compile-time grammar inspection (bytecode shape), see `pegc`.
See TOOLS.md for the full contract: subcommand semantics, JSONL schemas,
exit codes, partial-match diagnostics.
";

const CAPTURES_HELP: &str = "\
Usage: pegdb captures <verb> [options] [args]

Verbs:
    dump          Print every capture as JSONL (one object per capture).

See `pegdb captures <verb> --help` for the per-verb contract, or
TOOLS.md for the full schema.
";

const CAPTURES_DUMP_HELP: &str = "\
Usage: pegdb captures dump -g <grammar.peg> [--max-literal=N] [<path>]

Print one capture per line as a JSON object (keys: start, end, kind,
depth, literal). Exits 1 with a stderr partial-match marker on
incomplete parses. The grammar source is required. Recovery diagnostics
live in `pegdb recoveries dump` and `pegdb recoveries explain`.

See TOOLS.md for the full contract.
";

const RECOVERIES_HELP: &str = "\
Usage: pegdb recoveries <verb> [options] [args]

Verbs:
    dump          One JSONL row per recovery span (capture + diagnostic inline).
    explain       Cluster `*^` recoveries by rule-stack suffix.

See `pegdb recoveries <verb> --help` for the per-verb contract, or
TOOLS.md for the full schema.
";

const RECOVERIES_DUMP_HELP: &str = "\
Usage: pegdb recoveries dump -g <grammar.peg> [--max-literal=N] [<path>]

Print one JSON object per surviving recovery span (keys: start, end,
kind, label, pos, rule_stack, literal). One row per span — adjacent
single-byte recovery captures from the same `*^` loop collapse into
a single object. `rule_stack` is the full ignore-trimmed call stack
root-to-leaf at the deepest dive that contributed to the span.

When capture/diagnostic accounting mismatches (a known bug class —
see TOOLS.md), additional `\"sanity\":\"orphan_*\"` rows surface the
unmatched halves. Silent on well-formed inputs.

Exits 1 with a stderr partial-match marker on incomplete parses. The
grammar source is required.

See TOOLS.md for the full contract.
";

const RECOVERIES_EXPLAIN_HELP: &str = "\
Usage: pegdb recoveries explain -g <grammar.peg> [<path>]

Cluster `*^` recoveries by rule-stack suffix and print one line per
cluster, sorted by count descending. Each cluster reports the number
of recovery firings and the deepest rule reached during the failed
iterations that produced them. The grammar source is required.

When capture/diagnostic accounting mismatches, additional `[sanity]`-
prefixed lines surface the unmatched halves. Silent on well-formed
inputs.

See TOOLS.md for the full contract.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        print!("{}", TOP_HELP);
        return ExitCode::SUCCESS;
    }
    let (noun, rest) = args.split_first().unwrap();
    match noun.as_str() {
        "captures" => dispatch_captures(rest),
        "recoveries" => dispatch_recoveries(rest),
        "-h" | "--help" => {
            print!("{}", TOP_HELP);
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("pegdb: unknown noun {:?}", other);
            eprintln!();
            eprint!("{}", TOP_HELP);
            ExitCode::from(2)
        }
    }
}

fn dispatch_captures(args: &[String]) -> ExitCode {
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        print!("{}", CAPTURES_HELP);
        return ExitCode::SUCCESS;
    }
    let (verb, rest) = args.split_first().unwrap();
    match verb.as_str() {
        "dump" => run_captures_dump(rest),
        other => {
            eprintln!("pegdb captures: unknown verb {:?}", other);
            eprintln!();
            eprint!("{}", CAPTURES_HELP);
            ExitCode::from(2)
        }
    }
}

fn dispatch_recoveries(args: &[String]) -> ExitCode {
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        print!("{}", RECOVERIES_HELP);
        return ExitCode::SUCCESS;
    }
    let (verb, rest) = args.split_first().unwrap();
    match verb.as_str() {
        "dump" => run_recoveries_dump(rest),
        "explain" => run_recoveries_explain(rest),
        other => {
            eprintln!("pegdb recoveries: unknown verb {:?}", other);
            eprintln!();
            eprint!("{}", RECOVERIES_HELP);
            ExitCode::from(2)
        }
    }
}

fn run_captures_dump(args: &[String]) -> ExitCode {
    let sub = "captures dump";
    if wants_help(args) {
        print!("{}", CAPTURES_DUMP_HELP);
        return ExitCode::SUCCESS;
    }
    let (max_literal, rest) = match extract_max_literal(args, sub) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let parsed = match parse_fixture_args(&rest, sub, CAPTURES_DUMP_HELP) {
        FixtureArgs::Help => return ExitCode::SUCCESS,
        FixtureArgs::Err(code) => return code,
        FixtureArgs::Ok(p) => p,
    };
    let (grammar, input, source_label) = match load_fixture(&parsed, sub) {
        Ok(t) => t,
        Err(code) => return code,
    };
    let mut p = match Parser::new(&grammar) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("pegdb {}: grammar error: {}", sub, e);
            return ExitCode::from(3);
        }
    };
    p.set_input(input.into_bytes());
    let kinds = p.capture_kinds();
    let complete = p.is_complete();
    let (matched, captures) = p.captures();
    let view_bytes = p.input();
    // Input arrived through `read_to_string` (or a `String` literal in
    // tests), so `Parser` only ever held valid UTF-8 — no failure path.
    let view = std::str::from_utf8(view_bytes)
        .expect("Parser input originated as String; bytes must round-trip as UTF-8");

    let mut out = std::io::stdout().lock();
    // Captures arrive in CaptureBegin order — start-ascending, with a
    // parent always appearing before its nested children (the parent's
    // Begin fires first). A linear stack-walk maps each capture to its
    // nesting depth: pop entries whose end has already passed our start
    // (siblings/ancestors that closed), then `depth = stack.len()`.
    let mut open_ends: Vec<usize> = Vec::new();
    for cap in captures.iter() {
        while open_ends.last().is_some_and(|&end| end <= cap.start) {
            open_ends.pop();
        }
        let depth = open_ends.len();
        open_ends.push(cap.end);

        let kind = kinds
            .get(cap.kind.0 as usize)
            .map(String::as_str)
            .unwrap_or("<unknown>");
        let raw = &view[cap.start..cap.end];
        let truncated: Cow<'_, str> = match max_literal {
            Some(n) => truncate_with_ellipsis(raw, n),
            None => Cow::Borrowed(raw),
        };
        let _ = writeln!(
            out,
            "{{\"start\":{},\"end\":{},\"kind\":{},\"depth\":{},\"literal\":{}}}",
            cap.start,
            cap.end,
            json_string(kind),
            depth,
            json_string(&truncated),
        );
    }
    if !complete {
        eprintln!(
            "partial-match {}: matched {} of {} bytes",
            source_label,
            matched,
            view.len()
        );
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn run_recoveries_dump(args: &[String]) -> ExitCode {
    let sub = "recoveries dump";
    if wants_help(args) {
        print!("{}", RECOVERIES_DUMP_HELP);
        return ExitCode::SUCCESS;
    }
    let (max_literal, rest) = match extract_max_literal(args, sub) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let parsed = match parse_fixture_args(&rest, sub, RECOVERIES_DUMP_HELP) {
        FixtureArgs::Help => return ExitCode::SUCCESS,
        FixtureArgs::Err(code) => return code,
        FixtureArgs::Ok(p) => p,
    };
    let (grammar, input, source_label) = match load_fixture(&parsed, sub) {
        Ok(t) => t,
        Err(code) => return code,
    };
    let mut p = match Parser::new(&grammar) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("pegdb {}: grammar error: {}", sub, e);
            return ExitCode::from(3);
        }
    };
    p = p.with_track_recovery_diagnostics(true);
    p.set_input(input.into_bytes());
    let kinds = p.capture_kinds();
    let rule_names = p.rule_names();
    let rule_is_ignore = p.rule_is_ignore();
    let label_names = p.label_kinds();
    let complete = p.is_complete();
    let (matched, captures) = p.captures();
    let diagnostics = p.recovery_diagnostics();
    let view_bytes = p.input();
    let view = std::str::from_utf8(view_bytes)
        .expect("Parser input originated as String; bytes must round-trip as UTF-8");

    let recovery_kind_idx = kinds.iter().position(|k| k == "recovery");
    let span_aggregates =
        compute_recovery_span_aggregates(captures, diagnostics, recovery_kind_idx);

    let mut out = std::io::stdout().lock();

    // Walk captures and emit one row per span. The contiguity scan
    // mirrors `compute_recovery_span_aggregates`'s internal logic: a
    // span is a maximal contiguous run of `kind == "recovery"` captures
    // where adjacent members touch.
    let is_recovery = |c: &Capture| recovery_kind_idx.is_some_and(|k| c.kind.0 as usize == k);

    let mut i = 0;
    while i < captures.len() {
        if !is_recovery(&captures[i]) {
            i += 1;
            continue;
        }
        // Found the start of a recovery span. The aggregate gives us
        // the argmax-pos diagnostic; if it's `None`, this is an
        // orphan-capture span (surfaced in the trailing sanity pass)
        // — skip emission here and continue the scan past the span.
        let span_start_idx = i;
        let mut span_end_idx = i + 1;
        while span_end_idx < captures.len()
            && is_recovery(&captures[span_end_idx])
            && captures[span_end_idx - 1].end == captures[span_end_idx].start
        {
            span_end_idx += 1;
        }
        let Some(reach) = span_aggregates[span_start_idx].as_ref() else {
            i = span_end_idx;
            continue;
        };
        let span_start = captures[span_start_idx].start;
        let span_end = captures[span_end_idx - 1].end;

        let trimmed = trim_ignore_tail(reach.rule_stack(), rule_is_ignore);
        let stack_names: Vec<&str> = trimmed
            .iter()
            .filter_map(|id| rule_names.get(id.0 as usize).map(String::as_str))
            .collect();
        let label = reach
            .label_id()
            .and_then(|l| label_names.get(l.0 as usize).map(String::as_str))
            .unwrap_or(match &reach.origin {
                RecoveryOrigin::Utf8 => "utf8",
                RecoveryOrigin::Grammar { .. } => "",
            });
        let kind = kinds
            .get(captures[span_start_idx].kind.0 as usize)
            .map(String::as_str)
            .unwrap_or("recovery");
        let lossy = String::from_utf8_lossy(&view_bytes[span_start..span_end]);
        let literal: Cow<'_, str> = match max_literal {
            Some(n) => Cow::Owned(truncate_with_ellipsis(&lossy, n).into_owned()),
            None => lossy,
        };

        let _ = writeln!(
            out,
            "{{\"start\":{},\"end\":{},\"kind\":{},\"origin\":{},\"label\":{},\"pos\":{},\"rule_stack\":{},\"literal\":{}}}",
            span_start,
            span_end,
            json_string(kind),
            json_string(reach.origin.tag()),
            json_string(label),
            reach.pos,
            json_string_array(&stack_names),
            json_string(&literal),
        );

        i = span_end_idx;
    }

    // Orphan accounting — silent in the happy path. See `detect_orphans`
    // for the indices; emit one row per orphan half.
    let (orphan_captures, orphan_diagnostics) =
        detect_orphans(captures, diagnostics, &span_aggregates, recovery_kind_idx);

    for idx in &orphan_captures {
        let cap = &captures[*idx];
        let raw = &view[cap.start..cap.end];
        let literal: Cow<'_, str> = match max_literal {
            Some(n) => truncate_with_ellipsis(raw, n),
            None => Cow::Borrowed(raw),
        };
        let _ = writeln!(
            out,
            "{{\"sanity\":\"orphan_capture\",\"start\":{},\"end\":{},\"literal\":{}}}",
            cap.start,
            cap.end,
            json_string(&literal),
        );
    }
    for idx in &orphan_diagnostics {
        let diag = &diagnostics[*idx];
        let trimmed = trim_ignore_tail(diag.rule_stack(), rule_is_ignore);
        let stack_names: Vec<&str> = trimmed
            .iter()
            .filter_map(|id| rule_names.get(id.0 as usize).map(String::as_str))
            .collect();
        let label = diag
            .label_id()
            .and_then(|l| label_names.get(l.0 as usize).map(String::as_str))
            .unwrap_or(match &diag.origin {
                RecoveryOrigin::Utf8 => "utf8",
                RecoveryOrigin::Grammar { .. } => "",
            });
        let _ = writeln!(
            out,
            "{{\"sanity\":\"orphan_diagnostic\",\"pos\":{},\"origin\":{},\"label\":{},\"rule_stack\":{}}}",
            diag.pos,
            json_string(diag.origin.tag()),
            json_string(label),
            json_string_array(&stack_names),
        );
    }

    if !complete {
        eprintln!(
            "partial-match {}: matched {} of {} bytes",
            source_label,
            matched,
            view.len()
        );
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn run_recoveries_explain(args: &[String]) -> ExitCode {
    let sub = "recoveries explain";
    if wants_help(args) {
        print!("{}", RECOVERIES_EXPLAIN_HELP);
        return ExitCode::SUCCESS;
    }
    let parsed = match parse_fixture_args(args, sub, RECOVERIES_EXPLAIN_HELP) {
        FixtureArgs::Help => return ExitCode::SUCCESS,
        FixtureArgs::Err(code) => return code,
        FixtureArgs::Ok(p) => p,
    };
    let (grammar, input, source_label) = match load_fixture(&parsed, sub) {
        Ok(t) => t,
        Err(code) => return code,
    };
    let mut p = match Parser::new(&grammar) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("pegdb {}: grammar error: {}", sub, e);
            return ExitCode::from(3);
        }
    };
    p = p.with_track_recovery_diagnostics(true);
    p.set_input(input.into_bytes());
    let kinds = p.capture_kinds();
    let rule_names = p.rule_names();
    let rule_is_ignore = p.rule_is_ignore();
    let complete = p.is_complete();
    let (matched, captures) = p.captures();
    let diagnostics = p.recovery_diagnostics();

    let recovery_kind_idx = kinds.iter().position(|k| k == "recovery");
    let span_aggregates =
        compute_recovery_span_aggregates(captures, diagnostics, recovery_kind_idx);
    let label_names = p.label_kinds();

    // Cluster by `(rule_stack, label_name)`. Every recovery firing
    // carries a label — labeled catches (`^label`) use the author's
    // identifier; `*^` uses the intern of its `recovery_kind` string.
    // Trailing rule_stack frames marked ignore (reached from a
    // `ignore = …` reserved-name root) are popped before the key is
    // built, so two diagnostics that bottom out in different ignore
    // merge into one cluster on the deepest semantic rule. BTreeMap
    // preserves a stable key order for deterministic output.
    type ClusterKey = (Vec<String>, String);
    let mut clusters: std::collections::BTreeMap<ClusterKey, usize> =
        std::collections::BTreeMap::new();
    for slot in &span_aggregates {
        let Some(reach) = slot else {
            continue;
        };
        let trimmed_stack = trim_ignore_tail(reach.rule_stack(), rule_is_ignore);
        let stack: Vec<String> = trimmed_stack
            .iter()
            .filter_map(|id| rule_names.get(id.0 as usize).cloned())
            .collect();
        let label = reach
            .label_id()
            .and_then(|l| label_names.get(l.0 as usize).cloned())
            .unwrap_or_else(|| match &reach.origin {
                RecoveryOrigin::Utf8 => "utf8".to_string(),
                RecoveryOrigin::Grammar { .. } => String::new(),
            });
        *clusters.entry((stack, label)).or_insert(0) += 1;
    }

    let mut entries: Vec<(ClusterKey, usize)> = clusters.into_iter().collect();
    // Sort by count descending, then by key lexicographically for
    // determinism on ties.
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let (orphan_captures, orphan_diagnostics) =
        detect_orphans(captures, diagnostics, &span_aggregates, recovery_kind_idx);

    let mut out = std::io::stdout().lock();
    for ((stack, label), count) in &entries {
        let leaf = stack.last().map(String::as_str).unwrap_or("<empty>");
        let _ = writeln!(
            out,
            "{} recoveries — farthest reach ends at {} (label: {})",
            count, leaf, label
        );
    }
    if entries.is_empty() && orphan_captures.is_empty() && orphan_diagnostics.is_empty() {
        let _ = writeln!(out, "no recoveries");
    }
    if !orphan_captures.is_empty() {
        let ranges: Vec<String> = orphan_captures
            .iter()
            .map(|&i| format!("{}..{}", captures[i].start, captures[i].end))
            .collect();
        let _ = writeln!(
            out,
            "[sanity] {} orphan recovery capture{} (no diagnostic): bytes {}",
            orphan_captures.len(),
            if orphan_captures.len() == 1 { "" } else { "s" },
            ranges.join(", "),
        );
    }
    if !orphan_diagnostics.is_empty() {
        for &idx in &orphan_diagnostics {
            let diag = &diagnostics[idx];
            let trimmed = trim_ignore_tail(diag.rule_stack(), rule_is_ignore);
            let leaf = trimmed
                .last()
                .and_then(|id| rule_names.get(id.0 as usize))
                .map(String::as_str)
                .unwrap_or("<empty>");
            let label = diag
                .label_id()
                .and_then(|l| label_names.get(l.0 as usize).map(String::as_str))
                .unwrap_or(match &diag.origin {
                    RecoveryOrigin::Utf8 => "utf8",
                    RecoveryOrigin::Grammar { .. } => "",
                });
            let _ = writeln!(
                out,
                "[sanity] orphan diagnostic (no surviving capture): pos={} rule={} label={}",
                diag.pos, leaf, label,
            );
        }
    }
    if !complete {
        eprintln!(
            "partial-match {}: matched {} of {} bytes",
            source_label,
            matched,
            p.input().len()
        );
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

/// Truncate `s` at or before `max` bytes on a UTF-8 char boundary,
/// appending a `…` ellipsis. Returns the input borrowed if it already
/// fits, so the no-truncation path skips the allocation.
fn truncate_with_ellipsis(s: &str, max: usize) -> Cow<'_, str> {
    if s.len() <= max {
        return Cow::Borrowed(s);
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    let mut out = s[..end].to_string();
    out.push('…');
    Cow::Owned(out)
}

/// Per-capture span-aggregate slot. `Some(diag)` on every capture in
/// a recovery span carries the same diagnostic (the one with the
/// largest `pos` across the span); `None` everywhere else. The
/// aggregate is the most actionable signal: the worst dive that
/// contributed to this contiguous recovery run.
type RecoverySpanAggregates = Vec<Option<RecoveryDiagnostic>>;

/// Walk `captures` and produce one slot per capture: the span's
/// argmax-`pos` diagnostic for recovery rows, `None` for everything
/// else. A span is a maximal contiguous run of `kind == "recovery"`
/// captures where adjacent members touch (`cap[i].end ==
/// cap[i+1].start`). Diagnostics arrive aligned to recovery captures
/// in emission order; we step a single index through them.
fn compute_recovery_span_aggregates(
    captures: &[Capture],
    diagnostics: &[RecoveryDiagnostic],
    recovery_kind_idx: Option<usize>,
) -> RecoverySpanAggregates {
    let mut out: RecoverySpanAggregates = vec![None; captures.len()];
    let Some(recovery_idx) = recovery_kind_idx else {
        return out;
    };
    let is_recovery = |c: &Capture| c.kind.0 as usize == recovery_idx;

    // Diagnostics align with recovery captures in emission (= start)
    // order. Build a parallel lookup keyed by capture_index so we
    // tolerate any drift (e.g. a partial-match path that drops some
    // captures) without crashing.
    let mut diag_by_index: std::collections::HashMap<usize, &RecoveryDiagnostic> =
        std::collections::HashMap::with_capacity(diagnostics.len());
    for d in diagnostics {
        diag_by_index.insert(d.capture_index, d);
    }

    let mut i = 0;
    while i < captures.len() {
        if !is_recovery(&captures[i]) {
            i += 1;
            continue;
        }
        // Found the start of a recovery span. Walk forward while the
        // next capture is also recovery and adjacent.
        let span_start = i;
        let mut span_end = i + 1;
        while span_end < captures.len()
            && is_recovery(&captures[span_end])
            && captures[span_end - 1].end == captures[span_end].start
        {
            span_end += 1;
        }
        // Argmax over the span's diagnostics by `pos`.
        let mut best: Option<&RecoveryDiagnostic> = None;
        for j in span_start..span_end {
            if let Some(d) = diag_by_index.get(&j).copied() {
                best = match best {
                    None => Some(d),
                    Some(prev) if d.pos > prev.pos => Some(d),
                    Some(prev) => Some(prev),
                };
            }
        }
        if let Some(b) = best {
            let cloned = b.clone();
            for slot in out.iter_mut().take(span_end).skip(span_start) {
                *slot = Some(cloned.clone());
            }
        }
        i = span_end;
    }
    out
}

/// Pop trailing ignore frames from `stack`. A frame is ignore when its
/// `MemoId.0` indexes a `true` slot in `rule_is_ignore` (cascaded from
/// the reserved `ignore = …` rule at compile time). Both
/// `recoveries explain` and `recoveries dump` use this to keep the
/// displayed leaf on a semantically interesting rule rather than `ws`
/// or `line_comment`.
fn trim_ignore_tail(stack: &[MemoId], rule_is_ignore: &[bool]) -> Vec<MemoId> {
    let mut out = stack.to_vec();
    while let Some(last) = out.last() {
        if rule_is_ignore
            .get(last.0 as usize)
            .copied()
            .unwrap_or(false)
        {
            out.pop();
        } else {
            break;
        }
    }
    out
}

/// Cross-check capture⇄diagnostic accounting. Returns `(orphan_captures,
/// orphan_diagnostics)` where each entry is an index into the
/// corresponding slice.
///
/// - **orphan_capture**: a `kind == "recovery"` capture whose span
///   aggregate is `None` — every diagnostic in the span got dropped or
///   misaligned somewhere upstream.
/// - **orphan_diagnostic**: a diagnostic whose `capture_index` does not
///   appear in any surviving span aggregate's correlation set.
///
/// Both lists are empty on well-formed runs. Non-empty lists indicate
/// the bug class PR #101 fixed (`finalize_recovery_diagnostics`
/// silently dropping multi-byte diagnostics) or any future filter
/// drift of the same shape.
fn detect_orphans(
    captures: &[Capture],
    diagnostics: &[RecoveryDiagnostic],
    span_aggregates: &[Option<RecoveryDiagnostic>],
    recovery_kind_idx: Option<usize>,
) -> (Vec<usize>, Vec<usize>) {
    let mut orphan_captures: Vec<usize> = Vec::new();
    let Some(recovery_idx) = recovery_kind_idx else {
        // Without a `recovery` kind there can be no recovery captures
        // and no diagnostics either; both lists are vacuously empty.
        return (orphan_captures, Vec::new());
    };
    for i in 0..captures.len() {
        if captures[i].kind.0 as usize == recovery_idx && span_aggregates[i].is_none() {
            orphan_captures.push(i);
        }
    }
    // The set of capture indices that belong to some recovery span —
    // i.e., slots where the aggregator placed a (cloned) diagnostic.
    // A diagnostic is "in a span" iff its `capture_index` indexes a
    // slot in this set. Slots collide on `capture_index` legitimately
    // when multiple iterations within one span attach to consecutive
    // capture slots; using the argmax's `capture_index` alone would
    // miss the rest of the span's diagnostics and surface false
    // orphans.
    let in_span: std::collections::HashSet<usize> = (0..span_aggregates.len())
        .filter(|&i| span_aggregates[i].is_some())
        .collect();
    let mut orphan_diagnostics: Vec<usize> = Vec::new();
    for (i, d) in diagnostics.iter().enumerate() {
        if !in_span.contains(&d.capture_index) {
            orphan_diagnostics.push(i);
        }
    }
    (orphan_captures, orphan_diagnostics)
}

/// Encode `s` as a JSON string literal (surrounding double quotes
/// included, control chars escaped, ESC `\x1b` escaped to neutralize
/// terminal-color injection through capture content).
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            c if (c as u32) < 0x20 || c == '\x7f' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Encode `items` as a JSON array of strings — `[ "a", "b", ... ]`.
fn json_string_array(items: &[&str]) -> String {
    let mut out = String::with_capacity(2 + items.iter().map(|s| s.len() + 4).sum::<usize>());
    out.push('[');
    let mut first = true;
    for item in items {
        if !first {
            out.push(',');
        }
        first = false;
        out.push_str(&json_string(item));
    }
    out.push(']');
    out
}

/// Parsed `-g <grammar.peg> [<path>]` shape. Subcommand-specific
/// flags are extracted by the handler before this shared parser runs
/// (see the `--max-literal=N` pre-pass on the `dump` verbs), which
/// keeps unknown flags genuinely unknown when a future subcommand
/// without that flag is added.
struct ParsedFixture<'a> {
    grammar_path: Option<&'a str>,
    path: Option<&'a str>,
}

enum FixtureArgs<'a> {
    Ok(ParsedFixture<'a>),
    Help,
    Err(ExitCode),
}

fn parse_fixture_args<'a>(args: &'a [String], sub: &str, help: &str) -> FixtureArgs<'a> {
    let mut grammar_path: Option<&'a str> = None;
    let mut path: Option<&'a str> = None;
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "-h" | "--help" => {
                print!("{}", help);
                return FixtureArgs::Help;
            }
            "-g" | "--grammar" => {
                let Some(arg) = args.get(i + 1) else {
                    eprintln!("pegdb {}: {} requires a path", sub, a);
                    return FixtureArgs::Err(ExitCode::from(2));
                };
                grammar_path = Some(arg.as_str());
                i += 2;
                continue;
            }
            other if other.starts_with("--grammar=") => {
                grammar_path = Some(&args[i]["--grammar=".len()..]);
            }
            other if other.starts_with('-') => {
                eprintln!("pegdb {}: unknown flag {:?}", sub, other);
                return FixtureArgs::Err(ExitCode::from(2));
            }
            other => {
                if path.is_some() {
                    eprintln!("pegdb {}: unexpected extra argument {:?}", sub, other);
                    return FixtureArgs::Err(ExitCode::from(2));
                }
                path = Some(other);
            }
        }
        i += 1;
    }
    FixtureArgs::Ok(ParsedFixture { grammar_path, path })
}

/// True iff `args` contains `-h` or `--help` anywhere. Subcommand
/// handlers check this first so that `--help` is reachable even when
/// other arguments would error (e.g. a malformed `--max-literal=banana`).
fn wants_help(args: &[String]) -> bool {
    args.iter().any(|a| a == "-h" || a == "--help")
}

/// Pre-extract `--max-literal=N` (and the space-separated `--max-literal N`
/// form, symmetric with `--grammar`) from `args`, returning the parsed
/// value (if any) and a copy of the remaining args. The `dump` verbs
/// run this before handing what's left to the shared
/// `parse_fixture_args`, which keeps the flag scoped to subcommands
/// that opt into it instead of leaking into the shared parser.
fn extract_max_literal(
    args: &[String],
    sub: &str,
) -> Result<(Option<usize>, Vec<String>), ExitCode> {
    let mut max_literal: Option<usize> = None;
    let mut rest: Vec<String> = Vec::with_capacity(args.len());
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if let Some(n) = a.strip_prefix("--max-literal=") {
            max_literal = Some(parse_max_literal_value(n, sub)?);
        } else if a == "--max-literal" {
            let Some(n) = args.get(i + 1) else {
                eprintln!("pegdb {}: --max-literal requires a value", sub);
                return Err(ExitCode::from(2));
            };
            max_literal = Some(parse_max_literal_value(n, sub)?);
            i += 1; // skip the value
        } else {
            rest.push(args[i].clone());
        }
        i += 1;
    }
    Ok((max_literal, rest))
}

fn parse_max_literal_value(s: &str, sub: &str) -> Result<usize, ExitCode> {
    s.parse::<usize>().map_err(|_| {
        eprintln!(
            "pegdb {}: --max-literal expects an integer, got {:?}",
            sub, s
        );
        ExitCode::from(2)
    })
}

/// Read the grammar source from the path supplied via `-g` and load the
/// fixture input from `<path>` or stdin. Returns (grammar_source, input,
/// label-for-error-messages). The grammar path is mandatory: there is no
/// language-name shortcut, no extension inference, no default — `pegdb`
/// is a debug tool for any grammar source, not just the bundled set.
fn load_fixture(p: &ParsedFixture<'_>, sub: &str) -> Result<(String, String, String), ExitCode> {
    let Some(grammar_path) = p.grammar_path else {
        eprintln!("pegdb {}: -g <grammar.peg> is required", sub);
        return Err(ExitCode::from(2));
    };
    let grammar = match std::fs::read_to_string(grammar_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("pegdb {}: {}: {}", sub, grammar_path, e);
            return Err(ExitCode::from(2));
        }
    };
    let (input, label) = match p.path {
        Some(path) => match std::fs::read_to_string(path) {
            Ok(s) => (s, path.to_string()),
            Err(e) => {
                eprintln!("pegdb {}: {}: {}", sub, path, e);
                return Err(ExitCode::from(2));
            }
        },
        None => {
            let mut buf = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
                eprintln!("pegdb {}: reading stdin: {}", sub, e);
                return Err(ExitCode::from(2));
            }
            (buf, "<stdin>".to_string())
        }
    };
    Ok((grammar, input, label))
}

#[cfg(test)]
mod tests {
    use super::*;
    use syntax_highlighter::pegvm::{CaptureKind, LabelId};

    fn cap(start: usize, end: usize, kind: u16) -> Capture {
        Capture {
            start,
            end,
            kind: CaptureKind(kind),
        }
    }

    fn diag(capture_index: usize, pos: usize, label: u16) -> RecoveryDiagnostic {
        RecoveryDiagnostic {
            capture_index,
            pos,
            origin: RecoveryOrigin::Grammar {
                rule_stack: Vec::new(),
                label: LabelId(label),
            },
        }
    }

    #[test]
    fn detect_orphans_empty_when_well_formed() {
        // One recovery capture (idx 0, kind 1 = "recovery") with a
        // matching diagnostic; one non-recovery (idx 1, kind 0). The
        // span aggregate has a `Some` at idx 0 carrying the diagnostic.
        let recovery_kind = Some(1usize);
        let captures = vec![cap(0, 1, 1), cap(1, 2, 0)];
        let diagnostics = vec![diag(0, 5, 0)];
        let aggregates = vec![Some(diagnostics[0].clone()), None];

        let (orphan_caps, orphan_diags) =
            detect_orphans(&captures, &diagnostics, &aggregates, recovery_kind);

        assert!(orphan_caps.is_empty(), "expected no orphan captures");
        assert!(orphan_diags.is_empty(), "expected no orphan diagnostics");
    }

    #[test]
    fn detect_orphans_flags_recovery_capture_with_no_diagnostic() {
        // The shape of PR #101's bug: a recovery capture survives but
        // its diagnostic was silently dropped by an upstream filter.
        // Span aggregate slot is `None` for that capture.
        let recovery_kind = Some(1usize);
        let captures = vec![cap(0, 1, 1), cap(1, 2, 0)];
        let diagnostics: Vec<RecoveryDiagnostic> = Vec::new();
        let aggregates: Vec<Option<RecoveryDiagnostic>> = vec![None, None];

        let (orphan_caps, orphan_diags) =
            detect_orphans(&captures, &diagnostics, &aggregates, recovery_kind);

        assert_eq!(orphan_caps, vec![0]);
        assert!(orphan_diags.is_empty());
    }

    #[test]
    fn detect_orphans_flags_diagnostic_with_no_surviving_capture() {
        // The inverse drift case: a diagnostic exists but its
        // `capture_index` doesn't point at any capture that made it
        // into the span aggregates. No recovery captures here.
        let recovery_kind = Some(1usize);
        let captures = vec![cap(0, 1, 0)]; // non-recovery
        let diagnostics = vec![diag(42, 7, 0)];
        let aggregates = vec![None];

        let (orphan_caps, orphan_diags) =
            detect_orphans(&captures, &diagnostics, &aggregates, recovery_kind);

        assert!(orphan_caps.is_empty());
        assert_eq!(orphan_diags, vec![0]);
    }

    #[test]
    fn detect_orphans_handles_both_halves_simultaneously() {
        let recovery_kind = Some(1usize);
        let captures = vec![cap(0, 1, 1), cap(5, 6, 1)];
        let diagnostics = vec![diag(99, 7, 0)]; // capture_index doesn't match any cap
        let aggregates: Vec<Option<RecoveryDiagnostic>> = vec![None, None];

        let (orphan_caps, orphan_diags) =
            detect_orphans(&captures, &diagnostics, &aggregates, recovery_kind);

        assert_eq!(orphan_caps, vec![0, 1]);
        assert_eq!(orphan_diags, vec![0]);
    }

    #[test]
    fn detect_orphans_returns_empty_when_no_recovery_kind() {
        // Grammar without any `recovery` capture kind: there can be
        // neither recovery captures nor diagnostics. Both lists empty.
        let recovery_kind = None;
        let captures = vec![cap(0, 1, 0), cap(1, 2, 0)];
        let diagnostics: Vec<RecoveryDiagnostic> = Vec::new();
        let aggregates: Vec<Option<RecoveryDiagnostic>> = vec![None, None];

        let (orphan_caps, orphan_diags) =
            detect_orphans(&captures, &diagnostics, &aggregates, recovery_kind);

        assert!(orphan_caps.is_empty());
        assert!(orphan_diags.is_empty());
    }
}
