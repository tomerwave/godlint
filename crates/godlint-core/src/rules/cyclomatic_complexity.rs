use crate::{
    analyzers::SourceFacts,
    config::{Config, CyclomaticComplexityRule, Severity},
    facts::FunctionFact,
    rules::{
        Finding, FunctionRule, Metric, Rule, RuleError, Violation, evaluate_function_rule,
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

impl FunctionRule for CyclomaticComplexity {
    fn check(
        function: &FunctionFact,
        _facts: &SourceFacts,
        configuration: &Self::Configuration,
    ) -> Option<Violation> {
        let actual = function.decision_points().value() + 1;
        let max = configuration.limit();

        (actual > max).then_some(Violation::Limit {
            metric: Metric::Complexity,
            actual,
            max,
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(
        config.rules.cyclomatic_complexity.as_ref(),
        |configuration| evaluate_function_rule::<CyclomaticComplexity>(facts, configuration),
    )
}
