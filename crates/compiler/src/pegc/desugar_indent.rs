//! Desugaring of the implicit indentation operators.
//!
//! The grammar author writes three prefix operators — `%root X`,
//! `%align`, and `%indent X` — and never sees the column-threading
//! machinery underneath. This pass, run first in
//! [`Grammar::compile`](super::parser::Grammar::compile), rewrites those
//! operators back into the lower-level IR the rest of the compiler and the
//! whole VM already understand: rules parameterized by an indentation
//! column, the `deeper` / `same` / `at_least` combinators
//! ([`Pattern::IndentCombinator`]), and parameterized calls
//! ([`Pattern::NonTerminal`] with `args`). After it runs, no
//! [`Pattern::IndentOp`] remains and `rule_params` is populated, so every
//! later pass sees exactly the form an author would have written by hand
//! against the explicit surface.
//!
//! # The single ambient anchor
//!
//! The operators expose one ambient anchor — the indentation column of
//! the construct currently being parsed. `%root` seeds a fresh anchor at
//! the current line's column; `%indent` opens a level strictly deeper than
//! the ambient anchor and rebinds it for its operand; `%align` asserts the
//! current line sits exactly at the ambient anchor. The anchor threads
//! down the call chain as an implicit parameter (`$indent`), so a rule
//! that reads it never has to name it.
//!
//! This single-anchor model covers every brace-replaceable, tree-shaped
//! indentation language (YAML, Starlark, Scala 3, F#). The underlying IR
//! keeps the richer capability — literal and multiple named columns, an
//! assert without reseed — but no surface exposes it.
//!
//! # Effect-selective threading
//!
//! Only a rule that actually reads an inherited anchor gets the `$indent`
//! parameter. "Reads an inherited anchor" means: at top level (outside any
//! `%root` / `%indent` scope, which seed their own) the rule has a
//! `%align` or `%indent`, or it calls a rule that itself reads one. This
//! is a monotone call-graph fixpoint ([`compute_needs_anchor`]). Rules
//! that read nothing — every rule in every non-indentation grammar, plus
//! the `%root`-only seeding rules — stay parameterless, so the memo
//! hot-path key (`ArgKey::None`) is untouched for them.

use std::collections::{HashMap, HashSet};

use super::compiler::CompileError;
use super::parser::Grammar;
use super::pattern::{IndentArg, IndentOp, IndentOpKind, Pattern, Span};

/// Synthetic name of the implicit anchor parameter. `$`-prefixed so it can
/// never collide with an author identifier — `is_ident_start` admits only
/// ASCII letters and `_`, never `$`.
const ANCHOR_PARAM: &str = "$indent";

/// Rewrite every `%root` / `%align` / `%indent` in `grammar` into the
/// column-threading IR, and populate `grammar.rule_params` with the
/// implicit `$indent` parameter for each rule that reads an inherited
/// anchor. Idempotent in effect on grammars that use no indentation
/// operators: nothing is rewritten and `rule_params` stays empty.
///
/// Errors with [`CompileError::IndentAnchorOutOfScope`] if a `%align` /
/// `%indent` or an anchored call can be reached from the grammar root
/// without an enclosing `%root` / `%indent` to establish the column.
pub(crate) fn desugar_indent(grammar: &mut Grammar) -> Result<(), CompileError> {
    let needs = compute_needs_anchor(&grammar.rules);

    // The root is invoked by the bootstrap `Call` with no arguments, so it
    // has nobody to inherit an anchor from. If it reads one, a `%align` /
    // `%indent` or an anchored call escaped every `%root` / `%indent`
    // scope — the column would have no provider at runtime.
    if needs.contains(&grammar.root) {
        let demand = describe_top_level_demand(&grammar.rules[&grammar.root], &needs)
            .unwrap_or_else(|| "an indentation operator".to_string());
        return Err(CompileError::IndentAnchorOutOfScope {
            rule: grammar.root.clone(),
            demand,
        });
    }

    let mut rewritten: HashMap<String, Pattern> = HashMap::with_capacity(grammar.rules.len());
    for (name, body) in &grammar.rules {
        let start = if needs.contains(name) {
            Some(ANCHOR_PARAM.to_string())
        } else {
            None
        };
        let mut body = body.clone();
        let mut fresh = 0usize;
        rewrite(&mut body, start.as_deref(), &needs, name, &mut fresh)?;
        rewritten.insert(name.clone(), body);
    }
    grammar.rules = rewritten;

    // Give every anchor-reading rule the implicit parameter so the
    // compiler emits `RuleEnter` with argc=1 and the threaded `ArgPush`
    // lines resolve against slot 0. The surface has no parameter syntax,
    // so `rule_params` was empty coming in.
    for name in &needs {
        grammar
            .rule_params
            .insert(name.clone(), vec![ANCHOR_PARAM.to_string()]);
    }

    Ok(())
}

