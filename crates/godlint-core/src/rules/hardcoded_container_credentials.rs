use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{Config, HardcodedContainerCredentialsRule, Severity},
    rules::{
        Finding, Languages, Rule, Violation, WorkflowRule, evaluate_workflow_rule, when_configured,
    },
    source::SourceRange,
};

pub struct HardcodedContainerCredentials;

impl Rule for HardcodedContainerCredentials {
    const ID: &'static str = "ci/hardcoded-container-credentials";

    const LANGUAGES: Languages = Languages::WORKFLOWS;

    type Configuration = HardcodedContainerCredentialsRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl WorkflowRule for HardcodedContainerCredentials {
    fn check(
        workflow: &WorkflowFacts,
        _configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)> {
        workflow
            .credentials()
            .iter()
            .filter(|credential| credential.is_literal())
            .map(|credential| {
                (
                    credential.range(),
                    Violation::HardcodedContainerCredential {
                        key: credential.key().to_owned(),
                        job: credential.job().to_owned(),
                    },
                )
            })
            .collect()
    }
}

pub fn evaluate(workflows: &[WorkflowFacts], config: &Config) -> Vec<Finding> {
    when_configured(
        config.rules.hardcoded_container_credentials.as_ref(),
        |rule| evaluate_workflow_rule::<HardcodedContainerCredentials>(workflows, rule),
    )
}
