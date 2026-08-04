use crate::{
    analyzers::SourceFacts,
    config::{Config, NoWeakHashRule, Severity},
    facts::CallFact,
    rules::{
        CallRule, Finding, Rule, Violation,
        catalogue::{Catalogue, spelled},
        evaluate_call_rule, when_configured,
    },
    source::{Dialect, Language},
};

const FACTORIES: Catalogue = Catalogue(&[
    ("crypto.createHash", Dialect::JavaScript),
    ("crypto.createHmac", Dialect::JavaScript),
    ("hashlib.new", Dialect::Python),
]);

const WEAK_ALGORITHMS: [&str; 7] = [
    "md2",
    "md4",
    "md5",
    "sha1",
    "ripemd",
    "ripemd128",
    "ripemd160",
];

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
    fn check(call: &CallFact, _configuration: &Self::Configuration) -> Option<Violation> {
        hash_violation(call, spelled(call))
    }
}

fn hash_violation(call: &CallFact, name: String) -> Option<Violation> {
    let language = call.source().language();
    let strong = strong_hash(language).to_owned();

    if WEAK.speaks(language, &name) {
        return Some(Violation::WeakHash { weak: name, strong });
    }

    if !FACTORIES.speaks(language, &name) {
        return None;
    }

    match &call.positional(0)?.literal {
        Some(algorithm) => {
            weak_algorithm(algorithm).map(|weak| Violation::WeakHash { weak, strong })
        }
        None => Some(Violation::UnverifiedHash { callee: name }),
    }
}

fn weak_algorithm(algorithm: &str) -> Option<String> {
    let normalized = algorithm.to_lowercase().replace(['-', '_'], "");

    WEAK_ALGORITHMS
        .contains(&normalized.as_str())
        .then_some(normalized)
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_weak_hash.as_ref(), |rule| {
        evaluate_call_rule::<NoWeakHash>(facts, rule)
    })
}

fn strong_hash(language: Language) -> &'static str {
    match language {
        Language::JavaScript | Language::TypeScript => "sha256",
        Language::Python => "hashlib.sha256",
        Language::Rust => "sha2::Sha256",
    }
}
