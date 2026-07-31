use godlint_core::{
    config::{Config, Severity},
    rules::{closest_rule_id, configured_severity, is_known_rule, rule_ids},
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
        "maintainability/condition-complexity" => "max-operators",
        "maintainability/cognitive-complexity" => "max-score",
        "maintainability/return-count" => "max-returns",
        "maintainability/function-statements" => "max-statements",
        "ci/no-inline-script" => "max-lines",
        "ci/no-monolithic-job" => "max-steps",
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

#[test]
fn a_near_miss_is_offered_the_rule_it_probably_meant() {
    assert_eq!(
        closest_rule_id("maintainability/function-siz"),
        Some("maintainability/function-size")
    );
    assert_eq!(
        closest_rule_id("testing/no-focussed-test"),
        Some("testing/no-focused-test"),
        "a doubled letter is one edit"
    );
    assert_eq!(
        closest_rule_id("maintainability/function-size"),
        Some("maintainability/function-size"),
        "an exact name is its own closest match"
    );
}

#[test]
fn a_name_no_rule_resembles_is_offered_nothing() {
    assert_eq!(closest_rule_id("testing/from-a-later-release"), None);
    assert_eq!(closest_rule_id(""), None);
    assert_eq!(
        closest_rule_id("maintainability/no-comments"),
        None,
        "the family is most of the name, so naming the wrong one is not a near miss"
    );
}

#[test]
fn the_suggestion_stops_where_a_guess_starts() {
    assert_eq!(
        closest_rule_id("style/no-commen"),
        Some("style/no-comments"),
        "three edits away is still worth offering"
    );
    assert_eq!(
        closest_rule_id("style/no-comm"),
        None,
        "four edits away is a different name, and guessing would be worse than silence"
    );
    assert_eq!(
        closest_rule_id("style/no-comments-v2"),
        Some("style/no-comments"),
        "three characters too many is still a near miss"
    );
    assert_eq!(
        closest_rule_id("style/no-comments-two"),
        None,
        "and four too many is not"
    );
    assert_eq!(
        closest_rule_id("cccstyle/no-comments"),
        Some("style/no-comments"),
        "the extra characters may be at the front"
    );
    assert_eq!(
        closest_rule_id("ccccstyle/no-comments"),
        None,
        "four at the front is four: a deletion counted one too cheaply hides here and nowhere else"
    );
}
