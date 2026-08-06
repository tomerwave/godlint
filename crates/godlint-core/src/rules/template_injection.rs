use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{Config, Severity, TemplateInjectionRule},
    rules::{
        Finding, Languages, Rule, Violation, WorkflowRule, evaluate_workflow_rule, when_configured,
        workflow_expression::{is_attacker_influenced, matches_context},
    },
    source::{SourceRange, range_contains},
};

const TRIGGER_INPUT_CONTEXTS: [&str; 2] = ["github.event.inputs.", "inputs."];

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
        _configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)> {
        workflow
            .expressions()
            .iter()
            .filter_map(|expression| {
                influence(expression.context()).map(|influence| (expression, influence))
            })
            .filter(|expression| {
                workflow.steps().iter().any(|step| {
                    step.run()
                        .is_some_and(|script| range_contains(script, expression.0.range()))
                })
            })
            .map(|(expression, influence)| {
                (
                    expression.range(),
                    Violation::TemplateInjection {
                        expression: expression.body().to_owned(),
                        certain: matches!(influence, Influence::Attacker),
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

#[derive(Clone, Copy)]
enum Influence {
    Attacker,
    TriggerInput,
}

fn influence(context: &str) -> Option<Influence> {
    if is_attacker_influenced(context) {
        Some(Influence::Attacker)
    } else if matches_context(&TRIGGER_INPUT_CONTEXTS, context) {
        Some(Influence::TriggerInput)
    } else {
        None
    }
}
