use godlint_core::rules::{Violation, no_focused_test};

use super::support::rule_violations;

const ENABLED: &str = "version: 1\nrules:\n  testing/no-focused-test:\n    severity: error\n";

fn violations(path: &str, source: &str) -> Vec<Violation> {
    rule_violations(no_focused_test::evaluate, path, source, ENABLED)
}

#[test]
fn reports_a_focused_test_and_a_focused_suite() {
    assert_eq!(violations("a.js", "it.only('x', () => {});").len(), 1);
    assert_eq!(violations("a.ts", "describe.only('x', () => {});").len(), 1);
}

#[test]
fn leaves_an_unfocused_test_alone() {
    assert!(violations("a.js", "it('x', () => {});").is_empty());
    assert!(
        violations("a.js", "it.skip('x', () => {});").is_empty(),
        "a skipped test is the other rule's business"
    );
    assert!(violations("a.rs", "#[test]\nfn x() {}").is_empty());
    assert!(violations("a.py", "def test_x():\n    check()").is_empty());
}

#[test]
fn can_disable_the_rule() {
    let configuration = "version: 1\nrules:\n  testing/no-focused-test:\n    severity: off\n";

    assert!(
        rule_violations(
            no_focused_test::evaluate,
            "a.js",
            "it.only('x', () => {});",
            configuration
        )
        .is_empty()
    );
}
