use godlint_core::rules::{Violation, network_timeout_required};

use super::support::rule_violations;

fn violations(source: &str) -> Vec<Violation> {
    rule_violations(
        network_timeout_required::evaluate,
        "src/client.py",
        source,
        "version: 1\nrules:\n  reliability/network-timeout-required:\n    severity: error\n",
    )
}

#[test]
fn reports_a_network_call_without_timeout() {
    assert_eq!(
        violations("import requests\nrequests.get(url)\n"),
        vec![Violation::NetworkTimeoutMissing {
            callee: "requests.get".to_owned()
        }]
    );
}

#[test]
fn accepts_an_explicit_timeout() {
    assert!(violations("import requests\nrequests.get(url, timeout=5)\n").is_empty());
}