/// Set of rules that read an inherited indentation anchor, by monotone
/// call-graph fixpoint. Seed: any rule with a top-level `%align` or
/// `%indent`. Grow: any rule that, at top level, calls a rule already in
/// the set. `%root` operands are *not* top level — they seed their own
/// anchor — so a rule whose only anchored work sits under a `%root` reads
/// nothing inherited and stays out of the set.
fn compute_needs_anchor(rules: &HashMap<String, Pattern>) -> HashSet<String> {
    let mut needs: HashSet<String> = HashSet::new();
    loop {
        let mut changed = false;
        for (name, body) in rules {
            if !needs.contains(name) && top_level_demands_anchor(body, &needs) {
                needs.insert(name.clone());
                changed = true;
            }
        }
        if !changed {
            return needs;
        }
    }
}

/// Whether `pat`, walked at the enclosing rule's top level, reads the
/// inherited anchor — via a `%align` / `%indent`, or a call to a rule in
/// `needs`. Does **not** descend into `%root` / `%indent` operands: those
/// open a fresh anchor scope, so what they contain says nothing about the
/// inherited one. (`%indent` itself reads the inherited anchor before
/// opening the deeper level, hence it counts here without descending.)
fn top_level_demands_anchor(pat: &Pattern, needs: &HashSet<String>) -> bool {
    match pat {
        Pattern::IndentOp {
            kind: IndentOpKind::Align | IndentOpKind::Indent,
            ..
        } => true,
        Pattern::IndentOp {
            kind: IndentOpKind::Root,
            ..
        } => false,
        Pattern::NonTerminal { name, .. } => needs.contains(name),
        Pattern::Sequence { items, .. } => items.iter().any(|p| top_level_demands_anchor(p, needs)),
        Pattern::OrderedChoice { alts, .. } => {
            alts.iter().any(|p| top_level_demands_anchor(p, needs))
        }
        Pattern::Repeat { inner, .. }
        | Pattern::RepeatOne { inner, .. }
        | Pattern::Optional { inner, .. }
        | Pattern::NotPredicate { inner, .. }
        | Pattern::AndPredicate { inner, .. }
        | Pattern::Capture { inner, .. }
        | Pattern::InferBoundaryCatch { inner, .. } => top_level_demands_anchor(inner, needs),
        Pattern::Catch {
            inner, recovery, ..
        } => top_level_demands_anchor(inner, needs) || top_level_demands_anchor(recovery, needs),
        Pattern::Literal { .. }
        | Pattern::CharClass { .. }
        | Pattern::AnyChar { .. }
        | Pattern::IndentCombinator { .. } => false,
    }
}

/// First top-level construct in `pat` that reads the inherited anchor,
/// rendered for the [`CompileError::IndentAnchorOutOfScope`] message.
/// Mirrors [`top_level_demands_anchor`]'s descent.
fn describe_top_level_demand(pat: &Pattern, needs: &HashSet<String>) -> Option<String> {
    match pat {
        Pattern::IndentOp {
            kind: IndentOpKind::Align,
            ..
        } => Some("`%align`".to_string()),
        Pattern::IndentOp {
            kind: IndentOpKind::Indent,
            ..
        } => Some("`%indent`".to_string()),
        Pattern::IndentOp {
            kind: IndentOpKind::Root,
            ..
        } => None,
        Pattern::NonTerminal { name, .. } if needs.contains(name) => {
            Some(format!("a call to indentation-anchored rule `{name}`"))
        }
        Pattern::NonTerminal { .. } => None,
        Pattern::Sequence { items, .. } => items
            .iter()
            .find_map(|p| describe_top_level_demand(p, needs)),
        Pattern::OrderedChoice { alts, .. } => alts
            .iter()
            .find_map(|p| describe_top_level_demand(p, needs)),
        Pattern::Repeat { inner, .. }
        | Pattern::RepeatOne { inner, .. }
        | Pattern::Optional { inner, .. }
        | Pattern::NotPredicate { inner, .. }
        | Pattern::AndPredicate { inner, .. }
        | Pattern::Capture { inner, .. }
        | Pattern::InferBoundaryCatch { inner, .. } => describe_top_level_demand(inner, needs),
        Pattern::Catch {
            inner, recovery, ..
        } => describe_top_level_demand(inner, needs)
            .or_else(|| describe_top_level_demand(recovery, needs)),
        Pattern::Literal { .. }
        | Pattern::CharClass { .. }
        | Pattern::AnyChar { .. }
        | Pattern::IndentCombinator { .. } => None,
    }
}

