use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use syntax_highlighter::pegc;
use syntax_highlighter::pegvm::{Capture, Program, VM};

const JS_GRAMMAR: &str = include_str!("../grammars/javascript.peg");

// Compile the grammar exactly once per test process. The init lock's
// poisoning is what concentrates a broken-grammar failure into one
// informative panic plus N-1 trivial "previously failed" panics across
// the rest of the file's tests.
static JS_PROGRAM: OnceLock<Program> = OnceLock::new();
static JS_INIT: Mutex<()> = Mutex::new(());

fn js_program() -> &'static Program {
    if let Some(p) = JS_PROGRAM.get() {
        return p;
    }
    let _guard = JS_INIT
        .lock()
        .expect("JavaScript grammar compile previously failed; see the first failure");
    JS_PROGRAM.get_or_init(|| pegc::compile(JS_GRAMMAR).expect("JavaScript grammar should compile"))
}

#[test]
fn grammar_compiles() {
    let _ = js_program();
}

fn run(input: &str) -> (usize, Vec<Capture>, Vec<String>, bool) {
    let prog = js_program();
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
    let prog = js_program();
    assert!(!prog.code.is_empty());
    assert!(!prog.capture_kinds.is_empty());
}

#[test]
fn capture_kinds_stay_in_theme_vocabulary() {
    let prog = js_program();
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
fn simple_function_decl_parses() {
    let (caps, kinds) = assert_complete_full("function main() {}\n");
    let k = kinds_for(&caps, &kinds);
    assert!(
        k.contains(&"keyword"),
        "expected `function` as keyword: {:?}",
        k
    );
    assert!(
        k.contains(&"function"),
        "expected main as function: {:?}",
        k
    );
}

#[test]
fn arrow_fn_with_paren_params_parses() {
    let input = "const add = (a, b) => a + b;\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"keyword"), "expected `const` keyword: {:?}", k);
    assert!(
        k.contains(&"operator"),
        "expected `=>` operator capture: {:?}",
        k
    );
}

#[test]
fn arrow_fn_with_single_ident_param_parses() {
    let (_, _) = assert_complete_full("const sq = x => x * x;\n");
}

#[test]
fn parenthesized_expr_without_arrow_parses() {
    // `(a, b)` not followed by `=>` — must still parse via expression
    // path after arrow backtracks.
    let (_, _) = assert_complete_full("const x = (1, 2);\n");
}

#[test]
fn template_literal_parses_with_interp() {
    let input = "const s = `hello ${name}, age ${age + 1}`;\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(
        k.contains(&"string"),
        "expected @string for template: {:?}",
        k
    );
}

#[test]
fn tagged_template_parses() {
    let (_, _) = assert_complete_full("const s = html`<div>${x}</div>`;\n");
}

#[test]
fn class_with_methods_parses() {
    let input = "class Foo extends Bar { constructor(x) { super(); this.x = x; } get x() { return this._x; } static make() { return new Foo(0); } }\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"keyword"), "expected class/extends: {:?}", k);
    assert!(k.contains(&"type"), "expected Foo/Bar @type: {:?}", k);
}

#[test]
fn class_field_parses() {
    let (_, _) = assert_complete_full("class Foo { x = 1; static y = 2; }\n");
}

#[test]
fn destructuring_object_parses() {
    let (_, _) = assert_complete_full("const { a, b: c, d = 1, ...rest } = obj;\n");
}

#[test]
fn destructuring_array_parses() {
    let (_, _) = assert_complete_full("const [a, , b = 1, ...rest] = arr;\n");
}

#[test]
fn object_literal_with_shorthand_and_computed_parses() {
    let input = "const o = { a, b: 1, [key]: value, get x() { return 1; }, method() {} };\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn spread_in_call_and_array_parses() {
    let (_, _) = assert_complete_full("const a = [1, ...b, 3]; f(...a);\n");
}

#[test]
fn for_of_and_for_in_parse() {
    let (_, _) =
        assert_complete_full("for (const x of arr) { log(x); }\nfor (let k in obj) { log(k); }\n");
}

