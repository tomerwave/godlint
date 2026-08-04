use std::path::PathBuf;

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    config::{PinThirdPartyActionsRule, Severity},
    rules::{Violation, evaluate_action_rule, pin_third_party_actions::PinThirdPartyActions},
    source::TextFile,
};

fn workflow(body: &str) -> WorkflowFacts {
    let file = TextFile::new(PathBuf::from(".github/workflows/ci.yml"), body.into())
        .unwrap_or_else(|error| panic!("creates workflow: {error}"));

    workflow::read(&file).unwrap_or_else(|error| panic!("reads workflow: {error}"))
}

fn steps(uses: &[&str]) -> String {
    let mut body = String::from("jobs:\n  build:\n    steps:\n");

    for used in uses {
        body.push_str(&format!("      - uses: {used}\n"));
    }

    body
}

fn configuration(trusted: &[&str]) -> PinThirdPartyActionsRule {
    PinThirdPartyActionsRule {
        severity: Severity::Error,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        trusted_owners: trusted.iter().map(|owner| (*owner).to_owned()).collect(),
    }
}

fn violations(uses: &[&str], trusted: &[&str]) -> Vec<Violation> {
    let facts = workflow(&steps(uses));

    evaluate_action_rule::<PinThirdPartyActions>(
        std::slice::from_ref(&facts),
        &configuration(trusted),
    )
    .into_iter()
    .map(|finding| finding.violation)
    .collect()
}

fn references(uses: &[&str], trusted: &[&str]) -> Vec<String> {
    violations(uses, trusted)
        .into_iter()
        .map(|violation| violation.to_string())
        .collect()
}

#[test]
fn a_mutable_ref_is_reported_however_it_is_spelled() {
    let reported = references(
        &[
            "vendor/setup@v3",
            "vendor/setup@main",
            "vendor/setup@1.2.3",
            "vendor/setup@0f1a2b3",
        ],
        &[],
    );

    assert_eq!(
        reported.len(),
        4,
        "a tag, a branch, a version and a short SHA can all move: {reported:?}"
    );
}

#[test]
fn a_commit_is_the_only_thing_that_counts_as_pinned() {
    assert!(
        violations(
            &["vendor/setup@0f1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c"],
            &[]
        )
        .is_empty(),
        "forty hex characters name one commit and nothing else"
    );
    assert!(
        violations(
            &["vendor/setup@0f1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3g"],
            &[]
        )
        .len()
            == 1,
        "forty characters that are not all hexadecimal are a ref, not a commit"
    );
}

#[test]
fn a_trusted_owner_is_exempt_and_only_that_owner() {
    assert!(
        violations(&["actions/checkout@v4"], &["actions"]).is_empty(),
        "a repository decides whose tags it trusts"
    );
    assert_eq!(
        violations(&["vendor/setup@v4"], &["actions"]).len(),
        1,
        "trusting one owner must not trust every owner"
    );
    assert!(
        violations(&["Actions/checkout@v4"], &["actions"]).is_empty(),
        "GitHub owner names are case-insensitive"
    );
    assert_eq!(
        violations(&["actions/checkout@v4"], &[]).len(),
        1,
        "an empty list trusts nobody, which is the strict reading"
    );
}

#[test]
fn something_nobody_owns_is_not_a_third_party_action() {
    assert!(
        violations(
            &[
                "./.github/actions/setup",
                "docker://alpine:3.20",
                "docker://alpine",
            ],
            &[]
        )
        .is_empty(),
        "an action in this repository and a container image are not somebody else's tag"
    );
}

#[test]
fn a_reference_with_no_version_says_what_it_actually_runs() {
    let reported = references(&["vendor/setup"], &[]);

    assert_eq!(reported.len(), 1);
    assert!(
        reported[0].contains("names no version at all"),
        "no version is a different mistake from a movable one: {reported:?}"
    );
    assert!(reported[0].contains("default branch"), "{reported:?}");
}

#[test]
fn the_message_names_the_reference_and_what_to_do() {
    let reported = references(&["vendor/setup@v3"], &[]);

    assert!(
        reported[0].starts_with("vendor/setup@v3 is a mutable ref"),
        "{reported:?}"
    );
    assert!(
        reported[0].ends_with("pin it to a commit SHA."),
        "{reported:?}"
    );
}

#[test]
fn the_rule_is_silent_when_it_is_switched_off() {
    let facts = workflow(&steps(&["vendor/setup@v3"]));
    let configuration = PinThirdPartyActionsRule {
        severity: Severity::Off,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        trusted_owners: Vec::new(),
    };

    assert!(
        evaluate_action_rule::<PinThirdPartyActions>(std::slice::from_ref(&facts), &configuration)
            .is_empty()
    );
}
