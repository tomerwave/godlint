use crate::{
    config::{Config, LockfileVersionDriftRule, Severity},
    repository::RepositoryFacts,
    rules::{Finding, Languages, Reporting, Rule, Violation, report, when_configured},
};

pub struct LockfileVersionDrift;

impl Rule for LockfileVersionDrift {
    const ID: &'static str = "dependencies/lockfile-version-drift";
    const LANGUAGES: Languages = Languages::REPOSITORY;
    type Configuration = LockfileVersionDriftRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(repository: &RepositoryFacts, config: &Config) -> Vec<Finding> {
    when_configured(config.rules.lockfile_version_drift.as_ref(), |rule| {
        report(
            Reporting::of::<LockfileVersionDrift>(rule),
            repository.version_drifts().iter().map(|fact| {
                (
                    fact.file(),
                    fact.range(),
                    Violation::DependencyPolicy { message: format!("{} declares version {} but {} records {}; regenerate and commit the lockfile.", fact.package(), fact.declared(), fact.lockfile().display(), fact.locked()) },
                )
            }),
        )
    })
}
