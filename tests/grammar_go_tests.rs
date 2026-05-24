use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use syntax_highlighter::pegc;
use syntax_highlighter::pegvm::{Capture, Program, VM};

const GO_GRAMMAR: &str = include_str!("../grammars/go.peg");

// Compile the grammar exactly once per test process. The init lock's
// poisoning is what concentrates a broken-grammar failure into one
// informative panic plus N-1 trivial "previously failed" panics across
// the rest of the file's tests.
static GO_PROGRAM: OnceLock<Program> = OnceLock::new();
static GO_INIT: Mutex<()> = Mutex::new(());

fn go_program() -> &'static Program {
    if let Some(p) = GO_PROGRAM.get() {
        return p;
    }
    let _guard = GO_INIT
        .lock()
        .expect("Go grammar compile previously failed; see the first failure");
    GO_PROGRAM.get_or_init(|| pegc::compile(GO_GRAMMAR).expect("Go grammar should compile"))
}

#[test]
fn grammar_compiles() {
    let _ = go_program();
}

fn run(input: &str) -> (usize, Vec<Capture>, Vec<String>, bool) {
    let prog = go_program();
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

/// Collect recovery captures whose text contains anything other than ASCII
/// whitespace. Trailing-newline recovery is a known pre-existing quirk (see
/// #71); this filter keeps callers focused on real recovery regressions.
fn non_ws_recovery_literals<'a>(
    captures: &[Capture],
    kinds: &[String],
    input: &'a str,
) -> Vec<&'a str> {
    captures
        .iter()
        .filter(|c| kinds[c.kind.0 as usize] == "recovery")
        .map(|c| &input[c.start..c.end])
        .filter(|s| s.chars().any(|ch| !ch.is_ascii_whitespace()))
        .collect()
}

#[test]
fn grammar_parses_and_compiles() {
    let prog = go_program();
    assert!(!prog.code.is_empty());
    assert!(!prog.capture_kinds.is_empty());
}

