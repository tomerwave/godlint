use crate::{
    analyzers::SourceFacts,
    config::{Config, FilenameCaseRule, NamingCase, Severity},
    glob,
    rules::{
        FileRule, Finding, Rule, Violation, evaluate_file_rule, module_path, scoped,
        when_configured,
    },
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

        let expected = expected_case(configuration, source);
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

fn expected_case(configuration: &FilenameCaseRule, source: &SourceFile) -> NamingCase {
    scoped::most_specific(&configuration.scopes, |scope| {
        scoped::longest_match(&scope.paths, source.path_text())
    })
    .and_then(|index| configuration.scopes.get(index))
    .map_or_else(|| conventional_case(source), |scope| scope.case)
}

fn conventional_case(source: &SourceFile) -> NamingCase {
    match source.language() {
        Language::Python | Language::Rust => NamingCase::Snake,
        Language::JavaScript | Language::TypeScript => ecmascript_case(source),
    }
}

fn ecmascript_case(source: &SourceFile) -> NamingCase {
    match module_path::last_segment(file_name(source), '.') {
        "jsx" | "tsx" => NamingCase::Pascal,
        _ => NamingCase::Kebab,
    }
}

fn matches(patterns: &[String], source: &SourceFile) -> bool {
    glob::matches_any(patterns.iter().map(String::as_str), source.path_text())
}

fn stem(source: &SourceFile) -> Option<String> {
    let stem = module_path::first_segment(file_name(source), ".");

    (!stem.is_empty()).then(|| stem.to_owned())
}

fn file_name(source: &SourceFile) -> &str {
    module_path::last_segment(source.path_text(), '/')
}

fn follows(name: &str, case: NamingCase) -> bool {
    match case {
        NamingCase::Kebab => separated(name, '-'),
        NamingCase::Snake => separated(name, '_'),
        NamingCase::Camel => bounded(name, char::is_ascii_lowercase),
        NamingCase::Pascal => bounded(name, char::is_ascii_uppercase),
    }
}

fn separated(name: &str, separator: char) -> bool {
    let trimmed = name.trim_matches(separator);

    !trimmed.is_empty() && trimmed.split(separator).all(is_lower_segment)
}

fn is_lower_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .chars()
            .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
}

fn bounded(name: &str, leads: impl Fn(&char) -> bool) -> bool {
    let mut characters = name.chars();

    characters.next().as_ref().is_some_and(leads)
        && characters.all(|character| character.is_ascii_alphanumeric())
}
