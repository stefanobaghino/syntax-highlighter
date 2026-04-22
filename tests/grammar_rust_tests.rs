use std::collections::HashSet;

use syntax_highlighter::pegc;
use syntax_highlighter::pegvm::{Capture, Program, VM};

const RUST_GRAMMAR: &str = include_str!("../grammars/rust.peg");

fn compile_rust() -> Program {
    pegc::compile(RUST_GRAMMAR).expect("Rust grammar should compile")
}

fn run(input: &str) -> (usize, Vec<Capture>, Vec<String>, bool) {
    let prog = compile_rust();
    let r = VM::new(&prog.code, input.as_bytes()).run();
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

fn kind_spans(captures: &[Capture], kinds: &[String], kind: &str) -> Vec<String> {
    captures
        .iter()
        .filter(|c| kinds[c.kind.0 as usize] == kind)
        .map(|c| format!("{}..{}", c.start, c.end))
        .collect()
}

#[test]
fn grammar_parses_and_compiles() {
    let prog = compile_rust();
    assert!(!prog.code.is_empty());
    assert!(!prog.capture_kinds.is_empty());
}

#[test]
fn capture_kinds_stay_in_theme_vocabulary() {
    // The grammar must not invent capture names outside the hardcoded
    // theme vocabulary in src/highlight/theme.rs.
    let prog = compile_rust();
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
fn only_whitespace_matches() {
    let (caps, _) = assert_complete_full("   \n\t\n");
    assert!(caps.is_empty());
}

#[test]
fn simple_fn_parses() {
    let (caps, kinds) = assert_complete_full("fn main() {}\n");
    // `fn` keyword + `main` function + two punctuation groups for
    // parens and braces (order and punctuation count don't matter —
    // we just want `fn` and `main` both flagged).
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"keyword"), "expected `fn` as keyword: {:?}", k);
    assert!(k.contains(&"function"), "expected main as function: {:?}", k);
}

#[test]
fn use_item_segments_are_typed() {
    // `std::collections::HashMap` — upper-case segment = @type,
    // lower-case segment = @variable.
    let input = "use std::collections::HashMap;\n";
    let (caps, kinds) = assert_complete_full(input);
    let kv = kinds_for(&caps, &kinds);
    assert!(kv.contains(&"keyword"), "missing `use` keyword: {:?}", kv);
    assert!(kv.contains(&"type"), "HashMap should be @type: {:?}", kv);
    assert!(
        kv.contains(&"variable"),
        "std/collections should be @variable: {:?}",
        kv
    );
}

#[test]
fn struct_field_is_property() {
    // `struct Foo { a: i32 }` — `a` is a @property, `Foo` and `i32`
    // are @type.
    let (caps, kinds) = assert_complete_full("struct Foo { a: i32 }\n");
    let kv = kinds_for(&caps, &kinds);
    assert!(kv.contains(&"property"), "field `a` should be @property: {:?}", kv);
    assert!(kv.contains(&"type"), "Foo/i32 should be @type: {:?}", kv);
}

#[test]
fn turbofish_parses() {
    let input = "fn f() { let x = parse::<i32>(\"42\"); }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn closure_disambiguates_from_bitor() {
    // Closure body uses `|` for params (not bitor).
    let input = "fn f() { let add = |a: i32, b: i32| -> i32 { a + b }; }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn bitor_still_works_outside_closure_position() {
    let input = "fn f() -> u32 { 0b1100 | 0b0011 }\n";
    let (caps, kinds) = assert_complete_full(input);
    let ops = kind_spans(&caps, &kinds, "operator");
    assert!(
        ops.iter().any(|s| s.contains("..")),
        "expected at least one operator span: {:?}",
        ops
    );
}

