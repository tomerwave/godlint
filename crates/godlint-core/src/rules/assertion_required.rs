use crate::{
    analyzers::SourceFacts,
    config::{AssertionRequiredRule, Config, Severity},
    facts::TestFact,
    rules::{
        Finding, Reporting, Rule, Violation,
        catalogue::spelled,
        collect_ranged,
        enclosing::{encloses_a_test, test_body},
        when_configured,
    },
};

pub struct AssertionRequired;

impl Rule for AssertionRequired {
    const ID: &'static str = "testing/assertion-required";

    type Configuration = AssertionRequiredRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.assertion_required.as_ref(), |rule| {
        collect_ranged(
            facts,
            Reporting::of::<AssertionRequired>(rule),
            SourceFacts::tests,
            |test, source| check(test, source, rule),
        )
    })
}

fn check(
    test: &TestFact,
    facts: &SourceFacts,
    configuration: &AssertionRequiredRule,
) -> Option<Violation> {
    let body = test_body(facts, test)?;

    (!body.body_is_empty() && !encloses_a_test(facts, test) && !asserts(facts, test, configuration))
        .then_some(Violation::MissingAssertion)
}

fn asserts(facts: &SourceFacts, test: &TestFact, configuration: &AssertionRequiredRule) -> bool {
    facts
        .assertions()
        .iter()
        .any(|assertion| test.contains(assertion.range()))
        || facts.calls().iter().any(|call| {
            test.contains(call.range()) && configuration.extra_assertions.contains(&spelled(call))
        })
}
