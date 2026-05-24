use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use syntax_highlighter::pegc;
use syntax_highlighter::pegvm::{Capture, Program, VM};

const CSS_GRAMMAR: &str = include_str!("../grammars/css.peg");

// Compile the grammar exactly once per test process. The init lock's
// poisoning is what concentrates a broken-grammar failure into one
// informative panic plus N-1 trivial "previously failed" panics across
// the rest of the file's tests.
static CSS_PROGRAM: OnceLock<Program> = OnceLock::new();
static CSS_INIT: Mutex<()> = Mutex::new(());

fn css_program() -> &'static Program {
    if let Some(p) = CSS_PROGRAM.get() {
        return p;
    }
    let _guard = CSS_INIT
        .lock()
        .expect("CSS grammar compile previously failed; see the first failure");
    CSS_PROGRAM.get_or_init(|| pegc::compile(CSS_GRAMMAR).expect("CSS grammar should compile"))
}

#[test]
fn grammar_compiles() {
    let _ = css_program();
}

fn run(input: &str) -> (usize, Vec<Capture>, Vec<String>, bool) {
    let prog = css_program();
    let r = VM::new_from_program(prog, input.as_bytes()).run();
    (
        r.matched,
        r.captures,
        prog.capture_kinds.clone(),
        r.complete,
    )
}

fn assert_complete_full(input: &str) -> (Vec<Capture>, Vec<String>) {
    let (matched, caps, kinds, complete) = run(input);
    assert!(
        complete,
        "expected complete match, got matched={} of {}",
        matched,
        input.len()
    );
    assert_eq!(matched, input.len(), "expected full-input match");
    (caps, kinds)
}

fn kinds_for<'a>(captures: &[Capture], kinds: &'a [String]) -> Vec<&'a str> {
    captures
        .iter()
        .map(|c| kinds[c.kind.0 as usize].as_str())
        .collect()
}

#[test]
fn grammar_parses_and_compiles() {
    let prog = css_program();
    assert!(!prog.code.is_empty());
    assert!(!prog.capture_kinds.is_empty());
}

#[test]
fn capture_kinds_stay_in_theme_vocabulary() {
    let prog = css_program();
    let allowed: HashSet<&str> = [
        "keyword",
        "string",
        "number",
        "comment",
        "operator",
        "punctuation",
        "type",
        "function",
        "constant",
        "property",
        "variable",
        "recovery",
    ]
    .into_iter()
    .collect();
    for k in &prog.capture_kinds {
        assert!(
            allowed.contains(k.as_str()),
            "capture kind {:?} is not in the theme vocabulary",
            k
        );
    }
}

#[test]
fn empty_document_matches() {
    let (caps, _) = assert_complete_full("");
    assert!(caps.is_empty());
}

#[test]
fn simple_rule_parses() {
    let input = "body { color: red; }\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"type"), "body should be @type: {:?}", k);
    assert!(
        k.contains(&"property"),
        "color should be @property: {:?}",
        k
    );
}

#[test]
fn class_id_selectors_parse() {
    let input = ".foo { color: blue; }\n#bar { margin: 10px; }\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(
        k.contains(&"property"),
        "class selector should be @property: {:?}",
        k
    );
    assert!(
        k.contains(&"constant"),
        "id selector should be @constant: {:?}",
        k
    );
}

#[test]
fn pseudo_selectors_parse() {
    let input = "a:hover { color: red; }\na::before { content: \"x\"; }\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(
        k.contains(&"function"),
        "pseudo selectors should be @function: {:?}",
        k
    );
}

#[test]
fn attribute_selector_parses() {
    let input = "input[type=\"text\"] { border: 1px solid black; }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn combinators_parse() {
    let input = "div > p { margin: 0; }\n.a + .b { color: red; }\n.a ~ .b { color: blue; }\n.a .b { color: green; }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn hex_colors_and_units_parse() {
    let input = ".c { color: #ff00ff; background: #abc; width: 100px; height: 50%; margin: 0.5em; line-height: 1.5; }\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"constant"), "hex color @constant: {:?}", k);
    assert!(k.contains(&"number"), "numbers with units @number: {:?}", k);
}

