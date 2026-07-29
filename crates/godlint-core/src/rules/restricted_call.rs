use crate::{
    analyzers::SourceFacts,
    config::{Config, RestrictedCall as RestrictedCallConfiguration, RestrictedCallRule, Severity},
    facts::CallFact,
    glob,
    rules::{CallRule, Finding, Rule, Violation, evaluate_call_rule, when_configured},
    source::Language,
};

#[derive(Clone, Copy, Eq, PartialEq)]
enum Dialect {
    JavaScript,
    Python,
    Rust,
}

const DEFAULTS: &[(&str, Dialect)] = &[
    ("process.exit", Dialect::JavaScript),
    ("sys.exit", Dialect::Python),
    ("os._exit", Dialect::Python),
    ("std::process::exit", Dialect::Rust),
];

pub struct RestrictedCall;

impl Rule for RestrictedCall {
    const ID: &'static str = "architecture/restricted-call";

    type Configuration = RestrictedCallRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl CallRule for RestrictedCall {
    fn check(call: &CallFact, configuration: &Self::Configuration) -> Option<Violation> {
        is_restricted(call, &configuration.calls).then(|| Violation::RestrictedCall {
            callee: spelled(call),
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.restricted_call.as_ref(), |rule| {
        evaluate_call_rule::<RestrictedCall>(facts, rule)
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
    let configured = restrictions
        .iter()
        .find(|restriction| restriction.name == name);
    let restricted = if is_built_in(&name) {
        is_built_in_of(call, &name)
    } else {
        configured.is_some()
    };

    restricted && !configured.is_some_and(|restriction| is_allowed(call, &restriction.allow_in))
}

fn is_built_in(name: &str) -> bool {
    DEFAULTS.iter().any(|(default, _)| *default == name)
}

fn is_built_in_of(call: &CallFact, name: &str) -> bool {
    let dialect = dialect(call.source().language());

    DEFAULTS
        .iter()
        .any(|(default, spoken)| *default == name && *spoken == dialect)
}

fn dialect(language: Language) -> Dialect {
    match language {
        Language::JavaScript | Language::TypeScript => Dialect::JavaScript,
        Language::Python => Dialect::Python,
        Language::Rust => Dialect::Rust,
    }
}

fn is_allowed(call: &CallFact, paths: &[String]) -> bool {
    glob::matches_any(
        paths.iter().map(String::as_str),
        &call.source().path().to_string_lossy(),
    )
}
