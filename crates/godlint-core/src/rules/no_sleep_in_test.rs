use crate::{
    analyzers::SourceFacts,
    config::{Config, NoSleepInTestRule, Severity},
    facts::CallFact,
    rules::{
        CallInTestRule, Finding, Rule, Violation,
        catalogue::{Catalogue, Dialect, spelled},
        evaluate_call_in_test_rule, when_configured,
    },
};

const SLEEPS: Catalogue = Catalogue(&[
    ("time.sleep", Dialect::Python),
    ("asyncio.sleep", Dialect::Python),
    ("thread::sleep", Dialect::Rust),
    ("std::thread::sleep", Dialect::Rust),
    ("tokio::time::sleep", Dialect::Rust),
    ("page.waitForTimeout", Dialect::JavaScript),
    ("browser.pause", Dialect::JavaScript),
]);

pub struct NoSleepInTest;

impl Rule for NoSleepInTest {
    const ID: &'static str = "testing/no-sleep-in-test";

    type Configuration = NoSleepInTestRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl CallInTestRule for NoSleepInTest {
    fn check(
        call: &CallFact,
        _facts: &SourceFacts,
        _configuration: &Self::Configuration,
    ) -> Option<Violation> {
        let name = spelled(call);

        SLEEPS
            .speaks(call.source().language(), &name)
            .then_some(Violation::SleepInTest { callee: name })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_sleep_in_test.as_ref(), |rule| {
        evaluate_call_in_test_rule::<NoSleepInTest>(facts, rule)
    })
}
