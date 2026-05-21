//! `pegc` — PEG compiler-side toolchain.
//!
//! Subcommands today: `stats`. Anchors the family for future
//! compile-time tooling (a disassembler, a serialized-bytecode
//! producer). The conceptual line is compile-time inspection of a
//! grammar source — input-independent, artifact-aware. Distinct from
//! `pegdb`, which is the parse-time / debug-time toolchain. See
//! `TOOLS.md` at the repo root for the full contract.

use std::fmt::Write as _;
use std::io::Write;
use std::process::ExitCode;

use syntax_highlighter::pegc;
use syntax_highlighter::pegc::analysis::{compute_follow, FollowElement};

const TOP_HELP: &str = "\
pegc — PEG compiler-side toolchain for syntax-highlighter

Usage:
    pegc <subcommand> [options] [args]

Subcommands:
    stats <grammar.peg>              Print bytecode counts as one JSON object on stdout.
    follow-set <grammar.peg> [rule]  Print FOLLOW sets as NDJSON; one rule per line.

Options:
    -h, --help               Show this help.

See TOOLS.md for the full contract.
";

const STATS_HELP: &str = "\
Usage: pegc stats <grammar.peg>

Compile a PEG grammar file and print its bytecode size as one JSON
object on stdout (keys: path, instructions, rules, capture_kinds_count,
capture_kinds).

Exit 0 on success, 2 on usage error, 3 on grammar-compile error.
See TOOLS.md for the full contract.
";

const FOLLOW_SET_HELP: &str = "\
Usage: pegc follow-set <grammar.peg> [rule]

Parse a PEG grammar file and emit the per-rule FOLLOW set as NDJSON
on stdout (one JSON object per line, rules sorted alphabetically).

If [rule] is given, emit only that rule's FOLLOW set (single line).

Element kinds in the `follow` array:
  literal      {\"type\":\"literal\",\"value\":<string>}
  char_class   {\"type\":\"char_class\",\"bitmap\":<hex>}     -- 64 hex chars
  rule         {\"type\":\"rule\",\"name\":<string>}
  capture      {\"type\":\"capture\",\"kind\":<string>,\"inner\":<element>}
  eof          {\"type\":\"eof\"}

The analysis is one rule reference deep: FOLLOW(R) surfaces immediate
rule references opaquely (e.g. `Rule(\"returning_clause\")`) rather than
expanding to terminal leaves. Trace through with another `follow-set`
or `compute_first` call as needed.

Exit 0 on success, 2 on usage error / unknown rule, 3 on grammar parse error.
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
        "stats" => run_stats(rest),
        "follow-set" => run_follow_set(rest),
        "-h" | "--help" => {
            print!("{}", TOP_HELP);
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("pegc: unknown subcommand {:?}", other);
            eprintln!();
            eprint!("{}", TOP_HELP);
            ExitCode::from(2)
        }
    }
}

fn run_stats(args: &[String]) -> ExitCode {
    if wants_help(args) {
        print!("{}", STATS_HELP);
        return ExitCode::SUCCESS;
    }
    let mut path: Option<&str> = None;
    for a in args {
        match a.as_str() {
            "-h" | "--help" => {
                print!("{}", STATS_HELP);
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') => {
                eprintln!("pegc stats: unknown flag {:?}", other);
                return ExitCode::from(2);
            }
            other => {
                if path.is_some() {
                    eprintln!("pegc stats: unexpected extra argument {:?}", other);
                    return ExitCode::from(2);
                }
                path = Some(other);
            }
        }
    }
    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("pegc stats: missing <grammar.peg>");
            return ExitCode::from(2);
        }
    };
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("pegc stats: {}: {}", path, e);
            return ExitCode::from(2);
        }
    };
    let prog = match pegc::compile(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("pegc stats: grammar error: {}", e);
            return ExitCode::from(3);
        }
    };
    let mut out = std::io::stdout().lock();
    let mut kinds_json = String::with_capacity(prog.capture_kinds.len() * 16 + 2);
    kinds_json.push('[');
    for (i, k) in prog.capture_kinds.iter().enumerate() {
        if i > 0 {
            kinds_json.push(',');
        }
        kinds_json.push_str(&json_string(k));
    }
    kinds_json.push(']');
    let _ = writeln!(
        out,
        "{{\"path\":{},\"instructions\":{},\"rules\":{},\"capture_kinds_count\":{},\"capture_kinds\":{}}}",
        json_string(path),
        prog.code.len(),
        prog.rule_count,
        prog.capture_kinds.len(),
        kinds_json,
    );
    ExitCode::SUCCESS
}