#[test]
fn capture_kinds_stay_in_theme_vocabulary() {
    let prog = go_program();
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
fn minimal_package_parses() {
    let (_, _) = assert_complete_full("package main\n");
}

#[test]
fn hello_world_parses() {
    let input =
        "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"hello, world\")\n}\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"keyword"), "expected keywords: {:?}", k);
    assert!(k.contains(&"string"), "expected string: {:?}", k);
}

#[test]
fn grouped_imports_parse() {
    let input = "package main\n\nimport (\n\t\"fmt\"\n\tio \"io\"\n\t. \"strings\"\n)\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn var_decl_parses() {
    let input = "package p\n\nvar (\n\tx int\n\ty int = 1\n\tz = 2\n)\n\nvar a, b = 1, 2\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn const_iota_parses() {
    let input = "package p\n\nconst (\n\tA = iota\n\tB\n\tC\n)\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn type_struct_parses() {
    let input = "package p\n\ntype Point struct {\n\tX, Y int\n\tName string `json:\"name\"`\n}\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"type"), "expected Point @type: {:?}", k);
    assert!(k.contains(&"property"), "expected field @property: {:?}", k);
}

#[test]
fn type_interface_parses() {
    let input = "package p\n\ntype Reader interface {\n\tRead(p []byte) (n int, err error)\n\tClose() error\n}\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn method_decl_parses() {
    let input = "package p\n\nfunc (p *Point) Distance() float64 { return 0 }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn generic_func_parses() {
    let input = "package p\n\nfunc Map[T, U any](xs []T, f func(T) U) []U { return nil }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn for_range_parses() {
    let input = "package p\n\nfunc f() {\n\tfor i, v := range xs {\n\t\t_ = v\n\t\t_ = i\n\t}\n}\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn for_c_style_parses() {
    let input = "package p\n\nfunc f() {\n\tfor i := 0; i < 10; i++ {\n\t\tprintln(i)\n\t}\n}\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn switch_type_switch_parses() {
    let input = "package p\n\nfunc f(x interface{}) {\n\tswitch v := x.(type) {\n\tcase int:\n\t\t_ = v\n\tcase string:\n\t\t_ = v\n\tdefault:\n\t\t_ = v\n\t}\n}\n";
    let (caps, kinds) = assert_complete_full(input);
    // Top-level `*^` recovery would let this pass even when the body fails
    // to parse — explicitly assert the body landed under no recovery and
    // that the guard produced the expected `switch`, `type`, and `case`
    // keywords.
    let leaks = non_ws_recovery_literals(&caps, &kinds, input);
    assert!(
        leaks.is_empty(),
        "type-switch body should parse cleanly, recovered: {:?}",
        leaks
    );
    let keyword_count = |kw: &str| {
        caps.iter()
            .filter(|c| kinds[c.kind.0 as usize] == "keyword")
            .filter(|c| &input[c.start..c.end] == kw)
            .count()
    };
    assert_eq!(keyword_count("switch"), 1);
    assert_eq!(keyword_count("type"), 1, "expected `type` inside the guard");
    assert_eq!(keyword_count("case"), 2);
}

#[test]
fn type_switch_with_init_parses() {
    let input = "package p\n\nfunc f(x interface{}) {\n\tswitch y := lookup(); v := y.(type) {\n\tcase int:\n\t\t_ = v\n\tdefault:\n\t\t_ = v\n\t}\n}\n";
    let (caps, kinds) = assert_complete_full(input);
    let leaks = non_ws_recovery_literals(&caps, &kinds, input);
    assert!(
        leaks.is_empty(),
        "type-switch with init clause should parse cleanly, recovered: {:?}",
        leaks
    );
}

#[test]
fn type_switch_without_assignment_parses() {
    let input = "package p\n\nfunc f(x interface{}) {\n\tswitch x.(type) {\n\tcase int:\n\tdefault:\n\t}\n}\n";
    let (caps, kinds) = assert_complete_full(input);
    let leaks = non_ws_recovery_literals(&caps, &kinds, input);
    assert!(
        leaks.is_empty(),
        "anonymous type-switch should parse cleanly, recovered: {:?}",
        leaks
    );
}

#[test]
fn select_parses() {
    let input = "package p\n\nfunc f(ch chan int) {\n\tselect {\n\tcase v := <-ch:\n\t\t_ = v\n\tcase ch <- 1:\n\tdefault:\n\t}\n}\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn composite_literal_parses() {
    let input = "package p\n\nfunc f() {\n\tp := Point{X: 1, Y: 2}\n\ta := []int{1, 2, 3}\n\tm := map[string]int{\"a\": 1}\n}\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn closure_and_defer_parse() {
    let input = "package p\n\nfunc f() {\n\tdefer func() { recover() }()\n\tgo func(n int) { println(n) }(42)\n}\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn channel_types_parse() {
    let input = "package p\n\nfunc f(a chan int, b <-chan int, c chan<- int) {}\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn if_with_init_parses() {
    let input =
        "package p\n\nfunc f() {\n\tif v, err := parse(); err != nil {\n\t\t_ = v\n\t}\n}\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn slicing_parses() {
    let input = "package p\n\nfunc f(xs []int) {\n\t_ = xs[0]\n\t_ = xs[1:]\n\t_ = xs[:2]\n\t_ = xs[1:3]\n\t_ = xs[1:3:4]\n}\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn string_forms_parse() {
    let input = "package p\n\nvar s1 = \"hello\"\nvar s2 = `raw\nstring`\nvar r = 'a'\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"string"), "expected strings: {:?}", k);
}

#[test]
fn numeric_forms_parse() {
    let input = "package p\n\nvar a = 42\nvar b = 3.14\nvar c = 0xff\nvar d = 0b1010\nvar e = 0o77\nvar f = 1_000\nvar g = 1e10\nvar h = 1i\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"number"), "expected numbers: {:?}", k);
}

#[test]
fn line_and_block_comment_parse() {
    let input = "package p\n// line\n/* block */\nfunc f() {}\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"comment"), "expected comments: {:?}", k);
}

#[test]
fn operator_longest_match_disambiguates_lr_cascade() {
    // Pins lookahead-class pitfalls in the LR-cascade port:
    //   - shift vs comparison      (`<<` is shift, not `<` + `<`)
    //   - logical vs bitwise       (`&&` is land, not `&` + `&`; same for `||` vs `|`)
    //   - `&^` (and-not) vs `&`    (`&^` matched as one token at expr_mul level)
    //   - compound-assign prefixes (`<<=`, `+=`, `&=` etc. don't get half-eaten)
    let inputs = [
        "package p\nfunc f() { var a int = 1; a <<= 2; a >>= 3; a += 4 }\n",
        "package p\nfunc f(a, b int) bool { return a < 1 << b }\n",
        "package p\nfunc f(a, b, c bool) bool { return a && b || c }\n",
        "package p\nfunc f(a, b uint) uint { return a &^ b }\n",
    ];
    for input in &inputs {
        let (caps, kinds) = assert_complete_full(input);
        let k = kinds_for(&caps, &kinds);
        assert!(k.contains(&"operator"), "expected operators in {:?}", input);
    }
}

#[test]
fn recovery_absorbs_malformed_item() {
    let input = "package p\n\nfunc a() {}\n@@@ garbage @@@\nfunc b() {}\n";
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
    let input = "package main\nfunc main() { x := 1; @@@; }\n";
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

#[test]
fn blank_lines_between_top_decls_emit_no_recovery() {
    // Regression: file-root `(top_decl)*^` used to drop inter-iteration ws,
    // sending blank-line bytes through the recovery byte-eater. See #71.
    let input = "package main\n\nfunc a() {}\n\nfunc b() {}\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(
        !k.contains(&"recovery"),
        "well-formed input should emit no @recovery captures, got: {:?}",
        k
    );
}

// --- Stage 2 UTF-8 tests ----------------------------------------------

#[test]
fn non_ascii_identifier_in_short_var_decl() {
    // Per Go spec "Identifiers": letter = unicode_letter | "_"; both
    // start and continuation admit code points classified as Letter.
    let input = "package main\n\nfunc f() { 世界 := 1; _ = 世界 }\n";
    let (caps, kinds) = assert_complete_full(input);
    let var_spans: Vec<&str> = caps
        .iter()
        .filter(|c| kinds[c.kind.0 as usize] == "variable")
        .map(|c| &input[c.start..c.end])
        .collect();
    assert!(
        var_spans.contains(&"世界"),
        "expected non-ASCII identifier `世界` as variable; got {var_spans:?}"
    );
}
