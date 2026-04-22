use std::collections::HashSet;

use syntax_highlighter::pegc;
use syntax_highlighter::pegvm::{Capture, Program, VM};

const JS_GRAMMAR: &str = include_str!("../grammars/javascript.peg");

fn compile_js() -> Program {
    pegc::compile(JS_GRAMMAR).expect("JavaScript grammar should compile")
}

fn run(input: &str) -> (usize, Vec<Capture>, Vec<String>, bool) {
    let prog = compile_js();
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
    let prog = compile_js();
    assert!(!prog.code.is_empty());
    assert!(!prog.capture_kinds.is_empty());
}

#[test]
fn capture_kinds_stay_in_theme_vocabulary() {
    let prog = compile_js();
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
fn labeled_statement_and_break_parse() {
    let (_, _) =
        assert_complete_full("outer: for (let i = 0; i < 10; i++) { if (i === 5) break outer; }\n");
}
