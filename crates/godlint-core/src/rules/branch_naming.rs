use crate::{
    config::{BranchNamingRule, Config, Severity},
    glob,
    repository::RepositoryFacts,
    rules::{Finding, Languages, Reporting, Rule, Violation, report, when_configured},
};

pub struct BranchNaming;

impl Rule for BranchNaming {
    const ID: &'static str = "git/branch-naming";

    const LANGUAGES: Languages = Languages::REPOSITORY;

    type Configuration = BranchNamingRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(repository: &RepositoryFacts, config: &Config) -> Vec<Finding> {
    when_configured(config.rules.branch_naming.as_ref(), |rule| {
        let Some(branch) = repository.branch() else {
            return Vec::new();
        };

        report(
            Reporting::of::<BranchNaming>(rule),
            (!valid(branch.text(), rule)).then(|| {
                (
                    branch,
                    branch.full_range(),
                    Violation::InvalidBranchName {
                        name: branch.text().to_owned(),
                        types: rule.types.clone(),
                    },
                )
            }),
        )
    })
}

fn valid(branch: &str, configuration: &BranchNamingRule) -> bool {
    glob::matches_any(configuration.allow.iter().map(String::as_str), branch)
        || branch.split_once('/').is_some_and(|(kind, description)| {
            configuration.types.iter().any(|entry| entry == kind) && slug(description)
        })
}

fn slug(value: &str) -> bool {
    value
        .starts_with(|character: char| character.is_ascii_lowercase() || character.is_ascii_digit())
        && value.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || "._/-".contains(character)
        })
}
