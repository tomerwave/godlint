use crate::{
    analyzers::SourceFacts,
    config::{Config, LineLimitRule, Severity},
    facts::FunctionFact,
    rules::{
        Finding, FunctionLimitRule, Metric, Rule, RuleError, evaluate_function_limit_rule,
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

impl FunctionLimitRule for FunctionSize {
    const METRIC: Metric = Metric::FunctionLines;

    fn measure(
        function: &FunctionFact,
        facts: &SourceFacts,
        configuration: &Self::Configuration,
    ) -> u32 {
        line_count::effective_line_count(facts, function.range(), skipped(configuration))
    }

    fn max(configuration: &Self::Configuration) -> u32 {
        configuration.max_lines.get()
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.function_size.as_ref(), |configuration| {
        evaluate_function_limit_rule::<FunctionSize>(facts, configuration)
    })
}

fn skipped(configuration: &LineLimitRule) -> line_count::Skipped {
    line_count::Skipped {
        blank_lines: configuration.skip_blank_lines,
        comments: configuration.skip_comments,
    }
}
