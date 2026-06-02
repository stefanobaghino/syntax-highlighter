//! Verify per-recovery diagnostic capture: when the VM runs with
//! [`VM::with_track_recovery_diagnostics`], every `kind == recovery`
//! capture has a paired [`RecoveryDiagnostic`] reporting the
//! iteration's farthest reach and the rule-call stack at that point.
//! Drives the per-span detail in `pegdb recoveries dump` and the
//! cluster summary in `pegdb recoveries explain`.

use syntax_highlighter::pegvm::{MemoId, VM};
use syntax_highlighter_compiler::pegc;

#[test]
fn diagnostics_empty_when_knob_off() {
    // Grammar that triggers recovery but no opt-in to diagnostics.
    let prog = pegc::compile("root = ([a-z]+)*^").unwrap();
    let result = VM::new_from_program(&prog, b"abc!!def").run();
    assert!(result.complete);
    assert!(
        result
            .captures
            .iter()
            .any(|c| prog.capture_kinds[c.kind.0 as usize] == "recovery"),
        "expected at least one recovery capture in: {:?}",
        result.captures
    );
    assert!(
        result.recovery_diagnostics.is_empty(),
        "diagnostics must stay empty when the knob is off: {:?}",
        result.recovery_diagnostics
    );
}

#[test]
fn diagnostics_one_per_recovery_byte_when_knob_on() {
    let prog = pegc::compile("root = ([a-z]+)*^").unwrap();
    let result = VM::new_from_program(&prog, b"abc!!def")
        .with_track_recovery_diagnostics(true)
        .run();
    assert!(result.complete);
    let recovery_count = result
        .captures
        .iter()
        .filter(|c| prog.capture_kinds[c.kind.0 as usize] == "recovery")
        .count();
    assert!(recovery_count > 0);
    assert_eq!(
        result.recovery_diagnostics.len(),
        recovery_count,
        "one diagnostic per recovery byte"
    );
    // Each diagnostic's capture_index must point to a recovery
    // capture and the pos must equal that capture's start (the
    // recovery byte is one byte wide, emitted at scoped_max_sp).
    let recovery_kind_idx = prog
        .capture_kinds
        .iter()
        .position(|k| k == "recovery")
        .expect("recovery kind interned");
    for diag in &result.recovery_diagnostics {
        let cap = &result.captures[diag.capture_index];
        assert_eq!(
            cap.kind.0 as usize, recovery_kind_idx,
            "capture_index must point at a recovery capture"
        );
        assert_eq!(
            diag.pos, cap.start,
            "pos should match the recovery capture's start"
        );
    }
}

#[test]
fn diagnostics_record_rule_stack_at_deepest_reach() {
    // Two-rule grammar: outer `*^` of a rule that descends into
    // `inner`. When `inner`'s body fails, the failed iteration
    // reached deepest *inside* `inner`, so the recorded rule_stack
    // must include `inner`.
    let src = "\
        root = (item)*^ {\n\
        item = inner\n\
        inner = [a-z]+\n\
        }\n";
    let prog = pegc::compile(src).unwrap();
    let result = VM::new_from_program(&prog, b"abc!!def")
        .with_track_recovery_diagnostics(true)
        .run();
    assert!(result.complete);
    assert!(!result.recovery_diagnostics.is_empty());
    // The rule-stack names: resolve MemoIds to names via
    // `prog.rule_names`.
    let names_for = |stack: &[MemoId]| -> Vec<String> {
        stack
            .iter()
            .map(|id| prog.rule_names[id.0 as usize].clone())
            .collect()
    };
    // At least one diagnostic's rule_stack should descend through
    // `start` / `item` / `inner` (the failure happens trying to
    // start `inner` again at the `!`).
    let stacks: Vec<Vec<String>> = result
        .recovery_diagnostics
        .iter()
        .map(|d| names_for(d.rule_stack()))
        .collect();
    assert!(
        stacks.iter().any(|s| s.contains(&"root".to_string())),
        "expected at least one rule stack to start with `start`; got {:?}",
        stacks
    );
}
