use crate::{
    analyzers::SourceFacts,
    config::{CognitiveComplexityRule, Config, Severity},
    facts::FunctionFact,
    rules::{
        Finding, FunctionLimitRule, Metric, Rule, evaluate_function_limit_rule, when_configured,
    },
};

pub struct CognitiveComplexity;

impl Rule for CognitiveComplexity {
    const ID: &'static str = "maintainability/cognitive-complexity";

    type Configuration = CognitiveComplexityRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl FunctionLimitRule for CognitiveComplexity {
    const METRIC: Metric = Metric::CognitiveScore;

    fn measure(
        function: &FunctionFact,
        _facts: &SourceFacts,
        _configuration: &Self::Configuration,
    ) -> u32 {
        function.cognitive_score().value()
    }

    fn max(configuration: &Self::Configuration) -> u32 {
        configuration.limit()
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(
        config.rules.cognitive_complexity.as_ref(),
        |configuration| evaluate_function_limit_rule::<CognitiveComplexity>(facts, configuration),
    )
}
