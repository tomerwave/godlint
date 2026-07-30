use crate::{
    analyzers::SourceFacts,
    config::{Config, NoProductionLogRule, Severity},
    facts::CallFact,
    rules::{
        CallRule, Finding, Rule, Violation,
        catalogue::{Catalogue, Dialect, matches, spelled},
        evaluate_call_rule, when_configured,
    },
};

const LOGGERS: Catalogue = Catalogue(&[
    ("console.log", Dialect::JavaScript),
    ("console.debug", Dialect::JavaScript),
    ("console.info", Dialect::JavaScript),
    ("console.trace", Dialect::JavaScript),
    ("print", Dialect::Python),
    ("pprint.pprint", Dialect::Python),
    ("dbg!", Dialect::Rust),
]);

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
        let source = call.source();

        (LOGGERS.speaks(source.language(), &name) && !matches(source, &configuration.allow_in))
            .then(|| Violation::ProductionLog {
                callee: name.clone(),
            })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_production_log.as_ref(), |rule| {
        evaluate_call_rule::<NoProductionLog>(facts, rule)
    })
}
