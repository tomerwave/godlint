use crate::{
    analyzers::SourceFacts,
    config::{Config, RestrictedCall as RestrictedCallConfiguration, RestrictedCallRule, Severity},
    facts::CallFact,
    glob,
    rules::{Finding, Rule, RuleError, Violation, evaluate_call_rule, when_configured},
    source::Language,
};

const JAVASCRIPT_RESTRICTIONS: &[&str] = &["process.exit", "console.log", "console.debug"];

const PYTHON_RESTRICTIONS: &[&str] = &["sys.exit", "os._exit", "print"];

const RUST_RESTRICTIONS: &[&str] = &["std::process::exit"];

const RUST_MACRO_RESTRICTIONS: &[&str] = &["dbg!"];

pub struct RestrictedCall;

impl Rule for RestrictedCall {
    const ID: &'static str = "architecture/restricted-call";

    type Configuration = RestrictedCallRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.restricted_call.as_ref(), |rule| {
        evaluate_call_rule(
            facts,
            RestrictedCall::severity(rule),
            RestrictedCall::ID,
            |call| {
                is_restricted(call, &rule.calls).then(|| Violation::RestrictedCall {
                    callee: spelled(call),
                })
            },
        )
    })
}

fn spelled(call: &CallFact) -> String {
    let callee = call.callee();

    if call.is_macro() {
        format!("{callee}!")
    } else {
        callee.to_owned()
    }
}

fn is_restricted(call: &CallFact, restrictions: &[RestrictedCallConfiguration]) -> bool {
    let name = spelled(call);

    match restrictions
        .iter()
        .find(|restriction| restriction.name == name)
    {
        Some(restriction) => !is_allowed(call, &restriction.allow_in),
        None => default_restrictions(call).contains(&name.as_str()),
    }
}

fn default_restrictions(call: &CallFact) -> &'static [&'static str] {
    match call.source().language() {
        Language::JavaScript | Language::TypeScript => JAVASCRIPT_RESTRICTIONS,
        Language::Python => PYTHON_RESTRICTIONS,
        Language::Rust if call.is_macro() => RUST_MACRO_RESTRICTIONS,
        Language::Rust => RUST_RESTRICTIONS,
    }
}

fn is_allowed(call: &CallFact, paths: &[String]) -> bool {
    let path = call.source().path().to_string_lossy();

    paths.iter().any(|pattern| glob::matches(pattern, &path))
}
