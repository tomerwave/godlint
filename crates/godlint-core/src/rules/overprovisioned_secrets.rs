use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{Config, OverprovisionedSecretsRule, Severity},
    facts::Setting,
    rules::{
        Finding, Languages, Rule, Violation, WorkflowRule, evaluate_workflow_rule, when_configured,
    },
    source::{SourceRange, range_contains},
};

pub struct OverprovisionedSecrets;

impl Rule for OverprovisionedSecrets {
    const ID: &'static str = "ci/overprovisioned-secrets";

    const LANGUAGES: Languages = Languages::WORKFLOWS;

    type Configuration = OverprovisionedSecretsRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl WorkflowRule for OverprovisionedSecrets {
    fn check(
        workflow: &WorkflowFacts,
        _configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)> {
        workflow
            .steps()
            .iter()
            .flat_map(|step| step.inputs().iter().chain(step.environment()))
            .flat_map(|setting| violations_for_setting(workflow, setting))
            .collect()
    }
}

pub fn evaluate(workflows: &[WorkflowFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.overprovisioned_secrets.as_ref(), |rule| {
        evaluate_workflow_rule::<OverprovisionedSecrets>(workflows, rule)
    })
}

fn violations_for_setting(
    workflow: &WorkflowFacts,
    setting: &Setting,
) -> Vec<(SourceRange, Violation)> {
    workflow
        .expressions()
        .iter()
        .filter(|expression| range_contains(setting.range(), expression.range()))
        .filter(|expression| is_whole_secrets(expression.context(), expression.body()))
        .map(|expression| {
            (
                expression.range(),
                Violation::OverprovisionedSecrets {
                    setting: setting.key().to_owned(),
                },
            )
        })
        .collect()
}

fn is_whole_secrets(context: &str, body: &str) -> bool {
    context == "secrets"
        || body
            .chars()
            .filter(|character| !character.is_ascii_whitespace())
            .collect::<String>()
            .eq_ignore_ascii_case("tojson(secrets)")
}
