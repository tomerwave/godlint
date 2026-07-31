use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{Config, NoMonolithicJobRule, Severity},
    glob,
    rules::{
        Finding, Languages, Metric, Rule, Violation, WorkflowRule, evaluate_workflow_rule,
        when_configured,
    },
    source::SourceRange,
};

pub struct NoMonolithicJob;

impl Rule for NoMonolithicJob {
    const ID: &'static str = "ci/no-monolithic-job";

    const LANGUAGES: Languages = Languages::WORKFLOWS;

    type Configuration = NoMonolithicJobRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl WorkflowRule for NoMonolithicJob {
    fn check(
        workflow: &WorkflowFacts,
        configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)> {
        if glob::matches_any(
            configuration.allow_in.iter().map(String::as_str),
            workflow.file().path_text(),
        ) {
            return Vec::new();
        }

        let max = configuration.limit();

        workflow
            .jobs()
            .iter()
            .filter_map(|job| {
                let actual = u32::try_from(job.step_count()).unwrap_or(u32::MAX);

                (actual > max)
                    .then_some((job.range(), Violation::limit(Metric::JobSteps, actual, max)))
            })
            .collect()
    }
}

pub fn evaluate(workflows: &[WorkflowFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_monolithic_job.as_ref(), |rule| {
        evaluate_workflow_rule::<NoMonolithicJob>(workflows, rule)
    })
}
