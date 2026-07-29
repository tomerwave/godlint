use crate::{
    analyzers::SourceFacts,
    config::{Config, DependencyBoundaryRule, Layer, Severity},
    facts::ImportFact,
    glob,
    rules::{
        Finding, ImportRule, Rule, Violation, evaluate_import_rule, module_path, when_configured,
    },
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
        let from = position(layers, |layer| contains(layer, import))?;
        let to = position(layers, |layer| names(layer, import.module()))?;

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

fn position(layers: &[Layer], belongs: impl Fn(&Layer) -> bool) -> Option<usize> {
    layers.iter().position(belongs)
}

fn contains(layer: &Layer, import: &ImportFact) -> bool {
    glob::matches_any(
        layer.paths.iter().map(String::as_str),
        &import.source().path().to_string_lossy(),
    )
}

fn names(layer: &Layer, module: &str) -> bool {
    layer
        .modules
        .iter()
        .any(|spelling| module_path::covers(spelling, module))
}
