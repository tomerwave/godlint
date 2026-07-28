use std::{
    path::Path,
    process::{Command, Output},
};

mod function_size;

pub(super) struct ExpectedFinding {
    pub(super) path: &'static str,
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) severity: &'static str,
    pub(super) rule_id: &'static str,
    pub(super) message: &'static str,
}

pub(super) fn run(current_directory: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_godlint"))
        .current_dir(current_directory)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("runs godlint: {error}"))
}

pub(super) fn assert_check(directory: &Path, expected: &[ExpectedFinding]) {
    let output = run(directory, &["check", "."]);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected_output(expected)
    );
    assert!(output.stderr.is_empty());
}

fn expected_output(expected: &[ExpectedFinding]) -> String {
    expected
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
