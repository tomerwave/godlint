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
