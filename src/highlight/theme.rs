pub const RESET: &str = "\x1b[0m";

/// Map a capture-kind name to an ANSI color escape string.
/// Unknown kinds get an empty string (no color change).
pub fn color_for(kind: &str) -> &'static str {
    match kind {
        "keyword" => "\x1b[1;34m",   // bold blue
        "string" => "\x1b[32m",      // green
        "number" => "\x1b[33m",      // yellow
        "comment" => "\x1b[2;37m",   // dim white/gray
        "operator" => "\x1b[36m",    // cyan
        "punctuation" => "\x1b[37m", // white
        "type" => "\x1b[33m",        // yellow
        "function" => "\x1b[1;33m",  // bold yellow
        "constant" => "\x1b[35m",    // magenta
        "property" => "\x1b[36m",    // cyan
        "variable" => "\x1b[37m",    // white
        _ => "",
    }
}
