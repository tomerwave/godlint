use crate::{
    analyzers::SourceFacts,
    config::{Config, RestrictedCall as RestrictedCallConfiguration, RestrictedCallRule, Severity},
    facts::CallFact,
    glob,
    rules::{Finding, Rule, RuleError, Violation, evaluate_call_rule},
    source::Language,
};

pub struct RestrictedCall;

impl Rule for RestrictedCall {
    const ID: &'static str = "architecture/restricted-call";

    type Configuration = RestrictedCallRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    let severity = config
        .rules
        .restricted_call
        .as_ref()
        .map_or(Severity::Error, RestrictedCall::severity);

    evaluate_call_rule(facts, severity, RestrictedCall::ID, |call| {
        let restricted = is_default_restriction(call)
            || config
                .rules
                .restricted_call
                .as_ref()
                .is_some_and(|rule| is_configured_restriction(call, &rule.calls));

        restricted.then(|| Violation::RestrictedCall {
            callee: call.callee().to_owned(),
        })
    })
}

fn is_default_restriction(call: &CallFact) -> bool {
    match call.source().language() {
        Language::JavaScript | Language::TypeScript => {
            matches!(
                call.callee(),
                "process.exit" | "console.log" | "console.debug"
            )
        }
        Language::Python => matches!(call.callee(), "sys.exit" | "os._exit" | "print"),
        Language::Rust => matches!(call.callee(), "std::process::exit" | "dbg" | "dbg!"),
    }
}

fn is_configured_restriction(
    call: &CallFact,
    restrictions: &[RestrictedCallConfiguration],
) -> bool {
    restrictions
        .iter()
        .any(|restriction| applies(restriction, call))
}

fn applies(restriction: &RestrictedCallConfiguration, call: &CallFact) -> bool {
    restriction.name == call.callee() && !is_allowed(call, &restriction.allow_in)
}

fn is_allowed(call: &CallFact, paths: &[String]) -> bool {
    let path = call.source().path().to_string_lossy();

    paths.iter().any(|pattern| glob::matches(pattern, &path))
}
