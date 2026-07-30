use crate::{
    analyzers::SourceFacts,
    config::{Config, NoSkippedTestRule, Severity},
    facts::{TestFact, TestFocus},
    rules::{Finding, Rule, TestRule, Violation, evaluate_test_rule, when_configured},
};

pub struct NoSkippedTest;

impl Rule for NoSkippedTest {
    const ID: &'static str = "testing/no-skipped-test";

    type Configuration = NoSkippedTestRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl TestRule for NoSkippedTest {
    fn check(test: &TestFact, _configuration: &Self::Configuration) -> Option<Violation> {
        (test.focus() == TestFocus::Skipped).then_some(Violation::SkippedTest)
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_skipped_test.as_ref(), |rule| {
        evaluate_test_rule::<NoSkippedTest>(facts, rule)
    })
}
