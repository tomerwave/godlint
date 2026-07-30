use godlint_core::rules::{Violation, no_weak_hash};

use godlint_core::config::Severity;

use super::support::{rule_findings, rule_violations};

const ENABLED: &str = "version: 1\nrules:\n  security/no-weak-hash:\n    severity: error\n";

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(no_weak_hash::evaluate, path, source, configuration)
}

#[test]
fn reports_a_broken_hash_where_the_algorithm_is_the_callee() {
    assert_eq!(
        violations("src/sign.py", "s = hashlib.md5(payload)", ENABLED).len(),
        1
    );
    assert_eq!(
        violations("src/sign.py", "s = hashlib.sha1(payload)", ENABLED).len(),
        1
    );
    assert_eq!(
        violations("src/sign.rs", "let s = md5::compute(payload);", ENABLED).len(),
        1
    );
    assert_eq!(
        violations("src/sign.rs", "let mut h = Sha1::new();", ENABLED).len(),
        1
    );
}

#[test]
fn names_a_replacement_the_language_can_use() {
    let python = violations("src/a.py", "s = hashlib.md5(p)", ENABLED);
    let rust = violations("src/a.rs", "let s = md5::compute(p);", ENABLED);

    assert!(
        python
            .first()
            .expect("reports python")
            .to_string()
            .contains("hashlib.sha256")
    );
    assert!(
        rust.first()
            .expect("reports rust")
            .to_string()
            .contains("sha2::Sha256")
    );
}

#[test]
fn keeps_a_collision_resistant_hash() {
    assert!(violations("src/a.py", "s = hashlib.sha256(p)", ENABLED).is_empty());
    assert!(violations("src/a.rs", "let mut h = Sha256::new();", ENABLED).is_empty());
    assert!(violations("src/a.py", "s = hmac.new(key, p, hashlib.sha256)", ENABLED).is_empty());
}

#[test]
fn reads_a_weak_algorithm_named_by_a_literal_argument() {
    assert_eq!(
        violations("src/a.js", "const h = crypto.createHash(\"md5\");", ENABLED).len(),
        1
    );
    assert_eq!(
        violations(
            "src/a.ts",
            "const h = crypto.createHmac(\"sha1\", key);",
            ENABLED
        )
        .len(),
        1
    );
    assert_eq!(
        violations("src/a.py", "h = hashlib.new(\"md5\")", ENABLED).len(),
        1
    );
}

#[test]
fn reads_the_algorithm_however_it_is_spelled() {
    for spelling in ["\"MD5\"", "\"Md5\"", "\"md-5\"", "\"md_5\""] {
        assert_eq!(
            violations(
                "src/a.js",
                &format!("const h = crypto.createHash({spelling});"),
                ENABLED
            )
            .len(),
            1,
            "{spelling} names the same algorithm"
        );
    }
}

#[test]
fn keeps_a_strong_algorithm_named_by_a_literal_argument() {
    assert!(violations("src/a.js", "crypto.createHash(\"sha256\");", ENABLED).is_empty());
    assert!(violations("src/a.py", "hashlib.new(\"sha512\")", ENABLED).is_empty());
}

fn severities(path: &str, source: &str, configuration: &str) -> Vec<Severity> {
    rule_findings(no_weak_hash::evaluate, path, source, configuration)
        .into_iter()
        .map(|finding| finding.severity)
        .collect()
}

#[test]
fn reports_an_algorithm_it_cannot_read_as_a_warning_rather_than_an_error() {
    assert_eq!(
        severities("src/a.js", "crypto.createHash(algo);", ENABLED),
        vec![Severity::Warning],
        "the rule is configured at error, and this finding is a question rather than an answer"
    );
    assert_eq!(
        severities("src/a.py", "hashlib.new(algo)", ENABLED),
        vec![Severity::Warning]
    );
    assert_eq!(
        severities("src/a.js", "crypto.createHash(pick() + s);", ENABLED),
        vec![Severity::Warning]
    );
}

#[test]
fn keeps_a_readable_algorithm_at_the_configured_severity() {
    assert_eq!(
        severities("src/a.js", "crypto.createHash(\"md5\");", ENABLED),
        vec![Severity::Error]
    );
}

#[test]
fn a_cap_lowers_a_severity_and_never_raises_one() {
    let configuration = "version: 1\nrules:\n  security/no-weak-hash:\n    severity: info\n";

    assert_eq!(
        severities("src/a.js", "crypto.createHash(algo);", configuration),
        vec![Severity::Info],
        "a repository that asked for info does not get a warning back"
    );
}

#[test]
fn stays_silent_when_there_is_no_algorithm_to_read() {
    assert!(
        violations("src/a.js", "crypto.createHash();", ENABLED).is_empty(),
        "no argument at all is not a weak hash"
    );
}

#[test]
fn names_the_replacement_in_the_language_it_reports() {
    let js = violations("src/a.js", "crypto.createHash(\"md5\");", ENABLED);

    assert!(
        js.first()
            .expect("reports javascript")
            .to_string()
            .contains("use sha256"),
        "the Node spelling, not the Rust crate"
    );
}

#[test]
fn binds_a_hash_to_the_language_that_spells_it() {
    assert!(violations("src/a.js", "hashlib.md5(p);", ENABLED).is_empty());
    assert!(
        violations("src/a.py", "h = Md5.new()", ENABLED).is_empty(),
        "the catalogue spells this Md5::new, and the spelling is the match"
    );
}

#[test]
fn permits_a_weak_hash_inside_an_approved_path() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  security/no-weak-hash:\n",
        "    severity: error\n",
        "    allow-in:\n",
        "      - src/cache/**\n"
    );

    assert!(violations("src/cache/key.py", "k = hashlib.md5(p)", configuration).is_empty());
    assert_eq!(
        violations("src/sign.py", "s = hashlib.md5(p)", configuration).len(),
        1
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration = "version: 1\nrules:\n  security/no-weak-hash:\n    severity: off\n";

    assert!(violations("src/a.py", "s = hashlib.md5(p)", configuration).is_empty());
}
