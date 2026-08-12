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

fn go_violations(source: &str) -> Vec<Violation> {
    rule_violations(
        network_timeout_required::evaluate,
        "src/client.go",
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

#[test]
fn accepts_positional_timeout_for_standard_library_clients() {
    assert!(violations("import urllib.request\nurllib.request.urlopen(url, 5)\n").is_empty());
    assert!(violations("import socket\nsocket.create_connection(address, 5)\n").is_empty());
    assert!(violations("requests.Session().get(url)\n").is_empty());
}

#[test]
fn enforces_go_timeout_variants_without_affecting_other_languages() {
    assert_eq!(
        go_violations("package client\nimport \"net/http\"\nfunc call() { http.Get(url) }\n").len(),
        1
    );
    assert!(go_violations("package client\nfunc call() { http.Get(url) }\n").len() == 1);
    assert!(
        go_violations(
            "package client\nimport \"net\"\nfunc call() { net.DialTimeout(\"tcp\", address, 5) }\n"
        )
        .is_empty()
    );
    assert_eq!(
        go_violations(
            "package client\nimport \"net\"\nfunc call() { net.Dial(\"tcp\", address) }\n"
        )
        .len(),
        1
    );
    assert!(violations("import requests\nrequests.get(url)\n").len() == 1);
    assert!(
        rule_violations(
            network_timeout_required::evaluate,
            "src/client.js",
            "http.Get(url);",
            "version: 1\nrules:\n  reliability/network-timeout-required:\n    severity: error\n",
        )
        .is_empty()
    );
}
