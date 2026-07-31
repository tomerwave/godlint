use std::path::PathBuf;

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    config::{NoMonolithicJobRule, Severity},
    rules::{Metric, Violation, evaluate_workflow_rule, no_monolithic_job::NoMonolithicJob},
    source::TextFile,
};

fn workflow(body: &str) -> WorkflowFacts {
    let file = TextFile::new(PathBuf::from(".github/workflows/ci.yml"), body.into())
        .unwrap_or_else(|error| panic!("creates workflow: {error}"));

    workflow::read(&file).unwrap_or_else(|error| panic!("reads workflow: {error}"))
}

fn configuration(max_steps: u32) -> NoMonolithicJobRule {
    NoMonolithicJobRule {
        severity: Severity::Error,
        max_steps,
    }
}

fn violations(body: &str, max_steps: u32) -> Vec<Violation> {
    let facts = workflow(body);

    evaluate_workflow_rule::<NoMonolithicJob>(
        std::slice::from_ref(&facts),
        &configuration(max_steps),
    )
    .into_iter()
    .map(|finding| finding.violation)
    .collect()
}

#[test]
fn block_and_flow_step_sequences_are_measured() {
    let cases = [
        concat!(
            "jobs:\n  build:\n    steps:\n",
            "      - run: one\n      - run: two\n",
        ),
        "jobs:\n  build:\n    steps: [{run: one}, {run: two}]\n",
        "jobs: {build: {steps: [{run: one}, {run: two}]}}\n",
    ];

    for body in cases {
        assert_eq!(
            violations(body, 1),
            vec![Violation::limit(Metric::JobSteps, 2, 1)],
            "{body:?}"
        );
    }
}

#[test]
fn a_job_at_the_limit_is_silent_and_one_over_is_reported() {
    let at_limit = concat!(
        "jobs:\n  build:\n    steps:\n",
        "      - run: one\n      - run: two\n",
    );
    let over_limit = concat!(
        "jobs:\n  build:\n    steps:\n",
        "      - run: one\n      - run: two\n      - run: three\n",
    );

    assert!(violations(at_limit, 2).is_empty());
    assert_eq!(
        violations(over_limit, 2),
        vec![Violation::limit(Metric::JobSteps, 3, 2)]
    );
}

#[test]
fn a_reusable_workflow_job_and_an_empty_job_have_no_steps_to_report() {
    assert!(
        violations(
            concat!(
                "jobs:\n",
                "  called:\n    uses: owner/repo/.github/workflows/ci.yml@main\n",
                "  empty:\n    steps: []\n",
            ),
            0,
        )
        .is_empty()
    );
}

#[test]
fn the_message_names_the_job_metric() {
    assert_eq!(
        violations("jobs:\n  build:\n    steps: [{run: one}, {run: two}]\n", 1,)[0].to_string(),
        "Job has 2 steps (max 1)."
    );
}

#[test]
fn the_rule_is_silent_when_it_is_switched_off() {
    let facts = workflow("jobs:\n  build:\n    steps: [{run: one}, {run: two}]\n");
    let configuration = NoMonolithicJobRule {
        severity: Severity::Off,
        max_steps: 1,
    };

    assert!(
        evaluate_workflow_rule::<NoMonolithicJob>(std::slice::from_ref(&facts), &configuration)
            .is_empty()
    );
}
