use crate::{
    analyzers::SourceFacts,
    config::{Config, NetworkTimeoutRequiredRule, Severity},
    facts::CallFact,
    rules::{
        Absence, CallRule, Finding, Languages, Rule, Violation,
        catalogue::{Catalogue, spelled},
        evaluate_call_rule, when_configured,
    },
    source::{Dialect, Language},
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
    ("http.Get", Dialect::Go),
    ("http.Post", Dialect::Go),
    ("http.PostForm", Dialect::Go),
    ("http.Head", Dialect::Go),
    ("(*http.Client).Do", Dialect::Go),
    ("net.Dial", Dialect::Go),
    ("net.DialContext", Dialect::Go),
    ("net.DialTimeout", Dialect::Go),
    ("tls.Dial", Dialect::Go),
]);

pub struct NetworkTimeoutRequired;

impl Rule for NetworkTimeoutRequired {
    const ID: &'static str = "reliability/network-timeout-required";
    const LANGUAGES: Languages = Languages::all_but(&[
        (Dialect::JavaScript, Absence::NoSuchConstruct),
        (Dialect::Rust, Absence::NoSuchConstruct),
        (Dialect::Workflow, Absence::NoSuchConstruct),
        (Dialect::Repository, Absence::NoSuchConstruct),
    ]);

    type Configuration = NetworkTimeoutRequiredRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl CallRule for NetworkTimeoutRequired {
    fn check(call: &CallFact, _configuration: &Self::Configuration) -> Option<Violation> {
        let name = spelled(call);
        let source = call.source();

        if !CLIENTS.speaks(source.language(), &name) {
            return None;
        }

        let has_timeout = call.named("timeout").is_some()
            || (source.language() == Language::Go
                && (name.ends_with("Timeout") || name.ends_with("WithContext")))
            || (name == "urllib.request.urlopen" && call.argument_count() >= 2)
            || (name == "socket.create_connection" && call.argument_count() >= 2);

        (!has_timeout).then_some(Violation::NetworkTimeoutMissing { callee: name })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.network_timeout_required.as_ref(), |rule| {
        evaluate_call_rule::<NetworkTimeoutRequired>(facts, rule)
    })
}
