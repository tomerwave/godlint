use crate::{
    analyzers::SourceFacts,
    config::{FunctionStatementsRule, Severity},
    facts::FunctionFact,
    rules::{Finding, Rule, RuleError},
};

pub struct FunctionStatements;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionStatementsViolation {
    pub statement_count: u32,
}

pub fn evaluate(
    facts: &[SourceFacts],
    configuration: &FunctionStatementsRule,
) -> Result<Vec<Finding>, RuleError> {
    let mut findings = Vec::new();

    for source_facts in facts {
        for function in source_facts.functions() {
            let Some(violation) = FunctionStatements::evaluate(function, configuration) else {
                continue;
            };

            findings.push(finding(function, violation, configuration)?);
        }
    }

    Ok(findings)
}

impl Rule for FunctionStatements {
    type Input = FunctionFact;
    type Configuration = FunctionStatementsRule;
    type Violation = FunctionStatementsViolation;

    const ID: &'static str = "maintainability/function-statements";

    fn evaluate(
        function: &Self::Input,
        configuration: &Self::Configuration,
    ) -> Option<Self::Violation> {
        if configuration.severity == Severity::Off {
            return None;
        }

        (function.statement_count() > configuration.max_statements).then_some(
            FunctionStatementsViolation {
                statement_count: function.statement_count(),
            },
        )
    }
}

fn finding(
    function: &FunctionFact,
    violation: FunctionStatementsViolation,
    configuration: &FunctionStatementsRule,
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
        rule_id: FunctionStatements::ID,
        message: format!(
            "Function has {} statements (max {}).",
            violation.statement_count, configuration.max_statements
        ),
    })
}
