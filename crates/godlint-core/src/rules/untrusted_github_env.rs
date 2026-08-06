use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{Config, Severity, UntrustedGithubEnvRule},
    rules::{
        Finding, Languages, Rule, Violation, WorkflowRule, evaluate_workflow_rule, when_configured,
        workflow_expression::is_attacker_influenced,
    },
    source::{SourceRange, range_contains},
};

pub struct UntrustedGithubEnv;

impl Rule for UntrustedGithubEnv {
    const ID: &'static str = "ci/untrusted-github-env";

    const LANGUAGES: Languages = Languages::WORKFLOWS;

    type Configuration = UntrustedGithubEnvRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl WorkflowRule for UntrustedGithubEnv {
    fn check(
        workflow: &WorkflowFacts,
        _configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)> {
        workflow
            .expressions()
            .iter()
            .filter(|expression| is_attacker_influenced(expression.context()))
            .filter(|expression| {
                workflow.steps().iter().any(|step| {
                    step.run().is_some_and(|script| {
                        range_contains(script, expression.range())
                            && writes_shared_environment(step.file().text(), script)
                    })
                })
            })
            .map(|expression| {
                (
                    expression.range(),
                    Violation::UntrustedGithubEnv {
                        expression: expression.body().to_owned(),
                    },
                )
            })
            .collect()
    }
}

pub fn evaluate(workflows: &[WorkflowFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.untrusted_github_env.as_ref(), |rule| {
        evaluate_workflow_rule::<UntrustedGithubEnv>(workflows, rule)
    })
}

fn writes_shared_environment(text: &str, script: SourceRange) -> bool {
    let script = &text[script.start()..script.end()];
    script.contains("$GITHUB_ENV") || script.contains("$GITHUB_PATH")
}
