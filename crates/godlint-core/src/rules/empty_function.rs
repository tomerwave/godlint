use crate::{
    analyzers::SourceFacts,
    config::{EmptyFunctionRule, Severity},
    facts::FunctionFact,
    rules::{Finding, Rule, RuleError},
};

pub struct EmptyFunction;

pub fn evaluate(
    facts: &[SourceFacts],
    configuration: &EmptyFunctionRule,
) -> Result<Vec<Finding>, RuleError> {
    let mut findings = Vec::new();

    for source_facts in facts {
        for function in source_facts.functions() {
            if EmptyFunction::evaluate(function, configuration).is_some() {
                findings.push(finding(function, configuration)?);
            }
        }
    }

    Ok(findings)
}

impl Rule for EmptyFunction {
    type Input = FunctionFact;
    type Configuration = EmptyFunctionRule;
    type Violation = ();

    const ID: &'static str = "maintainability/empty-function";

    fn evaluate(
        function: &Self::Input,
        configuration: &Self::Configuration,
    ) -> Option<Self::Violation> {
        if configuration.severity == Severity::Off
            || !function.body_is_empty()
            || name_is_allowed(function, configuration)
        {
            return None;
        }

        Some(())
    }
}

fn name_is_allowed(function: &FunctionFact, configuration: &EmptyFunctionRule) -> bool {
    function.name().is_some_and(|name| {
        configuration
            .allow_names
            .iter()
            .any(|allowed| allowed == name)
    })
}

fn finding(
    function: &FunctionFact,
    configuration: &EmptyFunctionRule,
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
        rule_id: EmptyFunction::ID,
        message: "Function has an empty body.".into(),
    })
}
