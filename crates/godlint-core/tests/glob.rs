#![allow(clippy::expect_used, clippy::unwrap_used)]

use godlint_core::glob::matches;

#[test]
fn matches_a_bare_name_against_any_segment() {
    assert!(matches("target", "target/debug/main.rs"));
    assert!(matches(".venv", "a/b/.venv/lib.py"));
    assert!(!matches("target", "src/targeting.rs"));
}

#[test]
fn matches_a_wildcard_within_a_segment() {
    assert!(matches("*.py", "src/example.py"));
    assert!(!matches("*.py", "src/example.rs"));
    assert!(matches("gen_*.rs", "gen_api.rs"));
}

#[test]
fn matches_a_single_character() {
    assert!(matches("a?c.rs", "abc.rs"));
    assert!(!matches("a?c.rs", "ac.rs"));
}

#[test]
fn matches_across_segments_with_a_double_star() {
    assert!(matches("src/**/generated.rs", "src/a/b/generated.rs"));
    assert!(matches("src/**/generated.rs", "src/generated.rs"));
    assert!(!matches("src/**/generated.rs", "lib/a/generated.rs"));
}

#[test]
fn anchors_a_rooted_pattern() {
    assert!(matches("src/example.rs", "src/example.rs"));
    assert!(!matches("src/example.rs", "crates/src/example.rs"));
}
