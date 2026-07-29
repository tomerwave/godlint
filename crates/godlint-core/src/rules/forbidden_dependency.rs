use crate::{
    analyzers::SourceFacts,
    config::{Config, ForbiddenDependencyRule, Severity},
    facts::ImportFact,
    glob,
    rules::{
        Finding, ImportRule, Rule, Violation, evaluate_import_rule, module_path, when_configured,
    },
};

pub struct ForbiddenDependency;

impl Rule for ForbiddenDependency {
    const ID: &'static str = "security/forbidden-dependency";

    type Configuration = ForbiddenDependencyRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl ImportRule for ForbiddenDependency {
    fn check(import: &ImportFact, configuration: &Self::Configuration) -> Option<Violation> {
        let package = module_path::package(import.module(), import.source().language())?;
        let forbidden = configuration
            .packages
            .iter()
            .find(|entry| entry.name == package)?;

        (!is_allowed(import, &forbidden.allow_in)).then(|| Violation::ForbiddenDependency {
            package: package.to_owned(),
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.forbidden_dependency.as_ref(), |rule| {
        evaluate_import_rule::<ForbiddenDependency>(facts, rule)
    })
}

fn is_allowed(import: &ImportFact, paths: &[String]) -> bool {
    glob::matches_any(
        paths.iter().map(String::as_str),
        import.source().path_text(),
    )
}
