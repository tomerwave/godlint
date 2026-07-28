use crate::{
    analyzers::SourceFacts,
    config::{Config, ParameterCountRule, Severity},
    facts::FunctionFact,
    rules::{
        Finding, FunctionLimitRule, Metric, Rule, RuleError, evaluate_function_limit_rule,
        when_configured,
    },
};

pub struct ParameterCount;

impl Rule for ParameterCount {
    const ID: &'static str = "maintainability/parameter-count";

    type Configuration = ParameterCountRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl FunctionLimitRule for ParameterCount {
    const METRIC: Metric = Metric::ParameterCount;

    fn measure(
        function: &FunctionFact,
        _facts: &SourceFacts,
        _configuration: &Self::Configuration,
    ) -> u32 {
        function.parameter_count().value()
    }

    fn max(configuration: &Self::Configuration) -> u32 {
        configuration.limit()
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.parameter_count.as_ref(), |configuration| {
        evaluate_function_limit_rule::<ParameterCount>(facts, configuration)
    })
}
