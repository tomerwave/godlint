use crate::{
    config::{Config, NoProductionLogRule, Severity},
    facts::DependencySpecFact,
    repository::RepositoryFacts,
    rules::{Finding, Languages, Reporting, Rule, Violation, report, when_configured},
};

const ALIAS: &str = "npm:";
const LATEST: &str = "latest";
const ANY_OF: &str = "||";
const OPERATORS: [&str; 7] = [">=", "<=", ">", "<", "=", "^", "~"];
const PARTS: usize = 3;
const GIT_PREFIXES: [&str; 7] = [
    "git+",
    "git://",
    "git@",
    "github:",
    "gitlab:",
    "bitbucket:",
    "gist:",
];
const TARBALLS: [&str; 3] = [".tgz", ".tar", ".tar.gz"];
const COMMIT_LENGTHS: std::ops::RangeInclusive<usize> = 7..=40;
const RANGE: &str = "which can resolve to more than one release; declare the exact version so every install gets the same one.";
const MOVABLE_REFERENCE: &str =
    "a git reference that can move; pin it to a commit hash so every install gets the same code.";

pub struct ExactVersions;

impl Rule for ExactVersions {
    const ID: &'static str = "dependencies/exact-versions";
    const LANGUAGES: Languages = Languages::REPOSITORY;
    type Configuration = NoProductionLogRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(repository: &RepositoryFacts, config: &Config) -> Vec<Finding> {
    when_configured(config.rules.exact_versions.as_ref(), |rule| {
        report(
            Reporting::of::<ExactVersions>(rule),
            repository.dependency_specs().iter().filter_map(|fact| {
                remedy(fact.spec())
                    .map(|remedy| (fact.file(), fact.range(), violation(fact, remedy)))
            }),
        )
    })
}

fn violation(fact: &DependencySpecFact, remedy: &str) -> Violation {
    Violation::DependencyPolicy {
        message: format!(
            "{} in {} is \"{}\", {remedy}",
            fact.package(),
            fact.section(),
            fact.spec()
        ),
    }
}

fn remedy(spec: &str) -> Option<&'static str> {
    let spec = spec.trim();

    if names_git(spec) {
        return (!pinned_to_commit(spec)).then_some(MOVABLE_REFERENCE);
    }

    let floats = match spec.strip_prefix(ALIAS) {
        Some(alias) => aliased_version(alias).is_none_or(is_range),
        None => is_range(spec),
    };

    floats.then_some(RANGE)
}

fn names_git(spec: &str) -> bool {
    GIT_PREFIXES.iter().any(|prefix| spec.starts_with(prefix)) || shorthand(spec)
}

fn shorthand(spec: &str) -> bool {
    let repository = spec
        .split_once('#')
        .map_or(spec, |(repository, _)| repository);

    let tarball = TARBALLS
        .iter()
        .any(|extension| repository.ends_with(extension));

    !tarball
        && repository
            .split_once('/')
            .is_some_and(|(owner, name)| repository_part(owner) && repository_part(name))
}

fn repository_part(part: &str) -> bool {
    part.starts_with(char::is_alphanumeric)
        && part
            .chars()
            .all(|character| character.is_alphanumeric() || "-_.".contains(character))
}

fn pinned_to_commit(spec: &str) -> bool {
    spec.rsplit_once('#').is_some_and(|(_, commit)| {
        COMMIT_LENGTHS.contains(&commit.len())
            && commit.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

fn aliased_version(alias: &str) -> Option<&str> {
    alias
        .strip_prefix('@')
        .unwrap_or(alias)
        .split_once('@')
        .map(|(_, version)| version)
}

fn is_range(spec: &str) -> bool {
    let spec = spec.trim();

    spec.is_empty() || spec == LATEST || (!spec.contains([':', '/']) && semver_range(spec))
}

fn semver_range(spec: &str) -> bool {
    let comparators: Vec<&str> = spec.split(ANY_OF).flat_map(str::split_whitespace).collect();

    comparators
        .iter()
        .all(|comparator| comparator_shaped(comparator))
        && (spec.contains(ANY_OF) || !matches!(comparators.as_slice(), [only] if exact(only)))
}

fn comparator_shaped(comparator: &str) -> bool {
    let (operator, version) = split_operator(comparator);

    comparator == "-" || (version.is_empty() && !operator.is_empty()) || version_shaped(version)
}

fn split_operator(comparator: &str) -> (&str, &str) {
    OPERATORS
        .into_iter()
        .find_map(|operator| {
            comparator
                .strip_prefix(operator)
                .map(|version| (operator, version))
        })
        .unwrap_or(("", comparator))
}

fn version_shaped(version: &str) -> bool {
    let parts = core_parts(version);

    (1..=PARTS).contains(&parts.len())
        && parts
            .iter()
            .all(|part| numeric(part) || matches!(*part, "x" | "X" | "*"))
}

fn exact(comparator: &str) -> bool {
    let (operator, version) = split_operator(comparator);
    let parts = core_parts(version);

    matches!(operator, "" | "=") && parts.len() == PARTS && parts.iter().all(|part| numeric(part))
}

fn core_parts(version: &str) -> Vec<&str> {
    let version = version.strip_prefix('v').unwrap_or(version);
    let core = version.split(['-', '+']).next().unwrap_or_default();

    core.split('.').collect()
}

fn numeric(part: &str) -> bool {
    !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())
}
