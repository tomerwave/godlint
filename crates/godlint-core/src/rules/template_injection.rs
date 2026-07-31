use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{Config, Severity, TemplateInjectionRule},
    glob,
    rules::{
        Finding, Languages, Rule, Violation, WorkflowRule, evaluate_workflow_rule, when_configured,
    },
    source::{SourceRange, range_contains},
};

const ATTACKER_CONTEXTS: [&str; 22] = [
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
    "github.event.inputs.",
    "inputs.",
];

pub struct TemplateInjection;

impl Rule for TemplateInjection {
    const ID: &'static str = "ci/template-injection";

    const LANGUAGES: Languages = Languages::WORKFLOWS;

    type Configuration = TemplateInjectionRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl WorkflowRule for TemplateInjection {
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

        workflow
            .expressions()
            .iter()
            .filter(|expression| attacker_influenced(expression.context()))
            .filter(|expression| {
                workflow.steps().iter().any(|step| {
                    step.run()
                        .is_some_and(|script| range_contains(script, expression.range()))
                })
            })
            .map(|expression| {
                (
                    expression.range(),
                    Violation::TemplateInjection {
                        expression: expression.body().to_owned(),
                    },
                )
            })
            .collect()
    }
}

pub fn evaluate(workflows: &[WorkflowFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.template_injection.as_ref(), |rule| {
        evaluate_workflow_rule::<TemplateInjection>(workflows, rule)
    })
}

fn attacker_influenced(context: &str) -> bool {
    ATTACKER_CONTEXTS.iter().any(|candidate| {
        context == *candidate || candidate.ends_with('.') && context.starts_with(candidate)
    })
}
