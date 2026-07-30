use crate::{
    analyzers::SourceFacts,
    config::{ConditionComplexityRule, Config, Severity},
    facts::ConditionFact,
    rules::{
        ConditionRule, Finding, Metric, Rule, Violation, evaluate_condition_rule, when_configured,
    },
};

pub struct ConditionComplexity;

impl Rule for ConditionComplexity {
    const ID: &'static str = "maintainability/condition-complexity";

    type Configuration = ConditionComplexityRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl ConditionRule for ConditionComplexity {
    fn check(condition: &ConditionFact, configuration: &Self::Configuration) -> Option<Violation> {
        let actual = condition.operator_count();
        let max = configuration.limit();

        (actual > max).then_some(Violation::Limit {
            metric: Metric::ConditionOperators,
            actual,
            max,
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(
        config.rules.condition_complexity.as_ref(),
        |configuration| evaluate_condition_rule::<ConditionComplexity>(facts, configuration),
    )
}
