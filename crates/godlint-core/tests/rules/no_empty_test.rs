use godlint_core::rules::{Violation, no_empty_test};

use super::support::rule_violations;

const ENABLED: &str = "version: 1\nrules:\n  testing/no-empty-test:\n    severity: error\n";

fn violations(path: &str, source: &str) -> Vec<Violation> {
    rule_violations(no_empty_test::evaluate, path, source, ENABLED)
}

#[test]
fn reports_an_empty_test_in_each_language() {
    assert_eq!(violations("a.js", "it('x', () => {});").len(), 1);
    assert_eq!(violations("a.py", "def test_x():\n    pass").len(), 1);
    assert_eq!(violations("a.rs", "#[test]\nfn x() {}").len(), 1);
}

#[test]
fn leaves_a_test_that_does_something_alone() {
    assert!(violations("a.js", "it('x', () => { check(); });").is_empty());
    assert!(violations("a.py", "def test_x():\n    check()").is_empty());
    assert!(violations("a.rs", "#[test]\nfn x() {\n    check();\n}").is_empty());
}

#[test]
fn says_nothing_about_a_test_with_no_body_to_read() {
    assert!(
        violations("a.js", "it.todo('later');").is_empty(),
        "a todo test has no callback at all, and not running is the other rule's finding"
    );
}

#[test]
fn ignores_an_empty_function_that_is_not_a_test() {
    assert!(
        violations("a.js", "function helper() {}").is_empty(),
        "an empty function outside a test is maintainability/empty-function's business"
    );
    assert!(violations("a.py", "def helper():\n    pass").is_empty());
}

#[test]
fn reads_the_test_body_rather_than_a_callback_inside_it() {
    assert!(
        violations("a.js", "it('x', () => { register(() => {}); });").is_empty(),
        "the test does something; the empty function it passes on is not the test's body"
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration = "version: 1\nrules:\n  testing/no-empty-test:\n    severity: off\n";

    assert!(
        rule_violations(
            no_empty_test::evaluate,
            "a.js",
            "it('x', () => {});",
            configuration
        )
        .is_empty()
    );
}
