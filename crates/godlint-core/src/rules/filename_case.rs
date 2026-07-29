use crate::{
    analyzers::SourceFacts,
    config::{Config, FilenameCaseRule, NamingCase, Severity},
    glob,
    rules::{FileRule, Finding, Rule, Violation, evaluate_file_rule, module_path, when_configured},
    source::{Language, SourceFile},
};

pub struct FilenameCase;

impl Rule for FilenameCase {
    const ID: &'static str = "architecture/filename-case";

    type Configuration = FilenameCaseRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl FileRule for FilenameCase {
    fn check(source: &SourceFile, configuration: &Self::Configuration) -> Option<Violation> {
        if matches(&configuration.allow, source) {
            return None;
        }

        let expected = expected_case(configuration, source)?;
        let name = stem(source)?;

        (!follows(&name, expected)).then(|| Violation::FilenameCase {
            name,
            case: expected.describe().to_owned(),
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.filename_case.as_ref(), |rule| {
        evaluate_file_rule::<FilenameCase>(facts, rule)
    })
}

fn expected_case(configuration: &FilenameCaseRule, source: &SourceFile) -> Option<NamingCase> {
    configuration
        .scopes
        .iter()
        .find(|scope| matches(&scope.paths, source))
        .map(|scope| scope.case)
        .or_else(|| conventional_case(source.language()))
}

fn conventional_case(language: Language) -> Option<NamingCase> {
    match language {
        Language::Python | Language::Rust => Some(NamingCase::Snake),
        Language::JavaScript | Language::TypeScript => None,
    }
}

fn matches(patterns: &[String], source: &SourceFile) -> bool {
    glob::matches_any(
        patterns.iter().map(String::as_str),
        &source.path().to_string_lossy(),
    )
}

fn stem(source: &SourceFile) -> Option<String> {
    let path = source.path().to_string_lossy();
    let name = module_path::last_segment(&path, '/');
    let stem = module_path::first_segment(name, ".");

    (!stem.is_empty()).then(|| stem.to_owned())
}

fn follows(name: &str, case: NamingCase) -> bool {
    match case {
        NamingCase::Kebab => separated(name, '-'),
        NamingCase::Snake => separated(name, '_'),
        NamingCase::Camel => bounded(name, char::is_lowercase),
        NamingCase::Pascal => bounded(name, char::is_uppercase),
    }
}

fn separated(name: &str, separator: char) -> bool {
    !name.is_empty()
        && name.split(separator).all(|segment| {
            !segment.is_empty()
                && segment
                    .chars()
                    .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        })
}

fn bounded(name: &str, leads: impl Fn(char) -> bool) -> bool {
    let mut characters = name.chars();

    characters.next().is_some_and(leads)
        && characters.all(|character| character.is_ascii_alphanumeric())
}
