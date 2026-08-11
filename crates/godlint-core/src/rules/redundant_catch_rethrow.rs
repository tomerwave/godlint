use crate::{
    analyzers::SourceFacts,
    config::{Config, NoProductionLogRule, Severity},
    facts::ErrorHandlerFact,
    rules::{
        Absence, ErrorHandlerRule, Finding, Languages, Rule, Violation,
        evaluate_error_handler_rule, when_configured,
    },
    source::Dialect,
};

pub struct RedundantCatchRethrow;

impl Rule for RedundantCatchRethrow {
    const ID: &'static str = "reliability/redundant-catch-rethrow";
    const LANGUAGES: Languages = Languages::all_but(&[
        (Dialect::Rust, Absence::NoSuchConstruct),
        (Dialect::Workflow, Absence::NoSuchConstruct),
    ]);
    type Configuration = NoProductionLogRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl ErrorHandlerRule for RedundantCatchRethrow {
    fn check(
        handler: &ErrorHandlerFact,
        _configuration: &Self::Configuration,
    ) -> Option<Violation> {
        handler
            .rethrows_only()
            .then_some(Violation::RedundantCatchRethrow)
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.redundant_catch_rethrow.as_ref(), |rule| {
        evaluate_error_handler_rule::<RedundantCatchRethrow>(facts, rule)
    })
}
