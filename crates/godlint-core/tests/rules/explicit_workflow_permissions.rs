use std::path::PathBuf;

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    config::{ExplicitWorkflowPermissionsRule, Severity},
    rules::{
        Violation, evaluate_workflow_rule,
        explicit_workflow_permissions::ExplicitWorkflowPermissions,
    },
    source::TextFile,
};

fn workflow(body: &str) -> WorkflowFacts {
    let file = TextFile::new(PathBuf::from(".github/workflows/ci.yml"), body.into())
        .unwrap_or_else(|error| panic!("creates workflow: {error}"));

    workflow::read(&file).unwrap_or_else(|error| panic!("reads workflow: {error}"))
}

fn violations(body: &str, require_per_job: bool) -> Vec<Violation> {
    let facts = workflow(body);
    let configuration = ExplicitWorkflowPermissionsRule {
        severity: Severity::Error,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        require_per_job,
    };

    evaluate_workflow_rule::<ExplicitWorkflowPermissions>(
        std::slice::from_ref(&facts),
        &configuration,
    )
    .into_iter()
    .map(|finding| finding.violation)
    .collect()
}

const NOTHING: &str =
    "jobs:\n  build:\n    steps:\n      - run: x\n  test:\n    steps:\n      - run: y\n";

const WORKFLOW_LEVEL: &str = concat!(
    "permissions:\n",
    "  contents: read\n",
    "jobs:\n",
    "  build:\n",
    "    steps:\n",
    "      - run: x\n",
);

const MIXED: &str = concat!(
    "jobs:\n",
    "  narrowed:\n",
    "    permissions:\n",
    "      contents: read\n",
    "    steps:\n",
    "      - run: x\n",
    "  open:\n",
    "    steps:\n",
    "      - run: y\n",
);

#[test]
fn a_workflow_declaring_nothing_anywhere_is_reported_once() {
    assert_eq!(
        violations(NOTHING, false),
        vec![Violation::UndeclaredPermissions],
        "two jobs and one missing line is one finding, not two"
    );
}

#[test]
fn a_workflow_level_declaration_covers_every_job() {
    assert!(violations(WORKFLOW_LEVEL, false).is_empty());
}

#[test]
fn only_the_job_that_is_still_open_is_reported_when_some_are_narrowed() {
    assert_eq!(
        violations(MIXED, false),
        vec![Violation::InheritedPermissions {
            job: "open".to_owned()
        }],
        "a workflow whose other jobs are narrowed does not need a blanket block"
    );
}

#[test]
fn requiring_per_job_narrowing_asks_for_it_even_when_the_workflow_declares_some() {
    assert_eq!(
        violations(WORKFLOW_LEVEL, true),
        vec![Violation::InheritedPermissions {
            job: "build".to_owned()
        }]
    );
    assert!(
        violations(
            concat!(
                "permissions:\n",
                "  contents: read\n",
                "jobs:\n",
                "  build:\n",
                "    permissions:\n",
                "      contents: read\n",
                "    steps:\n",
                "      - run: x\n",
            ),
            true
        )
        .is_empty()
    );
}

#[test]
fn a_workflow_with_no_jobs_has_nothing_to_report() {
    assert!(
        violations("name: CI\non:\n  push:\n", false).is_empty(),
        "a file that runs nothing grants nothing"
    );
    assert!(violations("name: CI\non:\n  push:\n", true).is_empty());
}

#[test]
fn a_commented_out_declaration_is_not_a_declaration() {
    let commented = concat!(
        "# permissions:\n",
        "#   contents: read\n",
        "jobs:\n",
        "  build:\n",
        "    steps:\n",
        "      - run: x\n",
    );

    assert_eq!(
        violations(commented, false),
        vec![Violation::UndeclaredPermissions],
        "reading the syntax is what tells a declaration from a comment about one"
    );
}

#[test]
fn the_message_names_the_job_and_what_it_inherits() {
    let reported = violations(MIXED, false)[0].to_string();

    assert!(
        reported.starts_with("open declares no permissions"),
        "{reported}"
    );
    assert!(
        reported.contains("repository grants by default"),
        "{reported}"
    );
}

#[test]
fn the_rule_is_silent_when_it_is_switched_off() {
    let facts = workflow(NOTHING);
    let configuration = ExplicitWorkflowPermissionsRule {
        severity: Severity::Off,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        require_per_job: false,
    };

    assert!(
        evaluate_workflow_rule::<ExplicitWorkflowPermissions>(
            std::slice::from_ref(&facts),
            &configuration
        )
        .is_empty()
    );
}
