use godlint_core::rules::Violation;

#[test]
fn violation_message_names_the_remedy() {
    assert_eq!(
        Violation::DependencyPolicy { message: "This step uses npm install without pinning to the committed lockfile; use npm ci instead.".to_owned() }
        .to_string(),
        "This step uses npm install without pinning to the committed lockfile; use npm ci instead."
    );
}
