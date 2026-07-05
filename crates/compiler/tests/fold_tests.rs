//! End-to-end tests for `fold_captures` over real `Parser`-emitted
//! captures. Two properties: a parse's captures fold into a
//! caller-typed AST (the typed-captures use case), and the fold's
//! nesting agrees exactly with the flat stack derivation every other
//! consumer of the pre-order capture contract uses.

use syntax_highlighter_compiler::parser::Parser;

/// Nested list of numbers with structural capture kinds — the
/// smallest grammar whose captures carry AST shape rather than
/// highlighting kinds.
const LIST_GRAMMAR: &str = "\
list_doc = item {
item = @list ('[' item (',' item)* ']')
     / @num (\\d+)
}
";

#[derive(Debug, PartialEq)]
enum Ast {
    Num(String),
    List(Vec<Ast>),
}

#[test]
fn fold_builds_typed_ast_from_parse() {
    let mut p = Parser::new(LIST_GRAMMAR).unwrap();
    p.set_input(b"[1,[2,3],4]".to_vec());
    assert!(p.is_complete());
    let input = p.input().to_vec();
    let roots = p.fold_captures(|kind, range, children| match kind {
        "num" => {
            assert!(children.is_empty(), "num must be a leaf");
            Ast::Num(String::from_utf8(input[range].to_vec()).unwrap())
        }
        "list" => Ast::List(children),
        other => panic!("unexpected capture kind {other}"),
    });
    assert_eq!(
        roots,
        vec![Ast::List(vec![
            Ast::Num("1".into()),
            Ast::List(vec![Ast::Num("2".into()), Ast::Num("3".into())]),
            Ast::Num("4".into()),
        ])]
    );
}

/// Pre-order-flatten the folded forest and compare (start, end, depth)
/// per node against the flat `open_ends`-stack derivation (the same
/// one pegdb's `captures dump` uses for its `depth` column). Equality
/// of the full sequences pins the fold's nesting to the established
/// interpretation of the capture stream.
fn assert_fold_matches_stack_derivation(p: &Parser) {
    let (_, captures) = p.captures();

    let mut open_ends: Vec<usize> = Vec::new();
    let mut expected: Vec<(usize, usize, usize)> = Vec::new();
    for c in captures {
        while open_ends.last().is_some_and(|&e| e <= c.start) {
            open_ends.pop();
        }
        expected.push((c.start, c.end, open_ends.len()));
        open_ends.push(c.end);
    }

    struct Node {
        range: std::ops::Range<usize>,
        children: Vec<Node>,
    }
    let roots = p.fold_captures(|_, range, children| Node { range, children });

    let mut actual = Vec::new();
    let mut work: Vec<(Node, usize)> = roots.into_iter().rev().map(|n| (n, 0)).collect();
    while let Some((n, depth)) = work.pop() {
        actual.push((n.range.start, n.range.end, depth));
        for child in n.children.into_iter().rev() {
            work.push((child, depth + 1));
        }
    }

    assert_eq!(
        actual.len(),
        captures.len(),
        "every capture must fold exactly once"
    );
    assert_eq!(
        actual, expected,
        "fold nesting must equal the flat stack derivation"
    );
}

#[test]
fn fold_depths_agree_with_flat_stack_derivation() {
    const JSON_GRAMMAR: &str = include_str!("../../../grammars/json.peg");
    let mut p = Parser::new(JSON_GRAMMAR).unwrap();
    p.set_input(br#"{"a": [1, {"b": null}], "c": "x"}"#.to_vec());
    assert!(p.is_complete());
    assert!(p.captures().1.len() > 10, "fixture should be non-trivial");
    assert_fold_matches_stack_derivation(&p);
}

#[test]
fn fold_works_unchanged_on_partial_parse() {
    const JSON_GRAMMAR: &str = include_str!("../../../grammars/json.peg");
    let mut p = Parser::new(JSON_GRAMMAR).unwrap();
    p.set_input(br#"{"a": [1"#.to_vec());
    assert!(!p.is_complete());
    assert_fold_matches_stack_derivation(&p);
}
