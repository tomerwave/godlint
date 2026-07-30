use crate::{
    analyzers::SourceFacts,
    config::{Config, NoRandomnessWithoutSeedRule, Severity},
    facts::CallFact,
    rules::{
        CallInTestRule, Finding, Rule, Violation,
        catalogue::{Catalogue, Dialect, GENERATORS, is_allowed, spelled},
        evaluate_call_in_test_rule, when_configured,
    },
};

const SEEDS: Catalogue = Catalogue(&[
    ("random.seed", Dialect::Python),
    ("random.Random", Dialect::Python),
    ("np.random.seed", Dialect::Python),
    ("numpy.random.seed", Dialect::Python),
    ("Faker.seed", Dialect::Python),
    ("faker.seed_instance", Dialect::Python),
    ("seedrandom", Dialect::JavaScript),
    ("Math.seedrandom", Dialect::JavaScript),
    ("faker.seed", Dialect::JavaScript),
]);

pub struct NoRandomnessWithoutSeed;

impl Rule for NoRandomnessWithoutSeed {
    const ID: &'static str = "testing/no-randomness-without-seed";

    type Configuration = NoRandomnessWithoutSeedRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl CallInTestRule for NoRandomnessWithoutSeed {
    fn check(
        call: &CallFact,
        facts: &SourceFacts,
        configuration: &Self::Configuration,
    ) -> Option<Violation> {
        let name = spelled(call);
        let source = call.source();

        (GENERATORS.speaks(source.language(), &name)
            && !is_allowed(source, &configuration.allow_in)
            && !seeds_its_generator(facts))
        .then_some(Violation::UnseededRandom { callee: name })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_randomness_without_seed.as_ref(), |rule| {
        evaluate_call_in_test_rule::<NoRandomnessWithoutSeed>(facts, rule)
    })
}

fn seeds_its_generator(facts: &SourceFacts) -> bool {
    let language = facts.source().language();

    facts
        .calls()
        .iter()
        .any(|call| SEEDS.speaks(language, &spelled(call)))
}
