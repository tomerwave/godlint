use std::path::PathBuf;

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    config::{Severity, TemplateInjectionRule},
    rules::{Violation, evaluate_workflow_rule, template_injection::TemplateInjection},
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

fn configuration(allow_in: &[&str]) -> TemplateInjectionRule {
    TemplateInjectionRule {
        severity: Severity::Error,
        allow_in: allow_in.iter().map(|path| (*path).to_owned()).collect(),
    }
}

fn violations(body: &str) -> Vec<Violation> {
    let facts = workflow(body);

    evaluate_workflow_rule::<TemplateInjection>(std::slice::from_ref(&facts), &configuration(&[]))
        .into_iter()
        .map(|finding| finding.violation)
        .collect()
}

fn severities(body: &str, severity: Severity) -> Vec<Severity> {
    let facts = workflow(body);
    let configuration = TemplateInjectionRule {
        severity,
        allow_in: Vec::new(),
    };

    evaluate_workflow_rule::<TemplateInjection>(std::slice::from_ref(&facts), &configuration)
        .into_iter()
        .map(|finding| finding.severity)
        .collect()
}

fn run(script: &str) -> String {
    format!("jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - run: {script}\n")
}

#[test]
fn a_plain_run_scalar_is_reported() {
    assert_eq!(
        violations(&run("echo ${{ github.event.issue.title }}")),
        vec![Violation::TemplateInjection {
            expression: "github.event.issue.title".to_owned(),
            certain: true,
        }]
    );
}

#[test]
fn a_double_quoted_run_scalar_is_reported() {
    assert_eq!(
        violations(&run("\"echo ${{ github.event.pull_request.body }}\"")),
        vec![Violation::TemplateInjection {
            expression: "github.event.pull_request.body".to_owned(),
            certain: true,
        }]
    );
}

#[test]
fn a_single_quoted_run_scalar_is_reported() {
    assert_eq!(
        violations(&run("'echo ${{ github.event.comment.body }}'")),
        vec![Violation::TemplateInjection {
            expression: "github.event.comment.body".to_owned(),
            certain: true,
        }]
    );
}

#[test]
fn a_literal_block_run_scalar_is_reported() {
    assert_eq!(
        violations(&run("|\n          echo ${{ github.event.review.body }}")),
        vec![Violation::TemplateInjection {
            expression: "github.event.review.body".to_owned(),
            certain: true,
        }]
    );
}

#[test]
fn a_folded_block_run_scalar_is_reported() {
    assert_eq!(
        violations(&run(
            ">\n          echo ${{ github.event.discussion.title }}"
        )),
        vec![Violation::TemplateInjection {
            expression: "github.event.discussion.title".to_owned(),
            certain: true,
        }]
    );
}

#[test]
fn every_documented_influenced_context_is_reported() {
    let contexts = [
        "github.event.issue.title",
        "github.event.issue.body",
        "github.event.pull_request.title",
        "github.event.pull_request.body",
        "github.event.pull_request.head.ref",
        "github.event.comment.body",
        "github.event.review.body",
        "github.event.review_comment.body",
        "github.event.discussion.title",
        "github.event.discussion.body",
        "github.event.head_commit.message",
        "github.event.head_commit.author.name",
        "github.event.head_commit.author.email",
        "github.event.commits",
        "github.event.pages",
        "github.event.pull_request.head.label",
        "github.event.pull_request.head.repo.default_branch",
        "github.event.workflow_run.head_branch",
        "github.event.workflow_run.head_commit.message",
        "github.head_ref",
        "github.event.inputs.release_name",
        "inputs.release_name",
    ];
    let scripts = contexts
        .iter()
        .map(|context| format!("echo ${{{{ {context} }}}}"))
        .collect::<Vec<_>>()
        .join("\n          ");
    let reported = violations(&run(&format!("|\n          {scripts}")));

    assert_eq!(reported.len(), contexts.len(), "{reported:?}");
}

#[test]
fn event_contexts_keep_the_configured_severity() {
    assert_eq!(
        severities(
            &run("echo ${{ github.event.pull_request.title }}"),
            Severity::Error
        ),
        vec![Severity::Error]
    );
}

