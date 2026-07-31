use std::path::PathBuf;

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    config::{OverprovisionedSecretsRule, Severity},
    rules::{Violation, evaluate_workflow_rule, overprovisioned_secrets::OverprovisionedSecrets},
    source::TextFile,
};

fn workflow(body: &str) -> WorkflowFacts {
    let file = TextFile::new(PathBuf::from(".github/workflows/ci.yml"), body.into())
        .unwrap_or_else(|error| panic!("creates workflow: {error}"));
    workflow::read(&file).unwrap_or_else(|error| panic!("reads workflow: {error}"))
}

fn findings(body: &str, severity: Severity) -> Vec<Violation> {
    let facts = workflow(body);
    let configuration = OverprovisionedSecretsRule { severity };
    evaluate_workflow_rule::<OverprovisionedSecrets>(std::slice::from_ref(&facts), &configuration)
        .into_iter()
        .map(|finding| finding.violation)
        .collect()
}

fn step(settings: &str) -> String {
    format!(
        "jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: owner/action@v1\n{settings}"
    )
}

#[test]
fn the_whole_secrets_context_in_an_input_is_reported() {
    assert_eq!(
        findings(
            &step("        with:\n          payload: ${{ secrets }}\n"),
            Severity::Error
        ),
        vec![Violation::OverprovisionedSecrets {
            setting: "payload".to_owned()
        }]
    );
}

#[test]
fn the_serialized_whole_secrets_context_in_an_environment_variable_is_reported() {
    assert_eq!(
        findings(
            &step("        env:\n          ALL_SECRETS: ${{ toJSON(secrets) }}\n"),
            Severity::Error
        ),
        vec![Violation::OverprovisionedSecrets {
            setting: "ALL_SECRETS".to_owned()
        }]
    );
}

#[test]
fn serialized_whole_context_matching_ignores_case_and_whitespace() {
    assert_eq!(
        findings(
            &step("        with:\n          payload: ${{ TOJSON( secrets ) }}\n"),
            Severity::Error
        )
        .len(),
        1
    );
}

#[test]
fn a_named_secret_and_a_serialized_named_secret_are_silent() {
    let body = step(concat!(
        "        with:\n",
        "          token: ${{ secrets.NPM_TOKEN }}\n",
        "          payload: ${{ toJSON(secrets.NPM_TOKEN) }}\n",
    ));

    assert!(findings(&body, Severity::Error).is_empty());
}

#[test]
fn a_whole_context_outside_inputs_and_environment_is_silent() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - if: ${{ secrets }}\n",
        "        run: echo ${{ toJSON(secrets) }}\n",
    );

    assert!(findings(body, Severity::Error).is_empty());
}

#[test]
fn the_message_names_the_setting_and_the_narrowing_fix() {
    let reported = findings(
        &step("        env:\n          CREDENTIALS: ${{ secrets }}\n"),
        Severity::Error,
    )[0]
    .to_string();

    assert!(reported.starts_with("CREDENTIALS"), "{reported}");
    assert!(reported.contains("whole secrets context"), "{reported}");
    assert!(reported.contains("named secret"), "{reported}");
}

#[test]
fn the_rule_is_silent_when_it_is_switched_off() {
    assert!(
        findings(
            &step("        with:\n          payload: ${{ secrets }}\n"),
            Severity::Off
        )
        .is_empty()
    );
}
