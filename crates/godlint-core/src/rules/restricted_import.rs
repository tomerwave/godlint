use crate::{
    analyzers::SourceFacts,
    config::{
        Config, RestrictedImport as RestrictedImportConfiguration, RestrictedImportRule, Severity,
    },
    facts::ImportFact,
    rules::{
        Finding, ImportRule, Rule, Violation, catalogue, evaluate_import_rule, module_path,
        when_configured,
    },
};

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

        (!catalogue::matches(import.source(), &restriction.allow_in)).then(|| {
            Violation::RestrictedImport {
                module: import.module().to_owned(),
            }
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
    let language = import.source().language();

    modules
        .iter()
        .find(|restriction| module_path::covers(&restriction.name, module, language))
}
