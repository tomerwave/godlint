use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{ExpectedFinding, assert_check, run};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

const EXPECTED_FINDINGS: [ExpectedFinding; 7] = [
    expected_finding("example.js"),
    expected_finding("example.jsx"),
    expected_finding("example.py"),
    expected_finding("example.pyi"),
    expected_finding("example.rs"),
    expected_finding("example.ts"),
    expected_finding("example.tsx"),
];

struct TemporaryFixture {
    path: PathBuf,
}

impl TemporaryFixture {
    fn parse_issue() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("godlint-parse-issue-{timestamp}-{id}"));

        fs::create_dir(&path).unwrap_or_else(|error| panic!("creates fixture: {error}"));
        fs::write(
            path.join("godlint.yaml"),
            "version: 1\nrules:\n  maintainability/function-size:\n    severity: error\n    max-lines: 3\n    skip-blank-lines: true\n    skip-comments: true\n",
        )
        .unwrap_or_else(|error| panic!("writes config: {error}"));
        fs::write(path.join("broken.js"), "function broken( {")
            .unwrap_or_else(|error| panic!("writes invalid source: {error}"));
        fs::write(
            path.join("valid.rs"),
            "fn valid() {\n    one();\n    two();\n}\n",
        )
        .unwrap_or_else(|error| panic!("writes valid source: {error}"));

        Self { path }
    }
}

impl Drop for TemporaryFixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path).unwrap_or_else(|error| panic!("removes fixture: {error}"));
    }
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/function-size")
}

const fn expected_finding(path: &'static str) -> ExpectedFinding {
    ExpectedFinding {
        path,
        line: 1,
        column: 1,
        severity: "error",
        rule_id: "maintainability/function-size",
        message: "Function has 4 effective lines (max 3).",
    }
}

#[test]
fn reports_expected_findings() {
    assert_check(&fixture_directory(), &EXPECTED_FINDINGS);
}

#[test]
fn checks_an_absolute_repository_path() {
    let fixture = fixture_directory();
    let output = run(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).as_path(),
        &["check", &fixture.display().to_string()],
    );

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains(EXPECTED_FINDINGS[4].message));
    assert!(output.stderr.is_empty());
}

#[test]
fn checks_a_parent_directory_within_the_repository() {
    let fixture = fixture_directory();
    let output = run(&fixture.join("nested"), &["check", ".."]);

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains(EXPECTED_FINDINGS[4].message));
    assert!(output.stderr.is_empty());
}

#[test]
fn continues_after_a_source_parse_error() {
    let fixture = TemporaryFixture::parse_issue();
    let output = run(&fixture.path, &["check", "."]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "valid.rs:1:1: error[maintainability/function-size] Function has 4 effective lines (max 3).\n"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "broken.js: invalid syntax in broken.js\n"
    );
}
