use crate::{
    analyzers::SourceFacts,
    config::{CyclomaticComplexityRule, Severity},
    facts::FunctionFact,
    rules::{Finding, Rule, RuleError},
};

pub struct CyclomaticComplexity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CyclomaticComplexityViolation {
    pub complexity: u32,
}

pub fn evaluate(
    facts: &[SourceFacts],
    configuration: &CyclomaticComplexityRule,
) -> Result<Vec<Finding>, RuleError> {
    let mut findings = Vec::new();

    for source_facts in facts {
        for function in source_facts.functions() {
            let Some(violation) = CyclomaticComplexity::evaluate(function, configuration) else {
                continue;
            };

            findings.push(finding(function, violation, configuration)?);
        }
    }

    Ok(findings)
}

impl Rule for CyclomaticComplexity {
    type Input = FunctionFact;
    type Configuration = CyclomaticComplexityRule;
    type Violation = CyclomaticComplexityViolation;

    const ID: &'static str = "maintainability/cyclomatic-complexity";

    fn evaluate(
        function: &Self::Input,
        configuration: &Self::Configuration,
    ) -> Option<Self::Violation> {
        if configuration.severity == Severity::Off {
            return None;
        }

        let complexity = function.decision_points() + 1;

        (complexity > configuration.max_complexity)
            .then_some(CyclomaticComplexityViolation { complexity })
    }
}

fn finding(
    function: &FunctionFact,
    violation: CyclomaticComplexityViolation,
    configuration: &CyclomaticComplexityRule,
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
        rule_id: CyclomaticComplexity::ID,
        message: format!(
            "Function has cyclomatic complexity {} (max {}).",
            violation.complexity, configuration.max_complexity
        ),
    })
}
