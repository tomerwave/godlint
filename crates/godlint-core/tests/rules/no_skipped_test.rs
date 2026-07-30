use godlint_core::rules::{Violation, no_skipped_test};

use super::support::rule_violations;

const ENABLED: &str = "version: 1\nrules:\n  testing/no-skipped-test:\n    severity: error\n";

fn violations(path: &str, source: &str) -> Vec<Violation> {
    rule_violations(no_skipped_test::evaluate, path, source, ENABLED)
}

#[test]
fn reports_a_skipped_test_in_each_language() {
    assert_eq!(violations("a.js", "it.skip('x', () => {});").len(), 1);
    assert_eq!(
        violations("a.js", "it.todo('x');").len(),
        1,
        "a todo test does not run, which is what skipped means"
    );
    assert_eq!(violations("a.rs", "#[test]\n#[ignore]\nfn x() {}").len(), 1);
    assert_eq!(
        violations("a.py", "@pytest.mark.skip\ndef test_x():\n    check()").len(),
        1
    );
}

#[test]
fn reads_the_rust_attributes_in_either_order() {
    assert_eq!(violations("a.rs", "#[ignore]\n#[test]\nfn x() {}").len(), 1);
}

#[test]
fn leaves_a_running_test_alone() {
    assert!(violations("a.js", "it('x', () => {});").is_empty());
    assert!(
        violations("a.js", "it.only('x', () => {});").is_empty(),
        "a focused test is the other rule's business"
    );
    assert!(violations("a.rs", "#[test]\nfn x() {}").is_empty());
    assert!(
        violations("a.rs", "#[ignore]\nfn x() {}").is_empty(),
        "ignore without test is not a test at all"
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration = "version: 1\nrules:\n  testing/no-skipped-test:\n    severity: off\n";

    assert!(
        rule_violations(
            no_skipped_test::evaluate,
            "a.js",
            "it.skip('x', () => {});",
            configuration
        )
        .is_empty()
    );
}
