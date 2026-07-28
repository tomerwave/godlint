use crate::{
    analyzers::SourceFacts,
    config::{Config, NoDynamicExecutionRule, Severity},
    facts::CallFact,
    rules::{Finding, Rule, RuleError, Violation, evaluate_call_rule, when_configured},
    source::Language,
};

pub struct NoDynamicExecution;

impl Rule for NoDynamicExecution {
    const ID: &'static str = "security/no-dynamic-execution";

    type Configuration = NoDynamicExecutionRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.no_dynamic_execution.as_ref(), |rule| {
        evaluate_call_rule(
            facts,
            NoDynamicExecution::severity(rule),
            NoDynamicExecution::ID,
            |call| {
                is_dynamic_execution(call).then(|| Violation::DynamicExecution {
                    callee: call.callee().to_owned(),
                })
            },
        )
    })
}

fn is_dynamic_execution(call: &CallFact) -> bool {
    match call.source().language() {
        Language::JavaScript | Language::TypeScript => matches!(call.callee(), "eval" | "Function"),
        Language::Python => matches!(call.callee(), "eval" | "exec"),
        Language::Rust => false,
    }
}