/// Rewrite `pat` in place, threading `anchor` (the in-scope anchor local's
/// name, or `None` outside any anchor scope) through the tree:
///
/// - `%align` → `same(anchor)`
/// - `%indent X` → `deeper(anchor) as fresh` then `X` desugared under `fresh`
/// - `%root X` → `at_least(0) as fresh` then `X` desugared under `fresh`
/// - a call to a `needs`-anchor rule → that call with `anchor` as its
///   single argument
///
/// `fresh` is a per-rule counter producing collision-proof bind names.
/// Hitting an anchor-reading construct with `anchor == None` is the
/// out-of-scope error; for a correctly-propagated grammar this never fires
/// below the root, since [`compute_needs_anchor`] gives every such rule an
/// anchor to start from.
fn rewrite(
    pat: &mut Pattern,
    anchor: Option<&str>,
    needs: &HashSet<String>,
    rule: &str,
    fresh: &mut usize,
) -> Result<(), CompileError> {
    match pat {
        Pattern::IndentOp {
            kind,
            operand,
            span,
        } => {
            let kind = *kind;
            let span = *span;
            let operand = operand.take();
            *pat = desugar_op(kind, operand, span, anchor, needs, rule, fresh)?;
            Ok(())
        }
        Pattern::NonTerminal { name, args, .. } => {
            if needs.contains(name) {
                let a = require_anchor(anchor, rule, || {
                    format!("a call to indentation-anchored rule `{name}`")
                })?;
                debug_assert!(
                    args.is_empty(),
                    "the surface has no call-argument syntax, so a parsed call carries none"
                );
                *args = vec![IndentArg::Local(a)];
            }
            Ok(())
        }
        Pattern::Sequence { items, .. } => {
            for it in items {
                rewrite(it, anchor, needs, rule, fresh)?;
            }
            Ok(())
        }
        Pattern::OrderedChoice { alts, .. } => {
            for a in alts {
                rewrite(a, anchor, needs, rule, fresh)?;
            }
            Ok(())
        }
        Pattern::Repeat { inner, .. }
        | Pattern::RepeatOne { inner, .. }
        | Pattern::Optional { inner, .. }
        | Pattern::NotPredicate { inner, .. }
        | Pattern::AndPredicate { inner, .. }
        | Pattern::Capture { inner, .. }
        | Pattern::InferBoundaryCatch { inner, .. } => rewrite(inner, anchor, needs, rule, fresh),
        Pattern::Catch {
            inner, recovery, ..
        } => {
            rewrite(inner, anchor, needs, rule, fresh)?;
            rewrite(recovery, anchor, needs, rule, fresh)
        }
        Pattern::Literal { .. }
        | Pattern::CharClass { .. }
        | Pattern::AnyChar { .. }
        | Pattern::IndentCombinator { .. } => Ok(()),
    }
}

/// Build the replacement for one `%`-operator. `%root` / `%indent` desugar
/// to the seeding/opening combinator followed by their operand desugared
/// under the freshly-bound anchor; nesting them in a `Sequence` is
/// transparent to the compiler (it emits the combinator's instructions
/// then the operand's), so the bytecode matches a hand-written flat
/// sequence.
fn desugar_op(
    kind: IndentOpKind,
    operand: Option<Box<Pattern>>,
    span: Span,
    anchor: Option<&str>,
    needs: &HashSet<String>,
    rule: &str,
    fresh: &mut usize,
) -> Result<Pattern, CompileError> {
    match kind {
        IndentOpKind::Align => {
            let a = require_anchor(anchor, rule, || "`%align`".to_string())?;
            Ok(Pattern::IndentCombinator {
                op: IndentOp::Same,
                arg: IndentArg::Local(a),
                bind: None,
                span,
            })
        }
        IndentOpKind::Indent => {
            let a = require_anchor(anchor, rule, || "`%indent`".to_string())?;
            let bind = fresh_anchor(fresh);
            let inner = desugar_operand(operand, &bind, needs, rule, fresh)?;
            Ok(opened_block(
                IndentOp::Deeper,
                IndentArg::Local(a),
                bind,
                inner,
                span,
            ))
        }
        IndentOpKind::Root => {
            let bind = fresh_anchor(fresh);
            let inner = desugar_operand(operand, &bind, needs, rule, fresh)?;
            Ok(opened_block(
                IndentOp::AtLeast,
                IndentArg::Lit(0),
                bind,
                inner,
                span,
            ))
        }
    }
}

