use crate::{
    analyzers::SourceFacts,
    config::{Config, NoWeakHashRule, Severity},
    facts::CallFact,
    rules::{
        CallRule, Finding, Rule, Violation,
        catalogue::{Catalogue, Dialect, is_allowed, spelled},
        evaluate_call_rule, when_configured,
    },
};

const WEAK: Catalogue = Catalogue(&[
    ("hashlib.md5", Dialect::Python),
    ("hashlib.sha1", Dialect::Python),
    ("md5::compute", Dialect::Rust),
    ("Md5::new", Dialect::Rust),
    ("Sha1::new", Dialect::Rust),
]);

pub struct NoWeakHash;

impl Rule for NoWeakHash {
    const ID: &'static str = "security/no-weak-hash";

    type Configuration = NoWeakHashRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl CallRule for NoWeakHash {
    fn check(call: &CallFact, configuration: &Self::Configuration) -> Option<Violation> {
        let name = spelled(call);
        let source = call.source();

        (WEAK.speaks(source.language(), &name) && !is_allowed(source, &configuration.allow_in))
            .then(|| Violation::WeakHash {
                callee: name.clone(),
                strong: strong_hash(&name).to_owned(),
            })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_weak_hash.as_ref(), |rule| {
        evaluate_call_rule::<NoWeakHash>(facts, rule)
    })
}

fn strong_hash(callee: &str) -> &'static str {
    if callee.starts_with("hashlib.") {
        "hashlib.sha256"
    } else {
        "sha2::Sha256"
    }
}
