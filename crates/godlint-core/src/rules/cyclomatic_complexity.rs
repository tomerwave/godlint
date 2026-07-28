use crate::{
    analyzers::SourceFacts,
    config::{Config, CyclomaticComplexityRule, Severity},
    facts::FunctionFact,
    rules::{
        Finding, FunctionLimitRule, Metric, Rule, RuleError, evaluate_function_limit_rule,
        when_configured,
    },
};

pub struct CyclomaticComplexity;

impl Rule for CyclomaticComplexity {
    const ID: &'static str = "maintainability/cyclomatic-complexity";

    type Configuration = CyclomaticComplexityRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl FunctionLimitRule for CyclomaticComplexity {
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

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(
        config.rules.cyclomatic_complexity.as_ref(),
        |configuration| evaluate_function_limit_rule::<CyclomaticComplexity>(facts, configuration),
    )
}
