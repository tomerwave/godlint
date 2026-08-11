use godlint_core::rules::Violation;

#[test]
fn violation_message_identifies_manifest_and_lockfile_versions() {
    assert_eq!(
        Violation::DependencyPolicy { message: "demo declares version 0.2.0 but Cargo.lock records 0.1.0; regenerate and commit the lockfile.".into() }
        .to_string(),
        "demo declares version 0.2.0 but Cargo.lock records 0.1.0; regenerate and commit the lockfile."
    );
}
