use std::path::PathBuf;

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    config::{NoWorkflowCommentsRule, Severity},
    rules::{Violation, evaluate_workflow_rule, no_workflow_comments::NoWorkflowComments},
    source::TextFile,
};

fn workflow(body: &str) -> WorkflowFacts {
    let file = TextFile::new(PathBuf::from(".github/workflows/ci.yml"), body.into())
        .unwrap_or_else(|error| panic!("creates workflow: {error}"));

    workflow::read(&file).unwrap_or_else(|error| panic!("reads workflow: {error}"))
}

fn violations(body: &str) -> Vec<Violation> {
    let facts = workflow(body);
    let configuration = NoWorkflowCommentsRule {
        severity: Severity::Error,
    };

    evaluate_workflow_rule::<NoWorkflowComments>(std::slice::from_ref(&facts), &configuration)
        .into_iter()
        .map(|finding| finding.violation)
        .collect()
}

#[test]
fn a_trailing_comment_on_uses_is_silent() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    steps:\n",
        "      - uses: vendor/action@0123456789012345678901234567890123456789 # v2\n",
    );

    assert!(violations(body).is_empty());
}

#[test]
fn a_whole_line_comment_above_uses_is_reported() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    steps:\n",
        "      # Install the action\n",
        "      - uses: vendor/action@0123456789012345678901234567890123456789\n",
    );

    assert_eq!(
        violations(body),
        vec![Violation::WorkflowCommentNotPermitted]
    );
}

#[test]
fn a_trailing_comment_on_run_is_reported() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    steps:\n",
        "      - run: cargo build # Build before testing\n",
    );

    assert_eq!(
        violations(body),
        vec![Violation::WorkflowCommentNotPermitted]
    );
}

#[test]
fn all_yaml_comments_are_reported() {
    let body = concat!(
        "# yaml-language-server: $schema=https://json.schemastore.org/github-workflow.json\n",
        "# yamllint disable rule:line-length\n",
        "jobs:\n",
        "  build:\n",
        "    steps:\n",
        "      - run: cargo build\n",
    );

    assert_eq!(violations(body).len(), 2);
}

#[test]
fn the_rule_is_silent_when_it_is_switched_off() {
    let facts = workflow("# Explain the workflow\njobs: {}\n");
    let configuration = NoWorkflowCommentsRule {
        severity: Severity::Off,
    };

    assert!(
        evaluate_workflow_rule::<NoWorkflowComments>(std::slice::from_ref(&facts), &configuration)
            .is_empty()
    );
}
