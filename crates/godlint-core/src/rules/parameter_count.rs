use crate::{
    analyzers::SourceFacts,
    config::{Config, ParameterCountRule, Severity},
    facts::FunctionFact,
    rules::{
        Finding, FunctionRule, Metric, Rule, RuleError, Violation, evaluate_function_rule,
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

impl FunctionRule for ParameterCount {
    fn check(
        function: &FunctionFact,
        _facts: &SourceFacts,
        configuration: &Self::Configuration,
    ) -> Option<Violation> {
        let actual = function.parameter_count().value();
        let max = configuration.limit();

        (actual > max).then_some(Violation::limit(Metric::ParameterCount, actual, max))
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.parameter_count.as_ref(), |configuration| {
        evaluate_function_rule::<ParameterCount>(facts, configuration)
    })
}
