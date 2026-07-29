#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{
    fs,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use godlint_core::{
    config::{Config, ConfigError, Severity},
    rules::{configured_severity, rule_ids},
    suites,
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn load(contents: &str) -> Result<Config, ConfigError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("godlint-suite-{timestamp}-{id}.yaml"));

    fs::write(&path, contents).unwrap_or_else(|error| panic!("writes config: {error}"));

    let result = Config::load(&path);

    fs::remove_file(path).unwrap_or_else(|error| panic!("removes config: {error}"));

    result
}

fn recommended() -> Config {
    load("version: 1\nsuites:\n  - recommended@1\n")
        .unwrap_or_else(|error| panic!("loads recommended: {error}"))
}

#[test]
fn a_bare_configuration_enforces_nothing() {
    let config = load("version: 1\n").unwrap_or_else(|error| panic!("loads: {error}"));

    assert!(
        config.suites.is_empty(),
        "a suite must be asked for by name"
    );
    assert!(config.rules.function_size.is_none());
    assert!(config.rules.no_comments.is_none());
    assert!(config.rules.restricted_call.is_none());
}

#[test]
fn recommended_enables_every_rule_at_error() {
    let config = recommended();
    let off: Vec<&str> = rule_ids()
        .filter(|id| configured_severity(&config, id) != Severity::Error)
        .collect();

    assert!(
        off.is_empty(),
        "recommended@1 must enforce every rule at error; these are not: {off:?}"
    );
}

fn documented_thresholds() -> Vec<(String, u32)> {
    let roadmap = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/rule-roadmap.md");
    let text = fs::read_to_string(roadmap).unwrap_or_else(|error| panic!("reads roadmap: {error}"));

    text.lines()
        .skip_while(|line| !is_recommended_header(line))
        .take_while(|line| line.starts_with('|'))
        .filter_map(documented_threshold)
        .collect()
}

fn is_recommended_header(line: &str) -> bool {
    line.starts_with('|') && line.contains("`recommended@1`")
}

fn documented_threshold(line: &str) -> Option<(String, u32)> {
    let mut cells = line.split('|').map(str::trim).skip(1);
    let rule = documented_rule(cells.next()?)?;
    let threshold = cells.next()?.parse().ok()?;

    Some((rule.to_owned(), threshold))
}

fn documented_rule(cell: &str) -> Option<&str> {
    cell.strip_prefix('`')
        .and_then(|rule| rule.strip_suffix('`'))
        .filter(|rule| rule.starts_with("maintainability/"))
}

fn configured_limits(config: &Config) -> Vec<(String, u32)> {
    let rules = &config.rules;

    vec![
        (
            "maintainability/file-size",
            rules.file_size.as_ref().expect("file").max_lines.get(),
        ),
        (
            "maintainability/function-size",
            rules.function_size.as_ref().expect("size").max_lines.get(),
        ),
        (
            "maintainability/function-nesting",
            rules.function_nesting.as_ref().expect("nesting").limit(),
        ),
        (
            "maintainability/parameter-count",
            rules.parameter_count.as_ref().expect("parameters").limit(),
        ),
        (
            "maintainability/decision-complexity",
            rules
                .decision_complexity
                .as_ref()
                .expect("complexity")
                .limit(),
        ),
        (
            "maintainability/return-count",
            rules.return_count.as_ref().expect("returns").limit(),
        ),
        (
            "maintainability/function-statements",
            rules
                .function_statements
                .as_ref()
                .expect("statements")
                .limit(),
        ),
    ]
    .into_iter()
    .map(|(rule, limit)| (rule.to_owned(), limit))
    .collect()
}

#[test]
fn recommended_carries_the_documented_thresholds() {
    let mut documented = documented_thresholds();
    let mut configured = configured_limits(&recommended());

    documented.sort();
    configured.sort();

    assert_eq!(
        documented, configured,
        "docs/rule-roadmap.md and recommended@1 disagree; the table is the source"
    );
}

#[test]
fn the_roadmap_documents_every_threshold() {
    let documented = documented_thresholds();

    assert_eq!(
        documented.len(),
        configured_limits(&recommended()).len(),
        "every threshold recommended@1 sets must appear in docs/rule-roadmap.md: {documented:?}"
    );
}

#[test]
fn a_rules_entry_overrides_the_suite() {
    let config = load(concat!(
        "version: 1\n",
        "suites:\n",
        "  - recommended@1\n",
        "rules:\n",
        "  maintainability/parameter-count:\n",
        "    severity: warning\n",
        "    max-parameters: 9\n"
    ))
    .unwrap_or_else(|error| panic!("loads: {error}"));
    let parameters = config.rules.parameter_count.as_ref().expect("parameters");

    assert_eq!(parameters.severity, Severity::Warning);
    assert_eq!(parameters.limit(), 9);
    assert_eq!(
        config
            .rules
            .function_nesting
            .as_ref()
            .expect("nesting")
            .limit(),
        2,
        "a rule the repository did not name still comes from the suite"
    );
}

#[test]
fn a_rules_entry_can_switch_a_suite_rule_off() {
    let config = load(concat!(
        "version: 1\n",
        "suites:\n",
        "  - recommended@1\n",
        "rules:\n",
        "  style/no-comments:\n",
        "    severity: off\n"
    ))
    .unwrap_or_else(|error| panic!("loads: {error}"));

    assert_eq!(
        config
            .rules
            .no_comments
            .as_ref()
            .expect("comments")
            .severity,
        Severity::Off,
        "a repository must be able to decline one rule without abandoning the suite"
    );
}

#[test]
fn rejects_an_unknown_suite() {
    let Err(error) = load("version: 1\nsuites:\n  - strict@9\n") else {
        panic!("an unknown suite must not load");
    };
    let message = error.to_string();

    assert!(message.contains("strict@9"), "{message}");
    assert!(message.contains(suites::RECOMMENDED), "{message}");
}

#[test]
fn every_named_suite_is_applicable() {
    for name in suites::NAMES {
        let config = load(&format!("version: 1\nsuites:\n  - {name}\n"))
            .unwrap_or_else(|error| panic!("loads {name}: {error}"));

        assert!(
            rule_ids().any(|id| configured_severity(&config, id) != Severity::Off),
            "{name} enables no rule"
        );
    }
}