#[test]
fn trigger_inputs_are_capped_at_warning() {
    assert_eq!(
        severities(
            &run("echo ${{ github.event.inputs.release_kind }}"),
            Severity::Error
        ),
        vec![Severity::Warning]
    );
    assert_eq!(
        severities(&run("echo ${{ inputs.version }}"), Severity::Error),
        vec![Severity::Warning]
    );
}

#[test]
fn a_trigger_input_cap_never_raises_the_configured_severity() {
    assert_eq!(
        severities(&run("echo ${{ inputs.version }}"), Severity::Info),
        vec![Severity::Info]
    );
}

#[test]
fn pull_request_review_comment_input_is_reported() {
    assert_eq!(
        violations(&run("echo ${{ github.event.review_comment.body }}")),
        vec![Violation::TemplateInjection {
            expression: "github.event.review_comment.body".to_owned(),
            certain: true,
        }]
    );
}

#[test]
fn workflow_run_branch_and_commit_inputs_are_reported() {
    let script = concat!(
        "|\n",
        "          echo ${{ github.event.workflow_run.head_branch }}\n",
        "          echo ${{ github.event.workflow_run.head_commit.message }}",
    );

    assert_eq!(violations(&run(script)).len(), 2);
}

#[test]
fn matching_uses_the_normalized_context_but_the_message_preserves_the_body() {
    let reported = violations(&run("echo ${{ GITHUB.EVENT.ISSUE.TITLE }}"));

    assert_eq!(
        reported,
        vec![Violation::TemplateInjection {
            expression: "GITHUB.EVENT.ISSUE.TITLE".to_owned(),
            certain: true,
        }]
    );
    assert!(
        reported[0]
            .to_string()
            .starts_with("\"GITHUB.EVENT.ISSUE.TITLE\"")
    );
}

#[test]
fn expressions_outside_run_are_not_reported() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - if: ${{ github.event.issue.title }}\n",
        "        uses: owner/action@v1\n",
        "        with:\n",
        "          title: ${{ github.event.pull_request.title }}\n",
        "        env:\n",
        "          COMMENT: ${{ github.event.comment.body }}\n",
        "      - run: echo \"$TITLE\"\n",
        "        env:\n",
        "          TITLE: ${{ github.event.discussion.body }}\n",
    );

    assert!(violations(body).is_empty());
}

#[test]
fn trusted_expressions_in_run_are_not_reported() {
    assert!(violations(&run("echo ${{ github.sha }} ${{ github.repository }}")).is_empty());
}

#[test]
fn allow_in_globs_exempt_only_matching_workflow_paths() {
    let body = run("echo ${{ github.head_ref }}");
    let allowed = workflow_at(".github/workflows/release.yml", &body);
    let checked = workflow_at(".github/workflows/ci.yml", &body);
    let findings = evaluate_workflow_rule::<TemplateInjection>(
        &[allowed, checked],
        &configuration(&["**/release.yml"]),
    );

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].path, PathBuf::from(".github/workflows/ci.yml"));
}

#[test]
fn the_message_names_the_expression_timing_and_quoted_environment_fix() {
    let message = violations(&run("echo ${{ github.event.issue.body }}"))[0].to_string();

    assert!(message.contains("\"github.event.issue.body\""), "{message}");
    assert!(
        message.contains("runner expands it into the script before the shell runs"),
        "{message}"
    );
    assert!(message.contains("env variable"), "{message}");
    assert!(message.contains("quoted"), "{message}");
}

#[test]
fn the_trigger_input_message_explains_the_uncertainty_and_fix() {
    let message = violations(&run("echo ${{ inputs.version }}"))[0].to_string();

    assert!(
        message.contains("whoever triggered the workflow"),
        "{message}"
    );
    assert!(
        message.contains("workflow_dispatch requires write access"),
        "{message}"
    );
    assert!(message.contains("calling workflow"), "{message}");
    assert!(message.contains("value it does not control"), "{message}");
    assert!(message.contains("env variable"), "{message}");
    assert!(message.contains("quoted"), "{message}");
}

#[test]
fn the_rule_is_silent_when_it_is_switched_off() {
    let facts = workflow(&run("echo ${{ github.event.issue.title }}"));
    let configuration = TemplateInjectionRule {
        severity: Severity::Off,
        allow_in: Vec::new(),
    };

    assert!(
        evaluate_workflow_rule::<TemplateInjection>(std::slice::from_ref(&facts), &configuration)
            .is_empty()
    );
}
