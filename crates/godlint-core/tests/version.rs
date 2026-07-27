#[test]
fn exposes_a_non_empty_version() {
    assert!(!godlint_core::VERSION.is_empty());
}
