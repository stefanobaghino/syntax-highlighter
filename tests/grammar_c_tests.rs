use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use syntax_highlighter::pegc;
use syntax_highlighter::pegvm::{Capture, Program, VM};

const C_GRAMMAR: &str = include_str!("../grammars/c.peg");

// Compile the grammar exactly once per test process. The init lock's
// poisoning is what concentrates a broken-grammar failure into one
// informative panic plus N-1 trivial "previously failed" panics across
// the rest of the file's tests.
static C_PROGRAM: OnceLock<Program> = OnceLock::new();
static C_INIT: Mutex<()> = Mutex::new(());

fn c_program() -> &'static Program {
    if let Some(p) = C_PROGRAM.get() {
        return p;
    }
    let _guard = C_INIT
        .lock()
        .expect("C grammar compile previously failed; see the first failure");
    C_PROGRAM.get_or_init(|| pegc::compile(C_GRAMMAR).expect("C grammar should compile"))
}

#[test]
fn grammar_compiles() {
    let _ = c_program();
}

fn run(input: &str) -> (usize, Vec<Capture>, Vec<String>, bool) {
    let prog = c_program();
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

#[test]
fn grammar_parses_and_compiles() {
    let prog = c_program();
    assert!(!prog.code.is_empty());
    assert!(!prog.capture_kinds.is_empty());
}

#[test]
fn capture_kinds_stay_in_theme_vocabulary() {
    let prog = c_program();
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
fn hello_world_parses() {
    let input =
        "#include <stdio.h>\n\nint main(void) {\n\tprintf(\"hello, world\\n\");\n\treturn 0;\n}\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(
        !k.contains(&"comment"),
        "no real comments in input — `#include` should not be tagged as comment: {:?}",
        k
    );
    assert!(k.contains(&"keyword"), "expected keywords: {:?}", k);
    assert!(k.contains(&"type"), "expected int as type: {:?}", k);
}

#[test]
fn preprocessor_directives_parse() {
    let input = "#define FOO 1\n#ifdef FOO\n#include \"x.h\"\n#endif\nint x = 0;\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"keyword"), "expected PP as keywords: {:?}", k);
}

#[test]
fn preprocessor_line_continuation_parses() {
    let input = "#define LONG \\\n    foo \\\n    bar\nint x;\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn struct_decl_parses() {
    let input = "struct Point { int x; int y; };\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"keyword"), "expected struct keyword: {:?}", k);
    assert!(k.contains(&"type"), "expected Point as type: {:?}", k);
    assert!(k.contains(&"property"), "expected x/y as property: {:?}", k);
}

#[test]
fn enum_decl_parses() {
    let input = "enum Colors { RED = 0, GREEN, BLUE };\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(
        k.contains(&"constant"),
        "enum items should be @constant: {:?}",
        k
    );
}

#[test]
fn typedef_parses() {
    let input = "typedef unsigned long size_t;\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn function_decl_parses() {
    let input = "int add(int a, int b) { return a + b; }\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"function"), "expected add @function: {:?}", k);
}

#[test]
fn variadic_function_parses() {
    let input = "int printf(const char *fmt, ...);\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn pointer_types_parse() {
    let input = "int *p; const char * const s = \"hi\"; int **pp;\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn array_types_parse() {
    let input = "int xs[10]; char buf[64]; int matrix[3][4];\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn for_loop_parses() {
    let input = "int main(void) {\n\tfor (int i = 0; i < 10; i++) { printf(\"%d\\n\", i); }\n\treturn 0;\n}\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn switch_case_parses() {
    let input = "int f(int x) {\n\tswitch (x) {\n\t\tcase 1: return 10;\n\t\tcase 2: return 20;\n\t\tdefault: return 0;\n\t}\n}\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn sizeof_parses() {
    let input = "int main(void) { int a = sizeof(int); int b = sizeof x; return a + b; }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn cast_parses() {
    let input = "int main(void) { int x = (int)3.14; return x; }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn string_concat_parses() {
    let input = "const char *s = \"hello, \" \"world\";\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn number_literals_parse() {
    let input =
        "int a = 42; long b = 0xdeadBEEFul; int c = 0b1010; float d = 3.14f; double e = 1.5e10;\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"number"), "expected numbers: {:?}", k);
}

#[test]
fn string_and_char_parse() {
    let input = "const char c = 'a'; const char *s = \"hi\";\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"string"), "expected strings: {:?}", k);
}

