use super::support::rule_violations;
use godlint_core::rules::{Violation, no_duplicate_string};

#[test]
fn reports_repeated_long_literal() {
    let config =
        "version: 1\nrules:\n  maintainability/no-duplicate-string:\n    severity: error\n";
    let findings = rule_violations(
        no_duplicate_string::evaluate,
        "x.py",
        "a = \"https://example.com/a-very-long-value\"\nb = \"https://example.com/a-very-long-value\"\n",
        config,
    );
    assert_eq!(findings.len(), 2);
    assert!(
        findings
            .iter()
            .all(|finding| matches!(finding, Violation::DuplicateString { .. }))
    );
}

#[test]
fn ignores_unterminated_values() {
    let config =
        "version: 1\nrules:\n  maintainability/no-duplicate-string:\n    severity: error\n";
    assert!(rule_violations(no_duplicate_string::evaluate, "x.py", "value =\n", config).is_empty());
}
