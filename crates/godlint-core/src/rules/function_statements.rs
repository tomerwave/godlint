use crate::{
    analyzers::SourceFacts,
    config::{Config, FunctionStatementsRule, Severity},
    facts::FunctionFact,
    rules::{
        Finding, FunctionLimitRule, Metric, Rule, evaluate_function_limit_rule, when_configured,
    },
};

pub struct FunctionStatements;

impl Rule for FunctionStatements {
    const ID: &'static str = "maintainability/function-statements";

    type Configuration = FunctionStatementsRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl FunctionLimitRule for FunctionStatements {
    const METRIC: Metric = Metric::StatementCount;

    fn measure(
        function: &FunctionFact,
        _facts: &SourceFacts,
        _configuration: &Self::Configuration,
    ) -> u32 {
        function.statement_count().value()
    }

    fn max(configuration: &Self::Configuration) -> u32 {
        configuration.limit()
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.function_statements.as_ref(), |configuration| {
        evaluate_function_limit_rule::<FunctionStatements>(facts, configuration)
    })
}
