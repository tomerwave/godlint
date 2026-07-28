use crate::{
    analyzers::SourceFacts,
    config::{Config, ReturnCountRule, Severity},
    facts::FunctionFact,
    rules::{
        Finding, FunctionLimitRule, Metric, Rule, RuleError, evaluate_function_limit_rule,
        when_configured,
    },
};

pub struct ReturnCount;

impl Rule for ReturnCount {
    const ID: &'static str = "maintainability/return-count";

    type Configuration = ReturnCountRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl FunctionLimitRule for ReturnCount {
    const METRIC: Metric = Metric::ReturnPaths;

    fn measure(
        function: &FunctionFact,
        _facts: &SourceFacts,
        _configuration: &Self::Configuration,
    ) -> u32 {
        function.return_paths().value()
    }

    fn max(configuration: &Self::Configuration) -> u32 {
        configuration.limit()
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.return_count.as_ref(), |configuration| {
        evaluate_function_limit_rule::<ReturnCount>(facts, configuration)
    })
}