/// Desugar a `%root` / `%indent` operand under the freshly-bound anchor
/// `bind`.
fn desugar_operand(
    operand: Option<Box<Pattern>>,
    bind: &str,
    needs: &HashSet<String>,
    rule: &str,
    fresh: &mut usize,
) -> Result<Pattern, CompileError> {
    let mut inner =
        *operand.expect("`%root` / `%indent` always carry an operand (parser enforced)");
    rewrite(&mut inner, Some(bind), needs, rule, fresh)?;
    Ok(inner)
}

/// `Sequence[ measure-and-assert, body ]` — the combinator that measures
/// the line and binds the anchor, then the operand that runs under it.
fn opened_block(op: IndentOp, arg: IndentArg, bind: String, body: Pattern, span: Span) -> Pattern {
    Pattern::Sequence {
        items: vec![
            Pattern::IndentCombinator {
                op,
                arg,
                bind: Some(bind),
                span,
            },
            body,
        ],
        span,
    }
}

/// The in-scope anchor name, or the out-of-scope error built from
/// `demand` (a description of what needed an anchor).
fn require_anchor(
    anchor: Option<&str>,
    rule: &str,
    demand: impl FnOnce() -> String,
) -> Result<String, CompileError> {
    anchor
        .map(str::to_string)
        .ok_or_else(|| CompileError::IndentAnchorOutOfScope {
            rule: rule.to_string(),
            demand: demand(),
        })
}

