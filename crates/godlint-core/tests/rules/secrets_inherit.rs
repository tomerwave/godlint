use std::path::PathBuf;

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    config::{SecretsInheritRule, Severity},
    rules::{Violation, evaluate_workflow_rule, secrets_inherit::SecretsInherit},
    source::TextFile,
};

fn workflow_at(path: &str, body: &str) -> WorkflowFacts {
    let file = TextFile::new(PathBuf::from(path), body.into())
        .unwrap_or_else(|error| panic!("creates workflow: {error}"));
    workflow::read(&file).unwrap_or_else(|error| panic!("reads workflow: {error}"))
}

fn workflow(body: &str) -> WorkflowFacts {
    workflow_at(".github/workflows/ci.yml", body)
}

fn configuration(severity: Severity, allow_in: &[&str]) -> SecretsInheritRule {
    SecretsInheritRule {
        severity,
        allow_in: allow_in.iter().map(|path| (*path).to_owned()).collect(),
    }
}

fn violations(secrets: &str) -> Vec<Violation> {
    let facts = workflow(&format!(
        "jobs:\n  publish:\n    uses: owner/repository/.github/workflows/publish.yml@main\n    secrets: {secrets}\n"
    ));
    evaluate_workflow_rule::<SecretsInherit>(
        std::slice::from_ref(&facts),
        &configuration(Severity::Error, &[]),
    )
    .into_iter()
    .map(|finding| finding.violation)
    .collect()
}

#[test]
fn a_plain_inherit_value_is_reported() {
    assert_eq!(
        violations("inherit"),
        vec![Violation::InheritedSecrets {
            job: "publish".to_owned()
        }]
    );
}

#[test]
fn a_double_quoted_inherit_value_is_reported() {
    assert_eq!(violations("\"inherit\""), violations("inherit"));
}

#[test]
fn a_single_quoted_inherit_value_is_reported() {
    assert_eq!(violations("'inherit'"), violations("inherit"));
}

#[test]
fn named_secrets_and_no_secrets_are_silent() {
    let facts = workflow(concat!(
        "jobs:\n",
        "  named:\n",
        "    uses: owner/repository/.github/workflows/publish.yml@main\n",
        "    secrets:\n",
        "      token: ${{ secrets.NPM_TOKEN }}\n",
        "  none:\n",
        "    uses: owner/repository/.github/workflows/test.yml@main\n",
    ));

    assert!(
        evaluate_workflow_rule::<SecretsInherit>(
            std::slice::from_ref(&facts),
            &configuration(Severity::Error, &[])
        )
        .is_empty()
    );
}

#[test]
fn allow_in_globs_exempt_only_matching_workflow_paths() {
    let body = "jobs:\n  publish:\n    uses: ./publish.yml\n    secrets: inherit\n";
    let allowed = workflow_at(".github/workflows/release.yml", body);
    let checked = workflow_at(".github/workflows/ci.yml", body);
    let findings = evaluate_workflow_rule::<SecretsInherit>(
        &[allowed, checked],
        &configuration(Severity::Error, &["**/release.yml"]),
    );

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].path, PathBuf::from(".github/workflows/ci.yml"));
}

#[test]
fn the_message_explains_the_cost_and_names_the_fix() {
    let message = violations("inherit")[0].to_string();

    assert!(message.contains("receives every secret"), "{message}");
    assert!(
        message.contains("not only the secrets it needs"),
        "{message}"
    );
    assert!(message.contains("name each secret explicitly"), "{message}");
}

#[test]
fn the_rule_is_silent_when_it_is_switched_off() {
    let facts = workflow("jobs:\n  publish:\n    uses: ./publish.yml\n    secrets: inherit\n");

    assert!(
        evaluate_workflow_rule::<SecretsInherit>(
            std::slice::from_ref(&facts),
            &configuration(Severity::Off, &[])
        )
        .is_empty()
    );
}
