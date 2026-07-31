use std::path::PathBuf;

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    config::{Severity, UnredactedSecretsRule},
    rules::{Violation, evaluate_workflow_rule, unredacted_secrets::UnredactedSecrets},
    source::TextFile,
};

fn workflow(body: &str) -> WorkflowFacts {
    let file = TextFile::new(PathBuf::from(".github/workflows/ci.yml"), body.into())
        .unwrap_or_else(|error| panic!("creates workflow: {error}"));
    workflow::read(&file).unwrap_or_else(|error| panic!("reads workflow: {error}"))
}

fn violations(script: &str, severity: Severity) -> Vec<Violation> {
    let body =
        format!("jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - run: {script}\n");
    let facts = workflow(&body);
    let configuration = UnredactedSecretsRule { severity };
    evaluate_workflow_rule::<UnredactedSecrets>(std::slice::from_ref(&facts), &configuration)
        .into_iter()
        .map(|finding| finding.violation)
        .collect()
}

#[test]
fn a_plain_run_scalar_writing_a_secret_to_github_env_is_reported() {
    assert_eq!(
        violations(
            "echo TOKEN=${{ secrets.TOKEN }} >> $GITHUB_ENV",
            Severity::Error
        ),
        vec![Violation::UnredactedSecret]
    );
}

#[test]
fn a_double_quoted_run_scalar_writing_a_secret_is_reported() {
    assert_eq!(
        violations(
            "\"echo TOKEN=${{ secrets.TOKEN }} >> $GITHUB_ENV\"",
            Severity::Error
        ),
        vec![Violation::UnredactedSecret]
    );
}

#[test]
fn a_single_quoted_run_scalar_writing_a_secret_is_reported() {
    assert_eq!(
        violations(
            "'echo TOKEN=${{ secrets.TOKEN }} >> $GITHUB_ENV'",
            Severity::Error
        ),
        vec![Violation::UnredactedSecret]
    );
}

#[test]
fn a_literal_block_run_scalar_writing_a_secret_is_reported() {
    assert_eq!(
        violations(
            "|\n          echo TOKEN=${{ secrets.TOKEN }} >> $GITHUB_OUTPUT",
            Severity::Error
        ),
        vec![Violation::UnredactedSecret]
    );
}

#[test]
fn a_folded_block_run_scalar_writing_a_secret_is_reported() {
    assert_eq!(
        violations(
            ">\n          echo TOKEN=${{ secrets.TOKEN }} >> $GITHUB_OUTPUT",
            Severity::Error
        ),
        vec![Violation::UnredactedSecret]
    );
}

#[test]
fn multiple_secret_expressions_in_one_script_produce_one_finding() {
    assert_eq!(
        violations(
            "|\n          echo A=${{ secrets.A }} >> $GITHUB_ENV\n          echo B=${{ secrets.B }} >> $GITHUB_OUTPUT",
            Severity::Error
        ),
        vec![Violation::UnredactedSecret]
    );
}

#[test]
fn merely_using_a_secret_is_silent() {
    assert!(
        violations(
            "npm publish --token ${{ secrets.NPM_TOKEN }}",
            Severity::Error
        )
        .is_empty()
    );
}

#[test]
fn writing_a_non_secret_value_to_a_github_sink_is_silent() {
    assert!(violations("echo channel=stable >> $GITHUB_OUTPUT", Severity::Error).is_empty());
}

#[test]
fn a_secret_in_environment_and_a_sink_in_run_are_silent() {
    let facts = workflow(concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - run: echo TOKEN=$TOKEN >> $GITHUB_ENV\n",
        "        env:\n",
        "          TOKEN: ${{ secrets.TOKEN }}\n",
    ));
    let configuration = UnredactedSecretsRule {
        severity: Severity::Error,
    };

    assert!(
        evaluate_workflow_rule::<UnredactedSecrets>(std::slice::from_ref(&facts), &configuration)
            .is_empty()
    );
}

#[test]
fn the_message_explains_where_masking_stops() {
    let message = violations(
        "echo TOKEN=${{ secrets.TOKEN }} >> $GITHUB_ENV",
        Severity::Error,
    )[0]
    .to_string();

    assert!(message.contains("GITHUB_ENV or GITHUB_OUTPUT"), "{message}");
    assert!(message.contains("masking does not follow"), "{message}");
}

#[test]
fn the_rule_is_silent_when_it_is_switched_off() {
    assert!(
        violations(
            "echo TOKEN=${{ secrets.TOKEN }} >> $GITHUB_ENV",
            Severity::Off
        )
        .is_empty()
    );
}
