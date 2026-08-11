use super::support::rule_violations;
use godlint_core::rules::{Violation, redundant_catch_rethrow};

#[test]
fn reports_bare_reraise() {
    let config =
        "version: 1\nrules:\n  reliability/redundant-catch-rethrow:\n    severity: error\n";
    assert!(matches!(
        rule_violations(
            redundant_catch_rethrow::evaluate,
            "x.py",
            "try:\n    work()\nexcept Exception:\n    raise\n",
            config
        )
        .as_slice(),
        [Violation::RedundantCatchRethrow]
    ));
}
