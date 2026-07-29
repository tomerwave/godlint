use crate::{
    analyzers::SourceFacts,
    config::{Config, NoProductionLogRule, Severity},
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

const LOGGERS: &[(&str, Dialect)] = &[
    ("console.log", Dialect::JavaScript),
    ("console.debug", Dialect::JavaScript),
    ("console.info", Dialect::JavaScript),
    ("console.trace", Dialect::JavaScript),
    ("print", Dialect::Python),
    ("pprint.pprint", Dialect::Python),
    ("dbg!", Dialect::Rust),
];

pub struct NoProductionLog;

impl Rule for NoProductionLog {
    const ID: &'static str = "logging/no-production-log";

    type Configuration = NoProductionLogRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl CallRule for NoProductionLog {
    fn check(call: &CallFact, configuration: &Self::Configuration) -> Option<Violation> {
        let name = spelled(call);

        (is_logger(call, &name) && !is_allowed(call, &configuration.allow_in)).then(|| {
            Violation::ProductionLog {
                callee: name.clone(),
            }
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_production_log.as_ref(), |rule| {
        evaluate_call_rule::<NoProductionLog>(facts, rule)
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

fn is_logger(call: &CallFact, name: &str) -> bool {
    let spoken = dialect(call.source().language());

    LOGGERS
        .iter()
        .any(|(logger, dialect)| *logger == name && *dialect == spoken)
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
