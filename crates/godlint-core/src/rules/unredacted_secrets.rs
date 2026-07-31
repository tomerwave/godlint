use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{Config, Severity, UnredactedSecretsRule},
    rules::{
        Finding, Languages, Rule, Violation, WorkflowRule, evaluate_workflow_rule, when_configured,
    },
    source::{SourceRange, range_contains},
};

pub struct UnredactedSecrets;

impl Rule for UnredactedSecrets {
    const ID: &'static str = "ci/unredacted-secrets";

    const LANGUAGES: Languages = Languages::WORKFLOWS;

    type Configuration = UnredactedSecretsRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl WorkflowRule for UnredactedSecrets {
    fn check(
        workflow: &WorkflowFacts,
        _configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)> {
        workflow
            .steps()
            .iter()
            .filter_map(|step| step.run().map(|script| (step, script)))
            .filter(|(step, script)| writes_unmasked_target(step.file().text(), *script))
            .filter(|(_, script)| {
                workflow.expressions().iter().any(|expression| {
                    range_contains(*script, expression.range())
                        && expression.context().starts_with("secrets.")
                })
            })
            .map(|(_, script)| (script, Violation::UnredactedSecret))
            .collect()
    }
}

pub fn evaluate(workflows: &[WorkflowFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.unredacted_secrets.as_ref(), |rule| {
        evaluate_workflow_rule::<UnredactedSecrets>(workflows, rule)
    })
}

fn writes_unmasked_target(text: &str, script: SourceRange) -> bool {
    let script = &text[script.start()..script.end()];
    script.contains("$GITHUB_ENV") || script.contains("$GITHUB_OUTPUT")
}
