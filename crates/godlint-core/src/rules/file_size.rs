use crate::{
    analyzers::SourceFacts,
    config::{Config, LineLimitRule, Severity},
    rules::{
        FileLimitRule, Finding, Metric, Rule, evaluate_file_limit_rule, line_count, when_configured,
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

impl FileLimitRule for FileSize {
    const METRIC: Metric = Metric::FileLines;

    fn measure(facts: &SourceFacts, configuration: &Self::Configuration) -> u32 {
        line_count::effective_line_count(facts, facts.source().full_range(), configuration.into())
    }

    fn max(configuration: &Self::Configuration) -> u32 {
        configuration.max_lines.get()
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.file_size.as_ref(), |configuration| {
        evaluate_file_limit_rule::<FileSize>(facts, configuration)
    })
}