fn run_follow_set(args: &[String]) -> ExitCode {
    if wants_help(args) {
        print!("{}", FOLLOW_SET_HELP);
        return ExitCode::SUCCESS;
    }
    let mut path: Option<&str> = None;
    let mut rule: Option<&str> = None;
    for a in args {
        match a.as_str() {
            "-h" | "--help" => {
                print!("{}", FOLLOW_SET_HELP);
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') => {
                eprintln!("pegc follow-set: unknown flag {:?}", other);
                return ExitCode::from(2);
            }
            other => {
                if path.is_none() {
                    path = Some(other);
                } else if rule.is_none() {
                    rule = Some(other);
                } else {
                    eprintln!("pegc follow-set: unexpected extra argument {:?}", other);
                    return ExitCode::from(2);
                }
            }
        }
    }
    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("pegc follow-set: missing <grammar.peg>");
            return ExitCode::from(2);
        }
    };
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("pegc follow-set: {}: {}", path, e);
            return ExitCode::from(2);
        }
    };
    let grammar = match pegc::parse(&source) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("pegc follow-set: grammar parse error: {}", e);
            return ExitCode::from(3);
        }
    };
    let follow = compute_follow(&grammar);
    let mut out = std::io::stdout().lock();
    if let Some(rule_name) = rule {
        let Some(set) = follow.get(rule_name) else {
            eprintln!("pegc follow-set: rule {:?} not found in grammar", rule_name);
            return ExitCode::from(2);
        };
        let _ = writeln!(out, "{}", json_record(rule_name, set));
    } else {
        let mut names: Vec<&String> = follow.keys().collect();
        names.sort();
        for name in names {
            let set = follow.get(name).expect("name from keys()");
            let _ = writeln!(out, "{}", json_record(name, set));
        }
    }
    ExitCode::SUCCESS
}

/// Format one rule's FOLLOW set as a single-line JSON object.
fn json_record(rule: &str, set: &std::collections::BTreeSet<FollowElement>) -> String {
    let mut s = String::new();
    s.push('{');
    s.push_str("\"rule\":");
    s.push_str(&json_string(rule));
    s.push_str(",\"follow\":[");
    for (i, elem) in set.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        json_follow_element(&mut s, elem);
    }
    s.push_str("]}");
    s
}

/// Append one FOLLOW element as JSON to `out`.
fn json_follow_element(out: &mut String, elem: &FollowElement) {
    match elem {
        FollowElement::Literal(bytes) => {
            out.push_str("{\"type\":\"literal\",\"value\":");
            out.push_str(&json_string(&String::from_utf8_lossy(bytes)));
            out.push('}');
        }
        FollowElement::CharClass(cs) => {
            out.push_str("{\"type\":\"char_class\",\"bitmap\":\"");
            for b in cs.bitmap() {
                let _ = write!(out, "{:02x}", b);
            }
            out.push_str("\"}");
        }
        FollowElement::Rule(name) => {
            out.push_str("{\"type\":\"rule\",\"name\":");
            out.push_str(&json_string(name));
            out.push('}');
        }
        FollowElement::Capture { kind, inner } => {
            out.push_str("{\"type\":\"capture\",\"kind\":");
            out.push_str(&json_string(kind));
            out.push_str(",\"inner\":");
            json_follow_element(out, inner);
            out.push('}');
        }
        FollowElement::Eof => {
            out.push_str("{\"type\":\"eof\"}");
        }
    }
}

/// True iff `args` contains `-h` or `--help` anywhere.
fn wants_help(args: &[String]) -> bool {
    args.iter().any(|a| a == "-h" || a == "--help")
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
