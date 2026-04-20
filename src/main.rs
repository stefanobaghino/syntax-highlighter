use std::io::{self, Read, Write};
use std::process::ExitCode;

use syntax_highlighter::highlight::Highlighter;

const JSON_GRAMMAR: &str = include_str!("../grammars/json.peg");

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let input = match args.next() {
        Some(path) => match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("syntax-highlighter: {}: {}", path, e);
                return ExitCode::from(2);
            }
        },
        None => {
            let mut buf = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buf) {
                eprintln!("syntax-highlighter: reading stdin: {}", e);
                return ExitCode::from(2);
            }
            buf
        }
    };

    let highlighter = match Highlighter::new(JSON_GRAMMAR) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("syntax-highlighter: grammar error: {}", e);
            return ExitCode::from(3);
        }
    };

    let out = highlighter.highlight(&input);
    if let Err(e) = io::stdout().write_all(out.as_bytes()) {
        eprintln!("syntax-highlighter: writing output: {}", e);
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}
