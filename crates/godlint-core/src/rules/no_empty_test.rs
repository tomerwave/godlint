use crate::{
    analyzers::SourceFacts,
    config::{Config, NoEmptyTestRule, Severity},
    facts::TestFact,
    rules::{
        Finding, Reporting, Rule, Violation, collect_ranged, enclosing::test_body, when_configured,
    },
};

pub struct NoEmptyTest;

impl Rule for NoEmptyTest {
    const ID: &'static str = "testing/no-empty-test";

    type Configuration = NoEmptyTestRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_empty_test.as_ref(), |rule| {
        collect_ranged(
            facts,
            Reporting::of::<NoEmptyTest>(rule),
            SourceFacts::tests,
            check,
        )
    })
}

fn check(test: &TestFact, facts: &SourceFacts) -> Option<Violation> {
    let body = test_body(facts, test)?;

    body.body_is_empty().then_some(Violation::EmptyTest)
}
