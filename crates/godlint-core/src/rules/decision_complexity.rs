use crate::{
    analyzers::SourceFacts,
    config::{Config, DecisionComplexityRule, Severity},
    facts::FunctionFact,
    rules::{
        Finding, FunctionLimitRule, Metric, Rule, evaluate_function_limit_rule, when_configured,
    },
};

pub struct DecisionComplexity;

impl Rule for DecisionComplexity {
    const ID: &'static str = "maintainability/decision-complexity";

    type Configuration = DecisionComplexityRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl FunctionLimitRule for DecisionComplexity {
    const METRIC: Metric = Metric::Complexity;

    fn measure(
        function: &FunctionFact,
        _facts: &SourceFacts,
        _configuration: &Self::Configuration,
    ) -> u32 {
        function.decision_points().value() + 1
    }

    fn max(configuration: &Self::Configuration) -> u32 {
        configuration.limit()
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.decision_complexity.as_ref(), |configuration| {
        evaluate_function_limit_rule::<DecisionComplexity>(facts, configuration)
    })
}
