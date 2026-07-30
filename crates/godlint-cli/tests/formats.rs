#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{path::Path, process::Command};

#[path = "support/temporary.rs"]
mod temporary;

use temporary::TemporaryDirectory;

fn scratch(files: &[(&str, &str)]) -> TemporaryDirectory {
    let directory = TemporaryDirectory::new("format");

    for (name, contents) in files {
        directory.write(name, contents);
    }

    directory
}

fn checked(directory: &Path, format: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_godlint"))
        .arg("check")
        .args(format)
        .current_dir(directory)
        .output()
        .unwrap_or_else(|error| panic!("runs godlint: {error}"));

    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn restricted_call() -> TemporaryDirectory {
    scratch(&[
        (
            "godlint.yaml",
            "version: 1\nrules:\n  architecture/restricted-call:\n    severity: error\n",
        ),
        ("main.rs", "fn main() {\n    std::process::exit(1);\n}\n"),
    ])
}

#[test]
fn an_annotation_keeps_a_colon_in_the_message_and_escapes_one_in_a_property() {
    let printed = checked(restricted_call().path(), &["--format", "github"]);

    assert!(
        printed.contains("::std::process::exit is restricted"),
        "a colon in the message must survive, or the message reads wrong: {printed}"
    );
    assert!(
        printed.contains("title=architecture/restricted-call::"),
        "the rule identifier is a property and has no colon to escape: {printed}"
    );
}

#[test]
fn an_annotation_names_the_file_the_line_and_the_column() {
    let printed = checked(restricted_call().path(), &["--format", "github"]);

    assert!(
        printed.starts_with("::error file=main.rs,line=2,col=5,"),
        "{printed}"
    );
}

#[test]
fn a_warning_and_an_error_take_different_annotation_levels() {
    let directory = scratch(&[
        (
            "godlint.yaml",
            concat!(
                "version: 1\n",
                "rules:\n",
                "  architecture/restricted-call:\n",
                "    severity: warning\n"
            ),
        ),
        ("main.rs", "fn main() {\n    std::process::exit(1);\n}\n"),
    ]);

    assert!(
        checked(directory.path(), &["--format", "github"]).starts_with("::warning "),
        "a warning is not an error annotation"
    );
}

#[test]
fn json_is_parseable_and_names_every_field() {
    let printed = checked(restricted_call().path(), &["--format=json"]);

    for fragment in [
        "\"version\":",
        "\"findings\":[",
        "\"path\":\"main.rs\"",
        "\"line\":2",
        "\"column\":5",
        "\"severity\":\"error\"",
        "\"rule\":\"architecture/restricted-call\"",
        "\"issues\":[",
    ] {
        assert!(
            printed.contains(fragment),
            "{fragment} missing from {printed}"
        );
    }
}

#[test]
fn json_escapes_a_quote_and_a_backslash_in_a_message() {
    let directory = scratch(&[
        (
            "godlint.yaml",
            concat!(
                "version: 1\n",
                "rules:\n",
                "  architecture/restricted-import:\n",
                "    severity: error\n",
                "    modules:\n",
                "      - name: \"a\\\\b\"\n"
            ),
        ),
        ("main.rs", "use a\\b::thing;\n"),
    ]);
    let printed = checked(directory.path(), &["--format", "json"]);

    assert!(
        !printed.contains("\\\"") || printed.contains("\\\\"),
        "a backslash in a message must be escaped, not passed through: {printed}"
    );
}

#[test]
fn sarif_carries_the_schema_the_level_and_one_rule_per_identifier() {
    let printed = checked(restricted_call().path(), &["--format", "sarif"]);

    for fragment in [
        "\"version\":\"2.1.0\"",
        "\"$schema\":",
        "\"name\":\"godlint\"",
        "\"ruleId\":\"architecture/restricted-call\"",
        "\"level\":\"error\"",
        "\"startLine\":2",
        "\"startColumn\":5",
    ] {
        assert!(
            printed.contains(fragment),
            "{fragment} missing from {printed}"
        );
    }
}

#[test]
fn a_machine_readable_format_emits_a_document_when_there_is_nothing_to_report() {
    let directory = scratch(&[("godlint.yaml", "version: 1\n")]);

    assert!(
        checked(directory.path(), &["--format", "json"]).contains("\"findings\":[]"),
        "a consumer parses a document, so an empty run is still a document"
    );
    assert!(checked(directory.path(), &["--format", "sarif"]).contains("\"results\":[]"));
    for format in ["github", "terminal"] {
        assert_eq!(
            checked(directory.path(), &["--format", format]).trim(),
            "No findings.",
            "a log a person reads says so, because silence reads as the tool not having run"
        );
    }
}

#[test]
fn the_default_format_is_the_terminal_one() {
    let directory = restricted_call();

    assert_eq!(
        checked(directory.path(), &[]),
        checked(directory.path(), &["--format", "terminal"])
    );
}

#[test]
fn an_unknown_format_is_refused_by_name() {
    let output = Command::new(env!("CARGO_BIN_EXE_godlint"))
        .args(["check", "--format", "toml"])
        .output()
        .unwrap_or_else(|error| panic!("runs godlint: {error}"));
    let printed = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(printed.contains("toml"), "{printed}");
    assert!(
        printed.contains("sarif"),
        "the message lists what is available: {printed}"
    );
}

#[test]
fn a_format_with_no_name_is_refused() {
    let output = Command::new(env!("CARGO_BIN_EXE_godlint"))
        .args(["check", "--format"])
        .output()
        .unwrap_or_else(|error| panic!("runs godlint: {error}"));

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("needs a format"));
}
