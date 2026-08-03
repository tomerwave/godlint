use crate::{
    analyzers::SourceFacts,
    config::{Config, IndependentSet, ModuleIndependenceRule, Severity},
    facts::ImportFact,
    rules::{Finding, ImportRule, Rule, Violation, evaluate_import_rule, scoped, when_configured},
};

pub struct ModuleIndependence;

impl Rule for ModuleIndependence {
    const ID: &'static str = "architecture/module-independence";

    type Configuration = ModuleIndependenceRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl ImportRule for ModuleIndependence {
    fn check(import: &ImportFact, configuration: &Self::Configuration) -> Option<Violation> {
        configuration
            .sets
            .iter()
            .find_map(|set| crossing(set, import))
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.module_independence.as_ref(), |rule| {
        evaluate_import_rule::<ModuleIndependence>(facts, rule)
    })
}

fn crossing(set: &IndependentSet, import: &ImportFact) -> Option<Violation> {
    let members = &set.members;
    let (from, to) = scoped::endpoints(members, import)?;

    (to != from).then(|| Violation::BrokeIndependence {
        set: set.name.clone(),
        from: members[from].name.clone(),
        to: members[to].name.clone(),
    })
}
