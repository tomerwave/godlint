use godlint_core::rules::{Violation, no_weak_hash};

use super::support::rule_violations;

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
fn does_not_read_an_algorithm_passed_as_an_argument() {
    assert!(
        violations("src/a.js", "const h = crypto.createHash(\"md5\");", ENABLED).is_empty(),
        "the algorithm is an argument here, and reporting the callee would also report sha256"
    );
    assert!(violations("src/a.py", "h = hashlib.new(\"md5\")", ENABLED).is_empty());
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
