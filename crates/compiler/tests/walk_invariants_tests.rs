//! End-to-end regression tests for `walk` over real `Parser`-emitted
//! captures (not synthetic ones). Locks the producer-side contract
//! that walk's tie-breaking depends on: captures arrive in
//! `CaptureBegin` order. If the VM ever switches to `CaptureClose`
//! emission order, the shared-end case below breaks loudly.

#[path = "common/mod.rs"]
mod common;

use common::segments;

/// Inner capture's `end` coincides with the outer's:
/// `start = @outer body`, `body = 'aaa' @inner 'bbb'`.
/// On input `aaabbb`, outer covers 0..6 and inner covers 3..6.
const SHARED_END_GRAMMAR: &str = "\
root = @outer body {
body = 'aaa' @inner 'bbb'
}
";

#[test]
fn shared_end_keeps_inner_kind_active_to_the_end() {
    let segs = segments(SHARED_END_GRAMMAR, "aaabbb");
    assert_eq!(
        segs,
        vec![
            (0..3, Some("outer".to_string())),
            (3..6, Some("inner".to_string())),
        ],
        "shared-end: inner kind must extend to the shared end"
    );
}
