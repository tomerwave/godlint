use crate::{
    analyzers::SourceFacts,
    config::{Config, LineLimitRule, Severity},
    facts::FunctionFact,
    rules::{
        Finding, FunctionRule, Metric, Rule, RuleError, Violation, evaluate_function_rule,
        line_count, when_configured,
    },
};

pub struct FunctionSize;

impl Rule for FunctionSize {
    const ID: &'static str = "maintainability/function-size";

    type Configuration = LineLimitRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl FunctionRule for FunctionSize {
    fn check(
        function: &FunctionFact,
        facts: &SourceFacts,
        configuration: &Self::Configuration,
    ) -> Option<Violation> {
        let actual = line_count::effective_line_count(
            facts,
            function.range(),
            configuration.skip_blank_lines,
            configuration.skip_comments,
        );
        let max = configuration.max_lines.get();

        (actual > max).then_some(Violation::limit(Metric::FunctionLines, actual, max))
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.function_size.as_ref(), |configuration| {
        evaluate_function_rule::<FunctionSize>(facts, configuration)
    })
}
