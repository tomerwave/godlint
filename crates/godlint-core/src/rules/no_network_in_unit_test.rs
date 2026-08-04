use crate::{
    analyzers::SourceFacts,
    config::{Config, NoNetworkInUnitTestRule, Severity},
    facts::CallFact,
    rules::{
        CallInTestRule, Finding, Rule, Violation,
        catalogue::{Catalogue, matches, spelled},
        evaluate_call_in_test_rule, when_configured,
    },
    source::Dialect,
};

const CLIENTS: Catalogue = Catalogue(&[
    ("requests.get", Dialect::Python),
    ("requests.post", Dialect::Python),
    ("requests.put", Dialect::Python),
    ("requests.patch", Dialect::Python),
    ("requests.delete", Dialect::Python),
    ("requests.head", Dialect::Python),
    ("requests.request", Dialect::Python),
    ("httpx.get", Dialect::Python),
    ("httpx.post", Dialect::Python),
    ("httpx.put", Dialect::Python),
    ("httpx.patch", Dialect::Python),
    ("httpx.delete", Dialect::Python),
    ("httpx.head", Dialect::Python),
    ("httpx.request", Dialect::Python),
    ("urllib.request.urlopen", Dialect::Python),
    ("socket.create_connection", Dialect::Python),
    ("fetch", Dialect::JavaScript),
    ("axios.get", Dialect::JavaScript),
    ("axios.post", Dialect::JavaScript),
    ("axios.put", Dialect::JavaScript),
    ("axios.patch", Dialect::JavaScript),
    ("axios.delete", Dialect::JavaScript),
    ("axios.request", Dialect::JavaScript),
    ("http.get", Dialect::JavaScript),
    ("http.request", Dialect::JavaScript),
    ("https.request", Dialect::JavaScript),
    ("https.get", Dialect::JavaScript),
    ("reqwest::get", Dialect::Rust),
    ("reqwest::blocking::get", Dialect::Rust),
    ("ureq::get", Dialect::Rust),
    ("ureq::post", Dialect::Rust),
    ("TcpStream::connect", Dialect::Rust),
]);

pub struct NoNetworkInUnitTest;

impl Rule for NoNetworkInUnitTest {
    const ID: &'static str = "testing/no-network-in-unit-test";

    type Configuration = NoNetworkInUnitTestRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl CallInTestRule for NoNetworkInUnitTest {
    fn check(
        call: &CallFact,
        _facts: &SourceFacts,
        configuration: &Self::Configuration,
    ) -> Option<Violation> {
        let name = spelled(call);
        let source = call.source();

        (matches(source, &configuration.unit_paths) && CLIENTS.speaks(source.language(), &name))
            .then_some(Violation::NetworkInUnitTest { callee: name })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_network_in_unit_test.as_ref(), |rule| {
        evaluate_call_in_test_rule::<NoNetworkInUnitTest>(facts, rule)
    })
}
