use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{Config, NoSilencedFailureRule, Severity},
    facts::{ExpressionFact, JobFact, StepFact},
    rules::{
        Finding, Languages, Rule, Violation, WorkflowRule, evaluate_workflow_rule, when_configured,
        workflow_condition::expressions_in_condition,
    },
    source::{SourceRange, range_contains},
};

pub struct NoSilencedFailure;

impl Rule for NoSilencedFailure {
    const ID: &'static str = "ci/no-silenced-failure";

    const LANGUAGES: Languages = Languages::WORKFLOWS;

    type Configuration = NoSilencedFailureRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl WorkflowRule for NoSilencedFailure {
    fn check(
        workflow: &WorkflowFacts,
        _configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)> {
        let mut violations = job_violations(workflow);
        violations.extend(step_violations(workflow));
        violations
    }
}

pub fn evaluate(workflows: &[WorkflowFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_silenced_failure.as_ref(), |rule| {
        evaluate_workflow_rule::<NoSilencedFailure>(workflows, rule)
    })
}

fn job_violations(workflow: &WorkflowFacts) -> Vec<(SourceRange, Violation)> {
    workflow
        .jobs()
        .iter()
        .filter_map(|job| {
            literal_true(workflow, job.continue_on_error())
                .map(|range| (range, Violation::JobContinuesOnError))
        })
        .collect()
}

fn step_violations(workflow: &WorkflowFacts) -> Vec<(SourceRange, Violation)> {
    workflow
        .jobs()
        .iter()
        .fold(Vec::new(), |mut violations, job| {
            for step in workflow
                .steps()
                .iter()
                .filter(|step| step.job() == job.name())
            {
                if let Some(range) = literal_true(workflow, step.continue_on_error())
                    && !outcome_is_read(workflow, job, step)
                {
                    violations.push((range, Violation::StepContinuesOnError));
                }

                if let Some(script) = step.run()
                    && let Some(violation) = swallowed_script(workflow, script)
                {
                    violations.push((script, violation));
                }
            }
            violations
        })
}

fn literal_true(workflow: &WorkflowFacts, range: Option<SourceRange>) -> Option<SourceRange> {
    range.filter(|range| workflow.file().text()[range.start()..range.end()].trim() == "true")
}

fn outcome_is_read(workflow: &WorkflowFacts, job: &JobFact, step: &StepFact) -> bool {
    let Some(id) = step.id() else {
        return false;
    };
    let outcome = format!("steps.{id}.outcome");
    let conclusion = format!("steps.{id}.conclusion");

    workflow.expressions().iter().any(|expression| {
        expression_is_in_job(expression, job)
            && (expression.body().contains(&outcome) || expression.body().contains(&conclusion))
    }) || conditions_in_job(workflow, job).any(|condition| {
        expressions_in_condition(workflow, condition)
            .into_iter()
            .any(|(_, body)| body.contains(&outcome) || body.contains(&conclusion))
    })
}

fn conditions_in_job<'workflow>(
    workflow: &'workflow WorkflowFacts,
    job: &'workflow JobFact,
) -> impl Iterator<Item = SourceRange> + 'workflow {
    job.condition().into_iter().chain(
        workflow
            .steps()
            .iter()
            .filter(|step| step.job() == job.name())
            .filter_map(StepFact::condition),
    )
}

fn expression_is_in_job(expression: &ExpressionFact, job: &JobFact) -> bool {
    range_contains(job.body(), expression.range())
}

fn swallowed_script(workflow: &WorkflowFacts, range: SourceRange) -> Option<Violation> {
    let script = workflow.file().text()[range.start()..range.end()]
        .trim_end()
        .trim_end_matches(['"', '\''])
        .trim_end();

    if script.ends_with("|| exit 0") {
        Some(Violation::ScriptExitsSuccessfully {
            ending: "|| exit 0",
        })
    } else if script.ends_with("; exit 0") {
        Some(Violation::ScriptExitsSuccessfully { ending: "; exit 0" })
    } else if script.ends_with("|| true") {
        Some(Violation::ScriptOrTrue)
    } else {
        None
    }
}
