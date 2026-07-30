use crate::{
    analyzers::SourceFacts,
    config::{Config, NoDynamicExecutionRule, Severity},
    facts::CallFact,
    rules::{CallRule, Finding, Rule, Violation, evaluate_call_rule, when_configured},
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

impl CallRule for NoDynamicExecution {
    fn check(call: &CallFact, _configuration: &Self::Configuration) -> Option<Violation> {
        is_dynamic_execution(call).then(|| Violation::DynamicExecution {
            callee: call.callee().to_owned(),
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_dynamic_execution.as_ref(), |rule| {
        evaluate_call_rule::<NoDynamicExecution>(facts, rule)
    })
}

const ECMASCRIPT_GLOBALS: [&str; 4] = ["globalThis", "window", "self", "global"];
const PYTHON_GLOBALS: [&str; 1] = ["builtins"];

fn is_dynamic_execution(call: &CallFact) -> bool {
    let language = call.source().language();
    let callee = unqualified(call.callee(), globals(language));

    match language {
        Language::JavaScript | Language::TypeScript => matches!(callee, "eval" | "Function"),
        Language::Python => matches!(callee, "eval" | "exec"),
        Language::Rust => false,
    }
}

fn globals(language: Language) -> &'static [&'static str] {
    match language {
        Language::JavaScript | Language::TypeScript => &ECMASCRIPT_GLOBALS,
        Language::Python => &PYTHON_GLOBALS,
        Language::Rust => &[],
    }
}

fn unqualified<'callee>(callee: &'callee str, globals: &[&str]) -> &'callee str {
    globals
        .iter()
        .find_map(|global| {
            callee
                .strip_prefix(global)
                .and_then(|rest| rest.strip_prefix('.'))
        })
        .unwrap_or(callee)
}
