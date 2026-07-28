use crate::{
    analyzers::SourceFacts,
    config::{ParameterCountRule, Severity},
    facts::FunctionFact,
    rules::{Finding, Rule, RuleError},
};

pub struct ParameterCount;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParameterCountViolation {
    pub parameter_count: u32,
}

pub fn evaluate(
    facts: &[SourceFacts],
    configuration: &ParameterCountRule,
) -> Result<Vec<Finding>, RuleError> {
    let mut findings = Vec::new();

    for source_facts in facts {
        for function in source_facts.functions() {
            let Some(violation) = ParameterCount::evaluate(function, configuration) else {
                continue;
            };

            findings.push(finding(function, violation, configuration)?);
        }
    }

    Ok(findings)
}

impl Rule for ParameterCount {
    type Input = FunctionFact;
    type Configuration = ParameterCountRule;
    type Violation = ParameterCountViolation;

    const ID: &'static str = "maintainability/parameter-count";

    fn evaluate(
        function: &Self::Input,
        configuration: &Self::Configuration,
    ) -> Option<Self::Violation> {
        if configuration.severity == Severity::Off {
            return None;
        }

        (function.parameter_count() > configuration.max_parameters).then_some(
            ParameterCountViolation {
                parameter_count: function.parameter_count(),
            },
        )
    }
}

fn finding(
    function: &FunctionFact,
    violation: ParameterCountViolation,
    configuration: &ParameterCountRule,
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
        rule_id: ParameterCount::ID,
        message: format!(
            "Function has {} parameters (max {}).",
            violation.parameter_count, configuration.max_parameters
        ),
    })
}