/// A fresh, collision-proof anchor bind name (`$anchor0`, `$anchor1`, …).
/// `$`-prefixed like [`ANCHOR_PARAM`], so it can't shadow an author local;
/// the exact text is irrelevant to bytecode, which addresses activation
/// slots by index.
fn fresh_anchor(fresh: &mut usize) -> String {
    let n = *fresh;
    *fresh += 1;
    format!("$anchor{n}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pegc::{compile, parse, Error};
    use syntax_highlighter::pegvm::CharSet;

    /// The `%`-surface form of a tiny indentation language: a root block of
    /// column-aligned lines, each of which may own a strictly-deeper
    /// sub-block. Exercises all three operators plus a threaded recursive
    /// call (`ln` → `sub` → `ln`).
    const PCT_SRC: &str = "root = doc {
    doc: atomic = %root ( ln ( sep %align ln )* )
    ln: atomic = @text [a-z] sub?
    sub: atomic = %indent ( ln ( sep %align ln )* )
    sep: atomic = '\\n'
}";

    /// The same language written by hand against the lower-level
    /// column-threading IR — parameterized rules, the
    /// `deeper` / `same` / `at_least` combinators, parameterized calls.
    /// This is the form `desugar_indent` must reproduce.
    fn handwritten_explicit() -> Grammar {
        let syn = Span::SYNTHETIC;
        let comb = |op: IndentOp, arg: IndentArg, bind: Option<&str>| Pattern::IndentCombinator {
            op,
            arg,
            bind: bind.map(str::to_string),
            span: syn,
        };
        let call = |name: &str, local: &str| Pattern::NonTerminal {
            name: name.to_string(),
            args: vec![IndentArg::Local(local.to_string())],
            span: syn,
        };
        // `<col> as i  ln(i)  ( sep  same(i)  ln(i) )*`
        let block = |open: Pattern| {
            Pattern::seq(vec![
                open,
                call("ln", "i"),
                Pattern::repeat(Pattern::seq(vec![
                    Pattern::nt("sep"),
                    comb(IndentOp::Same, IndentArg::Local("i".to_string()), None),
                    call("ln", "i"),
                ])),
            ])
        };

        let mut rules = HashMap::new();
        rules.insert("root".to_string(), Pattern::nt("doc"));
        rules.insert(
            "doc".to_string(),
            block(comb(IndentOp::AtLeast, IndentArg::Lit(0), Some("i"))),
        );
        rules.insert(
            "ln".to_string(),
            Pattern::seq(vec![
                Pattern::capture(
                    "text",
                    Pattern::char_class(CharSet::single_range('a', 'z').unwrap()),
                ),
                Pattern::optional(call("sub", "cur")),
            ]),
        );
        rules.insert(
            "sub".to_string(),
            block(comb(
                IndentOp::Deeper,
                IndentArg::Local("outer".to_string()),
                Some("i"),
            )),
        );
        rules.insert("sep".to_string(), Pattern::literal("\n"));

        let atomic_rules: HashSet<String> = ["doc", "ln", "sub", "sep"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut rule_params = HashMap::new();
        rule_params.insert("ln".to_string(), vec!["cur".to_string()]);
        rule_params.insert("sub".to_string(), vec!["outer".to_string()]);

        Grammar {
            rules,
            root: "root".to_string(),
            atomic_rules,
            percent_rules: HashSet::new(),
            preferred_rules: HashSet::new(),
            rule_headers: Vec::new(),
            rule_params,
            layout: None,
        }
    }

    /// The decisive test: the desugared `%`-grammar compiles to byte-for-
    /// byte the same bytecode as the hand-written explicit IR. Proves the
    /// surface is a pure restatement of the kept internals.
    #[test]
    fn desugars_to_byte_identical_bytecode() {
        let actual = compile(PCT_SRC).expect("%-grammar compiles");
        let expected = handwritten_explicit()
            .compile()
            .expect("explicit IR compiles");
        assert_eq!(
            actual.code, expected.code,
            "desugared bytecode diverged from the hand-written explicit IR"
        );
        assert_eq!(actual.capture_kinds, expected.capture_kinds);
        assert_eq!(actual.rule_names, expected.rule_names);
        assert_eq!(actual.char_sets, expected.char_sets);
    }

    /// Effect-selective threading: only rules that read an *inherited*
    /// anchor get the implicit `$indent` parameter. A `%root`-only seeding
    /// rule, and a pure-whitespace rule, stay parameterless — so their memo
    /// key stays the allocation-free zero-argument key.
    #[test]
    fn only_anchor_reading_rules_get_the_implicit_param() {
        let mut g = parse(PCT_SRC).expect("parses");
        desugar_indent(&mut g).expect("desugars");

        assert!(
            !g.rule_params.contains_key("doc"),
            "`doc` only seeds via %root; it reads no inherited anchor"
        );
        assert!(
            !g.rule_params.contains_key("sep"),
            "`sep` is pure whitespace"
        );
        let one_anchor = vec![ANCHOR_PARAM.to_string()];
        assert_eq!(g.rule_params.get("ln"), Some(&one_anchor));
        assert_eq!(g.rule_params.get("sub"), Some(&one_anchor));

        for (name, body) in &g.rules {
            assert!(
                !contains_indent_op(body),
                "no `%`-operator may survive desugaring (rule `{name}`)"
            );
        }
    }

    /// A `%align` reachable from the root with no enclosing `%root` /
    /// `%indent` to seed the column is a compile error, not a silent
    /// mis-parse.
    #[test]
    fn unanchored_operator_is_rejected() {
        let src = "root = bare {
    bare: atomic = %align
}";
        match compile(src) {
            Err(Error::Compile(CompileError::IndentAnchorOutOfScope { rule, .. })) => {
                assert_eq!(rule, "root");
            }
            other => panic!("expected IndentAnchorOutOfScope, got {other:?}"),
        }
    }

    fn contains_indent_op(pat: &Pattern) -> bool {
        match pat {
            Pattern::IndentOp { .. } => true,
            Pattern::Sequence { items, .. } => items.iter().any(contains_indent_op),
            Pattern::OrderedChoice { alts, .. } => alts.iter().any(contains_indent_op),
            Pattern::Repeat { inner, .. }
            | Pattern::RepeatOne { inner, .. }
            | Pattern::Optional { inner, .. }
            | Pattern::NotPredicate { inner, .. }
            | Pattern::AndPredicate { inner, .. }
            | Pattern::Capture { inner, .. }
            | Pattern::InferBoundaryCatch { inner, .. } => contains_indent_op(inner),
            Pattern::Catch {
                inner, recovery, ..
            } => contains_indent_op(inner) || contains_indent_op(recovery),
            Pattern::Literal { .. }
            | Pattern::CharClass { .. }
            | Pattern::AnyChar { .. }
            | Pattern::NonTerminal { .. }
            | Pattern::IndentCombinator { .. } => false,
        }
    }
}
