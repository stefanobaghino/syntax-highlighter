use std::io::{self, Read, Write};
use std::path::Path;
use std::process::ExitCode;

use syntax_highlighter::highlight::Highlighter;

const JSON_GRAMMAR: &str = include_str!("../grammars/json.peg");
const TOML_GRAMMAR: &str = include_str!("../grammars/toml.peg");
const SQLITE_GRAMMAR: &str = include_str!("../grammars/sqlite.peg");
const RUST_GRAMMAR: &str = include_str!("../grammars/rust.peg");
const JS_GRAMMAR: &str = include_str!("../grammars/javascript.peg");
const GO_GRAMMAR: &str = include_str!("../grammars/go.peg");
const C_GRAMMAR: &str = include_str!("../grammars/c.peg");
const CSS_GRAMMAR: &str = include_str!("../grammars/css.peg");

#[derive(Clone, Copy)]
enum Lang {
    Json,
    Toml,
    Sql,
    Rust,
    JavaScript,
    Go,
    C,
    Css,
}

impl Lang {
    fn grammar(self) -> &'static str {
        match self {
            Lang::Json => JSON_GRAMMAR,
            Lang::Toml => TOML_GRAMMAR,
            Lang::Sql => SQLITE_GRAMMAR,
            Lang::Rust => RUST_GRAMMAR,
            Lang::JavaScript => JS_GRAMMAR,
            Lang::Go => GO_GRAMMAR,
            Lang::C => C_GRAMMAR,
            Lang::Css => CSS_GRAMMAR,
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        match name {
            "json" => Some(Lang::Json),
            "toml" => Some(Lang::Toml),
            "sql" | "sqlite" => Some(Lang::Sql),
            "rs" | "rust" => Some(Lang::Rust),
            "js" | "javascript" | "mjs" | "cjs" => Some(Lang::JavaScript),
            "go" => Some(Lang::Go),
            "c" | "h" => Some(Lang::C),
            "css" => Some(Lang::Css),
            _ => None,
        }
    }

    fn from_extension(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_name)
    }
}

struct Cli {
    lang: Option<Lang>,
    path: Option<String>,
}

fn parse_args<I: Iterator<Item = String>>(args: I) -> Result<Cli, String> {
    let mut lang = None;
    let mut path = None;
    let mut it = args.peekable();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-l" | "--lang" => {
                let name = it.next().ok_or_else(|| {
                    format!("{} requires a value (json|toml|sql|rust|js|go|c|css)", arg)
                })?;
                lang = Some(Lang::from_name(&name).ok_or_else(|| {
                    format!(
                        "unknown language {:?} (expected json|toml|sql|rust|js|go|c|css)",
                        name
                    )
                })?);
            }
            other if other.starts_with("--lang=") => {
                let name = &other["--lang=".len()..];
                lang = Some(Lang::from_name(name).ok_or_else(|| {
                    format!(
                        "unknown language {:?} (expected json|toml|sql|rust|js|go|c|css)",
                        name
                    )
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
    Ok(Cli { lang, path })
}

fn pick_lang(cli: &Cli) -> Result<Lang, String> {
    if let Some(l) = cli.lang {
        return Ok(l);
    }
    match &cli.path {
        Some(p) => Lang::from_extension(Path::new(p)).ok_or_else(|| {
            format!(
                "cannot infer language from path {:?}; pass --lang json|toml|sql|rust|js|go|c|css",
                p
            )
        }),
        None => Ok(Lang::Json),
    }
}

fn main() -> ExitCode {
    let cli = match parse_args(std::env::args().skip(1)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("syntax-highlighter: {}", e);
            return ExitCode::from(2);
        }
    };

    let lang = match pick_lang(&cli) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("syntax-highlighter: {}", e);
            return ExitCode::from(2);
        }
    };

    let input = match &cli.path {
        Some(path) => match std::fs::read_to_string(path) {
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

    let mut highlighter = match Highlighter::new(lang.grammar()) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("syntax-highlighter: grammar error: {}", e);
            return ExitCode::from(3);
        }
    };

    highlighter.set_input(input);
    if let Err(e) = io::stdout().write_all(highlighter.highlight().as_bytes()) {
        eprintln!("syntax-highlighter: writing output: {}", e);
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}
