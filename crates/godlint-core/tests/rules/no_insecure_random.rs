use godlint_core::rules::{Violation, no_insecure_random};

use super::support::rule_violations;

const ENABLED: &str = "version: 1\nrules:\n  security/no-insecure-random:\n    severity: error\n";

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(no_insecure_random::evaluate, path, source, configuration)
}

#[test]
fn reports_a_general_purpose_generator_in_each_language() {
    assert_eq!(
        violations("src/example.js", "const v = Math.random();", ENABLED).len(),
        1
    );
    assert_eq!(
        violations("src/example.ts", "const v = Math.random();", ENABLED).len(),
        1
    );
    assert_eq!(
        violations("src/example.py", "v = random.random()", ENABLED).len(),
        1
    );
    assert_eq!(
        violations("src/example.rs", "let v = rand::random();", ENABLED).len(),
        1
    );
}

#[test]
fn names_the_secure_generator_of_the_language_it_reports() {
    let cases = [
        (
            "src/example.js",
            "const v = Math.random();",
            "crypto.getRandomValues",
        ),
        ("src/example.py", "v = random.random()", "secrets"),
        (
            "src/example.rs",
            "let v = rand::random();",
            "rand::rngs::OsRng",
        ),
    ];

    for (path, source, secure) in cases {
        let reported = violations(path, source, ENABLED);
        let message = reported
            .first()
            .unwrap_or_else(|| panic!("reports {path}"))
            .to_string();

        assert!(
            message.contains(secure),
            "a named rule earns its keep by naming the fix: {message}"
        );
    }
}

#[test]
fn reports_the_pseudo_random_bytes_helper() {
    assert_eq!(
        violations(
            "src/a.js",
            "const b = crypto.pseudoRandomBytes(16);",
            ENABLED
        )
        .len(),
        1,
        "the name says pseudo; it is the one JavaScript case a callee match can decide"
    );
}

#[test]
fn keeps_a_secure_generator() {
    assert!(violations("src/a.py", "v = secrets.token_urlsafe(24)", ENABLED).is_empty());
    assert!(
        violations(
            "src/a.js",
            "const v = crypto.getRandomValues(buffer);",
            ENABLED
        )
        .is_empty()
    );
    assert!(violations("src/a.py", "random.seed(1)", ENABLED).is_empty());
}

#[test]
fn binds_a_generator_to_the_language_that_spells_it() {
    assert!(violations("src/a.py", "v = Math.random()", ENABLED).is_empty());
    assert!(violations("src/a.js", "random.random();", ENABLED).is_empty());
    assert!(violations("src/a.rs", "let v = Math.random();", ENABLED).is_empty());
}

#[test]
fn permits_a_generator_inside_an_approved_path() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  security/no-insecure-random:\n",
        "    severity: error\n",
        "    allow-in:\n",
        "      - scripts/**\n"
    );

    assert!(
        violations(
            "scripts/jitter.js",
            "const v = Math.random();",
            configuration
        )
        .is_empty()
    );
    assert_eq!(
        violations("src/token.js", "const v = Math.random();", configuration).len(),
        1
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration = "version: 1\nrules:\n  security/no-insecure-random:\n    severity: off\n";

    assert!(violations("src/a.js", "const v = Math.random();", configuration).is_empty());
}
