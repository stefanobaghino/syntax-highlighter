use std::collections::HashSet;

use syntax_highlighter::pegc;
use syntax_highlighter::pegvm::{Capture, Program, VM};

const SQLITE_GRAMMAR: &str = include_str!("../grammars/sqlite.peg");

fn compile_sql() -> Program {
    pegc::compile(SQLITE_GRAMMAR).expect("SQLite grammar should compile")
}

fn run(input: &str) -> (usize, Vec<Capture>, Vec<String>, bool) {
    let prog = compile_sql();
    let r = VM::new(&prog.code, input.as_bytes()).run();
    (
        r.matched,
        r.captures,
        prog.capture_kinds.clone(),
        r.complete,
    )
}

fn kinds_for<'a>(captures: &[Capture], kinds: &'a [String]) -> Vec<&'a str> {
    captures
        .iter()
        .map(|c| kinds[c.kind.0 as usize].as_str())
        .collect()
}

fn spans_for<'a>(
    captures: &[Capture],
    kinds: &[String],
    kind: &str,
    input: &'a str,
) -> Vec<&'a str> {
    captures
        .iter()
        .filter(|c| kinds[c.kind.0 as usize] == kind)
        .map(|c| &input[c.start..c.end])
        .collect()
}

fn assert_complete_full(input: &str) -> (Vec<Capture>, Vec<String>) {
    let (matched, caps, kinds, complete) = run(input);
    assert!(
        complete,
        "expected complete match for {:?}, got matched={} kinds={:?}",
        input,
        matched,
        kinds_for(&caps, &kinds)
    );
    assert_eq!(
        matched,
        input.len(),
        "expected full-input match for {:?}",
        input
    );
    (caps, kinds)
}

#[test]
fn grammar_parses_and_compiles() {
    let prog = compile_sql();
    assert!(!prog.code.is_empty());
    assert!(!prog.capture_kinds.is_empty());
    eprintln!(
        "SQLite bytecode: {} rules, {} kinds, {} instr",
        pegc::parse(SQLITE_GRAMMAR).unwrap().rules.len(),
        prog.capture_kinds.len(),
        prog.code.len()
    );
}

#[test]
fn capture_kinds_are_only_theme_kinds() {
    let prog = compile_sql();
    let expected: HashSet<&str> = [
        "keyword",
        "string",
        "number",
        "comment",
        "operator",
        "punctuation",
        "type",
        "function",
        "constant",
        "variable",
    ]
    .into_iter()
    .collect();
    let actual: HashSet<&str> = prog.capture_kinds.iter().map(String::as_str).collect();
    assert!(
        actual.is_subset(&expected),
        "SQLite grammar must stay within the hardcoded-theme vocabulary; got extras: {:?}",
        actual.difference(&expected).collect::<Vec<_>>()
    );
}

#[test]
fn simple_select_literal() {
    let (caps, kinds) = assert_complete_full("SELECT 1;");
    assert_eq!(
        kinds_for(&caps, &kinds),
        vec!["keyword", "number", "punctuation"]
    );
}

#[test]
fn select_column_from_table() {
    let input = "SELECT col FROM tbl";
    let (caps, kinds) = assert_complete_full(input);
    assert_eq!(
        kinds_for(&caps, &kinds),
        vec!["keyword", "variable", "keyword", "type"]
    );
}

#[test]
fn select_star() {
    let input = "SELECT * FROM t";
    let (caps, kinds) = assert_complete_full(input);
    assert_eq!(
        kinds_for(&caps, &kinds),
        vec!["keyword", "operator", "keyword", "type"]
    );
}

#[test]
fn table_dot_star() {
    let input = "SELECT t.* FROM t";
    let (caps, kinds) = assert_complete_full(input);
    let ks = kinds_for(&caps, &kinds);
    assert!(
        ks.contains(&"operator"),
        "expected * as operator, got {:?}",
        ks
    );
    // Both `t`s should be @type.
    let types = spans_for(&caps, &kinds, "type", input);
    assert_eq!(
        types.len(),
        2,
        "expected two @type captures for `t`, got {:?}",
        types
    );
}

