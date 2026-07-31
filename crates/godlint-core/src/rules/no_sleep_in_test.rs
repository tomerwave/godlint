use crate::{
    analyzers::SourceFacts,
    config::{Config, NoSleepInTestRule, Severity},
    facts::CallFact,
    rules::{
        CallInTestRule, Finding, Rule, Violation,
        catalogue::{Catalogue, spelled},
        evaluate_call_in_test_rule, when_configured,
    },
    source::{Dialect, SourceRange, range_contains},
};

const TIMERS: Catalogue = Catalogue(&[
    ("setTimeout", Dialect::JavaScript),
    ("setInterval", Dialect::JavaScript),
]);

const PROMISE: &str = "Promise";

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
        facts: &SourceFacts,
        _configuration: &Self::Configuration,
    ) -> Option<Violation> {
        let name = spelled(call);
        let language = call.source().language();

        (SLEEPS.speaks(language, &name) || waits_on_a_promise(call, facts, &name))
            .then_some(Violation::SleepInTest { callee: name })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_sleep_in_test.as_ref(), |rule| {
        evaluate_call_in_test_rule::<NoSleepInTest>(facts, rule)
    })
}

fn waits_on_a_promise(call: &CallFact, facts: &SourceFacts, name: &str) -> bool {
    TIMERS.speaks(call.source().language(), name)
        && facts
            .calls()
            .iter()
            .filter(|other| other.callee() == PROMISE)
            .filter(|promise| range_contains(promise.extent(), call.extent()))
            .any(|promise| does_nothing_else(facts, promise.extent()))
}

fn does_nothing_else(facts: &SourceFacts, promise: SourceRange) -> bool {
    facts
        .calls()
        .iter()
        .filter(|call| call.extent() != promise && range_contains(promise, call.extent()))
        .count()
        == 1
}
