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
    match command.output() {
        Ok(output) => output,
        Err(error) => panic!("runs godlint: {error}"),
    }
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
fn lists_discovered_source_files() {
    let directory = std::env::temp_dir().join("godlint-cli-discovery-test");

    fs::create_dir_all(&directory).unwrap_or_else(|error| panic!("creates directory: {error}"));
    fs::write(directory.join("example.rs"), "fn main() {}")
        .unwrap_or_else(|error| panic!("writes source file: {error}"));
    fs::write(directory.join("README.md"), "ignored")
        .unwrap_or_else(|error| panic!("writes markdown file: {error}"));

    let output = run(godlint().args(["check", &directory.display().to_string()]));

    fs::remove_dir_all(&directory).unwrap_or_else(|error| panic!("removes directory: {error}"));

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "Discovered 1 supported source files:\n{}\n",
            directory.join("example.rs").display()
        )
    );
    assert!(output.stderr.is_empty());
}