#[test]
fn qualified_column_is_variable() {
    let input = "SELECT t.col FROM t";
    let (caps, kinds) = assert_complete_full(input);
    let vars = spans_for(&caps, &kinds, "variable", input);
    assert_eq!(vars, vec!["t", "col"]);
    let types = spans_for(&caps, &kinds, "type", input);
    assert_eq!(types, vec!["t"]);
}

#[test]
fn function_call_emits_function_capture() {
    let input = "SELECT COUNT(*) FROM t";
    let (caps, kinds) = assert_complete_full(input);
    let funcs = spans_for(&caps, &kinds, "function", input);
    assert_eq!(funcs, vec!["COUNT"]);
}

#[test]
fn where_clause_with_eq_and_and_is_null() {
    let input = "SELECT a FROM t WHERE a = 1 AND b IS NULL";
    let (caps, kinds) = assert_complete_full(input);
    let ks = kinds_for(&caps, &kinds);
    assert!(ks.contains(&"operator"), "missing operator in {:?}", ks);
    assert!(
        ks.contains(&"constant"),
        "missing constant (NULL) in {:?}",
        ks
    );
    // Keywords: SELECT, FROM, WHERE, AND, IS.
    let kw_count = ks.iter().filter(|k| **k == "keyword").count();
    assert!(
        kw_count >= 5,
        "expected >= 5 keywords, got {}: {:?}",
        kw_count,
        ks
    );
}

#[test]
fn case_expression() {
    let input = "SELECT CASE WHEN a THEN 1 ELSE 2 END FROM t";
    let (caps, kinds) = assert_complete_full(input);
    let ks = kinds_for(&caps, &kinds);
    // Four CASE-related keywords: CASE WHEN THEN ELSE END (5).
    let kw_count = ks.iter().filter(|k| **k == "keyword").count();
    assert!(
        kw_count >= 7,
        "expected many keywords, got {}: {:?}",
        kw_count,
        ks
    );
}

#[test]
fn cast_emits_type_on_target() {
    let input = "SELECT CAST(x AS INTEGER) FROM t";
    let (caps, kinds) = assert_complete_full(input);
    let types = spans_for(&caps, &kinds, "type", input);
    assert!(
        types.contains(&"INTEGER"),
        "expected INTEGER as @type, got {:?}",
        types
    );
    assert!(types.contains(&"t"), "expected t as @type, got {:?}", types);
}

#[test]
fn string_literal_with_doubled_quote() {
    let input = "SELECT 'it''s' FROM t";
    let (caps, kinds) = assert_complete_full(input);
    let strings = spans_for(&caps, &kinds, "string", input);
    assert_eq!(strings, vec!["'it''s'"]);
}

#[test]
fn blob_literal_is_string() {
    let input = "SELECT X'DEAD' FROM t";
    let (caps, kinds) = assert_complete_full(input);
    let strings = spans_for(&caps, &kinds, "string", input);
    assert_eq!(strings, vec!["X'DEAD'"]);
}

#[test]
fn integer_variants() {
    for input in ["SELECT 0", "SELECT 42", "SELECT 0xDEAD"] {
        let (caps, kinds) = assert_complete_full(input);
        let nums = spans_for(&caps, &kinds, "number", input);
        assert_eq!(nums.len(), 1, "input: {:?}", input);
    }
}

#[test]
fn float_variants() {
    for input in ["SELECT 3.14", "SELECT 1e10", "SELECT 6.022e23"] {
        let (caps, kinds) = assert_complete_full(input);
        let nums = spans_for(&caps, &kinds, "number", input);
        assert_eq!(nums.len(), 1, "input: {:?}", input);
    }
}

#[test]
fn null_true_false_are_constants() {
    for (input, expected) in [
        ("SELECT NULL", "NULL"),
        ("SELECT TRUE", "TRUE"),
        ("SELECT FALSE", "FALSE"),
    ] {
        let (caps, kinds) = assert_complete_full(input);
        let consts = spans_for(&caps, &kinds, "constant", input);
        assert_eq!(consts, vec![expected]);
    }
}

#[test]
fn line_and_block_comments_are_captured() {
    let input = "-- a line\nSELECT /* inline */ 1";
    let (caps, kinds) = assert_complete_full(input);
    let comments = spans_for(&caps, &kinds, "comment", input);
    assert_eq!(comments.len(), 2);
    assert!(comments[0].starts_with("--"));
    assert!(comments[1].starts_with("/*"));
}

