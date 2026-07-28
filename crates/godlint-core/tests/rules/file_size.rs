use std::path::PathBuf;

use godlint_core::{
    analyzers::analyze,
    config::{FileSizeRule, Severity},
    rules::{Rule, file_size::FileSize},
    source::SourceFile,
};

fn facts(path: &str, source: &str) -> godlint_core::analyzers::SourceFacts {
    let source = SourceFile::new(PathBuf::from(path), source.into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));

    analyze(&source).unwrap_or_else(|error| panic!("analyzes source: {error}"))
}

fn configuration(max_lines: u32, skip_blank_lines: bool, skip_comments: bool) -> FileSizeRule {
    FileSizeRule {
        severity: Severity::Error,
        max_lines,
        skip_blank_lines,
        skip_comments,
    }
}

#[test]
fn reports_a_file_that_exceeds_its_limit() {
    let facts = facts(
        "src/example.rs",
        "// detail\n\nfn example() {\n    run();\n}",
    );
    let violation = FileSize::evaluate(&facts, &configuration(2, true, true));

    assert_eq!(FileSize::ID, "maintainability/file-size");
    assert_eq!(
        violation.map(|violation| violation.effective_line_count),
        Some(3)
    );
}

#[test]
fn applies_comment_and_blank_line_configuration() {
    let facts = facts("src/example.py", "# detail\n\ndef example():\n    run()\n");

    assert_eq!(
        FileSize::evaluate(&facts, &configuration(2, true, true)),
        None
    );
    assert_eq!(
        FileSize::evaluate(&facts, &configuration(2, false, false))
            .map(|violation| violation.effective_line_count),
        Some(4)
    );
}

#[test]
fn disables_evaluation_when_the_rule_is_off() {
    let facts = facts("src/example.ts", "function example() {\n  run();\n}");
    let configuration = FileSizeRule {
        severity: Severity::Off,
        max_lines: 1,
        skip_blank_lines: true,
        skip_comments: true,
    };

    assert_eq!(FileSize::evaluate(&facts, &configuration), None);
}