#[test]
fn for_c_style_parses() {
    let (_, _) = assert_complete_full("for (let i = 0; i < 10; i++) { log(i); }\n");
}

#[test]
fn optional_chaining_parses() {
    let (_, _) = assert_complete_full("const x = a?.b?.c?.();\n");
}

#[test]
fn nullish_coalescing_parses() {
    let (_, _) = assert_complete_full("const x = a ?? b ?? c;\n");
}

#[test]
fn try_catch_finally_parses() {
    let (_, _) =
        assert_complete_full("try { f(); } catch (e) { log(e); } finally { cleanup(); }\n");
}

#[test]
fn switch_case_parses() {
    let input = "switch (x) { case 1: a(); break; case 2: b(); break; default: c(); }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn import_named_parses() {
    let input = "import { a, b as c } from 'mod';\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(
        k.contains(&"keyword"),
        "expected import/from keywords: {:?}",
        k
    );
    assert!(k.contains(&"string"), "expected string for module: {:?}", k);
}

#[test]
fn import_default_and_namespace_parse() {
    let (_, _) = assert_complete_full("import def from 'd';\n");
    let (_, _) = assert_complete_full("import * as M from 'm';\n");
    let (_, _) = assert_complete_full("import def, { a } from 'm';\n");
    let (_, _) = assert_complete_full("import def, * as M from 'm';\n");
}

#[test]
fn export_forms_parse() {
    let (_, _) = assert_complete_full("export const x = 1;\n");
    let (_, _) = assert_complete_full("export function f() {}\n");
    let (_, _) = assert_complete_full("export class C {}\n");
    let (_, _) = assert_complete_full("export default 42;\n");
    let (_, _) = assert_complete_full("export default function () {}\n");
    let (_, _) = assert_complete_full("export { a, b as c };\n");
    let (_, _) = assert_complete_full("export * from 'm';\n");
}

#[test]
fn new_expression_parses() {
    let input = "const x = new Foo(1, 2);\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(
        k.contains(&"type"),
        "expected Foo as @type after new: {:?}",
        k
    );
}

#[test]
fn typeof_instanceof_parse() {
    let (_, _) =
        assert_complete_full("const t = typeof x === 'string'; const y = obj instanceof Foo;\n");
}

#[test]
fn async_await_parse() {
    let input = "async function f() { const x = await g(); return x; }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn generator_and_yield_parse() {
    let input = "function* gen() { yield 1; yield* other; }\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn regex_like_division_parses() {
    // Without regex support, `/` is always division. Must not choke.
    let (_, _) = assert_complete_full("const r = a / b / c;\n");
}

#[test]
fn numeric_literals_parse() {
    let input = "const a = 42; const b = 3.14; const c = 0xdeadBEEF; const d = 0b1010; const e = 0o777; const f = 1_000_000; const g = 1n; const h = .5; const i = 5e10;\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"number"), "expected @number: {:?}", k);
}

#[test]
fn line_comment_parses() {
    let input = "// hello\nconst x = 1;\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"comment"), "expected @comment: {:?}", k);
}

#[test]
fn block_comment_parses() {
    let input = "/* block */ const x = 1;\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(k.contains(&"comment"), "expected @comment: {:?}", k);
}

