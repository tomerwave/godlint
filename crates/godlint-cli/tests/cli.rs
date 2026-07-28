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

#[test]
fn lists_every_suppression_for_audit() {
    let repository = Repository::new();

    repository.write(
        "godlint.yaml",
        "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n",
    );
    repository.write(
        "source.rs",
        "fn blank() {\n    // godlint-ignore-enclosing maintainability/empty-function \
         owner=tomer expires=2999-12-31 -- awaiting #482\n}\n",
    );

    let output = run(godlint()
        .arg("suppressions")
        .arg(".")
        .current_dir(repository.path()));

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "source.rs:2 godlint-ignore-enclosing maintainability/empty-function owner=tomer \
         expires=2999-12-31 -- awaiting #482\n\n1 suppressions.\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn reports_a_repository_with_no_suppressions() {
    let repository = Repository::new();

    repository.write("godlint.yaml", "version: 1\n");
    repository.write("source.rs", "fn active() {\n    work();\n}\n");

    let output = run(godlint()
        .arg("suppressions")
        .arg(".")
        .current_dir(repository.path()));

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "No suppressions.\n"
    );
}

#[test]
fn lists_a_suppression_that_cannot_account_for_itself() {
    let repository = Repository::new();

    repository.write("godlint.yaml", "version: 1\n");
    repository.write(
        "source.rs",
        "// godlint-ignore-next-line maintainability/empty-function\nfn blank() {}\n",
    );

    let output = run(godlint()
        .arg("suppressions")
        .arg(".")
        .current_dir(repository.path()));

    assert_eq!(output.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("-- (no justification)"),
        "the audit must show that a suppression has no reason, not omit the field"
    );
}

#[test]
fn does_not_apply_parent_policy_to_a_nested_repository() {
    let repository = Repository::new();

    repository.write(
        "godlint.yaml",
        "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n",
    );
    repository.write("outer.rs", "fn active() {\n    work();\n}\n");
    repository.write("nested/.git/HEAD", "ref: refs/heads/main\n");
    repository.write("nested/inner.rs", "fn ignored() {}\n");

    let output = run(godlint()
        .arg("check")
        .arg(".")
        .current_dir(repository.path()));

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "No findings.\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn checks_a_nested_repository_when_it_has_its_own_configuration() {
    let repository = Repository::new();

    repository.write(
        "godlint.yaml",
        "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n",
    );
    repository.write("nested/.git/HEAD", "ref: refs/heads/main\n");
    repository.write(
        "nested/godlint.yaml",
        "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n",
    );
    repository.write("nested/inner.rs", "fn reported() {}\n");

    let output = run(godlint()
        .arg("check")
        .arg(".")
        .current_dir(repository.path().join("nested")));

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("inner.rs:1:1"));
    assert!(output.stderr.is_empty());
}

#[test]
fn scans_a_nested_repository_when_the_parent_is_requested_first() {
    let repository = Repository::new();

    repository.write(
        "godlint.yaml",
        "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n",
    );
    repository.write("nested/.git", "gitdir: ../.git/modules/nested\n");
    repository.write("nested/inner.rs", "fn reported() {}\n");

    let output = run(godlint()
        .arg("check")
        .arg(".")
        .arg("nested")
        .current_dir(repository.path()));

    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("nested/inner.rs:1:1"),
        "naming the parent first is the documented way to scan a child: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn explains_that_a_nested_repository_needs_its_own_configuration() {
    let repository = Repository::new();

    repository.write(
        "godlint.yaml",
        "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n",
    );
    repository.write("nested/.git", "gitdir: ../.git/modules/nested\n");
    repository.write("nested/inner.rs", "fn reported() {}\n");

    let output = run(godlint()
        .arg("check")
        .arg(".")
        .current_dir(repository.path().join("nested")));

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("No godlint.yaml found"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn treats_a_gitdir_file_as_a_repository_boundary() {
    let repository = Repository::new();

    repository.write(
        "godlint.yaml",
        "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n",
    );
    repository.write("worktree/.git", "gitdir: /elsewhere/.git/worktrees/wt\n");
    repository.write("worktree/inner.rs", "fn ignored() {}\n");

    let output = run(godlint()
        .arg("check")
        .arg(".")
        .current_dir(repository.path()));

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "No findings.\n");
}

#[test]
fn skips_a_nested_repository_that_has_its_own_configuration() {
    let repository = Repository::new();

    repository.write(
        "godlint.yaml",
        "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n",
    );
    repository.write("nested/.git/HEAD", "ref: refs/heads/main\n");
    repository.write(
        "nested/godlint.yaml",
        "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n",
    );
    repository.write("nested/inner.rs", "fn ignored() {}\n");

    let output = run(godlint()
        .arg("check")
        .arg(".")
        .current_dir(repository.path()));

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "No findings.\n",
        "a child owning a configuration is still not the parent's to scan"
    );
}

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
