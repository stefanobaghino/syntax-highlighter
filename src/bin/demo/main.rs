use std::io::{self, Read, Write};
use std::path::Path;
use std::process::ExitCode;

mod highlight;
mod theme;

use highlight::Highlighter;

const JSON_GRAMMAR: &str = include_str!("../../../grammars/json.peg");
const TOML_GRAMMAR: &str = include_str!("../../../grammars/toml.peg");
const SQLITE_GRAMMAR: &str = include_str!("../../../grammars/sqlite.peg");
const RUST_GRAMMAR: &str = include_str!("../../../grammars/rust.peg");
const JS_GRAMMAR: &str = include_str!("../../../grammars/javascript.peg");
const GO_GRAMMAR: &str = include_str!("../../../grammars/go.peg");
const C_GRAMMAR: &str = include_str!("../../../grammars/c.peg");
const CSS_GRAMMAR: &str = include_str!("../../../grammars/css.peg");

/// Default grammar source used when neither `-l` nor a path-extension
/// hint is available (e.g. stdin without flags).
const DEFAULT: &str = JSON_GRAMMAR;

/// Pipe-separated list of canonical language names, suitable for
/// embedding in usage strings: `-l json|toml|sql|rust|js|go|c|css`.
const LANG_NAMES: &str = "json|toml|sql|rust|js|go|c|css";

/// Resolve a language name (canonical or common alias) to the embedded
/// grammar source. Aliases: `sql|sqlite`, `rs|rust`, `js|javascript|mjs|cjs`,
/// `c|h`.
fn by_name(name: &str) -> Option<&'static str> {
    match name {
        "json" => Some(JSON_GRAMMAR),
        "toml" => Some(TOML_GRAMMAR),
        "sql" | "sqlite" => Some(SQLITE_GRAMMAR),
        "rs" | "rust" => Some(RUST_GRAMMAR),
        "js" | "javascript" | "mjs" | "cjs" => Some(JS_GRAMMAR),
        "go" => Some(GO_GRAMMAR),
        "c" | "h" => Some(C_GRAMMAR),
        "css" => Some(CSS_GRAMMAR),
        _ => None,
    }
}

/// Resolve a path's extension to an embedded grammar source.
fn by_extension(path: &Path) -> Option<&'static str> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(by_name)
}

struct Cli {
    grammar: Option<&'static str>,
    path: Option<String>,
}

fn parse_args<I: Iterator<Item = String>>(args: I) -> Result<Cli, String> {
    let mut grammar: Option<&'static str> = None;
    let mut path = None;
    let mut it = args.peekable();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-l" | "--lang" => {
                let name = it
                    .next()
                    .ok_or_else(|| format!("{} requires a value ({})", arg, LANG_NAMES))?;
                grammar = Some(by_name(&name).ok_or_else(|| {
                    format!("unknown language {:?} (expected {})", name, LANG_NAMES)
                })?);
            }
            other if other.starts_with("--lang=") => {
                let name = &other["--lang=".len()..];
                grammar = Some(by_name(name).ok_or_else(|| {
                    format!("unknown language {:?} (expected {})", name, LANG_NAMES)
                })?);
            }
            _ => {
                if path.is_some() {
                    return Err(format!("unexpected extra argument {:?}", arg));
                }
                path = Some(arg);
            }
        }
    }
    Ok(Cli { grammar, path })
}

fn pick_grammar(cli: &Cli) -> Result<&'static str, String> {
    if let Some(g) = cli.grammar {
        return Ok(g);
    }
    match &cli.path {
        Some(p) => by_extension(Path::new(p)).ok_or_else(|| {
            format!(
                "cannot infer language from path {:?}; pass --lang {}",
                p, LANG_NAMES
            )
        }),
        None => Ok(DEFAULT),
    }
}

fn main() -> ExitCode {
    let cli = match parse_args(std::env::args().skip(1)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("demo: {}", e);
            return ExitCode::from(2);
        }
    };

    let grammar = match pick_grammar(&cli) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("demo: {}", e);
            return ExitCode::from(2);
        }
    };

    let input = match &cli.path {
        Some(path) => match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("demo: {}: {}", path, e);
                return ExitCode::from(2);
            }
        },
        None => {
            let mut buf = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buf) {
                eprintln!("demo: reading stdin: {}", e);
                return ExitCode::from(2);
            }
            buf
        }
    };

    let mut highlighter = match Highlighter::new(grammar) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("demo: grammar error: {}", e);
            return ExitCode::from(3);
        }
    };

    highlighter.set_input(input);
    if let Err(e) = io::stdout().write_all(highlighter.highlight().as_bytes()) {
        eprintln!("demo: writing output: {}", e);
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::{by_extension, by_name};
    use std::path::Path;

    #[test]
    fn aliases_resolve_to_canonical_grammars() {
        assert_eq!(by_name("sqlite"), by_name("sql"));
        assert_eq!(by_name("rs"), by_name("rust"));
        assert_eq!(by_name("javascript"), by_name("js"));
        assert_eq!(by_name("mjs"), by_name("js"));
        assert_eq!(by_name("cjs"), by_name("js"));
        assert_eq!(by_name("h"), by_name("c"));
    }

    #[test]
    fn unknown_names_return_none() {
        assert!(by_name("python").is_none());
        assert!(by_name("").is_none());
    }

    #[test]
    fn by_extension_uses_name_table() {
        assert!(by_extension(Path::new("foo.rs")).is_some());
        assert!(by_extension(Path::new("foo.json")).is_some());
        assert!(by_extension(Path::new("foo.unknown")).is_none());
        assert!(by_extension(Path::new("Makefile")).is_none());
    }
}
