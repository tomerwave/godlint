use crate::{
    analyzers::SourceFacts,
    config::{Config, NoTestHelperInProductionRule, Severity},
    facts::ImportFact,
    glob,
    rules::{Finding, ImportRule, Rule, Violation, evaluate_import_rule, when_configured},
    source::Language,
};

pub struct NoTestHelperInProduction;

impl Rule for NoTestHelperInProduction {
    const ID: &'static str = "testing/no-test-helper-in-production";

    type Configuration = NoTestHelperInProductionRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl ImportRule for NoTestHelperInProduction {
    fn check(import: &ImportFact, configuration: &Self::Configuration) -> Option<Violation> {
        let source = import.source();
        let module = import.module();

        if is_test(source.path_text(), configuration) || !is_local(module, source.language()) {
            return None;
        }

        names_a_test_tree(module, source.language(), configuration).map(|segment| {
            Violation::TestHelperInProduction {
                module: module.to_owned(),
                segment,
            }
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_test_helper_in_production.as_ref(), |rule| {
        evaluate_import_rule::<NoTestHelperInProduction>(facts, rule)
    })
}

fn is_test(path: &str, configuration: &NoTestHelperInProductionRule) -> bool {
    glob::matches_any(configuration.test_paths.iter().map(String::as_str), path)
}

fn is_local(module: &str, language: Language) -> bool {
    if language == Language::Rust {
        return module.starts_with("crate::") || module.starts_with("super::");
    }

    module.starts_with('.')
}

fn names_a_test_tree(
    module: &str,
    language: Language,
    configuration: &NoTestHelperInProductionRule,
) -> Option<String> {
    module
        .split(separator(language))
        .find(|segment| {
            configuration
                .helpers
                .iter()
                .any(|helper| helper.eq_ignore_ascii_case(segment))
        })
        .map(str::to_owned)
}

fn separator(language: Language) -> char {
    match language {
        Language::Python => '.',
        Language::Rust => ':',
        Language::JavaScript | Language::TypeScript => '/',
    }
}
