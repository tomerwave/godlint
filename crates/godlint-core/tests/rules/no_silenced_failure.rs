use std::path::PathBuf;

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    config::{NoSilencedFailureRule, Severity},
    rules::{Finding, evaluate_workflow_rule, no_silenced_failure::NoSilencedFailure},
    source::TextFile,
};

fn workflow(body: &str) -> WorkflowFacts {
    let file = TextFile::new(PathBuf::from(".github/workflows/ci.yml"), body.into())
        .unwrap_or_else(|error| panic!("creates workflow: {error}"));
    workflow::read(&file).unwrap_or_else(|error| panic!("reads workflow: {error}"))
}

fn findings(body: &str, severity: Severity) -> Vec<Finding> {
    let facts = workflow(body);
    let configuration = NoSilencedFailureRule { severity };
    evaluate_workflow_rule::<NoSilencedFailure>(std::slice::from_ref(&facts), &configuration)
}

fn step(properties: &str) -> String {
    format!(
        "jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - run: check\n{properties}"
    )
}

fn script(run: &str) -> String {
    format!("jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - run: {run}\n")
}

#[test]
fn an_unconsumed_step_continuing_on_error_is_a_warning() {
    let reported = findings(&step("        continue-on-error: true\n"), Severity::Error);

    assert_eq!(reported.len(), 1);
    assert_eq!(reported[0].severity, Severity::Warning);
    assert!(reported[0].message().contains("no later expression"));
}

#[test]
fn a_job_continuing_on_error_is_a_warning() {
    let reported = findings(
        concat!(
            "jobs:\n",
            "  build:\n",
            "    continue-on-error: true\n",
            "    runs-on: ubuntu-latest\n",
            "    steps:\n",
            "      - run: check\n",
        ),
        Severity::Error,
    );

    assert_eq!(reported.len(), 1);
    assert_eq!(reported[0].severity, Severity::Warning);
    assert!(reported[0].message().contains("workflow stays green"));
}

#[test]
fn a_step_whose_outcome_is_read_in_the_same_job_is_silent() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - id: probe\n",
        "        continue-on-error: true\n",
        "        run: check\n",
        "      - if: ${{ steps.probe.outcome == 'failure' }}\n",
        "        run: warn\n",
    );

    assert!(findings(body, Severity::Error).is_empty());
}

#[test]
fn an_unbraced_condition_reading_the_steps_outcome_is_silent() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - id: probe\n",
        "        continue-on-error: true\n",
        "        run: check\n",
        "      - if: steps.probe.outcome == 'failure'\n",
        "        run: warn\n",
    );

    assert!(findings(body, Severity::Error).is_empty());
}

#[test]
fn an_unbraced_condition_reading_another_steps_outcome_does_not_exempt_the_step() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - id: probe\n",
        "        continue-on-error: true\n",
        "        run: check\n",
        "      - if: steps.other.outcome == 'failure'\n",
        "        run: warn\n",
    );

    assert_eq!(findings(body, Severity::Error).len(), 1);
}

#[test]
fn a_step_whose_conclusion_is_read_in_the_same_job_is_silent() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - id: probe\n",
        "        continue-on-error: true\n",
        "        run: check\n",
        "      - env:\n",
        "          PROBE: ${{ steps.probe.conclusion }}\n",
        "        run: warn\n",
    );

    assert!(findings(body, Severity::Error).is_empty());
}

#[test]
fn a_step_whose_conclusion_is_read_in_a_with_value_is_silent() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - id: probe\n",
        "        continue-on-error: true\n",
        "        run: check\n",
        "      - uses: example/action@v1\n",
        "        with:\n",
        "          outcome: ${{ steps.probe.conclusion }}\n",
    );

    assert!(findings(body, Severity::Error).is_empty());
}

#[test]
fn a_step_with_no_outcome_reference_is_reported() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - id: probe\n",
        "        continue-on-error: true\n",
        "        run: check\n",
        "      - run: echo unrelated\n",
    );

    assert_eq!(findings(body, Severity::Error).len(), 1);
}

#[test]
fn an_outcome_read_in_another_job_does_not_exempt_the_step() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - id: probe\n",
        "        continue-on-error: true\n",
        "        run: check\n",
        "  report:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - run: echo ${{ steps.probe.outcome }}\n",
    );

    assert_eq!(findings(body, Severity::Error).len(), 1);
}

#[test]
fn false_and_expression_continue_on_error_values_are_literal_and_silent() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    continue-on-error: ${{ matrix.experimental }}\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - continue-on-error: false\n",
        "        run: check\n",
    );

    assert!(findings(body, Severity::Error).is_empty());
}

#[test]
fn a_script_ending_or_true_is_a_warning() {
    let reported = findings(&script("check || true"), Severity::Error);

    assert_eq!(reported.len(), 1);
    assert_eq!(reported[0].severity, Severity::Warning);
}

#[test]
fn a_script_ending_semicolon_exit_zero_is_an_error() {
    let reported = findings(&script("check; exit 0"), Severity::Error);

    assert_eq!(reported.len(), 1);
    assert_eq!(reported[0].severity, Severity::Error);
}

#[test]
fn a_script_ending_or_exit_zero_is_an_error() {
    let reported = findings(&script("check || exit 0"), Severity::Error);

    assert_eq!(reported.len(), 1);
    assert_eq!(reported[0].severity, Severity::Error);
}

#[test]
fn trailing_whitespace_after_a_swallow_is_ignored() {
    let reported = findings(
        &script("|\n          check || true\n          \n"),
        Severity::Error,
    );

    assert_eq!(reported.len(), 1);
}

#[test]
fn an_or_true_inside_a_quoted_string_is_reported_by_the_text_match() {
    let reported = findings(&script("echo 'example || true'"), Severity::Error);

    assert_eq!(reported.len(), 1);
}

#[test]
fn indirect_success_exit_status_is_outside_the_text_match() {
    let body = script("|\n          x=true\n          exit $x");

    assert!(findings(&body, Severity::Error).is_empty());
}

#[test]
fn a_swallow_before_another_command_is_silent() {
    let body = script("|\n          optional || true\n          check");

    assert!(findings(&body, Severity::Error).is_empty());
}

#[test]
fn the_rule_is_silent_when_switched_off() {
    assert!(findings(&script("check; exit 0"), Severity::Off).is_empty());
}
