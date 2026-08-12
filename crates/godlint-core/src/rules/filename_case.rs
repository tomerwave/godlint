use crate::{
    analyzers::SourceFacts,
    config::{Config, FilenameCaseRule, NamingCase, Severity},
    rules::{
        FileRule, Finding, Rule, Violation, catalogue, evaluate_file_rule, module_path, scoped,
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
        if catalogue::matches(source, &configuration.allow) || is_framework_route_file(source) {
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
        Language::Go | Language::Python | Language::Rust => NamingCase::Snake,
        Language::JavaScript | Language::TypeScript => ecmascript_case(source),
    }
}

fn ecmascript_case(source: &SourceFile) -> NamingCase {
    match module_path::last_segment(file_name(source), '.') {
        "jsx" | "tsx" => NamingCase::Pascal,
        _ => NamingCase::Kebab,
    }
}

fn stem(source: &SourceFile) -> Option<String> {
    let stem = module_path::first_segment(file_name(source), ".");

    (!stem.is_empty()).then(|| stem.to_owned())
}

fn file_name(source: &SourceFile) -> &str {
    module_path::last_segment(source.path_text(), '/')
}

fn is_framework_route_file(source: &SourceFile) -> bool {
    let name = file_name(source);

    dynamic_route_prefix(name, "[[...", "]]")
        || dynamic_route_prefix(name, "[...", "]")
        || dynamic_route_prefix(name, "[", "]")
}

fn dynamic_route_prefix(name: &str, prefix: &str, close: &str) -> bool {
    name.strip_prefix(prefix)
        .and_then(|rest| {
            rest.find(close)
                .map(|index| (&rest[..index], &rest[index + close.len()..]))
        })
        .is_some_and(|(parameter, suffix)| {
            !parameter.is_empty()
                && !parameter.starts_with("...")
                && !parameter.contains(['[', ']'])
                && suffix.starts_with('.')
        })
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
