use std::path::PathBuf;

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    config::{Config, Severity},
    rules::{Violation, stale_action_refs},
    source::TextFile,
};

const SHA_A: &str = "0f1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c";
const SHA_B: &str = "1f2a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c";

fn workflow_at(path: &str, body: &str) -> WorkflowFacts {
    let file = TextFile::new(PathBuf::from(path), body.into())
        .unwrap_or_else(|error| panic!("creates workflow: {error}"));

    workflow::read(&file).unwrap_or_else(|error| panic!("reads workflow: {error}"))
}

fn step(used: &str) -> String {
    format!("jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: {used}\n")
}

fn config(severity: &str, allow_in: &[&str]) -> Config {
    let paths = if allow_in.is_empty() {
        " []\n".to_owned()
    } else {
        format!(
            "\n{}",
            allow_in
                .iter()
                .map(|path| format!("      - \"{path}\"\n"))
                .collect::<String>()
        )
    };
    let source = format!(
        "version: 1\nrules:\n  ci/stale-action-refs:\n    severity: {severity}\n    allow-in:{paths}"
    );

    yaml_serde::from_str(&source).unwrap_or_else(|error| panic!("reads config: {error}"))
}

fn findings(workflows: &[WorkflowFacts], severity: &str) -> Vec<godlint_core::rules::Finding> {
    stale_action_refs::evaluate(workflows, &config(severity, &[]))
}

#[test]
fn an_unlabelled_commit_pin_is_a_warning() {
    let workflow = workflow_at(
        ".github/workflows/ci.yml",
        &step(&format!("vendor/setup@{SHA_A}")),
    );
    let found = findings(&[workflow], "error");

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].severity, Severity::Warning);
    assert!(matches!(
        found[0].violation,
        Violation::UnlabelledActionPin { .. }
    ));
}

#[test]
fn the_warning_cap_does_not_raise_a_lower_configured_severity() {
    let workflow = workflow_at(
        ".github/workflows/ci.yml",
        &step(&format!("vendor/setup@{SHA_A}")),
    );
    let found = findings(&[workflow], "info");

    assert_eq!(found[0].severity, Severity::Info);
}

#[test]
fn trailing_labels_are_read_from_every_action_scalar_form() {
    let body = format!(
        "jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: vendor/one@{SHA_A} # v1\n      - uses: 'vendor/two@{SHA_A}' # v2\n      - uses: \"vendor/three@{SHA_A}\" # 3.0.0\n  reuse:\n    uses: vendor/repository/.github/workflows/reuse.yml@{SHA_A} # v4\n"
    );
    let workflow = workflow_at(".github/workflows/ci.yml", &body);

    assert!(findings(&[workflow], "error").is_empty());
}

#[test]
fn a_comment_above_a_pin_is_not_its_label() {
    let body = format!(
        "jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      # v1\n      - uses: vendor/setup@{SHA_A}\n"
    );
    let workflow = workflow_at(".github/workflows/ci.yml", &body);

    assert!(matches!(
        findings(&[workflow], "error")[0].violation,
        Violation::UnlabelledActionPin { .. }
    ));
}

#[test]
fn the_same_action_and_sha_with_different_labels_reports_every_claim() {
    let first = workflow_at(
        ".github/workflows/ci.yml",
        &step(&format!("Actions/Checkout@{SHA_A} # v5")),
    );
    let second = workflow_at(
        ".github/workflows/release.yml",
        &step(&format!("actions/checkout@{} # v3", SHA_A.to_uppercase())),
    );
    let found = findings(&[first, second], "error");

    assert_eq!(found.len(), 2);
    assert!(found.iter().all(|finding| {
        finding.severity == Severity::Error
            && matches!(
                finding.violation,
                Violation::ContradictoryActionLabels { .. }
            )
    }));
}

#[test]
fn a_leading_v_does_not_change_a_label_within_or_across_workflows() {
    let first = workflow_at(
        ".github/workflows/ci.yml",
        &format!(
            "jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/upload-artifact@{SHA_A} # v4.6.2\n      - uses: actions/upload-artifact@{SHA_A} # 4.6.2\n"
        ),
    );
    let second = workflow_at(
        ".github/workflows/release.yml",
        &step(&format!("actions/upload-artifact@{SHA_A} # 4.6.2")),
    );

    assert!(findings(&[first, second], "error").is_empty());
}

