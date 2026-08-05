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
fn every_extension_carries_its_own_convention_without_configuration() {
    for good in [
        "src/line_count.rs",
        "src/module.py",
        "src/stub.pyi",
        "src/http-client.ts",
        "src/http-client.js",
        "src/http-client.mjs",
        "src/http-client.cjs",
        "src/http-client.mts",
        "src/http-client.cts",
        "src/Button.tsx",
        "src/Button.jsx",
    ] {
        assert!(
            violations(good, ENABLED).is_empty(),
            "{good} follows the convention for its extension"
        );
    }

    for bad in [
        "src/lineCount.rs",
        "src/LineCount.rs",
        "src/line-count.py",
        "src/httpClient.ts",
        "src/HttpClient.ts",
        "src/http_client.js",
        "src/button.tsx",
        "src/button-panel.tsx",
    ] {
        assert_eq!(
            violations(bad, ENABLED).len(),
            1,
            "{bad} does not follow the convention for its extension"
        );
    }
}

#[test]
fn a_component_extension_and_a_module_extension_differ_in_the_same_directory() {
    assert!(violations("src/ui/Button.tsx", ENABLED).is_empty());
    assert!(violations("src/ui/use-button.ts", ENABLED).is_empty());
    assert_eq!(
        violations("src/ui/Button.ts", ENABLED).len(),
        1,
        "a module is not a component just because it sits beside one"
    );
    assert_eq!(violations("src/ui/use-button.tsx", ENABLED).len(), 1);
}

#[test]
fn a_scope_declares_the_case_for_the_paths_it_names() {
    let camel = scoped("src/**/*.ts", "camel");

    assert!(violations("src/lineCount.ts", &camel).is_empty());
    assert_eq!(violations("src/line-count.ts", &camel).len(), 1);
    assert_eq!(
        violations("app/lineCount.ts", &camel).len(),
        1,
        "a path the scope does not name keeps the convention for its extension"
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
    assert!(
        violations("src/line-count.test.ts", ENABLED).is_empty(),
        "the name is what precedes the first dot"
    );
    assert_eq!(violations("src/lineCount.test.ts", ENABLED).len(), 1);
}

#[test]
fn framework_dynamic_route_segments_are_exempt_from_filename_conventions() {
    for path in [
        "src/pages/[slug].ts",
        "src/pages/[...slug].ts",
        "src/pages/[[...slug]].ts",
        "src/pages/[...slug].md.ts",
    ] {
        assert!(
            violations(path, ENABLED).is_empty(),
            "{path} is a framework-required route filename"
        );
    }

    for path in [
        "src/pages/[].ts",
        "src/pages/[...].ts",
        "src/pages/[[slug]].ts",
        "src/pages/[slug]value.ts",
    ] {
        assert_eq!(
            violations(path, ENABLED).len(),
            1,
            "{path} is not a framework route filename"
        );
    }
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
    assert!(violations("src/base64-encode.ts", ENABLED).is_empty());
    assert_eq!(
        violations("src/base--64.ts", ENABLED).len(),
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
        violations("src/.d.ts", ENABLED).is_empty(),
        "a file whose name begins with the dot has no stem to hold to a convention"
    );
}

#[test]
fn a_file_at_the_repository_root_is_judged_the_same_as_a_nested_one() {
    assert_eq!(violations("lineCount.rs", ENABLED).len(), 1);
    assert!(violations("line_count.rs", ENABLED).is_empty());
}

#[test]
fn a_dunder_or_private_python_name_is_snake_case() {
    for name in ["__init__", "__main__", "_private", "trailing_"] {
        assert!(
            violations(&format!("pkg/{name}.py"), ENABLED).is_empty(),
            "{name}.py is snake_case; PEP 8 includes a leading or trailing underscore"
        );
    }

    assert_eq!(
        violations("pkg/a__b.py", ENABLED).len(),
        1,
        "a doubled separator inside the name leaves an empty segment"
    );
    assert_eq!(
        violations("pkg/_.py", ENABLED).len(),
        1,
        "a name that is only separators is not a name"
    );
}

#[test]
fn the_most_specific_scope_wins_regardless_of_declared_order() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/filename-case:\n",
        "    severity: error\n",
        "    scopes:\n",
        "      - paths:\n",
        "          - \"ui/**\"\n",
        "        case: pascal\n",
        "      - paths:\n",
        "          - \"ui/legacy/**\"\n",
        "        case: camel\n"
    );

    assert!(
        violations("ui/legacy/oldWidget.ts", configuration).is_empty(),
        "the narrower scope decides, though it was declared second"
    );
    assert_eq!(violations("ui/legacy/OldWidget.ts", configuration).len(), 1);
    assert!(violations("ui/Widget.ts", configuration).is_empty());
}

#[test]
fn a_case_is_judged_in_ascii_consistently() {
    for path in ["ui/Éclair.tsx", "ui/café.ts", "pkg/café.py"] {
        assert_eq!(
            violations(path, ENABLED).len(),
            1,
            "{path} is judged against an ASCII convention in every position"
        );
    }
}

#[test]
fn a_finding_can_be_suppressed_like_any_other_file_level_finding() {
    assert!(
        godlint_core::rules::is_suppressible_rule("architecture/filename-case"),
        "a file name finding is suppressible, the same as a file size finding"
    );
}