#[test]
fn line_and_block_comments_parse() {
    let input = "// line\n/* block */\nint x = 0;\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"comment"), "expected comments: {:?}", k);
}

#[test]
fn struct_access_parses() {
    let input = "int main(void) { struct Point p = {0, 0}; int x = p.x; int *pp = &p; int y = pp->x; return x + y; }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn designated_initializer_parses() {
    let input = "struct P { int x; int y; }; struct P p = { .x = 1, .y = 2 };\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn operator_longest_match_disambiguates_lr_cascade() {
    // Pins the three lookahead-class pitfalls in the LR-cascade port:
    //   - compound-assign vs same-prefix binary  (`<<=` is one token, not `<<` + `=`)
    //   - two-char vs one-char relational        (`<<` is shift, not `<` + `<`)
    //   - logical-vs-bitwise                     (`&&` is land, not `&` + `&`)
    let inputs = [
        "int main(void) { int a = 1; a <<= 2; a >>= 3; a += 4; return a; }\n",
        "int main(void) { int a = 1, b = 2; if (a < 1 << b) return 1; return 0; }\n",
        "int main(void) { int a = 1, b = 2, c = 3; if (a & b && c) return 1; return 0; }\n",
    ];
    for input in &inputs {
        let (caps, kinds) = assert_complete_full(input);
        let k = kinds_for(&caps, &kinds);
        assert!(k.contains(&"operator"), "expected operators in {:?}", input);
    }
}

fn kind_at_pos<'a>(captures: &[Capture], kinds: &'a [String], pos: usize) -> Option<&'a str> {
    captures
        .iter()
        .find(|c| c.start == pos)
        .map(|c| kinds[c.kind.0 as usize].as_str())
}

#[test]
fn t_suffix_typedef_resolves_as_type() {
    // Regression: PEG `*` is possessive, so the old
    // `[a-z_] ident_body* '_t' !ident_body` rule never matched anything
    // (the greedy `ident_body*` swallowed the trailing `_t`). That caused
    // identifiers like `counter_size_t` to fail `decl_spec` entirely.
    let input = "counter_size_t x;\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(!k.contains(&"recovery"), "no recovery expected: {:?}", k);
    let type_pos = input.find("counter_size_t").unwrap();
    let var_pos = input.find('x').unwrap();
    assert_eq!(kind_at_pos(&caps, &kinds, type_pos), Some("type"));
    assert_eq!(kind_at_pos(&caps, &kinds, var_pos), Some("variable"));
}

#[test]
fn sizeof_struct_type_name_parses() {
    // Regression: the old `expr_unary` consumed `sizeof` as a unary
    // prefix, so `sizeof(struct Foo)` then had to parse via
    // `cast_or_paren` — which fails because `struct Foo` isn't an
    // `expr_primary`. Fixed by trying `sizeof_type` first in `expr_unary`.
    let input = "int f(void) { return sizeof(struct Foo); }\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(!k.contains(&"recovery"), "no recovery expected: {:?}", k);
    let sizeof_pos = input.find("sizeof").unwrap();
    let struct_pos = input.find("struct").unwrap();
    let foo_pos = input.find("Foo").unwrap();
    assert_eq!(kind_at_pos(&caps, &kinds, sizeof_pos), Some("keyword"));
    assert_eq!(kind_at_pos(&caps, &kinds, struct_pos), Some("keyword"));
    assert_eq!(kind_at_pos(&caps, &kinds, foo_pos), Some("type"));
}

#[test]
fn medium_fixture_parses_without_recovery() {
    let input = include_str!("../benches/fixtures/medium.c");
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(
        !k.contains(&"recovery"),
        "medium.c should parse without recovery, got kinds: {:?}",
        k
    );
    let function_literals: HashSet<&str> = caps
        .iter()
        .filter(|c| kinds[c.kind.0 as usize] == "function")
        .map(|c| &input[c.start..c.end])
        .collect();
    for name in ["counter_new", "counter_push", "classify", "sum", "main"] {
        assert!(
            function_literals.contains(name),
            "expected `{}` as @function in medium.c captures, got {:?}",
            name,
            function_literals
        );
    }
}

#[test]
fn recovery_absorbs_malformed_item() {
    let input = "int a() {}\n@@@ garbage @@@\nint b() {}\n";
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
    // `compound_stmt`'s `^block_close` catch resyncs at the closing `}`
    // when the body contains garbage before the brace.
    let input = "int main(void) { x = 1; @@@; return 0; }\n";
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
