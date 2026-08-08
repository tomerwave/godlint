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
                    step.run_range().is_some_and(|script| {
                        range_contains(script, expression.range())
                            && writes_shared_environment(
                                step.file().text(),
                                script,
                                expression.range(),
                            )
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

fn writes_shared_environment(text: &str, script: SourceRange, expression: SourceRange) -> bool {
    let script_text = &text[script.start()..script.end()];
    let expression_start = expression.start() - script.start();
    let command_start = script_text[..expression_start]
        .char_indices()
        .rev()
        .find_map(|(index, character)| command_separator(script_text, index, character))
        .map_or(0, |(_, end)| end);
    let command_end = script_text[expression_start..]
        .char_indices()
        .find_map(|(index, character)| {
            command_separator(script_text, expression_start + index, character)
        })
        .map_or(script_text.len(), |(start, _)| start);
    let command = &script_text[command_start..command_end];

    [
        "$GITHUB_ENV",
        "$GITHUB_PATH",
        "${GITHUB_ENV}",
        "${GITHUB_PATH}",
    ]
    .iter()
    .any(|sink| command.contains(sink))
}

fn command_separator(text: &str, index: usize, character: char) -> Option<(usize, usize)> {
    match character {
        '\n' | ';' if index == 0 || text.as_bytes()[index - 1] != b'\\' => {
            Some((index, index + character.len_utf8()))
        }
        '&' if text.as_bytes().get(index + 1) == Some(&b'&') => Some((index, index + 2)),
        '|' if text.as_bytes().get(index + 1) == Some(&b'|') => Some((index, index + 2)),
        _ => None,
    }
}
