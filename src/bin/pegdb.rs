//! `pegdb` — grammar debug tool.
//!
//! Subcommands: `dump-captures`. Emits JSONL (one JSON object per
//! `\n`-delimited line). See `TOOLS.md` at the repo root for the full
//! contract. Compile-time grammar inspection (`stats`, etc.) lives in
//! `pegc`.

use std::borrow::Cow;
use std::fmt::Write as _;
use std::io::{Read, Write};
use std::process::ExitCode;

use syntax_highlighter::parser::Parser;
use syntax_highlighter::pegvm::{Capture, RecoveryDiagnostic};

const TOP_HELP: &str = "\
pegdb — grammar-developer debug surface for syntax-highlighter

Usage:
    pegdb <subcommand> [options] [args]

Subcommands:
    dump-captures -g <grammar.peg> [--max-literal=N] [<path>]
                                                 Print captures as JSONL (one object per capture).
    explain-recoveries -g <grammar.peg> [<path>]
                                                 Cluster `*^` recoveries by rule-stack suffix.

Options:
    -h, --help                                   Show this help.

For compile-time grammar inspection (bytecode shape), see `pegc`.
See TOOLS.md for the full contract: subcommand semantics, JSONL schemas,
exit codes, partial-match diagnostics.
";

const DUMP_HELP: &str = "\
Usage: pegdb dump-captures -g <grammar.peg> [--max-literal=N] [<path>]

Print one capture per line as a JSON object (keys: start, end, kind,
literal, plus farthest_reach on recovery rows). Exits 1 with a stderr
partial-match marker on incomplete parses. The grammar source is
required.

See TOOLS.md for the full contract.
";

const EXPLAIN_HELP: &str = "\
Usage: pegdb explain-recoveries -g <grammar.peg> [<path>]

Cluster `*^` recoveries by rule-stack suffix and print one line per
cluster, sorted by count descending. Each cluster reports the number
of recovery captures and the deepest rule reached during the failed
iterations that produced them. The grammar source is required.

See TOOLS.md for the full contract.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        print!("{}", TOP_HELP);
        return ExitCode::SUCCESS;
    }
    let (sub, rest) = args.split_first().unwrap();
    match sub.as_str() {
        "dump-captures" => run_dump_captures(rest),
        "explain-recoveries" => run_explain_recoveries(rest),
        "-h" | "--help" => {
            print!("{}", TOP_HELP);
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("pegdb: unknown subcommand {:?}", other);
            eprintln!();
            eprint!("{}", TOP_HELP);
            ExitCode::from(2)
        }
    }
}

