use godlint_core::rules::{Violation, explicit_timer_delay};

use super::support::rule_violations;

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(explicit_timer_delay::evaluate, path, source, configuration)
}

#[test]
fn requires_a_delay_for_each_javascript_timer() {
    let configuration =
        "version: 1\nrules:\n  reliability/explicit-timer-delay:\n    severity: error\n";

    assert_eq!(
        violations("src/example.ts", "setTimeout(work);", configuration).len(),
        1
    );
    assert_eq!(
        violations("src/example.js", "setInterval(work);", configuration).len(),
        1
    );
}

#[test]
fn requires_a_duration_for_go_timers() {
    let configuration =
        "version: 1\nrules:\n  reliability/explicit-timer-delay:\n    severity: error\n";

    assert_eq!(
        violations(
            "src/example.go",
            "package example\n\nimport \"time\"\n\nfunc f() { time.After() }",
            configuration
        )
        .len(),
        1
    );
    assert_eq!(
        violations(
            "src/example.go",
            "package example\n\nimport \"time\"\n\nfunc f() { time.AfterFunc() }",
            configuration
        )
        .len(),
        1
    );
    assert!(
        violations(
            "src/example.go",
            "package example\n\nimport \"time\"\n\nfunc f() { time.NewTimer(time.Second) }",
            configuration
        )
        .is_empty()
    );
}

#[test]
fn permits_timers_with_a_delay_and_unrelated_calls() {
    let configuration =
        "version: 1\nrules:\n  reliability/explicit-timer-delay:\n    severity: error\n";

    assert!(violations("src/example.ts", "setTimeout(work, 50);", configuration).is_empty());
    assert!(violations("src/example.ts", "schedule(work);", configuration).is_empty());
}

#[test]
fn does_not_apply_a_javascript_default_to_other_languages() {
    let configuration =
        "version: 1\nrules:\n  reliability/explicit-timer-delay:\n    severity: error\n";

    assert!(violations("src/example.py", "setTimeout(work)", configuration).is_empty());
    assert!(violations("src/example.rs", "setTimeout(work);", configuration).is_empty());
}

#[test]
fn counts_a_commented_out_delay_as_absent() {
    let configuration =
        "version: 1\nrules:\n  reliability/explicit-timer-delay:\n    severity: error\n";

    assert_eq!(
        violations(
            "src/example.js",
            "setTimeout(work /*, 100 */);",
            configuration
        )
        .len(),
        1
    );
    assert!(
        violations(
            "src/example.js",
            "setTimeout(work, /* ms */ 100);",
            configuration
        )
        .is_empty()
    );
}

#[test]
fn reads_a_timer_reached_through_a_global_object() {
    let configuration =
        "version: 1\nrules:\n  reliability/explicit-timer-delay:\n    severity: error\n";

    for callee in ["window", "globalThis", "self"] {
        assert_eq!(
            violations(
                "src/example.js",
                &format!("{callee}.setTimeout(work);"),
                configuration
            )
            .len(),
            1,
            "{callee}.setTimeout should require a delay"
        );
    }

    assert!(
        violations(
            "src/example.js",
            "window.setTimeout(work, 100);",
            configuration
        )
        .is_empty()
    );
    assert!(violations("src/example.js", "timers.setTimeout(work);", configuration).is_empty());
}

#[test]
fn can_disable_the_rule() {
    let configuration =
        "version: 1\nrules:\n  reliability/explicit-timer-delay:\n    severity: off\n";

    assert!(violations("src/example.ts", "setTimeout(work);", configuration).is_empty());
}
