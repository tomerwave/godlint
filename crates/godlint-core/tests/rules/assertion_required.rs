use godlint_core::{
    config::Severity,
    rules::{Violation, assertion_required},
};

use super::support::{rule_findings, rule_violations};

const ENABLED: &str = "version: 1\nrules:\n  testing/assertion-required:\n    severity: error\n";

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(assertion_required::evaluate, path, source, configuration)
}

fn reported(path: &str, source: &str) -> Vec<Violation> {
    violations(path, source, ENABLED)
}

#[test]
fn reports_a_test_that_asserts_nothing_in_each_language() {
    let cases = [
        (
            "tests/test_refunds.py",
            "def test_refund():\n    process_refund(order)\n",
        ),
        (
            "tests/refunds.spec.js",
            "it('refunds', () => {\n  processRefund(order);\n});\n",
        ),
        (
            "tests/refunds.spec.ts",
            "it('refunds', () => {\n  processRefund(order);\n});\n",
        ),
        (
            "tests/refunds.rs",
            "#[test]\nfn refunds() {\n    process_refund(order);\n}\n",
        ),
    ];

    for (path, source) in cases {
        assert_eq!(
            reported(path, source),
            vec![Violation::MissingAssertion],
            "a test that only calls the code passes unless it raises: {path}"
        );
    }
}

#[test]
fn reports_at_warning_even_where_it_is_configured_at_error() {
    let findings = rule_findings(
        assertion_required::evaluate,
        "tests/test_refunds.py",
        "def test_refund():\n    process_refund(order)\n",
        ENABLED,
    );

    assert_eq!(
        findings.first().map(|finding| finding.severity),
        Some(Severity::Warning),
        "whether a helper asserts for the test is not decidable, so this rule cannot fail a build"
    );
}

#[test]
fn keeps_a_test_that_asserts() {
    let cases = [
        (
            "tests/test_refunds.py",
            "def test_refund():\n    assert process_refund(order).ok\n",
        ),
        (
            "tests/test_refunds.py",
            "def test_refund():\n    self.assertTrue(process_refund(order).ok)\n",
        ),
        (
            "tests/refunds.spec.js",
            "it('refunds', () => {\n  expect(processRefund(order)).toBe(1);\n});\n",
        ),
        (
            "tests/refunds.rs",
            "#[test]\nfn refunds() {\n    assert_eq!(process_refund(order), 1);\n}\n",
        ),
    ];

    for (path, source) in cases {
        assert!(reported(path, source).is_empty(), "{path} {source}");
    }
}

#[test]
fn keeps_a_test_whose_assertion_is_that_it_raises() {
    assert!(
        reported(
            "tests/test_refunds.py",
            "def test_refund():\n    with pytest.raises(ClosedOrder):\n        process_refund(o)\n"
        )
        .is_empty()
    );
    assert!(
        reported(
            "tests/refunds.rs",
            "#[test]\n#[should_panic]\nfn refunds() {\n    process_refund(order);\n}\n"
        )
        .is_empty(),
        "the attribute is the assertion; reporting it would hit every Rust repository"
    );
}

#[test]
fn keeps_an_empty_test_for_the_rule_that_owns_it() {
    assert!(
        reported("tests/test_refunds.py", "def test_later():\n    pass\n").is_empty(),
        "an empty test is no-empty-test's finding; two findings for one defect is noise"
    );
    assert!(reported("tests/refunds.rs", "#[test]\nfn later() {}\n").is_empty());
    assert!(
        reported("tests/refunds.spec.js", "it.todo('later');\n").is_empty(),
        "a test with no body to read is no-skipped-test's finding"
    );
}

#[test]
fn keeps_a_suite_that_holds_the_asserting_tests() {
    assert_eq!(
        reported(
            "tests/refunds.spec.js",
            concat!(
                "describe('refunds', () => {\n",
                "  it('refunds', () => {\n    expect(refund()).toBe(1);\n  });\n",
                "});\n"
            )
        )
        .len(),
        0,
        "a suite asserts through its tests; reporting the describe would double every finding"
    );
    assert_eq!(
        reported(
            "tests/refunds.spec.js",
            concat!(
                "describe('refunds', () => {\n",
                "  it('refunds', () => {\n    refund();\n  });\n",
                "});\n"
            )
        )
        .len(),
        1,
        "the test inside is still reported once, and the suite around it is not"
    );
}

#[test]
fn keeps_a_test_delegating_to_a_configured_helper() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  testing/assertion-required:\n",
        "    severity: error\n",
        "    extra-assertions:\n",
        "      - verify_refund\n",
        "      - order.assert_settled\n"
    );

    assert!(
        violations(
            "tests/test_refunds.py",
            "def test_refund():\n    verify_refund(order)\n",
            configuration,
        )
        .is_empty(),
        "a repository that asserts through helpers names them rather than turning the rule off"
    );
    assert!(
        violations(
            "tests/test_refunds.py",
            "def test_refund():\n    order.assert_settled()\n",
            configuration,
        )
        .is_empty()
    );
    assert_eq!(
        violations(
            "tests/test_refunds.py",
            "def test_refund():\n    check_refund(order)\n",
            configuration,
        )
        .len(),
        1,
        "a helper that was not named is still not an assertion"
    );
}

#[test]
fn keeps_a_function_that_is_not_a_test() {
    assert!(
        reported(
            "tests/test_refunds.py",
            "def helper():\n    process_refund(order)\n"
        )
        .is_empty()
    );
}

#[test]
fn does_not_count_an_assertion_in_a_neighbouring_test() {
    assert_eq!(
        reported(
            "tests/test_refunds.py",
            concat!(
                "def test_asserts():\n    assert refund().ok\n\n",
                "def test_does_not():\n    refund()\n"
            )
        )
        .len(),
        1,
        "assertions count per test, not per file"
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration = "version: 1\nrules:\n  testing/assertion-required:\n    severity: off\n";

    assert!(
        violations(
            "tests/test_refunds.py",
            "def test_refund():\n    process_refund(order)\n",
            configuration,
        )
        .is_empty()
    );
}
