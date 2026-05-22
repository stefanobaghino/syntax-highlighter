//! Integration tests for the `pegdb` binary.
//!
//! Drives the compiled binary via `std::process::Command` and asserts on
//! its exit codes plus the structural shape of its JSONL output. JSON
//! parsing is hand-rolled and minimal (no external dep) per the
//! project's zero-dep design goal — only the structural invariants are
//! checked (object brackets, expected keys, integer-vs-string fields),
//! not exact literal contents (those would shift with grammar edits).

use std::process::{Command, Stdio};

#[path = "common/mod.rs"]
mod common;

use common::json_field_str;

const PEGDB: &str = env!("CARGO_BIN_EXE_pegdb");

fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(PEGDB)
        .args(args)
        .output()
        .expect("spawning pegdb");
    let code = out.status.code().expect("exit code");
    (
        code,
        String::from_utf8(out.stdout).expect("utf-8 stdout"),
        String::from_utf8(out.stderr).expect("utf-8 stderr"),
    )
}

fn run_stdin(args: &[&str], stdin: &[u8]) -> (i32, String, String) {
    use std::io::Write;
    let mut child = Command::new(PEGDB)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawning pegdb");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin)
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait");
    (
        out.status.code().expect("exit code"),
        String::from_utf8(out.stdout).expect("utf-8 stdout"),
        String::from_utf8(out.stderr).expect("utf-8 stderr"),
    )
}

// ---------- dump-captures ----------

#[test]
fn dump_captures_emits_well_shaped_jsonl() {
    let (code, stdout, stderr) = run(&[
        "dump-captures",
        "-g",
        "grammars/json.peg",
        "benches/fixtures/small.json",
    ]);
    assert_eq!(code, 0, "stderr was: {stderr}");
    let mut prev_start: usize = 0;
    let mut row_count = 0;
    for line in stdout.lines() {
        assert!(
            line.starts_with('{') && line.ends_with('}'),
            "row not a JSON object: {line:?}"
        );
        let start: usize = json_field_str(line, "start")
            .expect("start present")
            .parse()
            .expect("start parses as int");
        let end: usize = json_field_str(line, "end")
            .expect("end present")
            .parse()
            .expect("end parses as int");
        assert!(start <= end, "start..end inverted: {line:?}");
        // start values come out non-decreasing; equal is fine for adjacent
        // captures with the same anchor — the parser doesn't promise strict.
        assert!(
            start >= prev_start,
            "start went backwards from {prev_start} to {start}"
        );
        prev_start = start;
        let kind = json_field_str(line, "kind").expect("kind present");
        assert!(
            kind.starts_with('"') && kind.ends_with('"'),
            "kind not JSON-string: {kind:?}"
        );
        let literal = json_field_str(line, "literal").expect("literal present");
        assert!(
            literal.starts_with('"') && literal.ends_with('"'),
            "literal not JSON-string: {literal:?}"
        );
        row_count += 1;
    }
    assert!(row_count > 0, "no capture rows emitted");
}

