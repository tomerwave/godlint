use crate::{
    analyzers::SourceFacts,
    config::{FunctionNestingRule, Severity},
    facts::FunctionFact,
    rules::{Finding, Rule, RuleError},
};

pub struct FunctionNesting;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionNestingViolation {
    pub nesting_depth: u32,
}

pub fn evaluate(
    facts: &[SourceFacts],
    configuration: &FunctionNestingRule,
) -> Result<Vec<Finding>, RuleError> {
    let mut findings = Vec::new();

    for source_facts in facts {
        for function in source_facts.functions() {
            let Some(violation) = FunctionNesting::evaluate(function, configuration) else {
                continue;
            };

            findings.push(finding(function, violation, configuration)?);
        }
    }

    Ok(findings)
}

impl Rule for FunctionNesting {
    type Input = FunctionFact;
    type Configuration = FunctionNestingRule;
    type Violation = FunctionNestingViolation;

    const ID: &'static str = "maintainability/function-nesting";

    fn evaluate(
        function: &Self::Input,
        configuration: &Self::Configuration,
    ) -> Option<Self::Violation> {
        if configuration.severity == Severity::Off {
            return None;
        }

        (function.nesting_depth() > configuration.max_depth).then_some(FunctionNestingViolation {
            nesting_depth: function.nesting_depth(),
        })
    }
}

fn finding(
    function: &FunctionFact,
    violation: FunctionNestingViolation,
    configuration: &FunctionNestingRule,
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
        rule_id: FunctionNesting::ID,
        message: format!(
            "Function is nested at depth {} (max {}).",
            violation.nesting_depth, configuration.max_depth
        ),
    })
}
