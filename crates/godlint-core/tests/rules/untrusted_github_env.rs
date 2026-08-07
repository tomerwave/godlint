use std::path::PathBuf;

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    config::{Severity, UntrustedGithubEnvRule},
    rules::{Violation, evaluate_workflow_rule, untrusted_github_env::UntrustedGithubEnv},
    source::TextFile,
};

fn workflow(body: &str) -> WorkflowFacts {
    let file = TextFile::new(PathBuf::from(".github/workflows/ci.yml"), body.into())
        .unwrap_or_else(|error| panic!("creates workflow: {error}"));
    workflow::read(&file).unwrap_or_else(|error| panic!("reads workflow: {error}"))
}

fn violations(script: &str) -> Vec<Violation> {
    let body =
        format!("jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - run: {script}\n");
    let facts = workflow(&body);
    let configuration = UntrustedGithubEnvRule {
        severity: Severity::Error,
        only_in: Vec::new(),
        allow_in: Vec::new(),
    };
    evaluate_workflow_rule::<UntrustedGithubEnv>(std::slice::from_ref(&facts), &configuration)
        .into_iter()
        .map(|finding| finding.violation)
        .collect()
}

#[test]
fn attacker_controlled_value_written_to_github_env_is_reported() {
    assert_eq!(
        violations("echo TITLE=${{ github.event.pull_request.title }} >> $GITHUB_ENV"),
        vec![Violation::UntrustedGithubEnv {
            expression: "github.event.pull_request.title".to_owned(),
        }]
    );
}

#[test]
fn attacker_controlled_value_written_to_github_path_is_reported() {
    assert_eq!(
        violations("echo ${{ github.event.issue.body }} >> $GITHUB_PATH"),
        vec![Violation::UntrustedGithubEnv {
            expression: "github.event.issue.body".to_owned(),
        }]
    );
}

#[test]
fn braced_github_env_expansion_is_reported() {
    assert_eq!(
        violations("echo TITLE=${{ github.event.issue.body }} >> ${GITHUB_ENV}"),
        vec![Violation::UntrustedGithubEnv {
            expression: "github.event.issue.body".to_owned(),
        }]
    );
}

#[test]
fn braced_github_path_expansion_is_reported() {
    assert_eq!(
        violations("echo ${{ github.event.issue.body }} >> ${GITHUB_PATH}"),
        vec![Violation::UntrustedGithubEnv {
            expression: "github.event.issue.body".to_owned(),
        }]
    );
}

#[test]
fn static_value_written_to_github_env_is_silent() {
    assert!(violations("echo TITLE=release >> $GITHUB_ENV").is_empty());
}

#[test]
fn attacker_controlled_value_without_a_shared_environment_write_is_silent() {
    assert!(violations("echo ${{ github.event.issue.body }}").is_empty());
}

#[test]
fn an_attacker_value_and_a_static_sink_in_separate_commands_are_silent() {
    assert!(
        violations(
            "|\n          echo ${{ github.event.issue.body }}\n          echo SAFE=1 >> $GITHUB_ENV"
        )
        .is_empty()
    );
}

#[test]
fn an_attacker_value_and_a_static_sink_after_a_command_separator_are_silent() {
    assert!(
        violations("echo ${{ github.event.issue.body }}; echo SAFE=1 >> $GITHUB_ENV").is_empty()
    );
}

#[test]
fn shell_boolean_command_separators_do_not_cross_trigger() {
    for separator in ["&&", "||"] {
        assert!(
            violations(&format!(
                "echo ${{{{ github.event.issue.body }}}} {separator} echo SAFE=1 >> $GITHUB_ENV"
            ))
            .is_empty(),
            "separator: {separator}"
        );
    }
}

#[test]
fn the_message_explains_that_later_steps_inherit_the_value() {
    let message = violations("echo TITLE=${{ github.event.pull_request.title }} >> $GITHUB_ENV")[0]
        .to_string();

    assert!(message.contains("later steps"), "{message}");
    assert!(message.contains("GITHUB_ENV or GITHUB_PATH"), "{message}");
}

#[test]
fn the_rule_is_silent_when_it_is_switched_off() {
    let body = "jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo ${{ github.event.issue.body }} >> $GITHUB_ENV\n";
    let facts = workflow(body);
    let configuration = UntrustedGithubEnvRule {
        severity: Severity::Off,
        only_in: Vec::new(),
        allow_in: Vec::new(),
    };

    assert!(
        evaluate_workflow_rule::<UntrustedGithubEnv>(std::slice::from_ref(&facts), &configuration)
            .is_empty()
    );
}
