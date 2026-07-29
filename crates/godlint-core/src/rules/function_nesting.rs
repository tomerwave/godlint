use crate::{
    analyzers::SourceFacts,
    config::{Config, FunctionNestingRule, Severity},
    facts::FunctionFact,
    rules::{
        Finding, FunctionLimitRule, Metric, Rule, evaluate_function_limit_rule, when_configured,
    },
};

pub struct FunctionNesting;

impl Rule for FunctionNesting {
    const ID: &'static str = "maintainability/function-nesting";

    type Configuration = FunctionNestingRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl FunctionLimitRule for FunctionNesting {
    const METRIC: Metric = Metric::BlockDepth;

    fn measure(
        function: &FunctionFact,
        _facts: &SourceFacts,
        _configuration: &Self::Configuration,
    ) -> u32 {
        function.block_depth().value()
    }

    fn max(configuration: &Self::Configuration) -> u32 {
        configuration.limit()
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.function_nesting.as_ref(), |configuration| {
        evaluate_function_limit_rule::<FunctionNesting>(facts, configuration)
    })
}
