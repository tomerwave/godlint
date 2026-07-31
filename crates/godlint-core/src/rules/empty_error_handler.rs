use crate::{
    analyzers::SourceFacts,
    config::{Config, EmptyErrorHandlerRule, Severity},
    facts::ErrorHandlerFact,
    rules::{
        Absence, ErrorHandlerRule, Finding, Languages, Rule, Violation,
        evaluate_error_handler_rule, when_configured,
    },
    source::Dialect,
};

pub struct EmptyErrorHandler;

impl Rule for EmptyErrorHandler {
    const ID: &'static str = "reliability/empty-error-handler";

    const LANGUAGES: Languages = Languages::all_but(&[(Dialect::Rust, Absence::NoSuchConstruct)]);

    type Configuration = EmptyErrorHandlerRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl ErrorHandlerRule for EmptyErrorHandler {
    fn check(
        error_handler: &ErrorHandlerFact,
        _configuration: &Self::Configuration,
    ) -> Option<Violation> {
        error_handler
            .body_is_empty()
            .then_some(Violation::EmptyErrorHandler)
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.empty_error_handler.as_ref(), |rule| {
        evaluate_error_handler_rule::<EmptyErrorHandler>(facts, rule)
    })
}
