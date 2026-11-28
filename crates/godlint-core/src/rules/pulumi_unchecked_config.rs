use crate::{
    analyzers::SourceFacts,
    config::{Config, NoProductionLogRule, Severity},
    facts::CallFact,
    rules::{
        Absence, Finding, Languages, Reporting, Rule, Violation, collect_findings, when_configured,
    },
    source::Dialect,
};

const SDK: &str = "@pulumi/pulumi";

const GETTERS: [&str; 4] = [
    "requireObject",
    "getObject",
    "requireSecretObject",
    "getSecretObject",
];

pub struct PulumiUncheckedConfig;

impl Rule for PulumiUncheckedConfig {
    const ID: &'static str = "infra/pulumi-unchecked-config";

    const LANGUAGES: Languages = Languages::all_but(&[
        (Dialect::Python, Absence::NotImplemented),
        (Dialect::Go, Absence::NoSuchConstruct),
        (Dialect::Rust, Absence::NoSuchConstruct),
        (Dialect::Workflow, Absence::NoSuchConstruct),
    ]);

    type Configuration = NoProductionLogRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.pulumi_unchecked_config.as_ref(), |rule| {
        collect_findings(
            facts,
            Reporting::of::<PulumiUncheckedConfig>(rule),
            SourceFacts::calls,
            |call, source| {
                (imports_the_sdk(source) && casts_configuration(call)).then(|| {
                    (
                        call.range(),
                        Violation::UncheckedConfig {
                            callee: call.callee().to_owned(),
                        },
                    )
                })
            },
        )
    })
}

fn imports_the_sdk(source: &SourceFacts) -> bool {
    source.imports().iter().any(|import| {
        import.module() == SDK
            || import
                .module()
                .strip_prefix(SDK)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

fn casts_configuration(call: &CallFact) -> bool {
    call.has_type_arguments()
        && call
            .callee()
            .rsplit_once('.')
            .is_some_and(|(_, method)| GETTERS.contains(&method))
}
