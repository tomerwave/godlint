use crate::{
    analyzers::SourceFacts,
    config::{Config, RestrictedCall as RestrictedCallConfiguration, RestrictedCallRule, Severity},
    facts::CallFact,
    glob,
    rules::{Finding, Rule, RuleError, Violation, evaluate_call_rule, when_configured},
    source::Language,
};

#[derive(Clone, Copy, Eq, PartialEq)]
enum Dialect {
    JavaScript,
    Python,
    Rust,
    RustMacro,
}

const DEFAULTS: &[(&str, Dialect)] = &[
    ("process.exit", Dialect::JavaScript),
    ("console.log", Dialect::JavaScript),
    ("console.debug", Dialect::JavaScript),
    ("sys.exit", Dialect::Python),
    ("os._exit", Dialect::Python),
    ("print", Dialect::Python),
    ("std::process::exit", Dialect::Rust),
    ("dbg!", Dialect::RustMacro),
];

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
    let restricted_here = is_default_restriction(call, &name);

    match restrictions
        .iter()
        .find(|restriction| restriction.name == name)
    {
        Some(_) if is_built_in(&name) && !restricted_here => false,
        Some(restriction) => !is_allowed(call, &restriction.allow_in),
        None => restricted_here,
    }
}

fn is_built_in(name: &str) -> bool {
    DEFAULTS.iter().any(|(default, _)| *default == name)
}

fn is_default_restriction(call: &CallFact, name: &str) -> bool {
    let dialect = dialect(call);

    DEFAULTS
        .iter()
        .any(|(default, spoken)| *default == name && *spoken == dialect)
}

fn dialect(call: &CallFact) -> Dialect {
    match call.source().language() {
        Language::JavaScript | Language::TypeScript => Dialect::JavaScript,
        Language::Python => Dialect::Python,
        Language::Rust if call.is_macro() => Dialect::RustMacro,
        Language::Rust => Dialect::Rust,
    }
}

fn is_allowed(call: &CallFact, paths: &[String]) -> bool {
    let path = call.source().path().to_string_lossy();

    paths.iter().any(|pattern| glob::matches(pattern, &path))
}