#[test]
fn dump_captures_partial_parse_emits_marker_and_exits_one() {
    // JSON missing its closing brace: VM matches a prefix but doesn't reach End.
    let (code, stdout, stderr) =
        run_stdin(&["dump-captures", "-g", "grammars/json.peg"], br#"{"a": 1"#);
    assert_eq!(code, 1, "stderr: {stderr}");
    // Stdout is a clean JSONL stream: at least one capture line.
    let first = stdout.lines().next().expect("at least one capture line");
    assert!(
        first.starts_with('{') && first.ends_with('}'),
        "first stdout line not a JSON object: {first:?}"
    );
    // Marker on stderr: `partial-match <label>: matched M of L bytes`.
    assert!(
        stderr.contains("partial-match"),
        "missing partial-match marker in stderr: {stderr}"
    );
    assert!(
        stderr.contains("<stdin>"),
        "partial-match marker missing source label: {stderr}"
    );
}

#[test]
fn dump_captures_depth_zero_for_flat_grammar() {
    // JSON's grammar produces no nested captures: every `@kind{rule}`
    // wraps a rule whose body has no further `@kind{...}` annotations.
    let (code, stdout, _) = run(&[
        "dump-captures",
        "-g",
        "grammars/json.peg",
        "benches/fixtures/small.json",
    ]);
    assert_eq!(code, 0);
    let mut row_count = 0;
    for line in stdout.lines() {
        let depth = json_field_str(line, "depth").expect("depth field present");
        assert_eq!(depth, "0", "JSON capture depth should be 0: {line}");
        row_count += 1;
    }
    assert!(row_count > 0, "no rows emitted");
}

#[test]
fn dump_captures_depth_surfaces_go_qualified_ident_nesting() {
    // Go's `type_name <- @type{qualified_ident}` wraps `qualified_ident
    // <- ident (@punctuation{'.'} ident)?` — so `pkg.Foo` produces an
    // outer `type` capture (depth=0) containing an inner `punctuation`
    // capture for the `.` (depth=1).
    let (code, stdout, stderr) = run_stdin(
        &["dump-captures", "-g", "grammars/go.peg"],
        b"package p\nvar x pkg.Foo\n",
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    let mut saw_depth_zero = false;
    let mut saw_depth_one = false;
    for line in stdout.lines() {
        match json_field_str(line, "depth").expect("depth present") {
            "0" => saw_depth_zero = true,
            "1" => saw_depth_one = true,
            other => panic!("unexpected depth {other:?} in: {line}"),
        }
    }
    assert!(saw_depth_zero, "expected a depth=0 row, got:\n{stdout}");
    assert!(
        saw_depth_one,
        "expected a depth=1 row from `pkg.Foo`'s inner punctuation capture, got:\n{stdout}"
    );
}

#[test]
fn dump_captures_max_literal_truncates() {
    let (code, stdout, _stderr) = run(&[
        "dump-captures",
        "-g",
        "grammars/json.peg",
        "--max-literal=3",
        "benches/fixtures/small.json",
    ]);
    assert_eq!(code, 0);
    let truncation_mark = "…";
    assert!(
        stdout.contains(truncation_mark),
        "expected ellipsis in truncated output, got:\n{stdout}"
    );
}

#[test]
fn dump_captures_rejects_max_literal_with_non_integer_value() {
    let (code, _, stderr) = run(&[
        "dump-captures",
        "-g",
        "grammars/json.peg",
        "--max-literal=not-a-num",
        "benches/fixtures/small.json",
    ]);
    assert_eq!(code, 2);
    assert!(stderr.contains("--max-literal"), "stderr: {stderr}");
}

#[test]
fn dump_captures_accepts_space_separated_max_literal() {
    // Symmetric with --grammar/-g, which already accepts both --grammar=X
    // and --grammar X.
    let (code, stdout, stderr) = run(&[
        "dump-captures",
        "-g",
        "grammars/json.peg",
        "--max-literal",
        "3",
        "benches/fixtures/small.json",
    ]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("…"),
        "expected ellipsis in truncated output, got:\n{stdout}"
    );
}

#[test]
fn dump_captures_max_literal_without_value_errors() {
    let (code, _, stderr) = run(&["dump-captures", "--max-literal"]);
    assert_eq!(code, 2);
    assert!(stderr.contains("requires a value"), "stderr: {stderr}");
}

#[test]
fn dump_captures_grammar_compile_error_exits_3() {
    // Use a file that isn't valid PEG source.
    let (code, _, stderr) = run(&[
        "dump-captures",
        "-g",
        "benches/fixtures/medium.json",
        "benches/fixtures/small.json",
    ]);
    assert_eq!(code, 3);
    assert!(stderr.contains("grammar"), "stderr: {stderr}");
}

#[test]
fn dump_captures_help_short_circuits_max_literal_parse_error() {
    // --help should always reach the help screen, even when an earlier
    // argument would have errored on its own.
    let (code, stdout, _) = run(&["dump-captures", "--max-literal=banana", "--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("dump-captures") && stdout.contains("TOOLS.md"),
        "expected dump-captures help, got: {stdout}"
    );
}

// ---------- help strings ----------

#[test]
fn top_level_help_is_non_empty() {
    for flag in ["--help", "-h"] {
        let (code, stdout, _) = run(&[flag]);
        assert_eq!(code, 0, "{flag} should exit 0");
        assert!(stdout.contains("pegdb"), "{flag} output: {stdout}");
        assert!(
            stdout.contains("dump-captures"),
            "missing dump-captures in help"
        );
    }
}

#[test]
fn no_args_prints_top_level_help() {
    let (code, stdout, _) = run(&[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("pegdb"));
    assert!(stdout.contains("Subcommands"));
}

#[test]
fn dump_captures_help_points_at_tools_md() {
    let (code, stdout, _) = run(&["dump-captures", "--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("dump-captures"));
    assert!(
        stdout.contains("TOOLS.md"),
        "dump-captures help should point at TOOLS.md for the full contract"
    );
}

#[test]
fn unknown_subcommand_is_usage_error() {
    let (code, _stdout, stderr) = run(&["bogus"]);
    assert_eq!(code, 2);
    assert!(stderr.contains("unknown subcommand"), "stderr: {stderr}");
}

// ---------- farthest_reach on dump-captures ----------

#[test]
fn dump_captures_emits_farthest_reach_on_recovery_rows() {
    // `@@@` is unparseable at item position; the rust grammar's top
    // `*^` byte-eater emits a `recovery` row per skipped byte. Each
    // recovery row must carry a `farthest_reach` object with a `pos`
    // integer and a non-empty `rule_stack` array.
    let (code, stdout, _) = run_stdin(
        &["dump-captures", "-g", "grammars/rust.peg"],
        b"fn ok() {}\n@@@\nfn ok2() {}\n",
    );
    assert_eq!(code, 0);
    let mut saw_recovery_with_reach = false;
    let mut saw_non_recovery_without_reach = false;
    for line in stdout.lines() {
        let kind = json_field_str(line, "kind").expect("kind present");
        if kind == "\"recovery\"" {
            let reach = json_field_str(line, "farthest_reach")
                .unwrap_or_else(|| panic!("recovery row missing farthest_reach: {line}"));
            assert!(
                reach.starts_with('{') && reach.ends_with('}'),
                "farthest_reach not a JSON object: {reach}"
            );
            assert!(
                json_field_str(reach, "pos").is_some(),
                "farthest_reach missing pos field: {reach}"
            );
            let rule_stack =
                json_field_str(reach, "rule_stack").expect("farthest_reach has rule_stack");
            assert!(
                rule_stack.starts_with('[') && rule_stack.ends_with(']'),
                "rule_stack not an array: {rule_stack}"
            );
            assert!(
                rule_stack.contains("\"rust_file\""),
                "rule_stack should at least mention the top rule (rust_file): {rule_stack}"
            );
            saw_recovery_with_reach = true;
        } else if json_field_str(line, "farthest_reach").is_none() {
            saw_non_recovery_without_reach = true;
        }
    }
    assert!(
        saw_recovery_with_reach,
        "expected at least one recovery row with farthest_reach, got:\n{stdout}"
    );
    assert!(
        saw_non_recovery_without_reach,
        "expected non-recovery rows without farthest_reach, got:\n{stdout}"
    );
}

#[test]
fn dump_captures_recovery_span_shares_farthest_reach() {
    // Adjacent recovery captures (no successful match between them)
    // form one logical span and must carry the same `farthest_reach`.
    let (code, stdout, _) = run_stdin(
        &["dump-captures", "-g", "grammars/rust.peg"],
        b"fn ok() {}\n@@@\nfn ok2() {}\n",
    );
    assert_eq!(code, 0);
    let recovery_rows: Vec<&str> = stdout
        .lines()
        .filter(|line| json_field_str(line, "kind") == Some("\"recovery\""))
        .collect();
    assert!(recovery_rows.len() >= 2, "expected multiple recovery rows");
    // Collect (start, end, farthest_reach) for adjacent grouping.
    let parsed: Vec<(usize, usize, String)> = recovery_rows
        .iter()
        .map(|line| {
            let start: usize = json_field_str(line, "start").unwrap().parse().unwrap();
            let end: usize = json_field_str(line, "end").unwrap().parse().unwrap();
            let reach = json_field_str(line, "farthest_reach")
                .expect("recovery row carries farthest_reach")
                .to_string();
            (start, end, reach)
        })
        .collect();
    // Within a contiguous run (end[i] == start[i+1]), all `farthest_reach`
    // values must be identical.
    for w in parsed.windows(2) {
        if w[0].1 == w[1].0 {
            assert_eq!(
                w[0].2, w[1].2,
                "adjacent recovery rows in the same span must share farthest_reach"
            );
        }
    }
}

// ---------- explain-recoveries ----------

#[test]
fn explain_recoveries_skips_trivia_when_picking_leaf() {
    // Shipped grammars define a `trivia <- ws` reserved-name root; the
    // compiler cascades the trivia bit through `ws → comment →
    // line_comment / block_comment`, and pegdb explain-recoveries pops
    // those trailing frames before clustering. The displayed leaf is
    // the deepest semantic (non-trivia) rule.
    let (code, stdout, _) = run_stdin(
        &["explain-recoveries", "-g", "grammars/rust.peg"],
        b"fn ok() {}\n@@@ garbage @@@\nfn ok2() {}\n",
    );
    assert_eq!(code, 0);
    let trivia_names = ["ws", "spacing", "comment", "line_comment", "block_comment"];
    for line in stdout.lines() {
        let leaf = line
            .split(" — farthest reach ends at ")
            .nth(1)
            .and_then(|rest| rest.split(" (label:").next())
            .map(str::trim)
            .unwrap_or_else(|| panic!("could not parse leaf from: {line}"));
        assert!(
            !trivia_names.contains(&leaf),
            "displayed leaf must not be a trivia rule, got `{leaf}` in: {line}"
        );
    }
}

#[test]
fn explain_recoveries_after_block_close_migration_rust() {
    // Rust's `block` rule uses the new `^^block_close ..= '}'` sugar.
    // Malformed block body should still surface a `block_close` cluster
    // through pegdb explain-recoveries, with the recovery span covering
    // the garbage and the closing `}` consumed as `@punctuation`.
    let (code, stdout, _) = run_stdin(
        &["explain-recoveries", "-g", "grammars/rust.peg"],
        b"fn ok() { @@@ garbage @@@ }\n",
    );
    assert_eq!(code, 0);
    assert!(
        stdout
            .lines()
            .any(|line| line.contains("label: block_close")),
        "expected a cluster labeled block_close; got:\n{stdout}"
    );
}

#[test]
fn explain_recoveries_clusters_by_rule_stack() {
    // Multiple `@@@` runs should collapse into a single top-level
    // cluster; non-trivial dive sites (e.g. `g` looking like the start
    // of a comment) sit in their own cluster.
    let (code, stdout, _) = run_stdin(
        &["explain-recoveries", "-g", "grammars/rust.peg"],
        b"fn ok() {}\n@@@ garbage @@@\nfn ok2() {}\n",
    );
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(!lines.is_empty(), "expected at least one cluster line");
    // Output shape: `<count> recoveries — farthest reach ends at <rule>`.
    for line in &lines {
        assert!(
            line.contains("recoveries") && line.contains("farthest reach ends at"),
            "unexpected cluster line shape: {line}"
        );
    }
    // Counts are sorted descending; parse the first count and verify
    // it's the largest.
    let counts: Vec<usize> = lines
        .iter()
        .map(|l| {
            l.split_whitespace()
                .next()
                .and_then(|tok| tok.parse().ok())
                .unwrap_or_else(|| panic!("expected leading count in: {l}"))
        })
        .collect();
    for w in counts.windows(2) {
        assert!(
            w[0] >= w[1],
            "cluster output not sorted descending: {counts:?}"
        );
    }
}

#[test]
fn explain_recoveries_emits_label_suffix_on_every_cluster() {
    // After #91, every recovery firing carries a label — `*^` uses
    // its `recovery_kind` string. The cluster output now includes a
    // `(label: …)` suffix on every line, where before unlabeled
    // recoveries had no suffix.
    let (code, stdout, _) = run_stdin(
        &["explain-recoveries", "-g", "grammars/rust.peg"],
        b"fn ok() {}\n@@@ garbage @@@\nfn ok2() {}\n",
    );
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(!lines.is_empty(), "expected at least one cluster line");
    for line in &lines {
        assert!(
            line.contains("recoveries")
                && line.contains("farthest reach ends at")
                && line.contains("(label:"),
            "unexpected cluster line (missing label suffix): {line}"
        );
    }
}

#[test]
fn explain_recoveries_uses_author_supplied_recovery_label() {
    // `*^:bad_doc` lowers the catch's label slot to the author's name.
    // The cluster suffix should pick that up — no special-case for the
    // hardcoded `"recovery"` default.
    use std::io::Write as _;
    let mut path = std::env::temp_dir();
    path.push(format!(
        "pegdb_explain_recoveries_label_{}.peg",
        std::process::id()
    ));
    let grammar = b"doc <- 'x'*^:bad_doc\n";
    let mut f = std::fs::File::create(&path).expect("create temp grammar");
    f.write_all(grammar).expect("write temp grammar");
    drop(f);
    let (code, stdout, _) = run_stdin(
        &["explain-recoveries", "-g", path.to_str().unwrap()],
        b"xx@@@xx",
    );
    let _ = std::fs::remove_file(&path);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("(label: bad_doc)"),
        "expected the author-supplied label in the cluster output, got:\n{stdout}"
    );
}

#[test]
fn explain_recoveries_reports_no_recoveries_on_clean_input() {
    let (code, stdout, _) = run(&[
        "explain-recoveries",
        "-g",
        "grammars/json.peg",
        "benches/fixtures/small.json",
    ]);
    assert_eq!(code, 0);
    assert!(
        stdout.trim() == "no recoveries",
        "expected \"no recoveries\" on a clean parse, got:\n{stdout}"
    );
}

#[test]
fn explain_recoveries_help_short_circuits() {
    let (code, stdout, _) = run(&["explain-recoveries", "--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("explain-recoveries"),
        "expected explain-recoveries help, got: {stdout}"
    );
}
