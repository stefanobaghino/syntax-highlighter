//! Integration tests for the `pegc` binary.
//!
//! Drives the compiled binary via `std::process::Command` and asserts on
//! its exit codes plus the structural shape of its JSONL output. JSON
//! parsing is hand-rolled and minimal (no external dep) per the
//! project's zero-dep design goal — only the structural invariants are
//! checked (object brackets, expected keys, integer-vs-string fields),
//! not exact literal contents (those would shift with grammar edits).

use std::path::Path;
use std::process::Command;

#[path = "common/mod.rs"]
mod common;

use common::json_field_str;

const PEGC: &str = env!("CARGO_BIN_EXE_pegc");

fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(PEGC)
        .args(args)
        .output()
        .expect("spawning pegc");
    let code = out.status.code().expect("exit code");
    (
        code,
        String::from_utf8(out.stdout).expect("utf-8 stdout"),
        String::from_utf8(out.stderr).expect("utf-8 stderr"),
    )
}

#[test]
fn stats_json_grammar_emits_expected_record() {
    let json_path = "grammars/json.peg";
    assert!(
        Path::new(json_path).exists(),
        "fixture missing: {json_path}"
    );
    let (code, stdout, stderr) = run(&["stats", json_path]);
    assert_eq!(code, 0, "stderr was: {stderr}");
    let mut lines = stdout.lines();
    let line = lines.next().expect("at least one line");
    assert!(
        line.starts_with('{') && line.ends_with('}'),
        "stats output not a JSON object: {line:?}"
    );
    assert_eq!(
        json_field_str(line, "path"),
        Some(format!("\"{json_path}\"").as_str())
    );
    let instructions: usize = json_field_str(line, "instructions")
        .expect("instructions present")
        .parse()
        .expect("instructions parses as integer");
    assert!(instructions > 0, "expected non-zero instructions");
    let rules: usize = json_field_str(line, "rules")
        .expect("rules present")
        .parse()
        .expect("rules parses as integer");
    assert!(rules > 0, "expected non-zero rules");
    let count: usize = json_field_str(line, "capture_kinds_count")
        .expect("capture_kinds_count present")
        .parse()
        .expect("capture_kinds_count int");
    let kinds_array = json_field_str(line, "capture_kinds").expect("capture_kinds present");
    assert!(
        kinds_array.starts_with('[') && kinds_array.ends_with(']'),
        "capture_kinds not a JSON array: {kinds_array:?}"
    );
    let inner = &kinds_array[1..kinds_array.len() - 1];
    let parts: Vec<&str> = if inner.is_empty() {
        Vec::new()
    } else {
        inner.split(',').collect()
    };
    assert_eq!(count, parts.len(), "kind count mismatch: {parts:?}");
    for p in &parts {
        assert!(
            p.starts_with('"') && p.ends_with('"'),
            "kind name not JSON-string-quoted: {p:?}"
        );
    }
    // One NDJSON per-rule record after the header; the count matches
    // the header's `rules` field, lines are sorted alphabetically, and
    // each carries the documented `rule` / `references` / `body_chars`
    // fields.
    let per_rule: Vec<&str> = lines.collect();
    assert_eq!(
        per_rule.len(),
        rules,
        "per-rule record count mismatch: {per_rule:?}"
    );
    let mut prev: Option<&str> = None;
    for r in &per_rule {
        assert!(
            r.starts_with('{') && r.ends_with('}'),
            "per-rule line not a JSON object: {r:?}"
        );
        let name = json_field_str(r, "rule").expect("rule present");
        assert!(
            name.starts_with('"') && name.ends_with('"'),
            "rule name not JSON-string-quoted: {r:?}"
        );
        json_field_str(r, "references")
            .expect("references present")
            .parse::<usize>()
            .expect("references parses as integer");
        json_field_str(r, "body_chars")
            .expect("body_chars present")
            .parse::<usize>()
            .expect("body_chars parses as integer");
        if let Some(p) = prev {
            assert!(p < name, "per-rule lines not sorted: {p:?} then {name:?}");
        }
        prev = Some(name);
    }
}

// `tests/fixtures/stats_canary.peg` is a deliberately tiny grammar
// (`r <- 'a' / @keyword{'b'}`) whose only purpose is pinning `stats`'s
// scalar output. The numbers shift only if pegc's bytecode lowering
// for literals / alternation / capture wrapping changes — a real
// signal worth catching. Shipped-grammar drift belongs elsewhere.
#[test]
fn stats_emits_correct_counts_for_canary_grammar() {
    let path = "tests/fixtures/stats_canary.peg";
    assert!(Path::new(path).exists(), "fixture missing: {path}");
    let (code, stdout, stderr) = run(&["stats", path]);
    assert_eq!(code, 0, "stderr was: {stderr}");
    let mut lines = stdout.lines();
    let header = lines.next().expect("at least one line");
    assert_eq!(json_field_str(header, "instructions"), Some("11"));
    assert_eq!(json_field_str(header, "rules"), Some("1"));
    assert_eq!(json_field_str(header, "capture_kinds_count"), Some("1"));
    assert_eq!(
        json_field_str(header, "capture_kinds"),
        Some("[\"keyword\"]")
    );
    // The lone rule has no NonTerminal references; body_chars pins the
    // trimmed source length of `'a' / @keyword{'b'}` (19 characters).
    let rule_line = lines.next().expect("per-rule record present");
    assert_eq!(json_field_str(rule_line, "rule"), Some("\"r\""));
    assert_eq!(json_field_str(rule_line, "references"), Some("0"));
    assert_eq!(json_field_str(rule_line, "body_chars"), Some("19"));
    assert!(lines.next().is_none(), "canary should emit one rule record");
}

