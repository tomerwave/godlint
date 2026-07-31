use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{Config, SecretsInheritRule, Severity},
    facts::Secrets,
    glob,
    rules::{
        Finding, Languages, Rule, Violation, WorkflowRule, evaluate_workflow_rule, when_configured,
    },
    source::SourceRange,
};

pub struct SecretsInherit;

impl Rule for SecretsInherit {
    const ID: &'static str = "ci/secrets-inherit";

    const LANGUAGES: Languages = Languages::WORKFLOWS;

    type Configuration = SecretsInheritRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl WorkflowRule for SecretsInherit {
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
            .jobs()
            .iter()
            .filter_map(|job| match job.secrets() {
                Some(Secrets::Inherit { range }) => Some((
                    *range,
                    Violation::InheritedSecrets {
                        job: job.name().to_owned(),
                    },
                )),
                _ => None,
            })
            .collect()
    }
}

pub fn evaluate(workflows: &[WorkflowFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.secrets_inherit.as_ref(), |rule| {
        evaluate_workflow_rule::<SecretsInherit>(workflows, rule)
    })
}
