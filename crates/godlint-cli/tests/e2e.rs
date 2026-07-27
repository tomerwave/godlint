use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

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

#[test]
fn reports_the_same_function_size_result_for_every_supported_language() {
    let fixture = fixture_directory();
    let output = Command::new(env!("CARGO_BIN_EXE_godlint"))
        .current_dir(fixture)
        .args(["check", "."])
        .output()
        .unwrap_or_else(|error| panic!("runs godlint: {error}"));

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        concat!(
            "example.js:1:1: error[maintainability/function-size] Function has 4 effective lines (max 3).\n",
            "example.jsx:1:1: error[maintainability/function-size] Function has 4 effective lines (max 3).\n",
            "example.py:1:1: error[maintainability/function-size] Function has 4 effective lines (max 3).\n",
            "example.pyi:1:1: error[maintainability/function-size] Function has 4 effective lines (max 3).\n",
            "example.rs:1:1: error[maintainability/function-size] Function has 4 effective lines (max 3).\n",
            "example.ts:1:1: error[maintainability/function-size] Function has 4 effective lines (max 3).\n",
            "example.tsx:1:1: error[maintainability/function-size] Function has 4 effective lines (max 3).\n",
        )
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn checks_an_absolute_repository_path() {
    let fixture = fixture_directory();
    let output = Command::new(env!("CARGO_BIN_EXE_godlint"))
        .args(["check", &fixture.display().to_string()])
        .output()
        .unwrap_or_else(|error| panic!("runs godlint: {error}"));

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "example.rs:1:1: error[maintainability/function-size] Function has 4 effective lines (max 3)."
    ));
    assert!(output.stderr.is_empty());
}

#[test]
fn checks_a_parent_directory_within_the_repository() {
    let fixture = fixture_directory();
    let output = Command::new(env!("CARGO_BIN_EXE_godlint"))
        .current_dir(fixture.join("nested"))
        .args(["check", ".."])
        .output()
        .unwrap_or_else(|error| panic!("runs godlint: {error}"));

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "example.rs:1:1: error[maintainability/function-size] Function has 4 effective lines (max 3)."
    ));
    assert!(output.stderr.is_empty());
}

#[test]
fn continues_after_a_source_parse_error() {
    let fixture = TemporaryFixture::parse_issue();
    let output = Command::new(env!("CARGO_BIN_EXE_godlint"))
        .current_dir(&fixture.path)
        .args(["check", "."])
        .output()
        .unwrap_or_else(|error| panic!("runs godlint: {error}"));

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
