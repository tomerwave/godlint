use godlint_core::rules::{Rule, no_committed_secret_file};

#[test]
fn rule_is_registered() {
    assert_eq!(
        no_committed_secret_file::NoCommittedSecretFile::ID,
        "repository/no-committed-secret-file"
    );
}
