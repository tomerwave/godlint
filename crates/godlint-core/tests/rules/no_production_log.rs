use godlint_core::rules::{Violation, no_production_log};

use super::support::rule_violations;

const ENABLED: &str = "version: 1\nrules:\n  logging/no-production-log:\n    severity: error\n";

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(no_production_log::evaluate, path, source, configuration)
}

#[test]
fn reports_a_logging_call_in_each_language() {
    assert_eq!(
        violations("src/example.js", "console.log(value);", ENABLED).len(),
        1
    );
    assert_eq!(
        violations("src/example.ts", "console.debug(value);", ENABLED).len(),
        1
    );
    assert_eq!(
        violations("src/example.py", "print(value)", ENABLED).len(),
        1
    );
    assert_eq!(
        violations("src/example.rs", "dbg!(value);", ENABLED).len(),
        1
    );
    assert_eq!(
        violations(
            "src/example.go",
            "package example\n\nimport \"log\"\n\nfunc f() { log.Println(\"value\") }",
            ENABLED
        )
        .len(),
        1
    );
}

#[test]
fn keeps_deliberate_user_facing_output() {
    assert!(violations("src/example.js", "console.error(value);", ENABLED).is_empty());
    assert!(violations("src/example.js", "console.warn(value);", ENABLED).is_empty());
    assert!(violations("src/example.rs", "println!(\"{value}\");", ENABLED).is_empty());
    assert!(violations("src/example.py", "logging.info(value)", ENABLED).is_empty());
}

#[test]
fn binds_a_logger_to_the_language_that_spells_it() {
    assert!(violations("src/example.ts", "print(value);", ENABLED).is_empty());
    assert!(violations("src/example.py", "console.log(value)", ENABLED).is_empty());
    assert!(violations("src/example.rs", "print(value);", ENABLED).is_empty());
}

#[test]
fn permits_logging_inside_an_approved_path() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  logging/no-production-log:\n",
        "    severity: error\n",
        "    allow-in:\n",
        "      - scripts/**\n"
    );

    assert!(violations("scripts/tool.js", "console.log(value);", configuration).is_empty());
    assert_eq!(
        violations("src/tool.js", "console.log(value);", configuration).len(),
        1
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration = "version: 1\nrules:\n  logging/no-production-log:\n    severity: off\n";

    assert!(violations("src/example.js", "console.log(value);", configuration).is_empty());
}

const ONLY_IN_SOURCE: &str = concat!(
    "version: 1\nrules:\n  logging/no-production-log:\n    severity: error\n",
    "    only-in:\n",
    "      - \"src/**\"\n",
);

const ONLY_IN_SOURCE_BUT_NOT_GENERATED: &str = concat!(
    "version: 1\nrules:\n  logging/no-production-log:\n    severity: error\n",
    "    only-in:\n",
    "      - \"src/**\"\n",
    "    allow-in:\n",
    "      - \"src/generated/**\"\n",
);

#[test]
fn only_in_reports_inside_the_paths_the_rule_is_about() {
    assert_eq!(
        violations("src/server.js", "console.log(value);", ONLY_IN_SOURCE).len(),
        1
    );
}

#[test]
fn only_in_says_nothing_outside_them() {
    assert!(
        violations("scripts/release.js", "console.log(value);", ONLY_IN_SOURCE).is_empty(),
        "a rule about production logging has nothing to say about a build script, and without \
         only-in the alternative is excluding that path from every rule at once"
    );
}

#[test]
fn allow_in_carves_an_exception_out_of_only_in() {
    assert_eq!(
        violations(
            "src/server.js",
            "console.log(value);",
            ONLY_IN_SOURCE_BUT_NOT_GENERATED
        )
        .len(),
        1
    );
    assert!(
        violations(
            "src/generated/client.js",
            "console.log(value);",
            ONLY_IN_SOURCE_BUT_NOT_GENERATED
        )
        .is_empty(),
        "the narrower of the two decides, so an exception inside only-in still silences the rule"
    );
}
