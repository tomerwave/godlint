use crate::{
    analyzers::SourceFacts,
    config::{Config, DirectEnvironmentReadRule, Severity},
    facts::{AccessFact, CallFact},
    rules::{
        AccessRule, CallRule, Finding, Rule, Violation, catalogue::Catalogue, evaluate_access_rule,
        evaluate_call_rule, when_configured,
    },
    source::Dialect,
};

const READS: Catalogue = Catalogue(&[
    ("process.env", Dialect::JavaScript),
    ("os.environ", Dialect::Python),
]);

const READERS: Catalogue = Catalogue(&[
    ("os.getenv", Dialect::Python),
    ("std::env::var", Dialect::Rust),
]);

pub struct DirectEnvironmentRead;

impl Rule for DirectEnvironmentRead {
    const ID: &'static str = "security/direct-environment-read";

    type Configuration = DirectEnvironmentReadRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl AccessRule for DirectEnvironmentRead {
    fn check(access: &AccessFact, _configuration: &Self::Configuration) -> Option<Violation> {
        is_environment_access(access).then(|| direct_read_violation(access.target()))
    }
}

impl CallRule for DirectEnvironmentRead {
    fn check(call: &CallFact, _configuration: &Self::Configuration) -> Option<Violation> {
        is_environment_call(call).then(|| direct_read_violation(call.callee()))
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.direct_environment_read.as_ref(), |rule| {
        let mut findings = evaluate_access_rule::<DirectEnvironmentRead>(facts, rule);

        findings.extend(evaluate_call_rule::<DirectEnvironmentRead>(facts, rule));

        findings
    })
}

fn is_environment_access(access: &AccessFact) -> bool {
    READS.speaks(access.source().language(), access.target())
}

fn is_environment_call(call: &CallFact) -> bool {
    READERS.speaks(call.source().language(), call.callee())
}

fn direct_read_violation(target: &str) -> Violation {
    Violation::DirectEnvironmentRead {
        target: target.to_owned(),
    }
}
