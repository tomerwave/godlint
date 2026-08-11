use super::support::rule_violations;
use godlint_core::rules::{Violation, no_control_flow_in_finally};

#[test]
fn reports_control_flow_in_finally() {
    let config =
        "version: 1\nrules:\n  reliability/no-control-flow-in-finally:\n    severity: error\n";
    assert!(matches!(
        rule_violations(
            no_control_flow_in_finally::evaluate,
            "x.py",
            "try:\n    work()\nfinally:\n    return\n",
            config
        )
        .as_slice(),
        [Violation::NoControlFlowInFinally]
    ));
}
