use crate::{
    analyzers::SourceFacts,
    config::{Config, NoProductionLogRule, Severity},
    facts::CallFact,
    rules::{
        CallRule, Finding, Rule, Violation,
        catalogue::{Catalogue, spelled},
        evaluate_call_rule, when_configured,
    },
    source::Dialect,
};

const LOGGERS: Catalogue = Catalogue(&[
    ("console.log", Dialect::JavaScript),
    ("console.debug", Dialect::JavaScript),
    ("console.info", Dialect::JavaScript),
    ("console.trace", Dialect::JavaScript),
    ("print", Dialect::Python),
    ("pprint.pprint", Dialect::Python),
    ("dbg!", Dialect::Rust),
    ("log.Print", Dialect::Go),
    ("log.Printf", Dialect::Go),
    ("log.Println", Dialect::Go),
    ("log.Fatal", Dialect::Go),
    ("log.Fatalf", Dialect::Go),
    ("log.Fatalln", Dialect::Go),
    ("log.Panic", Dialect::Go),
    ("log.Panicf", Dialect::Go),
    ("log.Panicln", Dialect::Go),
    ("fmt.Print", Dialect::Go),
    ("fmt.Printf", Dialect::Go),
    ("fmt.Println", Dialect::Go),
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
    fn check(call: &CallFact, _configuration: &Self::Configuration) -> Option<Violation> {
        let name = spelled(call);
        let source = call.source();

        LOGGERS
            .speaks(source.language(), &name)
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
