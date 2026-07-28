use crate::{
    analyzers::SourceFacts,
    config::{Config, FunctionStatementsRule, Severity},
    facts::FunctionFact,
    rules::{
        Finding, FunctionRule, Rule, RuleError, Violation, evaluate_function_rule, when_configured,
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

impl FunctionRule for FunctionStatements {
    fn check(
        function: &FunctionFact,
        _facts: &SourceFacts,
        configuration: &Self::Configuration,
    ) -> Option<Violation> {
        let actual = function.statement_count().value();
        let max = configuration.limit();

        (actual > max).then_some(Violation::StatementCount { actual, max })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.function_statements.as_ref(), |configuration| {
        evaluate_function_rule::<FunctionStatements>(facts, configuration)
    })
}
