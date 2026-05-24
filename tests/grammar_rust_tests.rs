use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use syntax_highlighter::pegc;
use syntax_highlighter::pegvm::{Capture, Program, VM};

const RUST_GRAMMAR: &str = include_str!("../grammars/rust.peg");

// Compile the grammar exactly once per test process. The init lock's
// poisoning is what concentrates a broken-grammar failure into one
// informative panic plus N-1 trivial "previously failed" panics across
// the rest of the file's tests.
static RUST_PROGRAM: OnceLock<Program> = OnceLock::new();
static RUST_INIT: Mutex<()> = Mutex::new(());

fn rust_program() -> &'static Program {
    if let Some(p) = RUST_PROGRAM.get() {
        return p;
    }
    let _guard = RUST_INIT
        .lock()
        .expect("Rust grammar compile previously failed; see the first failure");
    RUST_PROGRAM.get_or_init(|| pegc::compile(RUST_GRAMMAR).expect("Rust grammar should compile"))
}

#[test]
fn grammar_compiles() {
    let _ = rust_program();
}

fn run(input: &str) -> (usize, Vec<Capture>, Vec<String>, bool) {
    let prog = rust_program();
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

fn kind_spans(captures: &[Capture], kinds: &[String], kind: &str) -> Vec<String> {
    captures
        .iter()
        .filter(|c| kinds[c.kind.0 as usize] == kind)
        .map(|c| format!("{}..{}", c.start, c.end))
        .collect()
}

fn recovery_literals<'a>(captures: &[Capture], kinds: &[String], input: &'a str) -> Vec<&'a str> {
    captures
        .iter()
        .filter(|c| kinds[c.kind.0 as usize] == "recovery")
        .map(|c| &input[c.start..c.end])
        .collect()
}

#[test]
fn grammar_parses_and_compiles() {
    let prog = rust_program();
    assert!(!prog.code.is_empty());
    assert!(!prog.capture_kinds.is_empty());
}

#[test]
fn capture_kinds_stay_in_theme_vocabulary() {
    // The grammar must not invent capture names outside the hardcoded
    // theme vocabulary in src/highlight/theme.rs.
    let prog = rust_program();
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
    assert!(
        k.contains(&"function"),
        "expected main as function: {:?}",
        k
    );
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
    assert!(
        kv.contains(&"property"),
        "field `a` should be @property: {:?}",
        kv
    );
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
fn closure_with_tuple_destructure_param_parses() {
    // Regression: `pattern_or`'s greedy `*` used to swallow the closing
    // `|` of `expr_closure` when the param was an or-eligible
    // `pattern_atom` (`(_, &n)` here), dropping the body and the rest
    // of the enclosing item into recovery. `closure_param` now uses
    // `pattern_no_or` so the delimiter stays free.
    let input = "fn f() { let top = max_by_key(|(_, &n)| n); }\n";
    let (caps, kinds) = assert_complete_full(input);
    let kv = kinds_for(&caps, &kinds);
    assert!(
        !kv.contains(&"recovery"),
        "closure with tuple destructuring should not recover: {:?}",
        kv
    );
}

#[test]
fn method_chain_with_turbofish_parses() {
    let input = "fn f() { v.iter().map(|x| x + 1).collect::<Vec<_>>(); }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn match_with_guards_and_ranges_parses() {
    let input =
        "fn f(n: i32) -> i32 { match n { 0 => 1, 1..=10 => 2, x if x > 10 => 3, _ => 0, } }\n";
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
    assert!(
        recovery_literals(&caps, &kinds, input).is_empty(),
        "no recovery expected for single-char lifetime"
    );
}

#[test]
fn multi_char_lifetime_parses() {
    // `'static` and other multi-char lifetimes must reach the lifetime
    // rule's full ident_body sequence — see #81.
    for input in [
        "fn k() -> &'static str { \"x\" }\n",
        "fn k<'lt>(s: &'lt str) -> &'lt str { s }\n",
        "fn k<'long_name>(x: &'long_name u8) {}\n",
        "fn f<T>(x: T) where T: Clone + Send + 'static {}\n",
        "fn f() -> Vec<(i64, &'static str)> { vec![] }\n",
    ] {
        let (caps, kinds) = assert_complete_full(input);
        let recs = recovery_literals(&caps, &kinds, input);
        assert!(
            recs.is_empty(),
            "expected no recovery on `{}` — got {:?}",
            input.trim_end(),
            recs
        );
    }
}

#[test]
fn where_clause_parses() {
    let input = "fn f<T>(x: T) where T: Clone + Send + 'static {}\n";
    let (caps, kinds) = assert_complete_full(input);
    assert!(
        recovery_literals(&caps, &kinds, input).is_empty(),
        "where-clause with `'static` should not recover"
    );
}

#[test]
fn fn_trait_paren_sugar_parses() {
    // `Fn(T) -> U` / `FnMut(T)` / `FnOnce(T) -> U` are paren-sugar for
    // the Fn-trait family — see #82. `type_path_seg`'s tail must accept
    // both angle-bracket generics and the parenthesized form.
    for input in [
        "fn with_each<F: FnMut(i64)>(f: F) {}\n",
        "fn map_fn<F: Fn(i64) -> i64>(f: F) {}\n",
        "fn consume<F: FnOnce()>(f: F) {}\n",
        "fn boxed(f: Box<dyn Fn(i64) -> i64>) {}\n",
        "fn dual<F: Fn(i64) -> i64 + Send>(f: F) {}\n",
    ] {
        let (caps, kinds) = assert_complete_full(input);
        let recs = recovery_literals(&caps, &kinds, input);
        assert!(
            recs.is_empty(),
            "expected no recovery on `{}` — got {:?}",
            input.trim_end(),
            recs
        );
    }
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
fn recovery_absorbs_malformed_block_body() {
    // `block`'s `^block_close` catch resyncs at the closing `}` when
    // the body contains garbage before the brace. The enclosing
    // `fn_item` still completes — without the catch, the malformed
    // body would fail `fn_item` and fall through to top-level
    // byte-by-byte recovery, fragmenting the output.
    let input = "fn main() { let x = 1; @@@; let y = 2; }\n";
    let (matched, caps, kinds, complete) = run(input);
    assert!(complete, "block_close catch should keep parse complete");
    assert_eq!(matched, input.len());
    let kv = kinds_for(&caps, &kinds);
    assert!(
        kv.contains(&"recovery"),
        "expected a @recovery capture inside the block, got: {:?}",
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

#[test]
fn operator_longest_match_disambiguates_lr_cascade() {
    // Pins lookahead-class pitfalls in the LR-cascade port:
    //   - compound-assign vs binary  (`<<=`, `+=`, `&=` all stay one token)
    //   - shift vs comparison        (`<<` is shift, not `<` + `<`)
    //   - logical vs bitwise         (`&&` is land, not `&` + `&`; `||` vs `|`)
    //   - assignment vs equality     (`=` !'=' so `==` doesn't get half-eaten)
    let inputs = [
        "fn f() { let mut a = 1; a <<= 2; a >>= 3; a += 4; a &= 5; }\n",
        "fn f(a: u32, b: u32) -> bool { a < 1 << b }\n",
        "fn f(a: bool, b: u32, c: u32) -> bool { a && b & c == 0 }\n",
    ];
    for input in &inputs {
        let (caps, kinds) = assert_complete_full(input);
        let k = kinds_for(&caps, &kinds);
        assert!(k.contains(&"operator"), "expected operators in {:?}", input);
    }
}
