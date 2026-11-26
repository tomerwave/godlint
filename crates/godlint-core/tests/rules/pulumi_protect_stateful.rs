use godlint_core::rules::{Violation, pulumi_protect_stateful};

use super::support::rule_violations;

const CONFIGURATION: &str =
    "version: 1\nrules:\n  infra/pulumi-protect-stateful:\n    severity: error\n";

const PROVIDER: &str = "import * as aws from \"@pulumi/aws\";\n";

fn violations_in(path: &str, source: &str) -> Vec<Violation> {
    rule_violations(
        pulumi_protect_stateful::evaluate,
        path,
        source,
        CONFIGURATION,
    )
}

fn violations(program: &str) -> Vec<Violation> {
    violations_in("infra/index.ts", &format!("{PROVIDER}{program}"))
}

fn unprotected(resource: &str) -> Vec<Violation> {
    vec![Violation::UnprotectedStatefulResource {
        resource: resource.to_owned(),
    }]
}

#[test]
fn reports_every_catalogued_stateful_resource_without_options() {
    for resource in [
        "aws.rds.Instance",
        "aws.rds.Cluster",
        "aws.s3.Bucket",
        "aws.s3.BucketV2",
        "aws.dynamodb.Table",
        "aws.kms.Key",
        "aws.elasticache.ReplicationGroup",
        "aws.elasticache.ServerlessCache",
        "aws.elasticache.Cluster",
        "aws.efs.FileSystem",
        "aws.ebs.Volume",
        "aws.docdb.Cluster",
        "aws.neptune.Cluster",
        "aws.redshift.Cluster",
        "aws.memorydb.Cluster",
        "aws.opensearch.Domain",
        "aws.backup.Vault",
    ] {
        assert_eq!(
            violations(&format!("new {resource}(\"data\", {{}});")),
            unprotected(resource),
            "{resource}"
        );
    }
}

#[test]
fn reports_options_that_do_not_protect() {
    assert_eq!(
        violations("new aws.s3.Bucket(\"data\", {}, { dependsOn: [] });"),
        unprotected("aws.s3.Bucket")
    );
    assert_eq!(
        violations("new aws.s3.Bucket(\"data\", {}, { protect: false });"),
        unprotected("aws.s3.Bucket")
    );
    assert_eq!(
        violations("new aws.s3.Bucket(\"data\", {}, {});"),
        unprotected("aws.s3.Bucket")
    );
    assert_eq!(
        violations("new aws.s3.Bucket(\"data\");"),
        unprotected("aws.s3.Bucket"),
        "a resource with no arguments object has no options either"
    );
}

#[test]
fn accepts_protection_however_the_key_is_spelled() {
    assert!(violations("new aws.s3.Bucket(\"data\", {}, { protect: true });").is_empty());
    assert!(violations("new aws.s3.Bucket(\"data\", {}, { \"protect\": true });").is_empty());
    assert!(
        violations("new aws.s3.Bucket(\"data\", {}, { retainOnDelete: true, protect: true });")
            .is_empty()
    );
}

#[test]
fn stays_silent_on_options_it_cannot_read() {
    assert!(violations("new aws.s3.Bucket(\"data\", {}, options);").is_empty());
    assert!(violations("new aws.s3.Bucket(\"data\", {}, { ...options });").is_empty());
    assert!(violations("new aws.s3.Bucket(\"data\", {}, { protect: isProduction });").is_empty());
    assert!(violations("new aws.s3.Bucket(\"data\", {}, { protect });").is_empty());
}

#[test]
fn reads_named_and_subpath_imports_of_the_provider() {
    assert_eq!(
        violations_in(
            "infra/index.ts",
            "import { s3 } from \"@pulumi/aws\";\nnew s3.Bucket(\"data\");"
        ),
        unprotected("s3.Bucket")
    );
    assert_eq!(
        violations_in(
            "infra/index.js",
            "import * as rds from \"@pulumi/aws/rds\";\nnew rds.Instance(\"data\");"
        ),
        unprotected("rds.Instance")
    );
}

#[test]
fn ignores_a_file_that_does_not_import_the_provider() {
    assert!(violations_in("infra/index.ts", "new aws.s3.Bucket(\"data\");").is_empty());
    assert!(
        violations_in(
            "infra/index.ts",
            "import * as aws from \"@pulumi/awsx\";\nnew aws.s3.Bucket(\"data\");"
        )
        .is_empty()
    );
    assert!(
        violations_in(
            "infra/index.ts",
            "import * as aws from \"@pulumi/aws-native\";\nnew aws.s3.Bucket(\"data\");"
        )
        .is_empty()
    );
}

#[test]
fn ignores_resources_outside_the_catalogue() {
    assert!(violations("new aws.sns.Topic(\"events\");").is_empty());
    assert!(violations("new aws.s3.BucketPolicy(\"policy\", {});").is_empty());
    assert!(violations("new Bucket(\"data\");").is_empty());
    assert!(violations("new deep.aws.s3.Bucket(\"data\");").is_empty());
}

#[test]
fn does_not_read_other_languages() {
    assert!(
        violations_in(
            "infra/__main__.py",
            "import pulumi_aws as aws\naws.s3.Bucket(\"data\")\n"
        )
        .is_empty()
    );
}