#[test]
fn recovery_absorbs_malformed_item() {
    // `@@@` has no valid interpretation at statement position, so the
    // top-level `*^` recovery catches it.
    let input = "function a() {}\n@@@ garbage @@@\nfunction b() {}\n";
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
    // the body contains garbage before the brace.
    let input = "function f() { var x = 1; @@@; }\n";
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
fn blank_lines_between_top_stmts_emit_no_recovery() {
    // Regression: file-root `(stmt)*^` used to drop inter-iteration ws,
    // sending blank-line bytes through the recovery byte-eater. See #71.
    let input = "function a() {}\n\nfunction b() {}\n";
    let (caps, kinds) = assert_complete_full(input);
    let kv = kinds_for(&caps, &kinds);
    assert!(
        !kv.contains(&"recovery"),
        "well-formed input should emit no @recovery captures, got: {:?}",
        kv
    );
}

#[test]
fn method_chain_with_call_parses() {
    let input = "const r = arr.filter(x => x > 0).map(x => x * 2).reduce((a, b) => a + b, 0);\n";
    let (caps, kinds) = assert_complete_full(input);
    let fns = kind_spans(&caps, &kinds, "function");
    assert!(
        !fns.is_empty(),
        "expected chained method names as @function: {:?}",
        fns
    );
}

#[test]
fn assignment_operators_parse() {
    let input = "x = 1; x += 1; x -= 1; x *= 2; x /= 2; x **= 2; x &&= y; x ||= y; x ??= y;\n";
    let (_, _) = assert_complete_full(input);
}

#[test]
fn ternary_parses() {
    let (_, _) = assert_complete_full("const x = a ? b : c;\n");
}

#[test]
fn operator_longest_match_disambiguates_lr_cascade() {
    // Pins the lookahead-class pitfalls in the LR-cascade port:
    //   - compound-assign vs same-prefix binary  (`<<=` is one token, not `<<` + `=`)
    //   - two-char vs one-char relational        (`<<` is shift, not `<` + `<`)
    //   - logical-vs-bitwise                     (`&&` is land, not `&` + `&`)
    //   - strict-equality vs equality            (`===` before `==` in the alt)
    //   - exponent vs multiply                   (`**` is right-assoc pow, not `*` + `*`)
    let inputs = [
        "let a = 1; a <<= 2; a >>>= 3; a ??= 4;\n",
        "if (a < 1 << b) { let c = 1; }\n",
        "if (a & b && c) { let d = 1; }\n",
        "if (a === b !== c) { let d = 1; }\n",
        "let p = 2 ** 3 ** 2;\n",
    ];
    for input in &inputs {
        let (caps, kinds) = assert_complete_full(input);
        let k = kinds_for(&caps, &kinds);
        assert!(k.contains(&"operator"), "expected operators in {:?}", input);
    }
}

#[test]
fn labeled_statement_and_break_parse() {
    let (_, _) =
        assert_complete_full("outer: for (let i = 0; i < 10; i++) { if (i === 5) break outer; }\n");
}

// Pins the inter-statement-whitespace handling inside `block` and
// `switch_case`. Stmts whose syntactic tail is `}` (`if_stmt`,
// `for_stmt`, `while_stmt`, `block_stmt`, `try_stmt`, …) leave their
// trailing space unconsumed; the stmt-list rules must consume it
// between siblings, otherwise the enclosing `block` fails on the
// next stmt's keyword and the whole `fn_decl` collapses into
// recovery captures.

fn keyword_literals<'a>(input: &'a str, captures: &[Capture], kinds: &[String]) -> Vec<&'a str> {
    captures
        .iter()
        .filter(|c| kinds[c.kind.0 as usize] == "keyword")
        .map(|c| &input[c.start..c.end])
        .collect()
}

#[test]
fn block_with_if_then_return_keeps_function_keyword() {
    let input = "function f() { if (true) {} return; }\n";
    let (caps, kinds) = assert_complete_full(input);
    let kw = keyword_literals(input, &caps, &kinds);
    assert!(
        kw.contains(&"function") && kw.contains(&"if") && kw.contains(&"return"),
        "expected function/if/return as keywords, got: {:?}",
        kw
    );
}

#[test]
fn block_with_for_then_return_keeps_function_keyword() {
    let input = "function f() { for (let i=0;i<1;i++) {} return; }\n";
    let (caps, kinds) = assert_complete_full(input);
    let kw = keyword_literals(input, &caps, &kinds);
    assert!(
        kw.contains(&"function") && kw.contains(&"for") && kw.contains(&"return"),
        "expected function/for/return as keywords, got: {:?}",
        kw
    );
}

