use std::{num::NonZeroU32, path::PathBuf};

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    config::{LineLimitRule, Severity},
    rules::{Metric, Violation, evaluate_workflow_rule, no_inline_script::NoInlineScript},
    source::TextFile,
};

fn workflow(body: &str) -> WorkflowFacts {
    let file = TextFile::new(PathBuf::from(".github/workflows/ci.yml"), body.into())
        .unwrap_or_else(|error| panic!("creates workflow: {error}"));

    workflow::read(&file).unwrap_or_else(|error| panic!("reads workflow: {error}"))
}

fn configuration(max_lines: u32) -> LineLimitRule {
    LineLimitRule {
        severity: Severity::Error,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        max_lines: NonZeroU32::new(max_lines).unwrap_or_else(|| panic!("limit must be positive")),
        skip_blank_lines: true,
        skip_comments: true,
    }
}

fn violations(scalar: &str, max_lines: u32) -> Vec<Violation> {
    let facts = workflow(&format!(
        "jobs:\n  build:\n    steps:\n      - run: {scalar}\n"
    ));

    evaluate_workflow_rule::<NoInlineScript>(
        std::slice::from_ref(&facts),
        &configuration(max_lines),
    )
    .into_iter()
    .map(|finding| finding.violation)
    .collect()
}

#[test]
fn every_multiline_yaml_scalar_form_is_measured() {
    let cases = [
        "|\n          echo one\n          echo two",
        ">\n          echo one\n          echo two",
        "'echo one\n          echo two'",
        "\"echo one\n          echo two\"",
    ];

    for scalar in cases {
        assert_eq!(
            violations(scalar, 1),
            vec![Violation::limit(Metric::ScriptLines, 2, 1)],
            "{scalar:?}"
        );
    }
}

#[test]
fn a_script_at_the_limit_is_silent_and_one_over_is_reported() {
    assert!(
        violations("|\n          one\n          two", 2).is_empty(),
        "the configured limit is inclusive"
    );
    assert_eq!(
        violations("|\n          one\n          two\n          three", 2),
        vec![Violation::limit(Metric::ScriptLines, 3, 2)]
    );
}

#[test]
fn blank_and_shell_comment_lines_are_not_logic() {
    let scalar = concat!(
        "|\n",
        "          echo one\n",
        "\n",
        "          # explains the next command\n",
        "          echo two # this is still a command",
    );

    assert!(violations(scalar, 2).is_empty());
    assert_eq!(
        violations(scalar, 1),
        vec![Violation::limit(Metric::ScriptLines, 2, 1)]
    );
}

#[test]
fn configured_physical_lines_can_include_blanks_and_comments() {
    let facts = workflow(concat!(
        "jobs:\n  build:\n    steps:\n      - run: |\n",
        "          echo one\n",
        "\n",
        "          # explanation\n",
    ));
    let configuration = LineLimitRule {
        severity: Severity::Error,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        max_lines: NonZeroU32::new(1).expect("one is positive"),
        skip_blank_lines: false,
        skip_comments: false,
    };
    let findings =
        evaluate_workflow_rule::<NoInlineScript>(std::slice::from_ref(&facts), &configuration);

    assert_eq!(
        findings[0].violation,
        Violation::limit(Metric::ScriptLines, 3, 1)
    );
}

#[test]
fn a_single_source_line_is_silent_even_when_it_chains_commands() {
    assert!(violations("one && two && three", 1).is_empty());
}

#[test]
fn the_rule_is_silent_when_it_is_switched_off() {
    let facts = workflow(concat!(
        "jobs:\n  build:\n    steps:\n      - run: |\n",
        "          one\n          two\n",
    ));
    let mut configuration = configuration(1);

    configuration.severity = Severity::Off;

    assert!(
        evaluate_workflow_rule::<NoInlineScript>(std::slice::from_ref(&facts), &configuration)
            .is_empty()
    );
}
