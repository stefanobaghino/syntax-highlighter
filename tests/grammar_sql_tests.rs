use std::collections::HashSet;
use std::sync::OnceLock;

use syntax_highlighter::pegc;
use syntax_highlighter::pegvm::{Capture, Program, VM};

const SQLITE_GRAMMAR: &str = include_str!("../grammars/sqlite.peg");

fn compile_sql() -> &'static Program {
    static SQLITE_PROGRAM: OnceLock<Program> = OnceLock::new();
    SQLITE_PROGRAM
        .get_or_init(|| pegc::compile(SQLITE_GRAMMAR).expect("SQLite grammar should compile"))
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

fn assert_rejects(input: &str) {
    let (matched, _caps, _kinds, complete) = run(input);
    assert!(
        !complete || matched < input.len(),
        "expected rejection for {:?}, got complete full-input match",
        input
    );
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
fn bytecode_size_within_expected_range() {
    // Soft guard: the full dialect currently compiles to ~5.6k
    // instructions. A drift outside this bracket probably means
    // something either shrank surprisingly (a regression) or grew
    // surprisingly (review warranted). Recompute if the change is
    // intentional.
    let n_instr = compile_sql().code.len();
    assert!(
        (4500..=7500).contains(&n_instr),
        "bytecode size {n_instr} outside expected range [4500, 7500]; \
         recompute the bracket if this is an intentional change"
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

#[test]
fn non_reserved_words_parse_as_identifiers() {
    // Guards the longest-first invariant in `keyword_word`: words that
    // look like dialect vocabulary but are not reserved in SQLite must
    // still parse as bare identifiers in column/table position.
    assert_complete_full("SELECT integer FROM t");
    assert_complete_full("SELECT recursive FROM t");
    assert_complete_full("SELECT partition FROM t WHERE following = 1");
    assert_complete_full("SELECT x FROM natural_joins");
    // `key` and `do` are syntactically keywords (PRIMARY KEY, DO
    // UPDATE) but too common as column / alias names to reserve.
    assert_complete_full("SELECT key FROM config WHERE key = 'x'");
    assert_complete_full("INSERT INTO config (key, value) VALUES ('k', 'v')");
    assert_complete_full("SELECT * FROM orders AS do");
}

#[test]
fn rejects_blatantly_invalid_input() {
    // Seeds `assert_rejects` usage. Two consecutive SELECTs without a
    // separator or expression between them are never valid.
    assert_rejects("SELECT SELECT");
    assert_rejects("SELECT FROM");
}

#[test]
fn with_recursive_accepted() {
    assert_complete_full(
        "WITH RECURSIVE counter(n) AS (SELECT 1 UNION ALL SELECT n FROM counter) \
         SELECT n FROM counter",
    );
}

#[test]
fn with_non_recursive_still_works() {
    // The RECURSIVE token is optional — classic CTE shape must stay valid.
    assert_complete_full("WITH t AS (SELECT 1) SELECT * FROM t");
}

#[test]
fn window_function_empty_over() {
    assert_complete_full("SELECT COUNT(*) OVER () FROM t");
}

#[test]
fn window_function_partition_by() {
    assert_complete_full("SELECT SUM(x) OVER (PARTITION BY y) FROM t");
}

#[test]
fn window_function_order_by() {
    assert_complete_full("SELECT SUM(x) OVER (ORDER BY y) FROM t");
}

#[test]
fn window_function_partition_and_order() {
    assert_complete_full("SELECT SUM(x) OVER (PARTITION BY a ORDER BY b DESC) FROM t");
}

#[test]
fn window_frame_rows_between() {
    assert_complete_full(
        "SELECT SUM(x) OVER (ORDER BY y ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) FROM t",
    );
}

#[test]
fn window_frame_range_exclude_ties() {
    assert_complete_full(
        "SELECT AVG(x) OVER (ORDER BY y RANGE BETWEEN 5 PRECEDING AND 5 FOLLOWING EXCLUDE TIES) \
         FROM t",
    );
}

#[test]
fn window_frame_groups() {
    assert_complete_full(
        "SELECT MAX(x) OVER (ORDER BY y GROUPS 3 PRECEDING EXCLUDE CURRENT ROW) FROM t",
    );
}

#[test]
fn window_frame_exclude_no_others() {
    assert_complete_full(
        "SELECT SUM(x) OVER (ORDER BY y ROWS UNBOUNDED PRECEDING EXCLUDE NO OTHERS) FROM t",
    );
}

#[test]
fn filter_clause() {
    assert_complete_full("SELECT SUM(x) FILTER (WHERE y > 0) FROM t");
}

#[test]
fn filter_with_over() {
    assert_complete_full("SELECT SUM(x) FILTER (WHERE y > 0) OVER (PARTITION BY z) FROM t");
}

#[test]
fn over_named_window() {
    assert_complete_full("SELECT SUM(x) OVER w FROM t WINDOW w AS (PARTITION BY y)");
}

#[test]
fn top_level_window_clause() {
    assert_complete_full("SELECT x FROM t WINDOW w1 AS (PARTITION BY a), w2 AS (ORDER BY b)");
}

#[test]
fn insert_values_basic() {
    assert_complete_full("INSERT INTO t VALUES (1, 2, 3)");
}

#[test]
fn insert_with_column_list() {
    assert_complete_full("INSERT INTO t (a, b) VALUES (1, 2)");
}

#[test]
fn insert_multi_row_values() {
    assert_complete_full("INSERT INTO t (a) VALUES (1), (2), (3)");
}

#[test]
fn insert_default_values() {
    assert_complete_full("INSERT INTO t DEFAULT VALUES");
}

#[test]
fn insert_from_select() {
    assert_complete_full("INSERT INTO t SELECT * FROM s");
}

#[test]
fn insert_or_variants() {
    for prefix in [
        "INSERT OR REPLACE",
        "INSERT OR ROLLBACK",
        "INSERT OR ABORT",
        "INSERT OR FAIL",
        "INSERT OR IGNORE",
    ] {
        let input = format!("{prefix} INTO t VALUES (1)");
        assert_complete_full(&input);
    }
}

#[test]
fn replace_shorthand() {
    // Bare REPLACE INTO is equivalent to INSERT OR REPLACE.
    assert_complete_full("REPLACE INTO t VALUES (1)");
}

#[test]
fn insert_with_alias() {
    assert_complete_full("INSERT INTO t AS x (a) VALUES (1)");
}

#[test]
fn insert_on_conflict_do_nothing() {
    assert_complete_full("INSERT INTO t (a) VALUES (1) ON CONFLICT (a) DO NOTHING");
}

#[test]
fn insert_on_conflict_do_update() {
    assert_complete_full(
        "INSERT INTO t (a, b) VALUES (1, 2) \
         ON CONFLICT (a) DO UPDATE SET b = excluded.b WHERE b > 0",
    );
}

#[test]
fn insert_on_conflict_without_target() {
    assert_complete_full("INSERT INTO t VALUES (1) ON CONFLICT DO NOTHING");
}

#[test]
fn insert_returning_star() {
    assert_complete_full("INSERT INTO t VALUES (1) RETURNING *");
}

#[test]
fn insert_returning_columns() {
    assert_complete_full("INSERT INTO t (a, b) VALUES (1, 2) RETURNING a, b AS bb");
}

#[test]
fn insert_with_cte() {
    assert_complete_full("WITH src AS (SELECT 1 AS a) INSERT INTO t (a) SELECT a FROM src");
}

#[test]
fn insert_into_emits_type_on_table() {
    let input = "INSERT INTO t VALUES (1)";
    let (caps, kinds) = assert_complete_full(input);
    let types = spans_for(&caps, &kinds, "type", input);
    assert_eq!(types, vec!["t"]);
}

#[test]
fn update_basic() {
    assert_complete_full("UPDATE t SET a = 1");
}

#[test]
fn update_multi_set_with_where() {
    assert_complete_full("UPDATE t SET a = 1, b = 2 WHERE c = 3");
}

#[test]
fn update_column_tuple() {
    assert_complete_full("UPDATE t SET (a, b) = (1, 2)");
}

#[test]
fn update_or_rollback() {
    for prefix in [
        "UPDATE OR ROLLBACK",
        "UPDATE OR ABORT",
        "UPDATE OR REPLACE",
        "UPDATE OR FAIL",
        "UPDATE OR IGNORE",
    ] {
        let input = format!("{prefix} t SET a = 1");
        assert_complete_full(&input);
    }
}

#[test]
fn update_with_from() {
    assert_complete_full("UPDATE t SET a = s.a FROM s WHERE t.id = s.id");
}

#[test]
fn update_with_returning() {
    assert_complete_full("UPDATE t SET a = 1 WHERE b > 0 RETURNING a, b");
}

#[test]
fn update_with_order_limit() {
    assert_complete_full("UPDATE t SET a = a + 1 ORDER BY id LIMIT 10");
}

#[test]
fn delete_basic() {
    assert_complete_full("DELETE FROM t");
}

#[test]
fn delete_with_where() {
    assert_complete_full("DELETE FROM t WHERE a = 1");
}

#[test]
fn delete_with_returning() {
    assert_complete_full("DELETE FROM t WHERE a = 1 RETURNING *");
}

#[test]
fn delete_with_order_limit() {
    assert_complete_full("DELETE FROM t WHERE a = 1 ORDER BY id LIMIT 10");
}

#[test]
fn delete_with_cte() {
    assert_complete_full(
        "WITH cur AS (SELECT id FROM t) DELETE FROM t WHERE id IN (SELECT id FROM cur)",
    );
}

#[test]
fn create_table_basic() {
    assert_complete_full("CREATE TABLE t (a INTEGER)");
}

#[test]
fn create_table_typeless_columns() {
    assert_complete_full("CREATE TABLE t (a, b)");
}

#[test]
fn create_table_mixed_constraints() {
    assert_complete_full(
        "CREATE TABLE t (a INTEGER PRIMARY KEY, b TEXT NOT NULL UNIQUE, c INT NULL)",
    );
}

#[test]
fn create_table_check_constraint() {
    assert_complete_full("CREATE TABLE t (a INT CHECK (a > 0))");
}

#[test]
fn create_table_defaults() {
    assert_complete_full(
        "CREATE TABLE t (a INT DEFAULT 0, b TEXT DEFAULT 'x', c DATETIME DEFAULT (CURRENT_TIMESTAMP))",
    );
}

#[test]
fn create_table_column_collate() {
    assert_complete_full("CREATE TABLE t (a TEXT COLLATE NOCASE)");
}

#[test]
fn create_table_generated_column_shorthand() {
    assert_complete_full("CREATE TABLE t (a INT, b INT AS (a + 1) STORED)");
}

#[test]
fn create_table_generated_column_always() {
    assert_complete_full("CREATE TABLE t (a INT, b INT GENERATED ALWAYS AS (a + 1) VIRTUAL)");
}

#[test]
fn create_table_table_primary_key() {
    assert_complete_full("CREATE TABLE t (a INT, b INT, PRIMARY KEY (a, b))");
}

#[test]
fn create_table_named_constraint() {
    assert_complete_full("CREATE TABLE t (a INT, CONSTRAINT pk PRIMARY KEY (a))");
}

#[test]
fn create_table_foreign_key() {
    assert_complete_full(
        "CREATE TABLE t (a INT, b INT, FOREIGN KEY (a) REFERENCES u(x) ON DELETE CASCADE)",
    );
}

#[test]
fn create_table_foreign_key_deferrable() {
    assert_complete_full("CREATE TABLE t (a INT REFERENCES u(x) DEFERRABLE INITIALLY DEFERRED)");
}

#[test]
fn create_table_without_rowid_strict() {
    assert_complete_full("CREATE TABLE t (a INT PRIMARY KEY) WITHOUT ROWID, STRICT");
}

#[test]
fn create_table_if_not_exists() {
    assert_complete_full("CREATE TABLE IF NOT EXISTS t (a INT)");
}

#[test]
fn create_temp_table() {
    assert_complete_full("CREATE TEMP TABLE t (a INT)");
    assert_complete_full("CREATE TEMPORARY TABLE t (a INT)");
}

#[test]
fn create_table_as_select() {
    assert_complete_full("CREATE TABLE t AS SELECT * FROM s");
}

#[test]
fn create_table_multi_word_types() {
    assert_complete_full(
        "CREATE TABLE t (a DOUBLE PRECISION, b UNSIGNED BIG INT, c VARCHAR(255), d NUMERIC(10, 2))",
    );
}

#[test]
fn drop_table_basic() {
    assert_complete_full("DROP TABLE t");
}

#[test]
fn drop_table_if_exists() {
    assert_complete_full("DROP TABLE IF EXISTS t");
}

#[test]
fn alter_table_rename_to() {
    assert_complete_full("ALTER TABLE t RENAME TO t2");
}

#[test]
fn alter_table_rename_column() {
    assert_complete_full("ALTER TABLE t RENAME COLUMN a TO b");
    assert_complete_full("ALTER TABLE t RENAME a TO b");
}

#[test]
fn alter_table_add_column() {
    assert_complete_full("ALTER TABLE t ADD COLUMN c INT DEFAULT 0");
    assert_complete_full("ALTER TABLE t ADD c INT");
}

#[test]
fn alter_table_drop_column() {
    assert_complete_full("ALTER TABLE t DROP COLUMN c");
    assert_complete_full("ALTER TABLE t DROP c");
}

#[test]
fn create_index_basic() {
    assert_complete_full("CREATE INDEX i ON t (a)");
}

#[test]
fn create_unique_index_multi_col() {
    assert_complete_full("CREATE UNIQUE INDEX i ON t (a, b)");
}

#[test]
fn create_partial_index() {
    assert_complete_full("CREATE INDEX i ON t (a) WHERE a > 0");
}

#[test]
fn create_expression_index() {
    assert_complete_full("CREATE INDEX i ON t (LOWER(a))");
}

#[test]
fn create_index_if_not_exists() {
    assert_complete_full("CREATE INDEX IF NOT EXISTS i ON t (a)");
}

#[test]
fn drop_index_variants() {
    assert_complete_full("DROP INDEX i");
    assert_complete_full("DROP INDEX IF EXISTS main.i");
}

#[test]
fn create_view_basic() {
    assert_complete_full("CREATE VIEW v AS SELECT * FROM t");
}

#[test]
fn create_view_with_columns() {
    assert_complete_full("CREATE VIEW v (a, b) AS SELECT x, y FROM t");
}

#[test]
fn create_temp_view() {
    assert_complete_full("CREATE TEMP VIEW v AS SELECT 1");
    assert_complete_full("CREATE TEMPORARY VIEW v AS SELECT 1");
}

#[test]
fn drop_view_variants() {
    assert_complete_full("DROP VIEW v");
    assert_complete_full("DROP VIEW IF EXISTS v");
}

#[test]
fn create_trigger_after_insert() {
    assert_complete_full(
        "CREATE TRIGGER tr AFTER INSERT ON t BEGIN INSERT INTO log VALUES (1); END",
    );
}

#[test]
fn create_trigger_before_update() {
    assert_complete_full(
        "CREATE TRIGGER tr BEFORE UPDATE ON t BEGIN UPDATE log SET n = n + 1; END",
    );
}

#[test]
fn create_trigger_instead_of_delete() {
    assert_complete_full(
        "CREATE TRIGGER tr INSTEAD OF DELETE ON v BEGIN DELETE FROM u WHERE id = OLD.id; END",
    );
}

#[test]
fn create_trigger_update_of_columns() {
    assert_complete_full("CREATE TRIGGER tr AFTER UPDATE OF a, b ON t BEGIN SELECT 1; END");
}

#[test]
fn create_trigger_for_each_row() {
    assert_complete_full("CREATE TRIGGER tr AFTER INSERT ON t FOR EACH ROW BEGIN SELECT 1; END");
}

#[test]
fn create_trigger_when_clause() {
    assert_complete_full(
        "CREATE TRIGGER tr AFTER INSERT ON t FOR EACH ROW WHEN NEW.a > 0 \
         BEGIN INSERT INTO log VALUES (NEW.a); END",
    );
}

#[test]
fn create_trigger_multi_statement_body() {
    assert_complete_full(
        "CREATE TRIGGER tr AFTER INSERT ON t BEGIN \
           INSERT INTO log VALUES (1); \
           UPDATE stats SET n = n + 1; \
           DELETE FROM tmp WHERE id = NEW.id; \
         END",
    );
}

#[test]
fn create_trigger_if_not_exists() {
    assert_complete_full("CREATE TRIGGER IF NOT EXISTS tr AFTER INSERT ON t BEGIN SELECT 1; END");
}

#[test]
fn create_temp_trigger() {
    assert_complete_full("CREATE TEMP TRIGGER tr AFTER INSERT ON t BEGIN SELECT 1; END");
}

#[test]
fn drop_trigger_variants() {
    assert_complete_full("DROP TRIGGER tr");
    assert_complete_full("DROP TRIGGER IF EXISTS main.tr");
}

#[test]
fn begin_variants() {
    assert_complete_full("BEGIN");
    assert_complete_full("BEGIN TRANSACTION");
    assert_complete_full("BEGIN DEFERRED");
    assert_complete_full("BEGIN IMMEDIATE TRANSACTION");
    assert_complete_full("BEGIN EXCLUSIVE");
}

#[test]
fn commit_and_end_aliases() {
    assert_complete_full("COMMIT");
    assert_complete_full("END");
    assert_complete_full("COMMIT TRANSACTION");
    assert_complete_full("END TRANSACTION");
}

#[test]
fn rollback_variants() {
    assert_complete_full("ROLLBACK");
    assert_complete_full("ROLLBACK TRANSACTION");
    assert_complete_full("ROLLBACK TO sp");
    assert_complete_full("ROLLBACK TRANSACTION TO SAVEPOINT sp");
}

#[test]
fn savepoint_and_release() {
    assert_complete_full("SAVEPOINT sp");
    assert_complete_full("RELEASE sp");
    assert_complete_full("RELEASE SAVEPOINT sp");
}

#[test]
fn pragma_variants() {
    assert_complete_full("PRAGMA cache_size");
    assert_complete_full("PRAGMA cache_size = 2000");
    assert_complete_full("PRAGMA cache_size(2000)");
    assert_complete_full("PRAGMA main.cache_size = 1000");
    assert_complete_full("PRAGMA foreign_keys = ON");
    assert_complete_full("PRAGMA journal_mode = 'wal'");
}

#[test]
fn vacuum_variants() {
    assert_complete_full("VACUUM");
    assert_complete_full("VACUUM main");
    assert_complete_full("VACUUM INTO 'backup.db'");
}

#[test]
fn attach_detach() {
    assert_complete_full("ATTACH DATABASE 'x.db' AS backup");
    assert_complete_full("ATTACH 'x.db' AS backup");
    assert_complete_full("DETACH DATABASE backup");
    assert_complete_full("DETACH backup");
}

#[test]
fn reindex_variants() {
    assert_complete_full("REINDEX");
    assert_complete_full("REINDEX t");
    assert_complete_full("REINDEX main.t");
}

#[test]
fn analyze_variants() {
    assert_complete_full("ANALYZE");
    assert_complete_full("ANALYZE main.t");
}

#[test]
fn explain_variants() {
    assert_complete_full("EXPLAIN SELECT 1");
    assert_complete_full("EXPLAIN QUERY PLAN SELECT 1");
    assert_complete_full("EXPLAIN INSERT INTO t VALUES (1)");
}

#[test]
fn explain_explain_rejected() {
    // `EXPLAIN EXPLAIN ...` is not valid — explain_stmt wraps
    // statement_body, not statement.
    assert_rejects("EXPLAIN EXPLAIN SELECT 1");
}

#[test]
fn create_virtual_table_fts5() {
    assert_complete_full("CREATE VIRTUAL TABLE fts USING fts5(a, b)");
}

#[test]
fn create_virtual_table_if_not_exists() {
    assert_complete_full("CREATE VIRTUAL TABLE IF NOT EXISTS fts USING fts5(a UNINDEXED, b)");
}

#[test]
fn create_virtual_table_rtree() {
    assert_complete_full("CREATE VIRTUAL TABLE boxes USING rtree(id, x0, x1, y0, y1)");
}

#[test]
fn create_virtual_table_no_args() {
    assert_complete_full("CREATE VIRTUAL TABLE t USING module_name");
}

#[test]
fn trigger_then_select_multi_statement() {
    // Two top-level statements separated by `;`: a trigger (whose body
    // has its own inner `;`-terminated statements) and a SELECT.
    assert_complete_full("CREATE TRIGGER tr AFTER INSERT ON t BEGIN SELECT 1; END; SELECT 2");
}

#[test]
fn create_table_column_kinds() {
    let input = "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)";
    let (caps, kinds) = assert_complete_full(input);
    // Table name and type names are @type.
    let types = spans_for(&caps, &kinds, "type", input);
    assert!(
        types.contains(&"users"),
        "expected users as @type, got {types:?}"
    );
    assert!(
        types.contains(&"INTEGER"),
        "expected INTEGER as @type, got {types:?}"
    );
    assert!(
        types.contains(&"TEXT"),
        "expected TEXT as @type, got {types:?}"
    );
    // Column names are @variable.
    let vars = spans_for(&caps, &kinds, "variable", input);
    assert!(
        vars.contains(&"id"),
        "expected id as @variable, got {vars:?}"
    );
    assert!(
        vars.contains(&"name"),
        "expected name as @variable, got {vars:?}"
    );
}

#[test]
fn window_captures_include_over_and_partition() {
    let input = "SELECT SUM(x) OVER (PARTITION BY y ORDER BY z) FROM t";
    let (caps, kinds) = assert_complete_full(input);
    let ks = kinds_for(&caps, &kinds);
    let kw_count = ks.iter().filter(|k| **k == "keyword").count();
    // SELECT OVER PARTITION BY ORDER BY FROM = 7 keywords at minimum.
    assert!(
        kw_count >= 7,
        "expected >=7 keywords, got {kw_count}: {ks:?}"
    );
}
