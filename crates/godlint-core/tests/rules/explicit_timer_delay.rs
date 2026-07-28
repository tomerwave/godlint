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
fn can_disable_the_rule() {
    let configuration =
        "version: 1\nrules:\n  reliability/explicit-timer-delay:\n    severity: off\n";

    assert!(violations("src/example.ts", "setTimeout(work);", configuration).is_empty());
}
