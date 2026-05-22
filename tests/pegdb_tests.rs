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

// ---------- captures dump ----------

#[test]
fn captures_dump_emits_well_shaped_jsonl() {
    let (code, stdout, stderr) = run(&[
        "captures",
        "dump",
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
fn captures_dump_partial_parse_emits_marker_and_exits_one() {
    let (code, stdout, stderr) = run_stdin(
        &["captures", "dump", "-g", "grammars/json.peg"],
        br#"{"a": 1"#,
    );
    assert_eq!(code, 1, "stderr: {stderr}");
    let first = stdout.lines().next().expect("at least one capture line");
    assert!(
        first.starts_with('{') && first.ends_with('}'),
        "first stdout line not a JSON object: {first:?}"
    );
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
fn captures_dump_depth_zero_for_flat_grammar() {
    let (code, stdout, _) = run(&[
        "captures",
        "dump",
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
fn captures_dump_depth_surfaces_go_qualified_ident_nesting() {
    let (code, stdout, stderr) = run_stdin(
        &["captures", "dump", "-g", "grammars/go.peg"],
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
fn captures_dump_max_literal_truncates() {
    let (code, stdout, _stderr) = run(&[
        "captures",
        "dump",
        "-g",
        "grammars/json.peg",
        "--max-literal=3",
        "benches/fixtures/small.json",
    ]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("…"),
        "expected ellipsis in truncated output, got:\n{stdout}"
    );
}

#[test]
fn captures_dump_rejects_max_literal_with_non_integer_value() {
    let (code, _, stderr) = run(&[
        "captures",
        "dump",
        "-g",
        "grammars/json.peg",
        "--max-literal=not-a-num",
        "benches/fixtures/small.json",
    ]);
    assert_eq!(code, 2);
    assert!(stderr.contains("--max-literal"), "stderr: {stderr}");
}

#[test]
fn captures_dump_accepts_space_separated_max_literal() {
    let (code, stdout, stderr) = run(&[
        "captures",
        "dump",
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
fn captures_dump_max_literal_without_value_errors() {
    let (code, _, stderr) = run(&["captures", "dump", "--max-literal"]);
    assert_eq!(code, 2);
    assert!(stderr.contains("requires a value"), "stderr: {stderr}");
}

#[test]
fn captures_dump_grammar_compile_error_exits_3() {
    let (code, _, stderr) = run(&[
        "captures",
        "dump",
        "-g",
        "benches/fixtures/medium.json",
        "benches/fixtures/small.json",
    ]);
    assert_eq!(code, 3);
    assert!(stderr.contains("grammar"), "stderr: {stderr}");
}

#[test]
fn captures_dump_help_short_circuits_max_literal_parse_error() {
    let (code, stdout, _) = run(&["captures", "dump", "--max-literal=banana", "--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("captures dump") && stdout.contains("TOOLS.md"),
        "expected captures-dump help, got: {stdout}"
    );
}

#[test]
fn captures_dump_recovery_rows_carry_no_farthest_reach() {
    // Regression: `farthest_reach` was removed from `captures dump`
    // when recovery detail moved to `recoveries dump`. The field
    // must not reappear on any row.
    let (code, stdout, _) = run_stdin(
        &["captures", "dump", "-g", "grammars/rust.peg"],
        b"fn ok() {}\n@@@\nfn ok2() {}\n",
    );
    assert_eq!(code, 0);
    assert!(
        !stdout.contains("farthest_reach"),
        "captures dump must not emit farthest_reach; got:\n{stdout}"
    );
    // Sanity: the input still produces recovery captures — otherwise
    // the assertion above would be vacuously true.
    let saw_recovery = stdout
        .lines()
        .any(|line| json_field_str(line, "kind") == Some("\"recovery\""));
    assert!(
        saw_recovery,
        "test fixture should produce recovery captures; got:\n{stdout}"
    );
}

// ---------- recoveries dump ----------

#[test]
fn recoveries_dump_emits_one_row_per_span() {
    // The `@@@` run produces multiple recovery captures (one per
    // skipped byte) in `captures dump`, but `recoveries dump`
    // collapses them into one row per maximal contiguous span.
    let (cap_code, cap_stdout, _) = run_stdin(
        &["captures", "dump", "-g", "grammars/rust.peg"],
        b"fn ok() {}\n@@@\nfn ok2() {}\n",
    );
    assert_eq!(cap_code, 0);
    let cap_recovery_rows = cap_stdout
        .lines()
        .filter(|line| json_field_str(line, "kind") == Some("\"recovery\""))
        .count();
    assert!(
        cap_recovery_rows >= 2,
        "fixture should produce multiple recovery captures; got {cap_recovery_rows}"
    );

    let (code, stdout, _) = run_stdin(
        &["recoveries", "dump", "-g", "grammars/rust.peg"],
        b"fn ok() {}\n@@@\nfn ok2() {}\n",
    );
    assert_eq!(code, 0);
    let span_rows: Vec<&str> = stdout
        .lines()
        .filter(|line| json_field_str(line, "kind") == Some("\"recovery\""))
        .collect();
    assert!(!span_rows.is_empty(), "expected at least one span row");
    assert!(
        span_rows.len() < cap_recovery_rows,
        "recoveries dump should collapse {cap_recovery_rows} per-byte captures into fewer spans; got {} spans",
        span_rows.len()
    );
    for line in &span_rows {
        let start: usize = json_field_str(line, "start").unwrap().parse().unwrap();
        let end: usize = json_field_str(line, "end").unwrap().parse().unwrap();
        assert!(start < end, "span start..end empty: {line}");
    }
}

#[test]
fn recoveries_dump_carries_label_pos_rule_stack_literal() {
    let (code, stdout, _) = run_stdin(
        &["recoveries", "dump", "-g", "grammars/rust.peg"],
        b"fn ok() {}\n@@@ garbage @@@\nfn ok2() {}\n",
    );
    assert_eq!(code, 0);
    let mut row_count = 0;
    for line in stdout.lines() {
        if json_field_str(line, "sanity").is_some() {
            continue;
        }
        let label = json_field_str(line, "label").expect("label present");
        assert!(
            label.starts_with('"') && label.ends_with('"'),
            "label not JSON-string: {label:?}"
        );
        assert!(
            json_field_str(line, "pos").is_some(),
            "pos field missing: {line}"
        );
        let rule_stack = json_field_str(line, "rule_stack").expect("rule_stack present");
        assert!(
            rule_stack.starts_with('[') && rule_stack.ends_with(']'),
            "rule_stack not a JSON array: {rule_stack:?}"
        );
        let literal = json_field_str(line, "literal").expect("literal present");
        assert!(
            literal.starts_with('"') && literal.ends_with('"'),
            "literal not JSON-string: {literal:?}"
        );
        row_count += 1;
    }
    assert!(row_count > 0, "no recovery rows emitted; got:\n{stdout}");
}

#[test]
fn recoveries_dump_skips_trivia_in_leaf() {
    // Same `trivia <- ws` cascade story as `recoveries explain`: the
    // displayed leaf (last element of `rule_stack`) must not be a
    // trivia rule.
    let (code, stdout, _) = run_stdin(
        &["recoveries", "dump", "-g", "grammars/rust.peg"],
        b"fn ok() {}\n@@@ garbage @@@\nfn ok2() {}\n",
    );
    assert_eq!(code, 0);
    let trivia_names = [
        "\"ws\"",
        "\"spacing\"",
        "\"comment\"",
        "\"line_comment\"",
        "\"block_comment\"",
    ];
    let mut row_count = 0;
    for line in stdout.lines() {
        if json_field_str(line, "sanity").is_some() {
            continue;
        }
        let rule_stack = json_field_str(line, "rule_stack").expect("rule_stack present");
        // Find the last quoted name in the rule_stack array. The
        // structure is `["a","b",…,"leaf"]`; locate the last `"` pair.
        let leaf_token = rule_stack
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .next_back()
            .map(str::trim)
            .unwrap_or("");
        assert!(
            !trivia_names.contains(&leaf_token),
            "displayed leaf must not be a trivia rule, got {leaf_token:?} in: {line}"
        );
        row_count += 1;
    }
    assert!(row_count > 0, "no recovery rows emitted");
}

#[test]
fn recoveries_dump_full_stack_always_emitted() {
    // The proposal commits to always-full-stack (no `--leaf-only` flag).
    // For an input that triggers a recovery deeper than the root rule,
    // the stack must carry more than just the leaf.
    let (code, stdout, _) = run_stdin(
        &["recoveries", "dump", "-g", "grammars/rust.peg"],
        b"fn ok() { @@@ garbage @@@ }\n",
    );
    assert_eq!(code, 0);
    let mut saw_multi_frame = false;
    for line in stdout.lines() {
        if json_field_str(line, "sanity").is_some() {
            continue;
        }
        let rule_stack = json_field_str(line, "rule_stack").expect("rule_stack present");
        // Count commas inside the array — N commas implies N+1 frames.
        let inner = rule_stack.trim_start_matches('[').trim_end_matches(']');
        if !inner.is_empty() && inner.contains(',') {
            saw_multi_frame = true;
        }
    }
    assert!(
        saw_multi_frame,
        "expected at least one row with multiple rule_stack frames, got:\n{stdout}"
    );
}

#[test]
fn recoveries_dump_max_literal_truncates() {
    // The `@@@` span is 3 bytes; `--max-literal=2` forces the
    // literal to be truncated with an ellipsis.
    let (code, stdout, _) = run_stdin(
        &[
            "recoveries",
            "dump",
            "-g",
            "grammars/rust.peg",
            "--max-literal=2",
        ],
        b"fn ok() {}\n@@@ garbage @@@\nfn ok2() {}\n",
    );
    assert_eq!(code, 0);
    assert!(
        stdout.contains("…"),
        "expected ellipsis in truncated literal, got:\n{stdout}"
    );
}

#[test]
fn recoveries_dump_no_recoveries_emits_nothing() {
    // Clean input on the JSON grammar — no recoveries, no orphans,
    // nothing to emit.
    let (code, stdout, _) = run(&[
        "recoveries",
        "dump",
        "-g",
        "grammars/json.peg",
        "benches/fixtures/small.json",
    ]);
    assert_eq!(code, 0);
    assert!(
        stdout.trim().is_empty(),
        "expected empty stdout on clean parse, got:\n{stdout}"
    );
}

#[test]
fn recoveries_dump_partial_match_marker() {
    let (code, _stdout, stderr) = run_stdin(
        &["recoveries", "dump", "-g", "grammars/json.peg"],
        br#"{"a": 1"#,
    );
    assert_eq!(code, 1, "stderr: {stderr}");
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
fn recoveries_dump_help_short_circuits() {
    let (code, stdout, _) = run(&["recoveries", "dump", "--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("recoveries dump"),
        "expected recoveries-dump help, got: {stdout}"
    );
}

#[test]
fn recoveries_dump_help_points_at_tools_md() {
    let (code, stdout, _) = run(&["recoveries", "dump", "--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("TOOLS.md"),
        "recoveries-dump help should point at TOOLS.md for the full contract"
    );
}

#[test]
fn recoveries_dump_clean_run_emits_no_sanity_rows() {
    // Well-formed inputs produce no orphan accounting. The `sanity`
    // key must not appear anywhere in a clean parse's output.
    let (code, stdout, _) = run_stdin(
        &["recoveries", "dump", "-g", "grammars/rust.peg"],
        b"fn ok() {}\n@@@ garbage @@@\nfn ok2() {}\n",
    );
    assert_eq!(code, 0);
    assert!(
        !stdout.contains("\"sanity\""),
        "well-formed run should not emit sanity rows, got:\n{stdout}"
    );
}

// ---------- recoveries explain ----------

#[test]
fn recoveries_explain_skips_trivia_when_picking_leaf() {
    let (code, stdout, _) = run_stdin(
        &["recoveries", "explain", "-g", "grammars/rust.peg"],
        b"fn ok() {}\n@@@ garbage @@@\nfn ok2() {}\n",
    );
    assert_eq!(code, 0);
    let trivia_names = ["ws", "spacing", "comment", "line_comment", "block_comment"];
    for line in stdout.lines() {
        if line.starts_with("[sanity]") || line == "no recoveries" {
            continue;
        }
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
fn recoveries_explain_after_block_close_migration_rust() {
    let (code, stdout, _) = run_stdin(
        &["recoveries", "explain", "-g", "grammars/rust.peg"],
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
fn recoveries_explain_clusters_by_rule_stack() {
    let (code, stdout, _) = run_stdin(
        &["recoveries", "explain", "-g", "grammars/rust.peg"],
        b"fn ok() {}\n@@@ garbage @@@\nfn ok2() {}\n",
    );
    assert_eq!(code, 0);
    let cluster_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.starts_with("[sanity]") && *line != "no recoveries")
        .collect();
    assert!(
        !cluster_lines.is_empty(),
        "expected at least one cluster line"
    );
    for line in &cluster_lines {
        assert!(
            line.contains("recoveries") && line.contains("farthest reach ends at"),
            "unexpected cluster line shape: {line}"
        );
    }
    let counts: Vec<usize> = cluster_lines
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
fn recoveries_explain_emits_label_suffix_on_every_cluster() {
    let (code, stdout, _) = run_stdin(
        &["recoveries", "explain", "-g", "grammars/rust.peg"],
        b"fn ok() {}\n@@@ garbage @@@\nfn ok2() {}\n",
    );
    assert_eq!(code, 0);
    let cluster_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.starts_with("[sanity]") && *line != "no recoveries")
        .collect();
    assert!(
        !cluster_lines.is_empty(),
        "expected at least one cluster line"
    );
    for line in &cluster_lines {
        assert!(
            line.contains("recoveries")
                && line.contains("farthest reach ends at")
                && line.contains("(label:"),
            "unexpected cluster line (missing label suffix): {line}"
        );
    }
}

#[test]
fn recoveries_explain_uses_author_supplied_recovery_label() {
    use std::io::Write as _;
    let mut path = std::env::temp_dir();
    path.push(format!(
        "pegdb_recoveries_explain_label_{}.peg",
        std::process::id()
    ));
    let grammar = b"doc <- 'x'*^:bad_doc\n";
    let mut f = std::fs::File::create(&path).expect("create temp grammar");
    f.write_all(grammar).expect("write temp grammar");
    drop(f);
    let (code, stdout, _) = run_stdin(
        &["recoveries", "explain", "-g", path.to_str().unwrap()],
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
fn recoveries_explain_reports_no_recoveries_on_clean_input() {
    let (code, stdout, _) = run(&[
        "recoveries",
        "explain",
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
fn recoveries_explain_help_short_circuits() {
    let (code, stdout, _) = run(&["recoveries", "explain", "--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("recoveries explain"),
        "expected recoveries-explain help, got: {stdout}"
    );
}

#[test]
fn recoveries_explain_clean_run_emits_no_sanity_lines() {
    // Mirror of `recoveries_dump_clean_run_emits_no_sanity_rows` for
    // the text-format verb. Well-formed runs must not append any
    // `[sanity]` lines.
    let (code, stdout, _) = run_stdin(
        &["recoveries", "explain", "-g", "grammars/rust.peg"],
        b"fn ok() {}\n@@@ garbage @@@\nfn ok2() {}\n",
    );
    assert_eq!(code, 0);
    assert!(
        !stdout.contains("[sanity]"),
        "well-formed run should not emit [sanity] lines, got:\n{stdout}"
    );
}

// ---------- help strings ----------

#[test]
fn top_level_help_is_non_empty() {
    for flag in ["--help", "-h"] {
        let (code, stdout, _) = run(&[flag]);
        assert_eq!(code, 0, "{flag} should exit 0");
        assert!(stdout.contains("pegdb"), "{flag} output: {stdout}");
        assert!(stdout.contains("captures"), "missing captures noun in help");
        assert!(
            stdout.contains("recoveries"),
            "missing recoveries noun in help"
        );
    }
}

#[test]
fn no_args_prints_top_level_help() {
    let (code, stdout, _) = run(&[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("pegdb"));
    assert!(stdout.contains("Nouns"));
}

#[test]
fn captures_dump_help_points_at_tools_md() {
    let (code, stdout, _) = run(&["captures", "dump", "--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("captures dump"));
    assert!(
        stdout.contains("TOOLS.md"),
        "captures-dump help should point at TOOLS.md for the full contract"
    );
}

#[test]
fn captures_noun_help_lists_verbs() {
    let (code, stdout, _) = run(&["captures", "--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("dump"),
        "captures noun help should list its verbs, got: {stdout}"
    );
}

#[test]
fn recoveries_noun_help_lists_verbs() {
    let (code, stdout, _) = run(&["recoveries", "--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("dump") && stdout.contains("explain"),
        "recoveries noun help should list dump and explain verbs, got: {stdout}"
    );
}

#[test]
fn unknown_noun_is_usage_error() {
    let (code, _stdout, stderr) = run(&["bogus"]);
    assert_eq!(code, 2);
    assert!(stderr.contains("unknown noun"), "stderr: {stderr}");
}

#[test]
fn unknown_verb_under_known_noun_is_usage_error() {
    let (code, _stdout, stderr) = run(&["captures", "bogus"]);
    assert_eq!(code, 2);
    assert!(stderr.contains("unknown verb"), "stderr: {stderr}");
}
