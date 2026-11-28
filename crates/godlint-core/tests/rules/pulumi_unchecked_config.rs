use godlint_core::rules::{Rule, Violation, pulumi_unchecked_config};

use super::support::rule_violations;

const CONFIGURATION: &str =
    "version: 1\nrules:\n  infra/pulumi-unchecked-config:\n    severity: error\n";

const SDK: &str =
    "import * as pulumi from \"@pulumi/pulumi\";\nconst config = new pulumi.Config();\n";

fn violations_in(path: &str, source: &str) -> Vec<Violation> {
    rule_violations(
        pulumi_unchecked_config::evaluate,
        path,
        source,
        CONFIGURATION,
    )
}

fn violations(program: &str) -> Vec<Violation> {
    violations_in("infra/index.ts", &format!("{SDK}{program}"))
}

fn unchecked(callee: &str) -> Vec<Violation> {
    vec![Violation::UncheckedConfig {
        callee: callee.to_owned(),
    }]
}

#[test]
fn rule_is_registered() {
    assert_eq!(
        pulumi_unchecked_config::PulumiUncheckedConfig::ID,
        "infra/pulumi-unchecked-config"
    );
}

#[test]
fn reports_every_object_getter_given_a_type_argument() {
    for getter in [
        "requireObject",
        "getObject",
        "requireSecretObject",
        "getSecretObject",
    ] {
        assert_eq!(
            violations(&format!(
                "const value = config.{getter}<Settings>(\"key\");"
            )),
            unchecked(&format!("config.{getter}")),
            "{getter}"
        );
    }
}

#[test]
fn reports_every_shape_of_type_argument_and_receiver() {
    for (program, callee) in [
        (
            "config.requireObject<string[]>(\"k\");",
            "config.requireObject",
        ),
        (
            "config.requireObject<{ name: string }>(\"k\");",
            "config.requireObject",
        ),
        ("config.getObject<A | B>(\"k\");", "config.getObject"),
        ("config?.getObject<A>(\"k\");", "config.getObject"),
        (
            "stack.config.requireObject<A>(\"k\");",
            "stack.config.requireObject",
        ),
    ] {
        assert_eq!(violations(program), unchecked(callee), "{program}");
    }
}

#[test]
fn reads_the_sdk_by_its_package_or_a_path_under_it() {
    let program = "config.requireObject<A>(\"k\");";

    for import in [
        "import { Config } from \"@pulumi/pulumi\";\n",
        "import * as config from \"@pulumi/pulumi/config\";\n",
    ] {
        assert_eq!(
            violations_in("index.ts", &format!("{import}{program}")),
            unchecked("config.requireObject"),
            "{import}"
        );
    }
}

#[test]
fn accepts_a_getter_without_a_type_argument() {
    assert!(violations("const value = config.requireObject(\"key\");").is_empty());
    assert!(violations("const value = Settings.parse(config.getObject(\"key\"));").is_empty());
}

#[test]
fn ignores_other_methods_and_bare_functions() {
    assert!(violations("config.require<string>(\"key\");").is_empty());
    assert!(violations("config.get<number>(\"key\");").is_empty());
    assert!(violations("requireObject<A>(\"key\");").is_empty());
    assert!(violations("config.requireObjects<A>(\"key\");").is_empty());
}

#[test]
fn ignores_a_file_that_does_not_import_the_sdk() {
    for import in [
        "import * as aws from \"@pulumi/aws\";\n",
        "import * as pulumi from \"@pulumi/pulumi-extra\";\n",
        "import { config } from \"./config\";\n",
    ] {
        assert!(
            violations_in(
                "index.ts",
                &format!("{import}config.requireObject<A>(\"k\");")
            )
            .is_empty(),
            "{import}"
        );
    }
}

#[test]
fn reports_the_callee_with_the_schema_remedy() {
    assert_eq!(
        Violation::UncheckedConfig {
            callee: "config.requireObject".to_owned()
        }
        .to_string(),
        "config.requireObject casts configuration to its type argument without checking it; drop the type argument and validate the value with a schema, such as `schema.parse(config.requireObject(\"key\"))`."
    );
}
