//! Layout-preserving `.peg` re-emission by source splicing.
//!
//! The [`Pattern`](super::Pattern) AST cannot be pretty-printed back
//! into faithful grammar source: `skip_ws` discards comments, and the
//! parser desugars eagerly (`{n}` expansion, `i"…"` case classes,
//! `..`/`..=` catch sugar, sync sets, `CharSet` normalization, scope
//! mangling) with no provenance, so an unedited rule printed from the
//! AST would not match what the author wrote. This module therefore
//! never prints unedited grammar: it *splices* — every byte of output
//! is either copied verbatim from the original source or is the text
//! of an explicit [`LayoutEdit`].
//!
//! Two contracts follow:
//!
//! - **Fidelity / tiling**: the offsets a [`RuleLayout`] records, plus
//!   the verbatim gaps between them, tile the whole source file, so
//!   [`emit`] with no edits reproduces the input byte-for-byte —
//!   comments, blank lines, and spelling included. The
//!   `layout_tests.rs` identity gate holds this over every shipped
//!   grammar.
//! - **Splice, don't print**: callers must work from the *parsed*
//!   grammar's layout ([`Grammar::layout`](super::Grammar::layout)),
//!   never from a compiled or otherwise transformed one — the layout's
//!   offsets are only meaningful against the exact source string that
//!   produced them.

use std::collections::{HashMap, HashSet};

use super::parser::RuleLayout;

/// One edit to apply during re-emission. Rules are addressed by flat
/// (mangled) name — the same key as
/// [`Grammar::rules`](super::Grammar::rules), e.g. `parent::child`.
#[derive(Debug, Clone)]
pub enum LayoutEdit {
    /// Replace a rule's body (its [`RuleLayout::body_range`]) with
    /// `text`, leaving everything around it — including trailing
    /// comments after the body — untouched.
    ReplaceBody { rule: String, text: String },
}

/// Failure mode of [`emit_with_edits`].
#[derive(Debug)]
pub enum EmitError {
    /// An edit addressed a rule name not present in the layout tree.
    UnknownRule(String),
}

impl std::fmt::Display for EmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmitError::UnknownRule(name) => {
                write!(f, "edit addresses unknown rule `{name}`")
            }
        }
    }
}

impl std::error::Error for EmitError {}

/// Re-emit `source` from its layout with no edits. Returns the input
/// byte-for-byte (the fidelity contract in the module docs).
///
/// `layout` must be the layout that [`parse`](super::parse) produced
/// for this exact `source`.
pub fn emit(source: &str, layout: &RuleLayout) -> String {
    emit_with_edits(source, layout, &[]).expect("emission without edits cannot fail")
}

/// Re-emit `source` with `edits` applied. Unedited bytes are copied
/// verbatim; each edited rule's body range is replaced by the edit's
/// text. When several edits address the same rule, the last one wins.
///
/// `layout` must be the layout that [`parse`](super::parse) produced
/// for this exact `source`.
pub fn emit_with_edits(
    source: &str,
    layout: &RuleLayout,
    edits: &[LayoutEdit],
) -> Result<String, EmitError> {
    let mut bodies: HashMap<&str, &str> = HashMap::new();
    for edit in edits {
        let LayoutEdit::ReplaceBody { rule, text } = edit;
        bodies.insert(rule.as_str(), text.as_str());
    }
    if !bodies.is_empty() {
        let mut known: HashSet<&str> = HashSet::new();
        collect_names(layout, &mut known);
        for rule in bodies.keys() {
            if !known.contains(rule) {
                return Err(EmitError::UnknownRule((*rule).to_string()));
            }
        }
    }

    let mut out = String::with_capacity(source.len());
    let mut cursor = 0usize;
    walk(source, layout, &bodies, &mut cursor, &mut out);
    // Trailing file trivia after the root's extent.
    out.push_str(&source[cursor..]);
    Ok(out)
}

fn collect_names<'a>(layout: &'a RuleLayout, out: &mut HashSet<&'a str>) {
    out.insert(layout.name.as_str());
    if let Some(block) = &layout.block {
        for child in &block.children {
            collect_names(child, out);
        }
    }
}

/// Copy `source[cursor..end]` to `out` and advance the cursor. The
/// half-open pieces handed to successive calls must be ordered — that
/// is the tiling invariant.
fn take(source: &str, cursor: &mut usize, end: usize, out: &mut String) {
    debug_assert!(*cursor <= end, "layout pieces out of order");
    debug_assert!(end <= source.len(), "layout offset past end of source");
    out.push_str(&source[*cursor..end]);
    *cursor = end;
}

/// Emit one rule: the gap before each recorded piece verbatim, then
/// the piece itself (or the replacement body), recursing into the
/// scope block's children between its braces.
fn walk(
    source: &str,
    rule: &RuleLayout,
    bodies: &HashMap<&str, &str>,
    cursor: &mut usize,
    out: &mut String,
) {
    take(source, cursor, rule.name_range.end, out);
    if let Some(asc) = &rule.ascriptions {
        take(source, cursor, asc.end, out);
    }
    take(source, cursor, rule.eq_byte + 1, out);
    take(source, cursor, rule.body_range.start, out);
    match bodies.get(rule.name.as_str()) {
        Some(text) => {
            out.push_str(text);
            *cursor = rule.body_range.end;
        }
        None => take(source, cursor, rule.body_range.end, out),
    }
    if let Some(block) = &rule.block {
        take(source, cursor, block.open_brace + 1, out);
        for child in &block.children {
            walk(source, child, bodies, cursor, out);
        }
        take(source, cursor, block.close_brace + 1, out);
    }
}
