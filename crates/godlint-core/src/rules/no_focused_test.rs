use crate::{
    analyzers::SourceFacts,
    config::{Config, NoFocusedTestRule, Severity},
    facts::{TestFact, TestFocus},
    rules::{
        Absence, Finding, Languages, Rule, TestRule, Violation, evaluate_test_rule, when_configured,
    },
    source::Dialect,
};

pub struct NoFocusedTest;

impl Rule for NoFocusedTest {
    const ID: &'static str = "testing/no-focused-test";

    const LANGUAGES: Languages = Languages::all_but(&[
        (Dialect::Python, Absence::NoSuchConstruct),
        (Dialect::Rust, Absence::NoSuchConstruct),
        (Dialect::Workflow, Absence::NoSuchConstruct),
    ]);

    type Configuration = NoFocusedTestRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl TestRule for NoFocusedTest {
    fn check(test: &TestFact, _configuration: &Self::Configuration) -> Option<Violation> {
        (test.focus() == TestFocus::Only).then_some(Violation::FocusedTest)
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_focused_test.as_ref(), |rule| {
        evaluate_test_rule::<NoFocusedTest>(facts, rule)
    })
}
