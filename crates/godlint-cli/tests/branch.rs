#![cfg(unix)]

use std::{fs, path::Path, process::Command};

#[path = "support/temporary.rs"]
mod temporary;

use temporary::TemporaryDirectory;

fn repository() -> TemporaryDirectory {
    TemporaryDirectory::new("branch")
}

fn godlint() -> Command {
    Command::new(env!("CARGO_BIN_EXE_godlint"))
}

fn path_with(directory: &Path) -> std::ffi::OsString {
    let mut parts = vec![directory.to_path_buf()];

    if let Some(existing) = std::env::var_os("PATH") {
        parts.extend(std::env::split_paths(&existing));
    }

    std::env::join_paths(parts).unwrap_or_else(|error| panic!("joins PATH: {error}"))
}

fn executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .unwrap_or_else(|error| panic!("reads {}: {error}", path.display()))
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .unwrap_or_else(|error| panic!("chmod {}: {error}", path.display()));
}

fn git(directory: &TemporaryDirectory, body: &str) -> std::path::PathBuf {
    let path = directory.write("bin/git", body);
    fs::create_dir_all(directory.path().join("bin"))
        .unwrap_or_else(|error| panic!("creates bin: {error}"));
    executable(&path);
    directory.path().join("bin")
}

#[test]
fn falls_back_to_git_when_the_pull_request_branch_is_missing() {
    let directory = repository();
    directory.write("godlint.yaml", "version: 1\nsuites: [recommended@1]\n");
    let log = directory.path().join("git.log");
    let bin = git(
        &directory,
        &format!(
            "#!/usr/bin/env bash\nprintf '%s\\n' \"$*\" >> '{}'\necho feat/from-git\n",
            log.display()
        ),
    );

    let output = godlint()
        .args(["check", "."])
        .current_dir(directory.path())
        .env_remove("GITHUB_HEAD_REF")
        .env("PATH", path_with(&bin))
        .output()
        .unwrap_or_else(|error| panic!("runs: {error}"));

    assert_eq!(output.status.code(), Some(0));
    let invoked = fs::read_to_string(log).unwrap_or_else(|error| panic!("reads log: {error}"));

    assert!(invoked.contains("branch --show-current"));
}

#[test]
fn prefers_the_pull_request_branch_to_git() {
    let directory = repository();
    directory.write("godlint.yaml", "version: 1\nsuites: [recommended@1]\n");
    let bin = git(&directory, "#!/usr/bin/env bash\nexit 1\n");

    let output = godlint()
        .args(["check", "."])
        .current_dir(directory.path())
        .env("GITHUB_HEAD_REF", "feat/from-pr")
        .env("PATH", path_with(&bin))
        .output()
        .unwrap_or_else(|error| panic!("runs: {error}"));

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn skips_the_repository_default_branch() {
    let directory = repository();
    directory.write("godlint.yaml", "version: 1\nsuites: [recommended@1]\n");
    let bin = git(
        &directory,
        "#!/usr/bin/env bash\nif [[ \"$*\" == *symbolic-ref* ]]; then echo origin/trunk; else echo trunk; fi\n",
    );

    let output = godlint()
        .args(["check", "."])
        .current_dir(directory.path())
        .env_remove("GITHUB_HEAD_REF")
        .env("PATH", path_with(&bin))
        .output()
        .unwrap_or_else(|error| panic!("runs: {error}"));

    assert_eq!(output.status.code(), Some(0));
}

const BRANCH_NAMING: &str = "version: 1\nrules:\n  git/branch-naming:\n    severity: error\n";

fn check_on_branch_without_origin_head(
    branch: &str,
    configuration: &str,
    pull_request_branch: Option<&str>,
) -> Option<i32> {
    let directory = repository();
    directory.write("godlint.yaml", configuration);
    let initialised = Command::new("git")
        .args(["init", "--quiet", "--initial-branch", branch])
        .current_dir(directory.path())
        .status()
        .unwrap_or_else(|error| panic!("runs git init: {error}"));
    assert!(initialised.success());

    let mut command = godlint();
    command.args(["check", "."]).current_dir(directory.path());
    match pull_request_branch {
        Some(name) => command.env("GITHUB_HEAD_REF", name),
        None => command.env_remove("GITHUB_HEAD_REF"),
    };

    command
        .output()
        .unwrap_or_else(|error| panic!("runs: {error}"))
        .status
        .code()
}

#[test]
fn accepts_conventional_default_branch_names_when_origin_head_is_missing() {
    for branch in ["main", "master"] {
        assert_eq!(
            check_on_branch_without_origin_head(branch, BRANCH_NAMING, None),
            Some(0),
            "{branch} must be accepted on a push checkout"
        );
    }
}

#[test]
fn accepts_typed_branches_when_origin_head_is_missing() {
    assert_eq!(
        check_on_branch_without_origin_head("feat/x", BRANCH_NAMING, None),
        Some(0)
    );
}

#[test]
fn rejects_untyped_branches_when_origin_head_is_missing() {
    for branch in ["scratch", "mainline", "main/x"] {
        assert_eq!(
            check_on_branch_without_origin_head(branch, BRANCH_NAMING, None),
            Some(1),
            "{branch} must be rejected"
        );
    }
}

#[test]
fn an_allow_list_accepts_an_untyped_branch_when_origin_head_is_missing() {
    let configuration = format!("{BRANCH_NAMING}    allow: [scratch]\n");

    assert_eq!(
        check_on_branch_without_origin_head("scratch", &configuration, None),
        Some(0)
    );
}

#[test]
fn a_pull_request_from_a_default_branch_name_is_still_rejected() {
    assert_eq!(
        check_on_branch_without_origin_head("feat/x", BRANCH_NAMING, Some("main")),
        Some(1)
    );
}
