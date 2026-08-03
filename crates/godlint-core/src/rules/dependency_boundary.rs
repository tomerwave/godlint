use crate::{
    analyzers::SourceFacts,
    config::{Config, DependencyBoundaryRule, Severity},
    facts::ImportFact,
    rules::{Finding, ImportRule, Rule, Violation, evaluate_import_rule, scoped, when_configured},
};

pub struct DependencyBoundary;

impl Rule for DependencyBoundary {
    const ID: &'static str = "architecture/dependency-boundary";

    type Configuration = DependencyBoundaryRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl ImportRule for DependencyBoundary {
    fn check(import: &ImportFact, configuration: &Self::Configuration) -> Option<Violation> {
        let layers = &configuration.layers;
        let (from, to) = scoped::endpoints(layers, import)?;

        (to < from).then(|| Violation::CrossedBoundary {
            from: layers[from].name.clone(),
            to: layers[to].name.clone(),
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.dependency_boundary.as_ref(), |rule| {
        evaluate_import_rule::<DependencyBoundary>(facts, rule)
    })
}