#[test]
fn an_uppercase_leading_v_does_not_change_a_label() {
    let first = workflow_at(
        ".github/workflows/ci.yml",
        &step(&format!("actions/checkout@{SHA_A} # V4")),
    );
    let second = workflow_at(
        ".github/workflows/release.yml",
        &step(&format!("actions/checkout@{SHA_A} # 4")),
    );

    assert!(findings(&[first, second], "error").is_empty());
}

#[test]
fn the_same_action_and_label_with_different_shas_reports_every_claim() {
    let first = workflow_at(
        ".github/workflows/ci.yml",
        &step(&format!("actions/checkout@{SHA_A} # v4")),
    );
    let second = workflow_at(
        ".github/workflows/release.yml",
        &step(&format!("actions/checkout@{SHA_B} # V4")),
    );
    let found = findings(&[first, second], "error");

    assert_eq!(found.len(), 2);
    assert!(
        found
            .iter()
            .all(|finding| matches!(finding.violation, Violation::ContradictoryActionPins { .. }))
    );
}

#[test]
fn prefixed_and_unprefixed_labels_group_different_shas() {
    let first = workflow_at(
        ".github/workflows/ci.yml",
        &step(&format!("actions/checkout@{SHA_A} # v4")),
    );
    let second = workflow_at(
        ".github/workflows/release.yml",
        &step(&format!("actions/checkout@{SHA_B} # 4")),
    );
    let found = findings(&[first, second], "error");

    assert_eq!(found.len(), 2);
    assert!(
        found
            .iter()
            .all(|finding| matches!(finding.violation, Violation::ContradictoryActionPins { .. }))
    );
}

#[test]
fn text_before_an_internal_v_remains_part_of_the_label() {
    let agreeing = workflow_at(
        ".github/workflows/ci.yml",
        &format!(
            "jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: vendor/stable@{SHA_A} # release/v1\n      - uses: vendor/stable@{SHA_A} # release/v1\n"
        ),
    );
    let conflicting = workflow_at(
        ".github/workflows/release.yml",
        &format!(
            "jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: vendor/conflict@{SHA_A} # release/v1\n      - uses: vendor/conflict@{SHA_A} # release/v2\n"
        ),
    );
    let found = findings(&[agreeing, conflicting], "error");

    assert_eq!(found.len(), 2);
    assert!(found.iter().all(|finding| {
        matches!(
            finding.violation,
            Violation::ContradictoryActionLabels { .. }
        )
    }));
}

#[test]
fn contradictions_can_be_proved_within_one_workflow() {
    let body = format!(
        "jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: vendor/setup@{SHA_A} # v1\n      - uses: vendor/setup@{SHA_B} # v1\n"
    );
    let workflow = workflow_at(".github/workflows/ci.yml", &body);

    assert_eq!(findings(&[workflow], "error").len(), 2);
}

#[test]
fn allow_in_removes_a_workflow_from_reporting_and_repository_evidence() {
    let allowed = workflow_at(
        ".github/workflows/legacy.yml",
        &step(&format!("actions/checkout@{SHA_A} # v3")),
    );
    let checked = workflow_at(
        ".github/workflows/ci.yml",
        &step(&format!("actions/checkout@{SHA_A} # v4")),
    );
    let configuration = config("error", &["**/legacy.yml"]);

    assert!(stale_action_refs::evaluate(&[allowed, checked], &configuration).is_empty());
}

#[test]
fn mutable_local_and_container_references_are_outside_the_rule() {
    let body = concat!(
        "jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n",
        "      - uses: vendor/setup@v1\n",
        "      - uses: ./.github/actions/setup\n",
        "      - uses: docker://alpine:3.20\n",
    );
    let workflow = workflow_at(".github/workflows/ci.yml", body);

    assert!(findings(&[workflow], "error").is_empty());
}

#[test]
fn the_rule_is_silent_when_switched_off() {
    let workflow = workflow_at(
        ".github/workflows/ci.yml",
        &step(&format!("vendor/setup@{SHA_A}")),
    );

    assert!(findings(&[workflow], "off").is_empty());
}
