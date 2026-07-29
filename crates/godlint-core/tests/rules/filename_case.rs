use godlint_core::rules::{Violation, filename_case};

use super::support::rule_violations;

const ENABLED: &str = "version: 1\nrules:\n  architecture/filename-case:\n    severity: error\n";

fn violations(path: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(filename_case::evaluate, path, "", configuration)
}

fn scoped(paths: &str, case: &str) -> String {
    format!(
        concat!(
            "version: 1\n",
            "rules:\n",
            "  architecture/filename-case:\n",
            "    severity: error\n",
            "    scopes:\n",
            "      - paths:\n",
            "          - \"{}\"\n",
            "        case: {}\n"
        ),
        paths, case
    )
}

#[test]
fn a_language_with_a_convention_is_held_to_it_without_configuration() {
    assert!(violations("src/line_count.rs", ENABLED).is_empty());
    assert!(violations("src/module.py", ENABLED).is_empty());
    assert_eq!(violations("src/lineCount.rs", ENABLED).len(), 1);
    assert_eq!(violations("src/LineCount.rs", ENABLED).len(), 1);
    assert_eq!(violations("src/line-count.py", ENABLED).len(), 1);
}

#[test]
fn a_language_with_no_single_convention_is_silent_until_asked() {
    for path in ["src/lineCount.ts", "src/LineCount.tsx", "src/line-count.js"] {
        assert!(
            violations(path, ENABLED).is_empty(),
            "{path} has no conventional case to assume"
        );
    }
}

#[test]
fn a_scope_declares_the_case_for_the_paths_it_names() {
    let kebab = scoped("src/**/*.ts", "kebab");

    assert!(violations("src/line-count.ts", &kebab).is_empty());
    assert_eq!(violations("src/lineCount.ts", &kebab).len(), 1);
    assert!(
        violations("app/lineCount.ts", &kebab).is_empty(),
        "a path the scope does not name keeps the language default"
    );
}

#[test]
fn a_scope_overrides_a_conventional_default() {
    let pascal = scoped("src/**/*.rs", "pascal");

    assert!(violations("src/LineCount.rs", &pascal).is_empty());
    assert_eq!(
        violations("src/line_count.rs", &pascal).len(),
        1,
        "the scope wins over the language convention"
    );
}

#[test]
fn every_case_accepts_its_own_spelling_and_rejects_the_others() {
    for (case, good, bad) in [
        ("kebab", "line-count", "line_count"),
        ("snake", "line_count", "line-count"),
        ("camel", "lineCount", "LineCount"),
        ("pascal", "LineCount", "lineCount"),
    ] {
        let configuration = scoped("**/*.ts", case);

        assert!(
            violations(&format!("src/{good}.ts"), &configuration).is_empty(),
            "{case} accepts {good}"
        );
        assert_eq!(
            violations(&format!("src/{bad}.ts"), &configuration).len(),
            1,
            "{case} rejects {bad}"
        );
    }
}

#[test]
fn a_compound_extension_is_not_part_of_the_name() {
    let kebab = scoped("**/*.ts", "kebab");

    assert!(
        violations("src/line-count.test.ts", &kebab).is_empty(),
        "the name is what precedes the first dot"
    );
    assert_eq!(violations("src/lineCount.test.ts", &kebab).len(), 1);
}

#[test]
fn an_allowed_path_is_exempt() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/filename-case:\n",
        "    severity: error\n",
        "    allow:\n",
        "      - \"**/lineCount.rs\"\n"
    );

    assert!(violations("src/lineCount.rs", configuration).is_empty());
    assert_eq!(violations("src/otherName.rs", configuration).len(), 1);
}

#[test]
fn a_digit_is_part_of_a_segment_but_not_a_segment_of_its_own() {
    let kebab = scoped("**/*.ts", "kebab");

    assert!(violations("src/base64-encode.ts", &kebab).is_empty());
    assert_eq!(
        violations("src/base--64.ts", &kebab).len(),
        1,
        "an empty segment is not a name"
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration = "version: 1\nrules:\n  architecture/filename-case:\n    severity: off\n";

    assert!(violations("src/lineCount.rs", configuration).is_empty());
}

#[test]
fn a_name_that_is_only_an_extension_is_judged_as_nothing() {
    assert!(
        violations("src/.d.ts", &scoped("**/*.ts", "kebab")).is_empty(),
        "a file whose name begins with the dot has no stem to hold to a convention"
    );
}

#[test]
fn a_file_at_the_repository_root_is_judged_the_same_as_a_nested_one() {
    assert_eq!(violations("lineCount.rs", ENABLED).len(), 1);
    assert!(violations("line_count.rs", ENABLED).is_empty());
}
