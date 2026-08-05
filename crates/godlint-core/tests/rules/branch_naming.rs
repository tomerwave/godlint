use std::path::PathBuf;

use godlint_core::{
    repository::RepositoryFacts,
    rules::{Violation, branch_naming},
    source::TextFile,
};

fn findings(branch: &str, configuration: &str) -> Vec<Violation> {
    let branch = TextFile::new(PathBuf::from("<branch>"), branch.into())
        .unwrap_or_else(|error| panic!("reads branch: {error}"));
    let repository = RepositoryFacts::new(Some(branch));
    let config = yaml_serde::from_str(configuration)
        .unwrap_or_else(|error| panic!("reads configuration: {error}"));

    branch_naming::evaluate(&repository, &config)
        .into_iter()
        .map(|finding| finding.violation)
        .collect()
}

const ENABLED: &str = "version: 1\nrules:\n  git/branch-naming:\n    severity: error\n";

#[test]
fn a_conventional_type_and_lowercase_slug_pass() {
    for branch in [
        "feat/import-fact",
        "fix/v1.2/repair_thing",
        "release/0.7.0",
        "feat/a.b_c/d-e",
    ] {
        assert!(
            findings(branch, ENABLED).is_empty(),
            "{branch} is conventional"
        );
    }
}

#[test]
fn an_unknown_type_or_non_lowercase_slug_fails() {
    for branch in ["codex/anything", "feat/Uppercase", "feat/"] {
        assert!(
            matches!(
                findings(branch, ENABLED).as_slice(),
                [Violation::InvalidBranchName { .. }]
            ),
            "{branch} must be rejected"
        );
    }
}

#[test]
fn configured_automation_patterns_are_allowed() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  git/branch-naming:\n",
        "    severity: error\n",
        "    allow:\n",
        "      - dependabot/**\n"
    );

    assert!(findings("dependabot/cargo/serde-1.0.0", configuration).is_empty());
}
