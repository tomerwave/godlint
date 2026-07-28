use crate::{
    analyzers::SourceFacts,
    config::{Config, FunctionNestingRule, Severity},
    facts::FunctionFact,
    rules::{
        Finding, FunctionRule, Metric, Rule, RuleError, Violation, evaluate_function_rule,
        when_configured,
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

impl FunctionRule for FunctionNesting {
    fn check(
        function: &FunctionFact,
        _facts: &SourceFacts,
        configuration: &Self::Configuration,
    ) -> Option<Violation> {
        let actual = function.block_depth().value();
        let max = configuration.limit();

        (actual > max).then_some(Violation::limit(Metric::BlockDepth, actual, max))
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.function_nesting.as_ref(), |configuration| {
        evaluate_function_rule::<FunctionNesting>(facts, configuration)
    })
}
