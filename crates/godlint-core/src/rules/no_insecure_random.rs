use crate::{
    analyzers::SourceFacts,
    config::{Config, NoInsecureRandomRule, Severity},
    facts::CallFact,
    rules::{
        CallRule, Finding, Rule, Violation,
        catalogue::{GENERATORS, spelled},
        evaluate_call_rule, when_configured,
    },
    source::Language,
};

pub struct NoInsecureRandom;

impl Rule for NoInsecureRandom {
    const ID: &'static str = "security/no-insecure-random";

    type Configuration = NoInsecureRandomRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl CallRule for NoInsecureRandom {
    fn check(call: &CallFact, _configuration: &Self::Configuration) -> Option<Violation> {
        let name = spelled(call);
        let source = call.source();

        GENERATORS
            .speaks(source.language(), &name)
            .then(|| Violation::InsecureRandom {
                callee: name.clone(),
                secure: secure_generator(source.language()).to_owned(),
            })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_insecure_random.as_ref(), |rule| {
        evaluate_call_rule::<NoInsecureRandom>(facts, rule)
    })
}

fn secure_generator(language: Language) -> &'static str {
    match language {
        Language::JavaScript | Language::TypeScript => "crypto.getRandomValues",
        Language::Python => "secrets",
        Language::Rust => "rand::rngs::OsRng",
    }
}
