use godlint_core::{
    config::{LineLimitRule, Severity},
    rules::{Metric, Rule, Violation, file_size::FileSize},
};

use super::support::{file_limits, limit};

fn configuration(max_lines: u32, skip_blank_lines: bool, skip_comments: bool) -> LineLimitRule {
    LineLimitRule {
        severity: Severity::Error,
        max_lines: limit(max_lines),
        skip_blank_lines,
        skip_comments,
    }
}

fn violations(path: &str, source: &str, configuration: &LineLimitRule) -> Vec<Violation> {
    file_limits::<FileSize>(path, source, configuration)
}

#[test]
fn reports_a_file_over_its_limit() {
    assert_eq!(FileSize::ID, "maintainability/file-size");
    assert_eq!(
        violations(
            "src/example.rs",
            "// detail\n\nfn example() {\n    run();\n}",
            &configuration(2, true, true)
        ),
        vec![Violation::limit(Metric::FileLines, 3, 2)]
    );
}

#[test]
fn accepts_a_file_at_its_limit() {
    assert!(
        violations(
            "src/example.rs",
            "// detail\n\nfn example() {\n    run();\n}",
            &configuration(3, true, true)
        )
        .is_empty()
    );
}

#[test]
fn applies_comment_and_blank_line_configuration() {
    let source = "# detail\n\ndef example():\n    run()\n";

    assert!(violations("src/example.py", source, &configuration(2, true, true)).is_empty());
    assert_eq!(
        violations("src/example.py", source, &configuration(2, false, false)),
        vec![Violation::limit(Metric::FileLines, 4, 2)]
    );
}

#[test]
fn ignores_a_byte_order_mark() {
    let plain = violations(
        "src/plain.rs",
        "// detail\nfn example() {}\n",
        &configuration(1, true, true),
    );
    let marked = violations(
        "src/marked.rs",
        "\u{feff}// detail\nfn example() {}\n",
        &configuration(1, true, true),
    );

    assert_eq!(plain, marked);
}

#[test]
fn counts_line_endings_consistently() {
    let cases = [
        ("src/unix.rs", "fn a() {}\nfn b() {}\n"),
        ("src/windows.rs", "fn a() {}\r\nfn b() {}\r\n"),
        ("src/unterminated.rs", "fn a() {}\nfn b() {}"),
    ];

    for (path, source) in cases {
        assert_eq!(
            violations(path, source, &configuration(1, true, true)),
            vec![Violation::limit(Metric::FileLines, 2, 1)],
            "{path}"
        );
    }
}
