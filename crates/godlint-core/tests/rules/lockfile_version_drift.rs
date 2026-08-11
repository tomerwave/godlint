use godlint_core::rules::Violation;

#[test]
fn violation_message_identifies_manifest_and_lockfile_versions() {
    assert_eq!(
        Violation::LockfileVersionDrift {
            package: "demo".into(),
            declared: "0.2.0".into(),
            locked: "0.1.0".into(),
            lockfile: "Cargo.lock".into(),
        }
        .to_string(),
        "demo declares version 0.2.0 but Cargo.lock records 0.1.0; regenerate and commit the lockfile."
    );
}
