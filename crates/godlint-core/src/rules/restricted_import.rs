use crate::{
    analyzers::SourceFacts,
    config::{
        Config, RestrictedImport as RestrictedImportConfiguration, RestrictedImportRule, Severity,
    },
    facts::ImportFact,
    glob,
    rules::{Finding, ImportRule, Rule, Violation, evaluate_import_rule, when_configured},
};

const SEPARATORS: [char; 3] = [':', '.', '/'];

pub struct RestrictedImport;

impl Rule for RestrictedImport {
    const ID: &'static str = "architecture/restricted-import";

    type Configuration = RestrictedImportRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl ImportRule for RestrictedImport {
    fn check(import: &ImportFact, configuration: &Self::Configuration) -> Option<Violation> {
        let restriction = restriction(import, &configuration.modules)?;

        (!is_allowed(import, &restriction.allow_in)).then(|| Violation::RestrictedImport {
            module: import.module().to_owned(),
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.restricted_import.as_ref(), |rule| {
        evaluate_import_rule::<RestrictedImport>(facts, rule)
    })
}

fn restriction<'a>(
    import: &ImportFact,
    modules: &'a [RestrictedImportConfiguration],
) -> Option<&'a RestrictedImportConfiguration> {
    let module = import.module();

    modules
        .iter()
        .find(|restriction| covers(&restriction.name, module))
}

fn covers(restricted: &str, module: &str) -> bool {
    let Some(rest) = module.strip_prefix(restricted) else {
        return false;
    };

    rest.is_empty() || rest.starts_with(SEPARATORS)
}

fn is_allowed(import: &ImportFact, paths: &[String]) -> bool {
    glob::matches_any(
        paths.iter().map(String::as_str),
        &import.source().path().to_string_lossy(),
    )
}
