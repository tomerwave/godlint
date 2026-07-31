use godlint_core::{
    config::Severity,
    rules::{Violation, no_duplicate_assertion},
};

use super::support::{rule_findings, rule_violations};

const ENABLED: &str =
    "version: 1\nrules:\n  testing/no-duplicate-assertion:\n    severity: error\n";

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(
        no_duplicate_assertion::evaluate,
        path,
        source,
        configuration,
    )
}

fn reported(path: &str, source: &str) -> Vec<Violation> {
    violations(path, source, ENABLED)
}

#[test]
fn reports_a_repeated_assertion_in_each_language() {
    let cases = [
        (
            "tests/test_total.py",
            "def test_total():\n    assert total == 100\n    assert total == 100\n",
        ),
        (
            "tests/total.rs",
            "#[test]\nfn total() {\n    assert_eq!(t, 100);\n    assert_eq!(t, 100);\n}\n",
        ),
        (
            "tests/total.spec.js",
            "it('t', () => {\n  expect(t).toBe(100);\n  expect(t).toBe(100);\n});\n",
        ),
    ];

    for (path, source) in cases {
        assert_eq!(
            reported(path, source).len(),
            1,
            "the repeat is reported once, not both occurrences: {path}"
        );
    }
}

#[test]
fn reports_at_warning_even_where_it_is_configured_at_error() {
    let findings = rule_findings(
        no_duplicate_assertion::evaluate,
        "tests/test_total.py",
        "def test_total():\n    assert total == 100\n    assert total == 100\n",
        ENABLED,
    );

    assert_eq!(
        findings.first().map(|finding| finding.severity),
        Some(Severity::Warning),
        "a repeat can be deliberate when something between the two changed state"
    );
}

#[test]
fn names_the_assertion_it_repeated() {
    let reported = reported(
        "tests/test_total.py",
        "def test_total():\n    assert total == 100\n    assert total == 100\n",
    );
    let message = reported.first().expect("reports the repeat").to_string();

    assert!(
        message.starts_with("assert total == 100 already ran"),
        "the message must quote the assertion: {message}"
    );
}

#[test]
fn keeps_two_assertions_that_check_different_things() {
    assert!(
        reported(
            "tests/test_total.py",
            "def test_total():\n    assert total == 100\n    assert state == 'charged'\n"
        )
        .is_empty()
    );
}

#[test]
fn keeps_two_matchers_on_the_same_value() {
    assert!(
        reported(
            "tests/total.spec.js",
            "it('t', () => {\n  expect(t).toBe(100);\n  expect(t).toBeGreaterThan(50);\n});\n"
        )
        .is_empty(),
        "the matcher is what the assertion checks, so these are two different assertions"
    );
}

#[test]
fn reads_an_assertion_through_the_whitespace_it_was_written_with() {
    assert_eq!(
        reported(
            "tests/total.rs",
            "#[test]\nfn total() {\n    assert_eq!(t, 100);\n    assert_eq!(t,  100);\n}\n"
        )
        .len(),
        1,
        "spacing is not what makes two assertions different"
    );
}

#[test]
fn keeps_the_same_assertion_in_two_different_tests() {
    assert!(
        reported(
            "tests/test_total.py",
            concat!(
                "def test_one():\n    assert total == 100\n\n",
                "def test_two():\n    assert total == 100\n"
            )
        )
        .is_empty(),
        "two tests checking the same thing are two tests, not a repeat"
    );
}

#[test]
fn keeps_a_suite_whose_tests_assert_the_same_thing() {
    assert!(
        reported(
            "tests/total.spec.js",
            concat!(
                "describe('s', () => {\n",
                "  it('one', () => {\n    expect(t).toBe(1);\n  });\n",
                "  it('two', () => {\n    expect(t).toBe(1);\n  });\n",
                "});\n"
            )
        )
        .is_empty(),
        "the suite encloses both, and reporting it would repeat every rule's finding per nesting level"
    );
}

#[test]
fn keeps_a_repeat_with_an_action_between_it() {
    assert!(
        reported(
            "tests/test_total.py",
            concat!(
                "def test_total():\n",
                "    rv = post('/more')\n    assert rv.status == 405\n",
                "    rv = delete('/more')\n    assert rv.status == 405\n"
            )
        )
        .is_empty(),
        "the second assertion checks the result of the second action, so it is not dead weight"
    );
}

#[test]
fn reads_a_chain_that_asserts_twice_as_one_span() {
    assert!(
        reported(
            "tests/total.spec.js",
            "it('t', () => {\n  request(app).get('/').expect(302).expect('Location', '/x');\n});\n"
        )
        .is_empty(),
        "a fluent chain asserting twice is one span, so the second is not a repeat of the first"
    );
}

#[test]
fn reports_every_repeat_beyond_the_first() {
    assert_eq!(
        reported(
            "tests/test_total.py",
            "def test_total():\n    assert t == 1\n    assert t == 1\n    assert t == 1\n"
        )
        .len(),
        2,
        "three copies is two repeats"
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration =
        "version: 1\nrules:\n  testing/no-duplicate-assertion:\n    severity: off\n";

    assert!(
        violations(
            "tests/test_total.py",
            "def test_total():\n    assert t == 1\n    assert t == 1\n",
            configuration,
        )
        .is_empty()
    );
}
