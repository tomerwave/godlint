use godlint_core::{
    config::{Config, Severity},
    rules::{configured_severity, is_known_rule, rule_ids},
};

fn config(body: &str) -> Config {
    yaml_serde::from_str(body).unwrap_or_else(|error| panic!("reads configuration: {error}"))
}

fn severity_of(rule_id: &str, enabled: &str) -> Severity {
    let body = format!(
        "version: 1\nrules:\n  {enabled}:\n    severity: error\n{}",
        limits(enabled)
    );

    configured_severity(&config(&body), rule_id)
}

fn limits(rule_id: &str) -> String {
    let key = match rule_id {
        "maintainability/function-size" | "maintainability/file-size" => "max-lines",
        "maintainability/function-nesting" => "max-depth",
        "maintainability/parameter-count" => "max-parameters",
        "maintainability/decision-complexity" => "max-complexity",
        "maintainability/return-count" => "max-returns",
        "maintainability/function-statements" => "max-statements",
        _ => return String::new(),
    };

    format!("    {key}: 1\n")
}

#[test]
fn every_registered_rule_reads_its_own_configuration() {
    let all: Vec<&str> = rule_ids().collect();

    assert!(all.len() > 1, "the registry lists nothing to check");

    for enabled in &all {
        assert_eq!(
            severity_of(enabled, enabled),
            Severity::Error,
            "{enabled} does not read its own severity"
        );

        for other in all.iter().filter(|other| *other != enabled) {
            assert_eq!(
                severity_of(other, enabled),
                Severity::Off,
                "{other} reads {enabled}'s configuration"
            );
        }
    }
}

#[test]
fn an_absent_rule_is_off_rather_than_unknown() {
    let empty = config("version: 1\n");

    for identifier in rule_ids() {
        assert_eq!(
            configured_severity(&empty, identifier),
            Severity::Off,
            "{identifier} must do nothing until a repository asks for it"
        );
        assert!(is_known_rule(identifier));
    }
}

#[test]
fn an_unregistered_identifier_is_not_known() {
    assert!(!is_known_rule("maintainability/no-such-rule"));
    assert_eq!(
        configured_severity(&config("version: 1\n"), "maintainability/no-such-rule"),
        Severity::Off
    );
}