#[test]
fn bind_parameters_are_variables() {
    for input in [
        "SELECT ? FROM t",
        "SELECT ?1 FROM t",
        "SELECT :name FROM t",
        "SELECT @name FROM t",
        "SELECT $name FROM t",
    ] {
        let (caps, kinds) = assert_complete_full(input);
        let ks = kinds_for(&caps, &kinds);
        // Bind param must surface as @variable.
        assert!(
            ks.contains(&"variable"),
            "expected @variable for bind param in {:?}, kinds={:?}",
            input,
            ks
        );
    }
}

#[test]
fn compound_select_union_all() {
    let input = "SELECT 1 UNION ALL SELECT 2";
    let (caps, kinds) = assert_complete_full(input);
    let ks = kinds_for(&caps, &kinds);
    let kw_count = ks.iter().filter(|k| **k == "keyword").count();
    assert!(
        kw_count >= 4,
        "expected 4 keywords (SELECT UNION ALL SELECT), got {:?}",
        ks
    );
}

#[test]
fn simple_cte() {
    let input = "WITH t AS (SELECT 1) SELECT 2 FROM t";
    let (caps, kinds) = assert_complete_full(input);
    let types = spans_for(&caps, &kinds, "type", input);
    assert!(
        types.iter().filter(|s| **s == "t").count() >= 2,
        "expected `t` captured as @type (CTE head + FROM), got {:?}",
        types
    );
}

#[test]
fn case_insensitive_keywords() {
    // Same query in lowercase and uppercase should both parse.
    assert_complete_full("SELECT 1");
    assert_complete_full("select 1");
    assert_complete_full("SeLeCt 1");
}

#[test]
fn join_on_clause() {
    let input = "SELECT * FROM u JOIN o ON u.id = o.uid";
    let (caps, kinds) = assert_complete_full(input);
    let ks = kinds_for(&caps, &kinds);
    assert!(ks.contains(&"operator"));
    // u, o at table positions; also u, o, id, uid at column positions.
    let types = spans_for(&caps, &kinds, "type", input);
    assert!(
        types.contains(&"u"),
        "expected `u` as @type, got {:?}",
        types
    );
    assert!(
        types.contains(&"o"),
        "expected `o` as @type, got {:?}",
        types
    );
}

#[test]
fn left_outer_join() {
    let input = "SELECT * FROM a LEFT OUTER JOIN b ON a.id = b.id";
    let (caps, _) = assert_complete_full(input);
    assert!(!caps.is_empty());
}

#[test]
fn realistic_multiclause_query() {
    let input = "\
SELECT u.id, COUNT(*) AS n
FROM users u
JOIN orders o ON u.id = o.user_id
WHERE u.active AND o.total > 100.0
GROUP BY u.id
ORDER BY n DESC
LIMIT 10;
";
    let (caps, kinds) = assert_complete_full(input);
    let ks: HashSet<&str> = kinds_for(&caps, &kinds).into_iter().collect();
    for expected in [
        "keyword",
        "number",
        "punctuation",
        "operator",
        "type",
        "variable",
        "function",
    ] {
        assert!(
            ks.contains(expected),
            "missing kind {} in {:?}",
            expected,
            ks
        );
    }
}

#[test]
fn in_list_and_in_subquery() {
    assert_complete_full("SELECT 1 FROM t WHERE a IN (1, 2, 3)");
    assert_complete_full("SELECT 1 FROM t WHERE a IN (SELECT x FROM s)");
}

#[test]
fn between_and_like() {
    assert_complete_full("SELECT 1 FROM t WHERE a BETWEEN 1 AND 10");
    assert_complete_full("SELECT 1 FROM t WHERE a LIKE 'x%'");
    assert_complete_full("SELECT 1 FROM t WHERE a NOT LIKE 'x%' ESCAPE '\\'");
}

#[test]
fn exists_subquery() {
    assert_complete_full("SELECT 1 WHERE EXISTS (SELECT 1)");
}

#[test]
fn quoted_identifiers() {
    assert_complete_full("SELECT \"a\".\"b\" FROM \"a\"");
    assert_complete_full("SELECT `a`.`b` FROM `a`");
    assert_complete_full("SELECT [a].[b] FROM [a]");
}
