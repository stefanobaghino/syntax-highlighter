//! CI gate: every fixture under `benches/fixtures/**` parses with zero
//! `recovery` captures.
//!
//! Per #75, this watches a channel the rest of CI doesn't: `*^` converts
//! grammar gaps into silent recovery captures with exit 0, so a bug only
//! becomes visible if someone runs `pegdb captures dump` and inspects
//! the JSONL. Asserting `recovery == 0` across the bench corpus makes
//! regressions fail at PR time.
//!
//! Mechanism is exact-match on raw `Capture` count, matching the
//! "one JSON object per capture" definition that `pegdb captures dump`
//! uses. Hand-written malformed-input tests that intentionally exercise
//! `*^` live alongside their grammar's `tests/grammar_<lang>_tests.rs`;
//! the channels stay separate.

use std::fs;
use std::path::PathBuf;

use syntax_highlighter::pegc;
use syntax_highlighter::pegvm::VM;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn grammar_for(ext: &str) -> Option<&'static str> {
    match ext {
        "c" => Some("c"),
        "css" => Some("css"),
        "go" => Some("go"),
        "js" => Some("javascript"),
        "json" => Some("json"),
        "rs" => Some("rust"),
        "sql" => Some("sqlite"),
        "toml" => Some("toml"),
        _ => None,
    }
}

fn count_recoveries(grammar_src: &str, input: &[u8]) -> usize {
    let prog = pegc::compile(grammar_src).expect("grammar should compile");
    // Grammars without `*^` don't register the `recovery` kind at all —
    // by construction they can't emit recoveries, so the count is 0.
    let Some(recovery_idx) = prog.capture_kinds.iter().position(|k| k == "recovery") else {
        return 0;
    };
    let r = VM::new(&prog.code, input).run();
    r.captures
        .iter()
        .filter(|c| c.kind.0 as usize == recovery_idx)
        .count()
}

#[test]
fn benches_fixtures_have_no_recovery_captures() {
    let fixtures_dir = crate_root().join("benches").join("fixtures");
    let mut entries: Vec<PathBuf> = fs::read_dir(&fixtures_dir)
        .expect("benches/fixtures/ exists")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .collect();
    entries.sort();

    assert!(
        !entries.is_empty(),
        "expected at least one fixture under {}",
        fixtures_dir.display()
    );

    let mut grammars: std::collections::HashMap<String, String> = Default::default();
    let mut mismatches: Vec<(PathBuf, usize)> = Vec::new();

    for fixture in &entries {
        let ext = fixture
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        let Some(grammar_name) = grammar_for(ext) else {
            continue;
        };
        let grammar_src = grammars
            .entry(grammar_name.to_string())
            .or_insert_with(|| {
                let path: PathBuf = crate_root()
                    .join("grammars")
                    .join(format!("{grammar_name}.peg"));
                fs::read_to_string(&path).unwrap_or_else(|e| {
                    panic!("read grammar {}: {}", path.display(), e);
                })
            })
            .clone();
        let input = fs::read(fixture).expect("read fixture");
        let count = count_recoveries(&grammar_src, &input);
        if count != 0 {
            mismatches.push((fixture.clone(), count));
        }
    }

    if !mismatches.is_empty() {
        let mut msg = String::from(
            "Unexpected `recovery` captures on bench fixtures (see #75).\n\
             Each entry below is a fixture where the parser fell into the `*^` recovery\n\
             loop. Likely cause: a grammar bug producing silent recoveries. Run\n\
             `pegdb captures dump -g grammars/<grammar>.peg <fixture>` to inspect.\n\n",
        );
        for (path, count) in &mismatches {
            let rel = path
                .strip_prefix(crate_root())
                .unwrap_or(path)
                .display()
                .to_string();
            msg.push_str(&format!("  {rel}: {count}\n"));
        }
        panic!("{msg}");
    }
}

#[test]
fn every_fixture_extension_has_a_grammar_mapping() {
    // Drift guard: if a new fixture lands under benches/fixtures/<size>.<ext>
    // and we don't have a grammar mapping for `.ext`, the recovery gate
    // would silently skip it. Force an explicit decision here.
    let fixtures_dir = crate_root().join("benches").join("fixtures");
    let mut unmapped: Vec<String> = Vec::new();
    for entry in fs::read_dir(&fixtures_dir).expect("benches/fixtures/ exists") {
        let path = entry.expect("dir entry").path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        if grammar_for(ext).is_none() {
            unmapped.push(format!(".{ext} ({})", path.display()));
        }
    }
    assert!(
        unmapped.is_empty(),
        "fixture extensions with no grammar mapping in recovery_baseline_tests.rs::grammar_for: {unmapped:?}"
    );
}

// Internal sanity: a deliberately malformed fixture *does* produce
// recoveries, so the gate is wired up (and not just observing zero
// because of a misconfigured count).
#[test]
fn gate_fires_on_malformed_input() {
    // `@@@` is unparseable at item position in every grammar covered by
    // the bench corpus, so it triggers the top-level `*^` byte-eater.
    // Pick Rust arbitrarily; the assertion is about the gate's
    // mechanics, not the grammar.
    let grammar =
        fs::read_to_string(crate_root().join("grammars/rust.peg")).expect("read rust.peg");
    let input = b"fn a() {}\n@@@ garbage @@@\nfn b() {}\n";
    let count = count_recoveries(&grammar, input);
    assert!(
        count > 0,
        "malformed input should produce recovery captures so the gate is real"
    );
}
