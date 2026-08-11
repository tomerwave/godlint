use crate::{
    config::{Config, NoProductionLogRule, Severity},
    repository::RepositoryFacts,
    rules::{Finding, Languages, Reporting, Rule, Violation, report, when_configured},
};

pub struct NoCommittedSecretFile;

impl Rule for NoCommittedSecretFile {
    const ID: &'static str = "repository/no-committed-secret-file";
    const LANGUAGES: Languages = Languages::REPOSITORY;
    type Configuration = NoProductionLogRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(repository: &RepositoryFacts, config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_committed_secret_file.as_ref(), |rule| {
        report(
            Reporting::of::<NoCommittedSecretFile>(rule),
            repository
                .secret_files()
                .iter()
                .map(|file| (file, file.full_range(), Violation::CommittedSecretFile)),
        )
    })
}