#[test]
fn method_chain_with_turbofish_parses() {
    let input = "fn f() { v.iter().map(|x| x + 1).collect::<Vec<_>>(); }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn match_with_guards_and_ranges_parses() {
    let input = "fn f(n: i32) -> i32 { match n { 0 => 1, 1..=10 => 2, x if x > 10 => 3, _ => 0, } }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn lifetime_parses_as_type() {
    let input = "fn f<'a>(s: &'a str) -> &'a str { s }\n";
    let (caps, kinds) = assert_complete_full(input);
    let kv = kinds_for(&caps, &kinds);
    assert!(
        kv.contains(&"type"),
        "lifetime `'a` should be captured as @type: {:?}",
        kv
    );
}

#[test]
fn where_clause_parses() {
    let input = "fn f<T>(x: T) where T: Clone + Send + 'static {}\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn impl_with_generic_params_parses() {
    let input = "impl<T: Clone> Foo<T> { pub fn new() -> Self { Self { a: T::default() } } }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn attribute_parses_before_item() {
    let input = "#[derive(Debug, Clone)]\npub struct Foo;\n";
    let (caps, kinds) = assert_complete_full(input);
    let kv = kinds_for(&caps, &kinds);
    assert!(kv.contains(&"keyword"), "missing keyword: {:?}", kv);
    assert!(kv.contains(&"type"), "missing @type Foo: {:?}", kv);
}

#[test]
fn inner_attribute_parses() {
    let input = "#![allow(dead_code)]\nfn main() {}\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn raw_string_parses() {
    // Raw strings with hash fences. `r#"..."#` contains `"` freely.
    let input = "fn f() { let s = r#\"he said \"hi\" to her\"#; }\n";
    let (caps, kinds) = assert_complete_full(input);
    let kv = kinds_for(&caps, &kinds);
    assert!(kv.contains(&"string"), "raw string missing: {:?}", kv);
}

#[test]
fn macro_invocation_function_color() {
    let input = "fn main() { println!(\"hi\"); }\n";
    let (caps, kinds) = assert_complete_full(input);
    let fns = kind_spans(&caps, &kinds, "function");
    // Both `main` and `println` should be @function.
    assert!(
        fns.len() >= 2,
        "expected both `main` and `println` as @function, got: {:?}",
        fns
    );
}

#[test]
fn recovery_absorbs_malformed_item() {
    // Top-level `*^` recovery keeps the surrounding items parsed even
    // when a middle item is garbage. `@@@` is unparseable at item
    // position (no rule begins with it) so the outer `*^` catches it.
    let input = "fn a() {}\n@@@ garbage @@@\nfn b() {}\n";
    let (matched, caps, kinds, complete) = run(input);
    assert!(complete, "recovery should keep parse complete");
    assert_eq!(matched, input.len());
    let kv = kinds_for(&caps, &kinds);
    assert!(
        kv.contains(&"recovery"),
        "expected a @recovery capture for the @@@ region, got: {:?}",
        kv
    );
}

#[test]
fn trailing_comma_tolerated_in_struct_fields() {
    let input = "struct Foo { a: i32, b: u32, }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn nested_generics_parse() {
    let input = "fn f() -> Option<Result<Vec<u8>, String>> { None }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn as_cast_parses() {
    let input = "fn f() { let y = (x as u32) + (z as u64); }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn numeric_literal_with_suffix_parses() {
    let input = "fn f() { let a = 42u32; let b = 3.14f64; let c = 0xdeadBEEFu64; }\n";
    let (caps, kinds) = assert_complete_full(input);
    let kv = kinds_for(&caps, &kinds);
    assert!(kv.contains(&"number"), "expected @number: {:?}", kv);
}

#[test]
fn block_comment_parses() {
    let input = "/* a block */ fn f() {}\n";
    let (caps, kinds) = assert_complete_full(input);
    let kv = kinds_for(&caps, &kinds);
    assert!(kv.contains(&"comment"), "expected @comment: {:?}", kv);
}

#[test]
fn line_comment_parses() {
    let input = "// a line\nfn f() {}\n";
    let (caps, kinds) = assert_complete_full(input);
    let kv = kinds_for(&caps, &kinds);
    assert!(kv.contains(&"comment"), "expected @comment: {:?}", kv);
}

#[test]
fn raw_identifier_parses() {
    let input = "fn r#match() {}\n";
    let (_, _) = assert_complete_full(input);
}
