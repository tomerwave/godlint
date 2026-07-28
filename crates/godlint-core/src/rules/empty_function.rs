use crate::{
    analyzers::SourceFacts,
    config::{Config, EmptyFunctionRule, Severity},
    facts::FunctionFact,
    rules::{
        Finding, FunctionRule, Rule, RuleError, Violation, evaluate_function_rule, when_configured,
    },
};

pub struct EmptyFunction;

impl Rule for EmptyFunction {
    const ID: &'static str = "maintainability/empty-function";

    type Configuration = EmptyFunctionRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl FunctionRule for EmptyFunction {
    /// Reports a body that declares no work and says nothing about why.
    ///
    /// Three kinds of emptiness are deliberate and are not reported: an interface stub,
    /// where every body is a placeholder by construction; a declaration the language
    /// marks as having no implementation, such as an abstract method or a constructor
    /// whose parameters carry the assignment; and a body holding a comment, which is the
    /// author already explaining the omission.
    fn check(
        function: &FunctionFact,
        _facts: &SourceFacts,
        configuration: &Self::Configuration,
    ) -> Option<Violation> {
        if function.source().is_interface_stub()
            || function.is_abstract()
            || !function.body_is_empty()
            || name_is_allowed(function, configuration)
        {
            return None;
        }

        Some(Violation::EmptyBody)
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

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.empty_function.as_ref(), |configuration| {
        evaluate_function_rule::<EmptyFunction>(facts, configuration)
    })
}
