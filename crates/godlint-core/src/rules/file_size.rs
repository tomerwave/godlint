use crate::{
    analyzers::SourceFacts,
    config::{Config, LineLimitRule, Severity},
    rules::{
        FileRule, Finding, Metric, Rule, RuleError, Violation, evaluate_file_rule, line_count,
        when_configured,
    },
};

pub struct FileSize;

impl Rule for FileSize {
    const ID: &'static str = "maintainability/file-size";

    type Configuration = LineLimitRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl FileRule for FileSize {
    fn check(facts: &SourceFacts, configuration: &Self::Configuration) -> Option<Violation> {
        let actual = line_count::effective_line_count(
            facts,
            facts.source().full_range(),
            configuration.skip_blank_lines,
            configuration.skip_comments,
        );
        let max = configuration.max_lines.get();

        (actual > max).then_some(Violation::Limit {
            metric: Metric::FileLines,
            actual,
            max,
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.file_size.as_ref(), |configuration| {
        evaluate_file_rule::<FileSize>(facts, configuration)
    })
}
