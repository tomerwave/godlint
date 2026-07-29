use godlint_core::rules::{Violation, restricted_import};

use super::support::rule_violations;

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(restricted_import::evaluate, path, source, configuration)
}

fn restricting(name: &str) -> String {
    format!(
        concat!(
            "version: 1\n",
            "rules:\n",
            "  architecture/restricted-import:\n",
            "    severity: error\n",
            "    modules:\n",
            "      - name: \"{}\"\n"
        ),
        name
    )
}

#[test]
fn reads_an_import_in_each_language() {
    assert_eq!(
        violations(
            "src/example.rs",
            "use crate::internal::store;",
            &restricting("crate::internal")
        )
        .len(),
        1
    );
    assert_eq!(
        violations(
            "src/example.py",
            "from legacy.db import rows",
            &restricting("legacy.db")
        )
        .len(),
        1
    );
    assert_eq!(
        violations(
            "src/example.ts",
            "import { store } from \"@app/internal\";",
            &restricting("@app/internal")
        )
        .len(),
        1
    );
    assert_eq!(
        violations(
            "src/example.js",
            "export { store } from \"@app/internal\";",
            &restricting("@app/internal")
        )
        .len(),
        1,
        "a re-export is an import edge too"
    );
}

#[test]
fn a_restricted_module_covers_what_lies_beneath_it() {
    let configuration = restricting("crate::internal");

    assert_eq!(
        violations("src/a.rs", "use crate::internal;", &configuration).len(),
        1
    );
    assert_eq!(
        violations(
            "src/a.rs",
            "use crate::internal::deep::thing;",
            &configuration
        )
        .len(),
        1
    );
}

#[test]
fn a_name_that_only_shares_a_prefix_is_a_different_module() {
    let configuration = restricting("crate::internal");

    assert!(
        violations("src/a.rs", "use crate::internals::thing;", &configuration).is_empty(),
        "internals is not internal"
    );
    assert!(violations("src/a.rs", "use crate::public::thing;", &configuration).is_empty());
}

#[test]
fn permits_an_import_inside_an_approved_path() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/restricted-import:\n",
        "    severity: error\n",
        "    modules:\n",
        "      - name: crate::internal\n",
        "        allow-in:\n",
        "          - src/wiring/**\n"
    );

    assert!(
        violations(
            "src/wiring/app.rs",
            "use crate::internal::store;",
            configuration
        )
        .is_empty()
    );
    assert_eq!(
        violations("src/other.rs", "use crate::internal::store;", configuration).len(),
        1
    );
}

#[test]
fn an_unnamed_module_is_not_restricted() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/restricted-import:\n",
        "    severity: error\n"
    );

    assert!(violations("src/a.rs", "use std::collections::BTreeMap;", configuration).is_empty());
}

#[test]
fn can_disable_the_rule() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/restricted-import:\n",
        "    severity: off\n",
        "    modules:\n",
        "      - name: crate::internal\n"
    );

    assert!(violations("src/a.rs", "use crate::internal::store;", configuration).is_empty());
}
