use std::path::Path;

use crate::{
    analyzers::SourceFacts,
    config::{Config, DirectEnvironmentReadRule, Severity},
    facts::{AccessFact, CallFact},
    glob,
    rules::{
        AccessRule, CallRule, Finding, Rule, Violation, evaluate_access_rule, evaluate_call_rule,
        when_configured,
    },
    source::Language,
};

pub struct DirectEnvironmentRead;

impl Rule for DirectEnvironmentRead {
    const ID: &'static str = "security/direct-environment-read";

    type Configuration = DirectEnvironmentReadRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl AccessRule for DirectEnvironmentRead {
    fn check(access: &AccessFact, configuration: &Self::Configuration) -> Option<Violation> {
        (is_environment_access(access)
            && !is_allowed(access.source().path(), &configuration.allow_in))
        .then(|| direct_read_violation(access.target()))
    }
}

impl CallRule for DirectEnvironmentRead {
    fn check(call: &CallFact, configuration: &Self::Configuration) -> Option<Violation> {
        (is_environment_call(call) && !is_allowed(call.source().path(), &configuration.allow_in))
            .then(|| direct_read_violation(call.callee()))
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
    match access.source().language() {
        Language::JavaScript | Language::TypeScript => access.target() == "process.env",
        Language::Python => access.target() == "os.environ",
        Language::Rust => false,
    }
}

fn is_environment_call(call: &CallFact) -> bool {
    match call.source().language() {
        Language::Python => call.callee() == "os.getenv",
        Language::Rust => call.callee() == "std::env::var",
        Language::JavaScript | Language::TypeScript => false,
    }
}

fn direct_read_violation(target: &str) -> Violation {
    Violation::DirectEnvironmentRead {
        target: target.to_owned(),
    }
}

fn is_allowed(path: &Path, allow_in: &[String]) -> bool {
    glob::matches_any(allow_in.iter().map(String::as_str), &path.to_string_lossy())
}