#[test]
fn block_with_while_then_return_keeps_function_keyword() {
    let input = "function f() { while (true) {} return; }\n";
    let (caps, kinds) = assert_complete_full(input);
    let kw = keyword_literals(input, &caps, &kinds);
    assert!(
        kw.contains(&"function") && kw.contains(&"while") && kw.contains(&"return"),
        "expected function/while/return as keywords, got: {:?}",
        kw
    );
}

#[test]
fn block_stmt_then_return_keeps_function_keyword() {
    let input = "function f() { {} return; }\n";
    let (caps, kinds) = assert_complete_full(input);
    let kw = keyword_literals(input, &caps, &kinds);
    assert!(
        kw.contains(&"function") && kw.contains(&"return"),
        "expected function/return as keywords, got: {:?}",
        kw
    );
}

#[test]
fn switch_case_with_block_tail_stmt_parses() {
    let input = "function f() { switch (x) { case 1: if (y) {} break; } }\n";
    let (caps, kinds) = assert_complete_full(input);
    let kw = keyword_literals(input, &caps, &kinds);
    assert!(
        kw.contains(&"function")
            && kw.contains(&"switch")
            && kw.contains(&"case")
            && kw.contains(&"if")
            && kw.contains(&"break"),
        "expected function/switch/case/if/break as keywords, got: {:?}",
        kw
    );
}

#[test]
fn try_then_return_keeps_function_keyword() {
    let input = "function f() { try {} catch (e) {} return; }\n";
    let (caps, kinds) = assert_complete_full(input);
    let kw = keyword_literals(input, &caps, &kinds);
    assert!(
        kw.contains(&"function")
            && kw.contains(&"try")
            && kw.contains(&"catch")
            && kw.contains(&"return"),
        "expected function/try/catch/return as keywords, got: {:?}",
        kw
    );
}

// Regression tests for #73: reserved words must be accepted at
// IdentifierName positions (member access, property keys, imported
// /exported names) where ES allows them but rejects only at binding
// positions.

#[test]
fn reserved_word_after_dot_parses() {
    let input = "main().catch(e => log(e));\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(
        !k.contains(&"recovery"),
        "no recovery expected for `.catch(...)`: {:?}",
        k
    );
}

#[test]
fn reserved_word_after_dot_without_call_is_property() {
    let input = "p.catch;\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(
        !k.contains(&"recovery"),
        "no recovery expected for `.catch;`: {:?}",
        k
    );
    let catch_pos = input.find("catch").unwrap();
    let catch_cap = caps.iter().find(|c| c.start == catch_pos).unwrap();
    assert_eq!(kinds[catch_cap.kind.0 as usize], "property");
}

#[test]
fn reserved_word_after_optional_chain_parses() {
    let input = "obj?.delete; obj?.return(); obj?.['x'];\n";
    let (caps, kinds) = assert_complete_full(input);
    let k = kinds_for(&caps, &kinds);
    assert!(
        !k.contains(&"recovery"),
        "no recovery expected for optional-chain `?.<reserved>`: {:?}",
        k
    );
}

#[test]
fn reserved_word_as_object_key_parses() {
    let (_, _) = assert_complete_full("const o = { catch: 1, default: 2, new: 3 };\n");
}

#[test]
fn reserved_word_as_object_method_shorthand_parses() {
    let (_, _) = assert_complete_full("const o = { catch() { return 1; }, default() {} };\n");
}

#[test]
fn reserved_word_as_class_method_and_field_parses() {
    let (_, _) = assert_complete_full("class C { catch() {} default = 1; static return() {} }\n");
}

#[test]
fn reserved_word_in_object_destructuring_key_parses() {
    let (_, _) = assert_complete_full("const { default: d, catch: c } = obj;\n");
}

#[test]
fn import_specifier_allows_reserved_imported_name() {
    let (_, _) = assert_complete_full("import { default as foo, class as Cls } from 'm';\n");
}

#[test]
fn export_specifier_allows_reserved_exported_name() {
    let (_, _) = assert_complete_full("export { foo as default, bar as class };\n");
}

#[test]
fn export_star_as_reserved_parses() {
    let (_, _) = assert_complete_full("export * as default from 'm';\n");
}
