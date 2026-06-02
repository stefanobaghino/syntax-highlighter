use std::io::{self, Read, Write};
use std::path::Path;
use std::process::ExitCode;

mod highlight;
mod theme;

use highlight::Highlighter;

/// AOT-precompiled grammar bytecode produced by `build.rs` into
/// `$OUT_DIR/<lang>.pegb`. Embedded here so the demo never compiles
/// a grammar at startup; `Highlighter::from_pegb` decodes the blob
/// (microseconds) and the parser runs directly off the bytecode.
const JSON_PEGB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/json.pegb"));
const TOML_PEGB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/toml.pegb"));
const SQLITE_PEGB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sqlite.pegb"));
const RUST_PEGB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rust.pegb"));
const JS_PEGB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/javascript.pegb"));
const GO_PEGB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/go.pegb"));
const C_PEGB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/c.pegb"));
const CSS_PEGB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/css.pegb"));
const STARLARK_PEGB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/starlark.pegb"));
const YAML_PEGB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/yaml.pegb"));

/// Default grammar used when neither `-l` nor a path-extension hint
/// is available (e.g. stdin without flags).
const DEFAULT: &[u8] = JSON_PEGB;

/// Pipe-separated list of canonical language names, suitable for
/// embedding in usage strings.
const LANG_NAMES: &str = "json|toml|sql|rust|js|go|c|css|starlark|yaml";

/// Resolve a language name (canonical or common alias) to the embedded
/// `pegb` bytecode. Aliases: `sql|sqlite`, `rs|rust`, `js|javascript|mjs|cjs`,
/// `c|h`, `starlark|star|bzl`, `yaml|yml`. Starlark and YAML are
/// deliberately-pruned subsets (issue #43); see `grammars/{starlark,yaml}.peg`.
fn by_name(name: &str) -> Option<&'static [u8]> {
    match name {
        "json" => Some(JSON_PEGB),
        "toml" => Some(TOML_PEGB),
        "sql" | "sqlite" => Some(SQLITE_PEGB),
        "rs" | "rust" => Some(RUST_PEGB),
        "js" | "javascript" | "mjs" | "cjs" => Some(JS_PEGB),
        "go" => Some(GO_PEGB),
        "c" | "h" => Some(C_PEGB),
        "css" => Some(CSS_PEGB),
        "starlark" | "star" | "bzl" => Some(STARLARK_PEGB),
        "yaml" | "yml" => Some(YAML_PEGB),
        _ => None,
    }
}

/// Resolve a path's extension to embedded `pegb` bytecode.
fn by_extension(path: &Path) -> Option<&'static [u8]> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(by_name)
}

struct Cli {
    grammar: Option<&'static [u8]>,
    path: Option<String>,
}

fn parse_args<I: Iterator<Item = String>>(args: I) -> Result<Cli, String> {
    let mut grammar: Option<&'static [u8]> = None;
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

fn pick_grammar(cli: &Cli) -> Result<&'static [u8], String> {
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

    let mut highlighter = match Highlighter::from_pegb(grammar) {
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
        assert_eq!(by_name("star"), by_name("starlark"));
        assert_eq!(by_name("bzl"), by_name("starlark"));
        assert_eq!(by_name("yml"), by_name("yaml"));
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
        assert!(by_extension(Path::new("foo.star")).is_some());
        assert!(by_extension(Path::new("foo.yaml")).is_some());
        assert!(by_extension(Path::new("foo.unknown")).is_none());
        assert!(by_extension(Path::new("Makefile")).is_none());
    }
}
