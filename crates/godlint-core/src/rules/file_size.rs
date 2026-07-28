use crate::{
    analyzers::SourceFacts,
    config::{FileSizeRule, Severity},
    rules::{Finding, Rule, RuleError, line_count},
    source::SourceFile,
};

pub struct FileSize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileSizeViolation {
    pub effective_line_count: usize,
}

pub fn evaluate(
    facts: &[SourceFacts],
    configuration: &FileSizeRule,
) -> Result<Vec<Finding>, RuleError> {
    facts
        .iter()
        .filter_map(|source_facts| {
            FileSize::evaluate(source_facts, configuration)
                .map(|violation| finding(source_facts.source(), violation, configuration))
        })
        .collect()
}

impl Rule for FileSize {
    type Input = SourceFacts;
    type Configuration = FileSizeRule;
    type Violation = FileSizeViolation;

    const ID: &'static str = "maintainability/file-size";

    fn evaluate(
        source_facts: &Self::Input,
        configuration: &Self::Configuration,
    ) -> Option<Self::Violation> {
        if configuration.severity == Severity::Off {
            return None;
        }

        let source = source_facts.source();
        let effective_line_count = line_count::effective_line_count(
            source,
            source.full_range(),
            configuration.skip_blank_lines,
            configuration.skip_comments,
        );

        (effective_line_count > configuration.max_lines as usize).then_some(FileSizeViolation {
            effective_line_count,
        })
    }
}

fn finding(
    source: &SourceFile,
    violation: FileSizeViolation,
    configuration: &FileSizeRule,
) -> Result<Finding, RuleError> {
    let location = source
        .location(source.full_range())
        .map_err(|source| RuleError::LocatesSource { source })?;

    Ok(Finding {
        path: source.path().to_path_buf(),
        line: location.start.line,
        column: location.start.column,
        severity: configuration.severity,
        rule_id: FileSize::ID,
        message: format!(
            "File has {} effective lines (max {}).",
            violation.effective_line_count, configuration.max_lines
        ),
    })
}
