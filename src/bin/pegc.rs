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

const TOP_HELP: &str = "\
pegc — PEG compiler-side toolchain for syntax-highlighter

Usage:
    pegc <subcommand> [options] [args]

Subcommands:
    stats <grammar.peg>      Print bytecode counts as one JSON object on stdout.

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

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        print!("{}", TOP_HELP);
        return ExitCode::SUCCESS;
    }
    let (sub, rest) = args.split_first().unwrap();
    match sub.as_str() {
        "stats" => run_stats(rest),
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
