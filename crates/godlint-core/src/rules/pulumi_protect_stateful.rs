use crate::{
    analyzers::SourceFacts,
    config::{Config, PulumiProtectStatefulRule, Severity},
    facts::{CallArgument, CallFact},
    rules::{
        Absence, Finding, Languages, Reporting, Rule, Violation, collect_findings, when_configured,
    },
    source::Dialect,
};

const PROVIDER: &str = "@pulumi/aws";

const STATEFUL: [&str; 17] = [
    "rds.Instance",
    "rds.Cluster",
    "s3.Bucket",
    "s3.BucketV2",
    "dynamodb.Table",
    "kms.Key",
    "elasticache.ReplicationGroup",
    "elasticache.ServerlessCache",
    "elasticache.Cluster",
    "efs.FileSystem",
    "ebs.Volume",
    "docdb.Cluster",
    "neptune.Cluster",
    "redshift.Cluster",
    "memorydb.Cluster",
    "opensearch.Domain",
    "backup.Vault",
];

const OPTIONS: usize = 2;

pub struct PulumiProtectStateful;

impl Rule for PulumiProtectStateful {
    const ID: &'static str = "infra/pulumi-protect-stateful";

    const LANGUAGES: Languages = Languages::all_but(&[
        (Dialect::Python, Absence::NotImplemented),
        (Dialect::Go, Absence::NotImplemented),
        (Dialect::Rust, Absence::NoSuchConstruct),
        (Dialect::Workflow, Absence::NoSuchConstruct),
    ]);

    type Configuration = PulumiProtectStatefulRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.pulumi_protect_stateful.as_ref(), |rule| {
        collect_findings(
            facts,
            Reporting::of::<PulumiProtectStateful>(rule),
            SourceFacts::calls,
            |call, source| {
                (imports_the_provider(source) && is_unprotected(call)).then(|| {
                    (
                        call.range(),
                        Violation::UnprotectedStatefulResource {
                            resource: call.callee().to_owned(),
                        },
                    )
                })
            },
        )
    })
}

fn imports_the_provider(source: &SourceFacts) -> bool {
    source.imports().iter().any(|import| {
        import.module() == PROVIDER
            || import
                .module()
                .strip_prefix(PROVIDER)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

fn is_unprotected(call: &CallFact) -> bool {
    names_a_stateful_resource(call.callee())
        && call.positional(OPTIONS).is_none_or(declines_protection)
}

fn names_a_stateful_resource(callee: &str) -> bool {
    let segments: Vec<&str> = callee.split('.').collect();

    match segments.as_slice() {
        [.., service, resource] if segments.len() <= 3 => {
            STATEFUL.contains(&format!("{service}.{resource}").as_str())
        }
        _ => false,
    }
}

fn declines_protection(options: &CallArgument) -> bool {
    options.properties.is_some()
        && options.property("protect").is_none_or(|protect| {
            protect
                .literal
                .as_deref()
                .is_some_and(|value| value != "true")
        })
}
