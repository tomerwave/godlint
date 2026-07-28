use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedResult {
    #[serde(rename = "exit-code")]
    exit_code: i32,
    #[serde(default)]
    findings: Vec<ExpectedFinding>,
    #[serde(default)]
    stdout: Option<String>,
    #[serde(default)]
    stderr: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedFinding {
    path: String,
    line: usize,
    column: usize,
    severity: String,
    #[serde(rename = "rule-id")]
    rule_id: String,
    message: String,
}

#[test]
fn checks_every_rule_fixture() {
    for fixture in rule_fixtures() {
        assert_fixture(&fixture);
    }
}

fn rule_fixtures() -> Vec<PathBuf> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rules");
    let entries =
        fs::read_dir(&directory).unwrap_or_else(|error| panic!("reads fixtures: {error}"));
    let mut fixtures = entries
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("reads fixture entry: {error}"))
                .path()
        })
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();

    fixtures.sort();

    fixtures
}

fn assert_fixture(fixture: &Path) {
    let expected = expected_result(fixture);
    let output = run(fixture);

    assert_eq!(
        output.status.code(),
        Some(expected.exit_code),
        "{}",
        fixture.display()
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected_stdout(&expected),
        "{}",
        fixture.display()
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        expected_stderr(&expected),
        "{}",
        fixture.display()
    );
}

fn expected_result(fixture: &Path) -> ExpectedResult {
    let path = fixture.join("expected.yaml");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("reads {}: {error}", path.display()));

    yaml_serde::from_str(&source)
        .unwrap_or_else(|error| panic!("parses {}: {error}", path.display()))
}

fn run(fixture: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_godlint"))
        .current_dir(fixture)
        .args(["check", "."])
        .output()
        .unwrap_or_else(|error| panic!("runs godlint: {error}"))
}

fn expected_stdout(expected: &ExpectedResult) -> String {
    if let Some(stdout) = &expected.stdout {
        return stdout.clone();
    }

    expected
        .findings
        .iter()
        .map(|finding| {
            format!(
                "{}:{}:{}: {}[{}] {}\n",
                finding.path,
                finding.line,
                finding.column,
                finding.severity,
                finding.rule_id,
                finding.message
            )
        })
        .collect()
}

fn expected_stderr(expected: &ExpectedResult) -> String {
    if expected.stderr.is_empty() {
        return String::new();
    }

    format!("{}\n", expected.stderr.join("\n"))
}
