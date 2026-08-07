use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{Config, LineLimitRule, Severity},
    rules::{
        Finding, Languages, Metric, Rule, Violation, WorkflowRule, evaluate_workflow_rule,
        line_count::{Skipped, effective_script_line_count},
        when_configured,
    },
    source::SourceRange,
};

pub struct NoInlineScript;

impl Rule for NoInlineScript {
    const ID: &'static str = "ci/no-inline-script";

    const LANGUAGES: Languages = Languages::WORKFLOWS;

    type Configuration = LineLimitRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl WorkflowRule for NoInlineScript {
    fn check(
        workflow: &WorkflowFacts,
        configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)> {
        let max = configuration.max_lines.get();

        workflow
            .steps()
            .iter()
            .filter_map(|step| step.run_range())
            .filter_map(|range| {
                let actual = effective_script_line_count(
                    workflow.file(),
                    range,
                    Skipped::from(configuration),
                );

                (actual > max)
                    .then_some((range, Violation::limit(Metric::ScriptLines, actual, max)))
            })
            .collect()
    }
}

pub fn evaluate(workflows: &[WorkflowFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_inline_script.as_ref(), |rule| {
        evaluate_workflow_rule::<NoInlineScript>(workflows, rule)
    })
}
