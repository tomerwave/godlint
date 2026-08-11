use crate::{
    analyzers::SourceFacts,
    config::{Config, NoInternalImportRule, Severity},
    facts::ImportFact,
    glob,
    rules::{
        Absence, Finding, ImportRule, Languages, Rule, Violation, evaluate_import_rule,
        module_path, when_configured,
    },
    source::{Dialect, Language},
};

const HIDDEN: [&str; 4] = ["internal", "private", "impl", "_internal"];

const BUILT: [&str; 3] = ["dist", "src", "build"];

pub struct NoInternalImport;

impl Rule for NoInternalImport {
    const ID: &'static str = "architecture/no-internal-import";

    const LANGUAGES: Languages = Languages::all_but(&[
        (Dialect::Go, Absence::NoSuchConstruct),
        (Dialect::Rust, Absence::NoSuchConstruct),
        (Dialect::Workflow, Absence::NoSuchConstruct),
    ]);

    type Configuration = NoInternalImportRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl ImportRule for NoInternalImport {
    fn check(import: &ImportFact, configuration: &Self::Configuration) -> Option<Violation> {
        let module = import.module();

        if is_own(module, import.source().language()) || is_permitted(module, configuration) {
            return None;
        }

        reached_past(module, import.source().language()).map(|marker| Violation::InternalImport {
            certain: !BUILT.contains(&marker.as_str()),
            module: module.to_owned(),
            marker,
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_internal_import.as_ref(), |rule| {
        evaluate_import_rule::<NoInternalImport>(facts, rule)
    })
}

fn is_own(module: &str, language: Language) -> bool {
    language == Language::Rust
        || module.starts_with('.')
        || module.starts_with('/')
        || module.starts_with('#')
}

fn is_permitted(module: &str, configuration: &NoInternalImportRule) -> bool {
    glob::matches_any(configuration.allow.iter().map(String::as_str), module)
}

fn reached_past(module: &str, language: Language) -> Option<String> {
    let reached: Vec<&str> = module_path::segments(module, language)
        .skip(own_segments(module))
        .filter(|segment| is_marker(segment, language))
        .collect();

    reached
        .iter()
        .find(|segment| !BUILT.contains(*segment))
        .or(reached.first())
        .map(|segment| (*segment).to_owned())
}

fn own_segments(module: &str) -> usize {
    if module.starts_with('@') { 2 } else { 1 }
}

fn is_marker(segment: &str, language: Language) -> bool {
    HIDDEN.contains(&segment)
        || BUILT.contains(&segment)
        || (language == Language::Python && is_author_private(segment))
}

fn is_author_private(segment: &str) -> bool {
    segment.starts_with('_') && !(segment.starts_with("__") && segment.ends_with("__"))
}
