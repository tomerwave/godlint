use godlint_core::rules::{Violation, module_independence};

use super::support::rule_violations;

const FEATURES: &str = concat!(
    "version: 1\n",
    "rules:\n",
    "  architecture/module-independence:\n",
    "    severity: error\n",
    "    sets:\n",
    "      - name: features\n",
    "        members:\n",
    "          - name: billing\n",
    "            paths: [src/billing/**]\n",
    "            modules: [crate::billing]\n",
    "          - name: notifications\n",
    "            paths: [src/notifications/**]\n",
    "            modules: [crate::notifications]\n",
);

fn violations(path: &str, source: &str) -> Vec<Violation> {
    rule_violations(module_independence::evaluate, path, source, FEATURES)
}

fn broke(from: &str, to: &str) -> Violation {
    Violation::BrokeIndependence {
        set: "features".to_owned(),
        from: from.to_owned(),
        to: to.to_owned(),
    }
}

#[test]
fn reports_a_member_reaching_another_member() {
    assert_eq!(
        violations("src/billing/charge.rs", "use crate::notifications::send;"),
        vec![broke("billing", "notifications")]
    );
}

#[test]
fn reads_go_module_paths() {
    let configuration = FEATURES
        .replace("crate::billing", "github.com/acme/billing")
        .replace("crate::notifications", "github.com/acme/notifications");
    assert_eq!(
        rule_violations(
            module_independence::evaluate,
            "src/billing/charge.go",
            "package billing\n\nimport \"github.com/acme/notifications/send\"",
            &configuration
        )
        .len(),
        1
    );
}

#[test]
fn reports_the_reverse_direction_too() {
    assert_eq!(
        violations("src/notifications/send.rs", "use crate::billing::charge;"),
        vec![broke("notifications", "billing")],
        "independence is mutual, unlike a layer order where only one direction is wrong"
    );
}

#[test]
fn permits_a_member_importing_itself() {
    assert!(
        violations("src/billing/charge.rs", "use crate::billing::ledger;").is_empty(),
        "a module's own internals are not a foreign dependency"
    );
}

#[test]
fn permits_a_file_outside_the_set_importing_a_member() {
    assert!(
        violations("src/shared/report.rs", "use crate::billing::charge;").is_empty(),
        "the set constrains its members' dependencies, not everyone else's"
    );
}

#[test]
fn permits_a_member_importing_something_outside_the_set() {
    assert!(violations("src/billing/charge.rs", "use crate::shared::money;").is_empty());
}

#[test]
fn ignores_an_import_when_no_set_is_configured() {
    let empty = concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/module-independence:\n",
        "    severity: error\n",
    );

    assert!(
        rule_violations(
            module_independence::evaluate,
            "src/billing/charge.rs",
            "use crate::notifications::send;",
            empty,
        )
        .is_empty(),
        "the rule enforces nothing until a repository names a set"
    );
}

#[test]
fn can_disable_the_rule() {
    let off = FEATURES.replace("severity: error", "severity: off");

    assert!(
        rule_violations(
            module_independence::evaluate,
            "src/billing/charge.rs",
            "use crate::notifications::send;",
            &off,
        )
        .is_empty()
    );
}