// Multi-rule fixture pinning per-rule reference counting: `a` is
// referenced three times (twice from `start`, once from `b`), `b` once
// (from `start`), and `start` zero times.
#[test]
fn stats_per_rule_records_count_references() {
    let path = "tests/fixtures/stats_refs_canary.peg";
    assert!(Path::new(path).exists(), "fixture missing: {path}");
    let (code, stdout, stderr) = run(&["stats", path]);
    assert_eq!(code, 0, "stderr was: {stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 4, "expected 1 header + 3 per-rule: {lines:?}");
    let header = lines[0];
    assert_eq!(json_field_str(header, "rules"), Some("3"));
    let mut by_rule: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
    for line in &lines[1..] {
        let rule = json_field_str(line, "rule").expect("rule present");
        by_rule.insert(rule, line);
    }
    let a = by_rule["\"a\""];
    assert_eq!(json_field_str(a, "references"), Some("3"));
    let b = by_rule["\"b\""];
    assert_eq!(json_field_str(b, "references"), Some("1"));
    let start = by_rule["\"start\""];
    assert_eq!(json_field_str(start, "references"), Some("0"));
}

#[test]
fn stats_missing_path_fails_with_usage_error() {
    let (code, _stdout, stderr) = run(&["stats"]);
    assert_eq!(code, 2);
    assert!(stderr.contains("missing"), "stderr: {stderr}");
}

#[test]
fn stats_grammar_compile_error_exits_3() {
    // Use a fixture that isn't valid PEG source.
    let fixture = "benches/fixtures/medium.json";
    let (code, _stdout, _stderr) = run(&["stats", fixture]);
    assert_eq!(code, 3);
}

#[test]
fn top_level_help_is_non_empty() {
    for flag in ["--help", "-h"] {
        let (code, stdout, _) = run(&[flag]);
        assert_eq!(code, 0, "{flag} should exit 0");
        assert!(stdout.contains("pegc"), "{flag} output: {stdout}");
        assert!(stdout.contains("stats"), "missing stats in help");
    }
}

#[test]
fn no_args_prints_top_level_help() {
    let (code, stdout, _) = run(&[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("pegc"));
    assert!(stdout.contains("Subcommands"));
}

#[test]
fn stats_help_short_circuits() {
    let (code, stdout, _) = run(&["stats", "--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("stats"));
    assert!(stdout.contains("TOOLS.md"));
}

#[test]
fn unknown_subcommand_is_usage_error() {
    let (code, _stdout, stderr) = run(&["bogus"]);
    assert_eq!(code, 2);
    assert!(stderr.contains("unknown subcommand"), "stderr: {stderr}");
}

#[test]
fn follow_set_emits_ndjson_per_rule() {
    let path = "grammars/json.peg";
    assert!(Path::new(path).exists(), "fixture missing: {path}");
    let (code, stdout, stderr) = run(&["follow-set", path]);
    assert_eq!(code, 0, "stderr was: {stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(!lines.is_empty(), "follow-set produced no output");
    for line in &lines {
        assert!(
            line.starts_with('{') && line.ends_with('}'),
            "follow-set line not a JSON object: {line:?}"
        );
        assert!(
            json_field_str(line, "rule").is_some(),
            "missing rule field: {line:?}"
        );
        let follow = json_field_str(line, "follow").expect("follow field");
        assert!(
            follow.starts_with('[') && follow.ends_with(']'),
            "follow not a JSON array: {follow:?}"
        );
    }
}

#[test]
fn follow_set_single_rule_filter_emits_one_line() {
    let path = "grammars/sqlite.peg";
    assert!(Path::new(path).exists(), "fixture missing: {path}");
    let (code, stdout, stderr) = run(&["follow-set", path, "result_column"]);
    assert_eq!(code, 0, "stderr was: {stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "expected exactly one record, got: {lines:?}"
    );
    let line = lines[0];
    assert_eq!(
        json_field_str(line, "rule"),
        Some("\"result_column\""),
        "wrong rule: {line:?}"
    );
}

#[test]
fn follow_set_unknown_rule_exits_two() {
    let (code, _stdout, stderr) = run(&["follow-set", "grammars/sqlite.peg", "no_such_rule"]);
    assert_eq!(code, 2);
    assert!(stderr.contains("not found"), "stderr: {stderr}");
}

#[test]
fn follow_set_bad_grammar_exits_three() {
    // medium.json is not valid PEG source.
    let fixture = "benches/fixtures/medium.json";
    let (code, _stdout, _stderr) = run(&["follow-set", fixture]);
    assert_eq!(code, 3);
}
