#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_CONFIG_ID: AtomicU64 = AtomicU64::new(0);

fn godlint() -> Command {
    Command::new(env!("CARGO_BIN_EXE_godlint"))
}

fn run(command: &mut Command) -> std::process::Output {
    command
        .output()
        .unwrap_or_else(|error| panic!("runs godlint: {error}"))
}

fn config_file(contents: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let id = NEXT_CONFIG_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("godlint-cli-{timestamp}-{id}.yaml"));

    fs::write(&path, contents).unwrap_or_else(|error| panic!("writes config: {error}"));

    path
}

#[test]
fn prints_its_version() {
    let output = run(godlint().arg("--version"));

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "godlint 0.1.0\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn rejects_unknown_arguments() {
    let output = run(godlint().arg("unknown"));

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Unknown command or arguments: unknown")
    );
}

#[test]
fn validates_a_function_size_configuration() {
    let path = config_file(
        "version: 1\nrules:\n  maintainability/function-size:\n    severity: error\n    max-lines: 30\n    skip-blank-lines: true\n    skip-comments: true\n",
    );
    let output = run(godlint().args([
        "config",
        "validate",
        "--config",
        &path.display().to_string(),
    ]));

    fs::remove_file(path).unwrap_or_else(|error| panic!("removes config: {error}"));

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Configuration is valid:"));
    assert!(output.stderr.is_empty());
}

#[test]
fn reports_an_invalid_configuration() {
    let path = config_file("version: 2\n");
    let output = run(godlint().args([
        "config",
        "validate",
        "--config",
        &path.display().to_string(),
    ]));

    fs::remove_file(path).unwrap_or_else(|error| panic!("removes config: {error}"));

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unsupported configuration version: 2")
    );
}

/// A repository with nothing to report must succeed, which no fixture asserted before.
#[test]
fn reports_a_clean_repository() {
    let repository = Repository::new();

    repository.write(
        "godlint.yaml",
        "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n",
    );
    repository.write("source.rs", "fn active() {\n    work();\n}\n");

    let output = run(godlint()
        .arg("check")
        .arg(".")
        .current_dir(repository.path()));

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "No findings.\n");
    assert!(output.stderr.is_empty());
}

/// Without a configuration the operator needs to be told that, not shown a read error.
#[test]
fn explains_a_missing_configuration() {
    let repository = Repository::new();

    repository.write(".git/HEAD", "ref: refs/heads/main\n");
    repository.write("source.rs", "fn active() {\n    work();\n}\n");

    let output = run(godlint()
        .arg("check")
        .arg(".")
        .current_dir(repository.path()));

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("No godlint.yaml found"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Configuration discovery must stop at the repository boundary, so a stray file in a
/// parent directory cannot silently govern an unrelated repository.
#[test]
fn does_not_adopt_a_configuration_from_outside_the_repository() {
    let outer = Repository::new();

    outer.write(
        "godlint.yaml",
        "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n",
    );
    outer.write("inner/.git/HEAD", "ref: refs/heads/main\n");
    outer.write("inner/source.rs", "fn reported() {}\n");

    let output = run(godlint()
        .arg("check")
        .arg(".")
        .current_dir(outer.path().join("inner")));

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("No godlint.yaml found"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// A temporary directory that cleans itself up, so a failing assertion cannot leak it.
struct Repository {
    path: PathBuf,
}

impl Repository {
    fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let id = NEXT_CONFIG_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("godlint-repo-{timestamp}-{id}"));

        fs::create_dir_all(&path).unwrap_or_else(|error| panic!("creates repository: {error}"));

        Self { path }
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.path.join(relative);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|error| panic!("creates parent: {error}"));
        }

        fs::write(path, contents).unwrap_or_else(|error| panic!("writes {relative}: {error}"));
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
