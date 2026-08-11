use crate::{
    analyzers::SourceFacts,
    config::{Config, ForbiddenDependencyRule, Severity},
    facts::ImportFact,
    rules::{
        Finding, ImportRule, Rule, Violation, catalogue, evaluate_import_rule, module_path,
        when_configured,
    },
    source::Language,
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
        let forbidden = configuration.packages.iter().find(|entry| {
            if import.source().language() == Language::Go {
                module_path::covers(&entry.name, import.module(), Language::Go)
            } else {
                entry.name == package
            }
        })?;

        let package = if import.source().language() == Language::Go {
            import.module()
        } else {
            package
        };

        (!catalogue::matches(import.source(), &forbidden.allow_in)).then(|| {
            Violation::ForbiddenDependency {
                package: package.to_owned(),
            }
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.forbidden_dependency.as_ref(), |rule| {
        evaluate_import_rule::<ForbiddenDependency>(facts, rule)
    })
}
