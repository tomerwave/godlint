use crate::{
    analyzers::SourceFacts,
    config::{Config, DependencyBoundaryRule, Layer, Severity},
    facts::ImportFact,
    rules::{
        Finding, ImportRule, Rule, Violation, evaluate_import_rule, module_path, scoped,
        when_configured,
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
        let from = scoped::most_specific(layers, |layer| contains(layer, import))?;
        let to = scoped::most_specific(layers, |layer| names(layer, import))?;

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

fn contains(layer: &Layer, import: &ImportFact) -> Option<usize> {
    scoped::longest_match(&layer.paths, import.source().path_text())
}

fn names(layer: &Layer, import: &ImportFact) -> Option<usize> {
    let module = import.module();
    let language = import.source().language();

    layer
        .modules
        .iter()
        .filter(|spelling| module_path::covers(spelling, module, language))
        .map(String::len)
        .max()
}