fn run_dump_captures(args: &[String]) -> ExitCode {
    if wants_help(args) {
        print!("{}", DUMP_HELP);
        return ExitCode::SUCCESS;
    }
    let (max_literal, rest) = match extract_max_literal(args) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let parsed = match parse_fixture_args(&rest, "dump-captures", DUMP_HELP) {
        FixtureArgs::Help => return ExitCode::SUCCESS,
        FixtureArgs::Err(code) => return code,
        FixtureArgs::Ok(p) => p,
    };
    let (grammar, input, source_label) = match load_fixture(&parsed, "dump-captures") {
        Ok(t) => t,
        Err(code) => return code,
    };
    let mut p = match Parser::new(&grammar) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("pegdb dump-captures: grammar error: {}", e);
            return ExitCode::from(3);
        }
    };
    p = p.with_track_recovery_diagnostics(true);
    p.set_input(input.into_bytes());
    let kinds = p.capture_kinds();
    let rule_names = p.rule_names();
    let complete = p.is_complete();
    let (matched, captures) = p.captures();
    let diagnostics = p.recovery_diagnostics();
    let view_bytes = p.input();
    // Input arrived through `read_to_string` (or a `String` literal in
    // tests), so `Parser` only ever held valid UTF-8 — no failure path.
    let view = std::str::from_utf8(view_bytes)
        .expect("Parser input originated as String; bytes must round-trip as UTF-8");

    // Pre-pass for the `farthest_reach` field. A recovery span is a
    // maximal contiguous run of `kind == "recovery"` captures where
    // `cap[i].end == cap[i+1].start` (the `*^` loop emits one
    // single-byte recovery capture per failed iteration). For each
    // span, take the diagnostic with the largest `pos` — that's the
    // worst dive that contributed to this run, and the rule stack
    // there is the most actionable signal for "what broke?". Every
    // row in the span carries the same span-level value, so consumers
    // can either dedup with `jq` or pull the canonical record from
    // `pegdb explain-recoveries`.
    let recovery_kind_idx = kinds.iter().position(|k| k == "recovery");
    let span_aggregates =
        compute_recovery_span_aggregates(captures, diagnostics, recovery_kind_idx);

    let mut out = std::io::stdout().lock();
    // Captures arrive in CaptureBegin order — start-ascending, with a
    // parent always appearing before its nested children (the parent's
    // Begin fires first). A linear stack-walk maps each capture to its
    // nesting depth: pop entries whose end has already passed our start
    // (siblings/ancestors that closed), then `depth = stack.len()`.
    let mut open_ends: Vec<usize> = Vec::new();
    for (idx, cap) in captures.iter().enumerate() {
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
        // `farthest_reach` appears only on recovery rows. Encoded
        // unconditionally when the row's span has a diagnostic — the
        // existing `recovery_baseline_tests.rs` consumer filters on
        // `kind` and ignores additive fields, so the JSONL contract is
        // backwards-compatible.
        let farthest_reach = span_aggregates.get(idx).and_then(|opt| opt.as_ref());
        match farthest_reach {
            Some(reach) => {
                let _ = writeln!(
                    out,
                    "{{\"start\":{},\"end\":{},\"kind\":{},\"depth\":{},\"literal\":{},\"farthest_reach\":{}}}",
                    cap.start,
                    cap.end,
                    json_string(kind),
                    depth,
                    json_string(&truncated),
                    format_farthest_reach(reach, rule_names),
                );
            }
            None => {
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
        }
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

fn run_explain_recoveries(args: &[String]) -> ExitCode {
    if wants_help(args) {
        print!("{}", EXPLAIN_HELP);
        return ExitCode::SUCCESS;
    }
    let parsed = match parse_fixture_args(args, "explain-recoveries", EXPLAIN_HELP) {
        FixtureArgs::Help => return ExitCode::SUCCESS,
        FixtureArgs::Err(code) => return code,
        FixtureArgs::Ok(p) => p,
    };
    let (grammar, input, source_label) = match load_fixture(&parsed, "explain-recoveries") {
        Ok(t) => t,
        Err(code) => return code,
    };
    let mut p = match Parser::new(&grammar) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("pegdb explain-recoveries: grammar error: {}", e);
            return ExitCode::from(3);
        }
    };
    p = p.with_track_recovery_diagnostics(true);
    p.set_input(input.into_bytes());
    let kinds = p.capture_kinds();
    let rule_names = p.rule_names();
    let complete = p.is_complete();
    let (matched, captures) = p.captures();
    let diagnostics = p.recovery_diagnostics();

    let recovery_kind_idx = kinds.iter().position(|k| k == "recovery");
    let span_aggregates =
        compute_recovery_span_aggregates(captures, diagnostics, recovery_kind_idx);
    let label_names = p.label_kinds();

    // Cluster by `(rule_stack, label_name)`. Every recovery
    // firing carries a label — labeled catches (`^label`) use the
    // author's identifier; `*^` uses the intern of its
    // `recovery_kind` string. v1 uses the entire stack as the key;
    // suffix-clustering is a follow-up. BTreeMap preserves a stable
    // key order for deterministic output.
    type ClusterKey = (Vec<String>, String);
    let mut clusters: std::collections::BTreeMap<ClusterKey, usize> =
        std::collections::BTreeMap::new();
    for slot in &span_aggregates {
        let Some(reach) = slot else {
            continue;
        };
        let stack: Vec<String> = reach
            .rule_stack
            .iter()
            .filter_map(|id| rule_names.get(id.0 as usize).cloned())
            .collect();
        let label = label_names
            .get(reach.label.0 as usize)
            .cloned()
            .unwrap_or_default();
        *clusters.entry((stack, label)).or_insert(0) += 1;
    }

    let mut entries: Vec<(ClusterKey, usize)> = clusters.into_iter().collect();
    // Sort by count descending, then by key lexicographically for
    // determinism on ties.
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let mut out = std::io::stdout().lock();
    for ((stack, label), count) in &entries {
        let leaf = stack.last().map(String::as_str).unwrap_or("<empty>");
        let _ = writeln!(
            out,
            "{} recoveries — farthest reach ends at {} (label: {})",
            count, leaf, label
        );
    }
    if entries.is_empty() {
        let _ = writeln!(out, "no recoveries");
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

/// Format a [`RecoveryDiagnostic`] as the inner JSON object of the
/// `farthest_reach` field — `{"pos":N,"rule_stack":["name",...]}`.
/// `rule_names` resolves [`crate::pegvm::MemoId`] indices back to
/// human-readable rule names; ids outside the table are skipped (a
/// hand-built [`crate::pegvm::Program`] could in principle leave
/// `rule_names` shorter than the highest live `MemoId`).
fn format_farthest_reach(reach: &RecoveryDiagnostic, rule_names: &[String]) -> String {
    let mut s = String::with_capacity(48 + reach.rule_stack.len() * 16);
    s.push_str("{\"pos\":");
    use std::fmt::Write as _;
    let _ = write!(s, "{}", reach.pos);
    s.push_str(",\"rule_stack\":[");
    let mut first = true;
    for id in &reach.rule_stack {
        let Some(name) = rule_names.get(id.0 as usize) else {
            continue;
        };
        if !first {
            s.push(',');
        }
        first = false;
        s.push_str(&json_string(name));
    }
    s.push_str("]}");
    s
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

/// Parsed `-g <grammar.peg> [<path>]` shape. Subcommand-specific
/// flags are extracted by the handler before this shared parser runs
/// (see `run_dump_captures`'s pre-pass for `--max-literal=N`), which
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
/// value (if any) and a copy of the remaining args. `dump-captures`
/// runs this before handing what's left to the shared
/// `parse_fixture_args`, which keeps the flag scoped to subcommands
/// that opt into it instead of leaking into the shared parser.
fn extract_max_literal(args: &[String]) -> Result<(Option<usize>, Vec<String>), ExitCode> {
    let mut max_literal: Option<usize> = None;
    let mut rest: Vec<String> = Vec::with_capacity(args.len());
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if let Some(n) = a.strip_prefix("--max-literal=") {
            max_literal = Some(parse_max_literal_value(n)?);
        } else if a == "--max-literal" {
            let Some(n) = args.get(i + 1) else {
                eprintln!("pegdb dump-captures: --max-literal requires a value");
                return Err(ExitCode::from(2));
            };
            max_literal = Some(parse_max_literal_value(n)?);
            i += 1; // skip the value
        } else {
            rest.push(args[i].clone());
        }
        i += 1;
    }
    Ok((max_literal, rest))
}

fn parse_max_literal_value(s: &str) -> Result<usize, ExitCode> {
    s.parse::<usize>().map_err(|_| {
        eprintln!(
            "pegdb dump-captures: --max-literal expects an integer, got {:?}",
            s
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
