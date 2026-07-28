use godlint_core::{
    config::{LineLimitRule, Severity},
    rules::{FileRule, Metric, Rule, Violation, file_size::FileSize},
};

use super::support::{facts, limit};

fn configuration(max_lines: u32, skip_blank_lines: bool, skip_comments: bool) -> LineLimitRule {
    LineLimitRule {
        severity: Severity::Error,
        max_lines: limit(max_lines),
        skip_blank_lines,
        skip_comments,
    }
}

#[test]
fn reports_a_file_over_its_limit() {
    let facts = facts(
        "src/example.rs",
        "// detail\n\nfn example() {\n    run();\n}",
    );

    assert_eq!(FileSize::ID, "maintainability/file-size");
    assert_eq!(
        FileSize::check(&facts, &configuration(2, true, true)),
        Some(Violation::Limit {
            metric: Metric::FileLines,
            actual: 3,
            max: 2
        })
    );
}

#[test]
fn accepts_a_file_at_its_limit() {
    let facts = facts(
        "src/example.rs",
        "// detail\n\nfn example() {\n    run();\n}",
    );

    assert_eq!(FileSize::check(&facts, &configuration(3, true, true)), None);
}

#[test]
fn applies_comment_and_blank_line_configuration() {
    let facts = facts("src/example.py", "# detail\n\ndef example():\n    run()\n");

    assert_eq!(FileSize::check(&facts, &configuration(2, true, true)), None);
    assert_eq!(
        FileSize::check(&facts, &configuration(2, false, false)),
        Some(Violation::Limit {
            metric: Metric::FileLines,
            actual: 4,
            max: 2
        })
    );
}

#[test]
fn ignores_a_byte_order_mark() {
    let plain = facts("src/plain.rs", "// detail\nfn example() {}\n");
    let marked = facts("src/marked.rs", "\u{feff}// detail\nfn example() {}\n");

    assert_eq!(
        FileSize::check(&plain, &configuration(1, true, true)),
        FileSize::check(&marked, &configuration(1, true, true))
    );
}

#[test]
fn counts_line_endings_consistently() {
    let unix = facts("src/unix.rs", "fn a() {}\nfn b() {}\n");
    let windows = facts("src/windows.rs", "fn a() {}\r\nfn b() {}\r\n");
    let unterminated = facts("src/unterminated.rs", "fn a() {}\nfn b() {}");

    for subject in [&unix, &windows, &unterminated] {
        assert_eq!(
            FileSize::check(subject, &configuration(1, true, true)),
            Some(Violation::Limit {
                metric: Metric::FileLines,
                actual: 2,
                max: 1
            })
        );
    }
}
