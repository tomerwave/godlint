use std::path::Path;

use crate::{
    analyzers::SourceFacts,
    config::{Config, DirectEnvironmentReadRule, Severity},
    facts::{AccessFact, CallFact},
    glob,
    rules::{
        Finding, Rule, RuleError, Violation, evaluate_access_rule, evaluate_call_rule,
        when_configured,
    },
    source::Language,
};

const DEFAULT_ALLOWED_PATHS: [&str; 2] = ["**/config.*", "**/config/**"];

pub struct DirectEnvironmentRead;

impl Rule for DirectEnvironmentRead {
    const ID: &'static str = "security/direct-environment-read";

    type Configuration = DirectEnvironmentReadRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.direct_environment_read.as_ref(), |rule| {
        let severity = DirectEnvironmentRead::severity(rule);
        let allowed = |path: &Path| is_allowed(&path.to_string_lossy(), &rule.allow_in);
        let mut findings =
            evaluate_access_rule(facts, severity, DirectEnvironmentRead::ID, |access| {
                is_environment_access(access)
                    .then(|| direct_read_violation(access.target()))
                    .filter(|_| !allowed(access.source().path()))
            })?;

        findings.extend(evaluate_call_rule(
            facts,
            severity,
            DirectEnvironmentRead::ID,
            |call| {
                is_environment_call(call)
                    .then(|| direct_read_violation(call.callee()))
                    .filter(|_| !allowed(call.source().path()))
            },
        )?);

        Ok(findings)
    })
}

fn is_environment_access(access: &AccessFact) -> bool {
    (matches!(
        access.source().language(),
        Language::JavaScript | Language::TypeScript
    ) && access.target() == "process.env")
        || (access.source().language() == Language::Python && access.target() == "os.environ")
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

fn is_allowed(path: &str, allow_in: &[String]) -> bool {
    DEFAULT_ALLOWED_PATHS
        .iter()
        .copied()
        .chain(allow_in.iter().map(String::as_str))
        .any(|pattern| glob::matches(pattern, path))
}
