use std::path::PathBuf;

use godlint_core::{
    analyzers::manifest::dependency_specs,
    repository::RepositoryFacts,
    rules::{Rule, Violation, exact_versions},
    source::TextFile,
};

use super::support::config;

const CONFIGURATION: &str =
    "version: 1\nrules:\n  dependencies/exact-versions:\n    severity: error\n";

fn findings_in(path: &str, manifest: &str, configuration: &str) -> Vec<(usize, usize, String)> {
    let file = TextFile::new(PathBuf::from(path), manifest.to_owned())
        .unwrap_or_else(|error| panic!("creates manifest: {error}"));
    let repository = RepositoryFacts::new(None).with_dependency_specs(dependency_specs(&file));

    exact_versions::evaluate(&repository, &config(configuration))
        .into_iter()
        .map(|finding| (finding.line, finding.column, finding.violation.to_string()))
        .collect()
}

fn reported(spec: &str) -> bool {
    let manifest = format!("{{\"dependencies\":{{\"a\":\"{spec}\"}}}}");

    !findings_in("package.json", &manifest, CONFIGURATION).is_empty()
}

#[test]
fn rule_is_registered() {
    assert_eq!(
        exact_versions::ExactVersions::ID,
        "dependencies/exact-versions"
    );
}

#[test]
fn reports_every_range_form() {
    for spec in [
        "^1.2.3",
        "~1.2.3",
        ">1.2.3",
        ">=1.2.3",
        "<2.0.0",
        "<=2.0.0",
        ">= 1.2.3",
        ">=1.0.0 <2.0.0",
        "1.2.3 - 2.0.0",
        "*",
        "x",
        "X",
        "1.x",
        "1.2.X",
        "1.*",
        "1",
        "1.2",
        "1.0.0 || 2.0.0",
        "1.0.0 ||",
        "||",
        "latest",
        " latest ",
        "",
        "   ",
        "^1.2.3-beta.1",
        "npm:other@^1.2.3",
        "npm:@scope/other@~2",
        "npm:other",
        "npm:@scope/other",
        "npm:",
        " npm:other@1",
    ] {
        assert!(reported(spec), "{spec:?} is a range");
    }
}

#[test]
fn accepts_every_exact_or_unversioned_form() {
    for spec in [
        "1.2.3",
        " 1.2.3 ",
        "v1.2.3",
        "=1.2.3",
        "=v1.2.3",
        "1.2.3-beta.1",
        "1.2.3+build.5",
        "10.20.30",
        "npm:other@1.2.3",
        "npm:@scope/other@1.2.3",
        "workspace:*",
        "workspace:^1.2.3",
        "workspace:",
        "file:../shared",
        "link:../shared",
        "github:owner/repo#0123456789abcdef0123456789abcdef01234567",
        "git+https://example.com/owner/repo.git#0123456789abcdef",
        "git+ssh://git@github.com:owner/repo.git#abcdef1",
        "git://github.com/owner/repo.git#ABCDEF0",
        "gitlab:owner/repo#0123456",
        "owner/repo#0123456789abcdef0123456789abcdef01234567",
        "https://example.com/package.tgz",
        "../shared",
        "./shared",
        "/opt/shared",
        "~/shared",
        "vendor/package.tgz",
        "vendor/package.tar",
        "vendor/package.tar.gz",
        "owner/repo/extra",
        "-owner/repo",
        "owner/.repo",
        "own+er/repo",
        "@scope/package",
        "next",
        "beta",
        "1.2.3.4",
        "~>1.2",
        "1.2.a",
        "1..2",
    ] {
        assert!(!reported(spec), "{spec:?} is not reported");
    }
}

#[test]
fn reports_a_git_reference_that_is_not_a_commit() {
    for spec in [
        "github:owner/repo",
        "github:owner/repo#main",
        "github:owner/repo#semver:^1.2.0",
        "gitlab:owner/repo#v1.2.3",
        "bitbucket:owner/repo",
        "gist:abc123",
        "git+https://github.com/owner/repo.git",
        "git+ssh://git@github.com:owner/repo.git#release",
        "git://github.com/owner/repo.git#012345",
        "git@github.com:owner/repo.git",
        "owner/repo",
        "owner/repo#main",
        "Owner_1/repo.js-2",
        "owner/repo#0123456789abcdef0123456789abcdef012345678",
        "owner/repo#0123456789abcdeg",
    ] {
        assert_eq!(
            findings_in(
                "package.json",
                &format!("{{\"dependencies\":{{\"a\":\"{spec}\"}}}}"),
                CONFIGURATION
            )
            .into_iter()
            .map(|(_, _, message)| message)
            .collect::<Vec<_>>(),
            vec![format!(
                "a in dependencies is \"{spec}\", a git reference that can move; pin it to a commit hash so every install gets the same code."
            )],
            "{spec:?} can move"
        );
    }
}

#[test]
fn reports_the_package_section_and_spec_at_the_quoted_value() {
    assert_eq!(
        findings_in(
            "web/package.json",
            "{\n  \"devDependencies\": {\n    \"typescript\": \"^5.4.0\"\n  }\n}\n",
            CONFIGURATION
        ),
        vec![(
            3,
            19,
            "typescript in devDependencies is \"^5.4.0\", which can resolve to more than one release; declare the exact version so every install gets the same one.".to_owned()
        )]
    );
}

#[test]
fn stays_silent_where_allowed_and_when_unconfigured() {
    let manifest = "{\"dependencies\":{\"a\":\"^1\"}}";
    let allowed = format!("{CONFIGURATION}    allow-in:\n      - tools/**\n");

    assert!(findings_in("tools/package.json", manifest, &allowed).is_empty());
    assert_eq!(findings_in("web/package.json", manifest, &allowed).len(), 1);
    assert!(findings_in("package.json", manifest, "version: 1\n").is_empty());
}

#[test]
fn reports_as_a_dependency_policy_violation() {
    let file = TextFile::new(
        PathBuf::from("package.json"),
        "{\"optionalDependencies\":{\"fsevents\":\"*\"}}".to_owned(),
    )
    .unwrap_or_else(|error| panic!("creates manifest: {error}"));
    let repository = RepositoryFacts::new(None).with_dependency_specs(dependency_specs(&file));
    let findings = exact_versions::evaluate(&repository, &config(CONFIGURATION));

    assert!(matches!(
        findings.as_slice(),
        [finding] if matches!(finding.violation, Violation::DependencyPolicy { .. })
            && finding.rule_id == "dependencies/exact-versions"
    ));
}
