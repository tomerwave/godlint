use crate::{
    analyzers::SourceFacts,
    config::{ReturnCountRule, Severity},
    facts::FunctionFact,
    rules::{Finding, Rule, RuleError},
};

pub struct ReturnCount;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReturnCountViolation {
    pub return_count: u32,
}

pub fn evaluate(
    facts: &[SourceFacts],
    configuration: &ReturnCountRule,
) -> Result<Vec<Finding>, RuleError> {
    let mut findings = Vec::new();

    for source_facts in facts {
        for function in source_facts.functions() {
            let Some(violation) = ReturnCount::evaluate(function, configuration) else {
                continue;
            };

            findings.push(finding(function, violation, configuration)?);
        }
    }

    Ok(findings)
}

impl Rule for ReturnCount {
    type Input = FunctionFact;
    type Configuration = ReturnCountRule;
    type Violation = ReturnCountViolation;

    const ID: &'static str = "maintainability/return-count";

    fn evaluate(
        function: &Self::Input,
        configuration: &Self::Configuration,
    ) -> Option<Self::Violation> {
        if configuration.severity == Severity::Off {
            return None;
        }

        (function.return_count() > configuration.max_returns).then_some(ReturnCountViolation {
            return_count: function.return_count(),
        })
    }
}

fn finding(
    function: &FunctionFact,
    violation: ReturnCountViolation,
    configuration: &ReturnCountRule,
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
        rule_id: ReturnCount::ID,
        message: format!(
            "Function has {} returns (max {}).",
            violation.return_count, configuration.max_returns
        ),
    })
}
