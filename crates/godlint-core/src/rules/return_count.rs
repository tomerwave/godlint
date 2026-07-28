use crate::{
    analyzers::SourceFacts,
    config::{Config, ReturnCountRule, Severity},
    facts::FunctionFact,
    rules::{
        Finding, FunctionRule, Metric, Rule, RuleError, Violation, evaluate_function_rule,
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

impl FunctionRule for ReturnCount {
    fn check(
        function: &FunctionFact,
        _facts: &SourceFacts,
        configuration: &Self::Configuration,
    ) -> Option<Violation> {
        let actual = function.return_paths().value();
        let max = configuration.limit();

        (actual > max).then_some(Violation::Limit {
            metric: Metric::ReturnPaths,
            actual,
            max,
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.return_count.as_ref(), |configuration| {
        evaluate_function_rule::<ReturnCount>(facts, configuration)
    })
}
