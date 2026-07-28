use crate::{
    analyzers::SourceFacts,
    config::{FunctionSizeRule, Severity},
    facts::FunctionFact,
    rules::{Finding, Rule, RuleError, line_count},
};

pub struct FunctionSize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionSizeViolation {
    pub effective_line_count: usize,
}

pub fn evaluate(
    facts: &[SourceFacts],
    configuration: &FunctionSizeRule,
) -> Result<Vec<Finding>, RuleError> {
    let mut findings = Vec::new();

    for source_facts in facts {
        for function in source_facts.functions() {
            let Some(violation) = FunctionSize::evaluate(function, configuration) else {
                continue;
            };

            findings.push(finding(function, violation, configuration)?);
        }
    }

    Ok(findings)
}

impl Rule for FunctionSize {
    type Input = FunctionFact;
    type Configuration = FunctionSizeRule;
    type Violation = FunctionSizeViolation;

    const ID: &'static str = "maintainability/function-size";

    fn evaluate(
        function: &Self::Input,
        configuration: &Self::Configuration,
    ) -> Option<Self::Violation> {
        if configuration.severity == Severity::Off {
            return None;
        }

        let effective_line_count = line_count::effective_line_count(
            function.source(),
            function.range(),
            configuration.skip_blank_lines,
            configuration.skip_comments,
        );

        (effective_line_count > configuration.max_lines as usize).then_some(FunctionSizeViolation {
            effective_line_count,
        })
    }
}

fn finding(
    function: &FunctionFact,
    violation: FunctionSizeViolation,
    configuration: &FunctionSizeRule,
) -> Result<Finding, RuleError> {
    let location = function
        .source()
        .location(function.range())
        .map_err(|source| RuleError::LocatesSource { source })?;

    Ok(Finding {
        path: function.source().path().to_path_buf(),
        line: location.start.line,
        column: location.start.column,
        severity: configuration.severity,
        rule_id: FunctionSize::ID,
        message: format!(
            "Function has {} effective lines (max {}).",
            violation.effective_line_count, configuration.max_lines
        ),
    })
}