#[test]
fn function_call_and_calc_parse() {
    let input = ".c { color: rgb(255, 0, 0); background: rgba(0, 0, 0, 0.5); width: calc(100% - 20px); transform: rotate(45deg); }\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(
        k.contains(&"function"),
        "value fns should be @function: {:?}",
        k
    );
}

#[test]
fn var_and_custom_property_parse() {
    let input = ":root { --main-color: red; --gap: 8px; }\n.c { color: var(--main-color); padding: var(--gap, 4px); }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn url_unquoted_parses() {
    let input = ".c { background-image: url(image.png); }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn url_quoted_parses() {
    let input = ".c { background-image: url(\"image.png\"); }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn at_media_parses() {
    let input = "@media (min-width: 600px) { .c { color: red; } }\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"keyword"), "@media as keyword: {:?}", k);
}

#[test]
fn at_keyframes_parses() {
    let input = "@keyframes fade { from { opacity: 0; } to { opacity: 1; } }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn at_import_parses() {
    let input = "@import \"reset.css\";\n@import url(\"theme.css\");\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn at_font_face_parses() {
    let input = "@font-face { font-family: \"MyFont\"; src: url(\"my.woff2\"); }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn nested_rule_parses() {
    let input = ".parent { color: red; .child { color: blue; &:hover { color: green; } } }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn important_flag_parses() {
    let input = ".c { color: red !important; }\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(
        k.contains(&"keyword"),
        "!important should be @keyword: {:?}",
        k
    );
}

#[test]
fn comments_parse() {
    let input = "/* header comment */\n.c { /* inline */ color: red; }\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"comment"), "expected @comment: {:?}", k);
}

#[test]
fn multi_value_comma_list_parses() {
    let input = ".c { font-family: Arial, \"Helvetica Neue\", sans-serif; transition: color 0.2s, background 0.3s ease-out; }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn recovery_absorbs_malformed_item() {
    let input = ".a { color: red; }\n@@@ garbage @@@\n.b { color: blue; }\n";
    let (matched, caps, kinds, complete) = run(input);
    assert!(complete, "recovery should keep parse complete");
    assert_eq!(matched, input.len());
    let k = kinds_for(&caps, &kinds);
    assert!(
        k.contains(&"recovery"),
        "expected a @recovery capture for the @@@ region, got: {:?}",
        k
    );
}

#[test]
fn recovery_absorbs_malformed_block_body() {
    // `block`'s `^block_close` catch resyncs at the closing `}` when
    // the body contains garbage before the brace.
    let input = ".btn { color: red; @@@; padding: 4px; }\n";
    let (matched, caps, kinds, complete) = run(input);
    assert!(complete, "block_close catch should keep parse complete");
    assert_eq!(matched, input.len());
    let k = kinds_for(&caps, &kinds);
    assert!(
        k.contains(&"recovery"),
        "expected a @recovery capture inside the block, got: {:?}",
        k
    );
}

// --- Stage 1 UTF-8 tests ----------------------------------------------

#[test]
fn non_ascii_identifier_in_class_selector() {
    // Per CSS Syntax Module L3 §4.6, non-ASCII code points are
    // ident-start code points.
    let input = ".café { color: red; }";
    let (_, caps, kinds, complete) = run(input);
    assert!(complete, "expected complete parse for {input:?}");
    let props: Vec<&str> = caps
        .iter()
        .filter(|c| kinds[c.kind.0 as usize] == "property")
        .map(|c| &input[c.start..c.end])
        .collect();
    assert!(
        props.contains(&".café"),
        "expected `.café` to capture as a property selector; got {props:?}"
    );
}

#[test]
fn hex_escape_in_class_selector() {
    // CSS Syntax Module L3 §4.6 admits `\\HHHHHH` escapes in identifiers.
    // The broader `\\` + any-non-newline form covers this case.
    let input = ".cl\\61ss { color: red; }";
    let (caps, kinds) = assert_complete_full(input);
    let props: Vec<&str> = caps
        .iter()
        .filter(|c| kinds[c.kind.0 as usize] == "property")
        .map(|c| &input[c.start..c.end])
        .collect();
    assert!(
        props.iter().any(|p| p.contains("\\61")),
        "expected hex-escaped selector to capture as property; got {props:?}"
    );
}
