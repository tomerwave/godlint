use godlint_core::rules::{Violation, dependency_boundary};

use super::support::rule_violations;

const LAYERS: &str = concat!(
    "version: 1\n",
    "rules:\n",
    "  architecture/dependency-boundary:\n",
    "    severity: error\n",
    "    layers:\n",
    "      - name: ui\n",
    "        paths:\n",
    "          - src/ui/**\n",
    "        modules:\n",
    "          - crate::ui\n",
    "      - name: application\n",
    "        paths:\n",
    "          - src/app/**\n",
    "        modules:\n",
    "          - crate::app\n",
    "      - name: domain\n",
    "        paths:\n",
    "          - src/domain/**\n",
    "        modules:\n",
    "          - crate::domain\n"
);

fn violations(path: &str, source: &str) -> Vec<Violation> {
    rule_violations(dependency_boundary::evaluate, path, source, LAYERS)
}

#[test]
fn reports_a_dependency_that_runs_against_the_declared_order() {
    assert_eq!(
        violations("src/domain/model.rs", "use crate::app::service;").len(),
        1
    );
    assert_eq!(
        violations("src/domain/model.rs", "use crate::ui::widget;").len(),
        1
    );
    assert_eq!(
        violations("src/app/service.rs", "use crate::ui::widget;").len(),
        1
    );
}

#[test]
fn permits_a_dependency_that_runs_with_the_order() {
    assert!(violations("src/ui/widget.rs", "use crate::app::service;").is_empty());
    assert!(violations("src/ui/widget.rs", "use crate::domain::model;").is_empty());
    assert!(violations("src/app/service.rs", "use crate::domain::model;").is_empty());
}

#[test]
fn permits_a_layer_depending_on_itself() {
    assert!(violations("src/app/service.rs", "use crate::app::helper;").is_empty());
    assert!(violations("src/domain/model.rs", "use crate::domain::value;").is_empty());
}

#[test]
fn ignores_a_file_or_module_outside_every_layer() {
    assert!(
        violations("src/domain/model.rs", "use std::collections::BTreeMap;").is_empty(),
        "a module no layer names is outside the policy"
    );
    assert!(
        violations("build.rs", "use crate::ui::widget;").is_empty(),
        "a file no layer contains is outside the policy"
    );
}

#[test]
fn names_a_layer_by_whole_segment() {
    assert!(
        violations("src/domain/model.rs", "use crate::application::thing;").is_empty(),
        "crate::application is not crate::app"
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration = LAYERS.replace("severity: error", "severity: off");

    assert!(
        rule_violations(
            dependency_boundary::evaluate,
            "src/domain/model.rs",
            "use crate::app::service;",
            &configuration
        )
        .is_empty()
    );
}
