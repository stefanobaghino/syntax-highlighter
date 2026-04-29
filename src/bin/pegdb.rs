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

const TOP_HELP: &str = "\
pegdb — grammar-developer debug surface for syntax-highlighter

Usage:
    pegdb <subcommand> [options] [args]

Subcommands:
    dump-captures -g <grammar.peg> [--max-literal=N] [<path>]
                                                 Print captures as JSONL (one object per capture).

Options:
    -h, --help                                   Show this help.

For compile-time grammar inspection (bytecode shape), see `pegc`.
See TOOLS.md for the full contract: subcommand semantics, JSONL schemas,
exit codes, partial-match diagnostics.
";

const DUMP_HELP: &str = "\
Usage: pegdb dump-captures -g <grammar.peg> [--max-literal=N] [<path>]

Print one capture per line as a JSON object (keys: start, end, kind,
literal). Exits 1 with a stderr partial-match marker on incomplete
parses. The grammar source is required.

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
    for cap in captures {
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
