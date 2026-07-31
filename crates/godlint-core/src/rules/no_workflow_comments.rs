use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{Config, NoWorkflowCommentsRule, Severity},
    rules::{
        Finding, Languages, Rule, Violation, WorkflowRule, evaluate_workflow_rule, when_configured,
    },
    source::SourceRange,
};

pub struct NoWorkflowComments;

impl Rule for NoWorkflowComments {
    const ID: &'static str = "ci/no-comments";

    const LANGUAGES: Languages = Languages::WORKFLOWS;

    type Configuration = NoWorkflowCommentsRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl WorkflowRule for NoWorkflowComments {
    fn check(
        workflow: &WorkflowFacts,
        _configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)> {
        workflow
            .comments()
            .iter()
            .copied()
            .filter(|comment| !trails_uses_value(workflow, *comment))
            .map(|range| (range, Violation::WorkflowCommentNotPermitted))
            .collect()
    }
}

fn trails_uses_value(workflow: &WorkflowFacts, comment: SourceRange) -> bool {
    workflow.actions().iter().any(|action| {
        let value = action.range();

        value.end() <= comment.start()
            && workflow.file().line(value.end()) == workflow.file().line(comment.start())
    })
}

pub fn evaluate(workflows: &[WorkflowFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_workflow_comments.as_ref(), |rule| {
        evaluate_workflow_rule::<NoWorkflowComments>(workflows, rule)
    })
}
