use godlint_core::rules::{Violation, forbidden_dependency};

use super::support::rule_violations;

fn forbidding(name: &str) -> String {
    format!(
        concat!(
            "version: 1\n",
            "rules:\n",
            "  security/forbidden-dependency:\n",
            "    severity: error\n",
            "    packages:\n",
            "      - name: \"{}\"\n"
        ),
        name
    )
}

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(forbidden_dependency::evaluate, path, source, configuration)
}

#[test]
fn forbids_a_package_however_deeply_it_is_imported() {
    let configuration = forbidding("lodash");

    assert_eq!(
        violations("src/a.ts", "import merge from \"lodash\";", &configuration).len(),
        1
    );
    assert_eq!(
        violations(
            "src/a.ts",
            "import merge from \"lodash/merge\";",
            &configuration
        )
        .len(),
        1,
        "a deep import is the same dependency"
    );
}

#[test]
fn reads_a_scoped_package_as_its_scope_and_name() {
    assert_eq!(
        violations(
            "src/a.ts",
            "import x from \"@corp/legacy/deep\";",
            &forbidding("@corp/legacy")
        )
        .len(),
        1
    );
    assert!(
        violations(
            "src/a.ts",
            "import x from \"@corp/allowed\";",
            &forbidding("@corp/legacy")
        )
        .is_empty(),
        "another package in the same scope is a different dependency"
    );
}

#[test]
fn names_a_package_exactly_rather_than_by_prefix() {
    assert!(
        violations(
            "src/a.ts",
            "import x from \"lodash-es\";",
            &forbidding("lodash")
        )
        .is_empty(),
        "lodash-es is its own package"
    );
    assert!(violations("src/a.py", "import requests_mock", &forbidding("requests")).is_empty());
}

#[test]
fn forbids_a_package_in_each_language() {
    assert_eq!(
        violations(
            "src/a.py",
            "from requests.adapters import x",
            &forbidding("requests")
        )
        .len(),
        1
    );
    assert_eq!(
        violations("src/a.rs", "use serde::de::Error;", &forbidding("serde")).len(),
        1
    );
    assert_eq!(
        violations("src/a.rs", "extern crate serde;", &forbidding("serde")).len(),
        1
    );
    assert_eq!(
        violations(
            "src/a.go",
            "package app\n\nimport \"github.com/acme/legacy/codec\"",
            &forbidding("github.com/acme/legacy")
        )
        .len(),
        1
    );
}

#[test]
fn first_party_code_is_not_a_dependency() {
    for (path, source) in [
        ("src/a.rs", "use crate::internal::store;"),
        ("src/a.rs", "use self::helper;"),
        ("src/a.rs", "use super::parent;"),
        ("src/a.ts", "import x from \"./sibling\";"),
        ("src/a.ts", "import x from \"../parent/thing\";"),
        ("src/a.py", "from .sibling import x"),
    ] {
        assert!(
            violations(path, source, &forbidding("crate")).is_empty()
                && violations(path, source, &forbidding("sibling")).is_empty(),
            "{source} names no dependency"
        );
    }
}

#[test]
fn a_platform_builtin_is_not_a_package() {
    assert!(
        violations(
            "src/a.ts",
            "import { z } from \"node:fs\";",
            &forbidding("node")
        )
        .is_empty(),
        "a protocol-qualified builtin is not a published package"
    );
}

#[test]
fn permits_a_dependency_inside_an_approved_path() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  security/forbidden-dependency:\n",
        "    severity: error\n",
        "    packages:\n",
        "      - name: lodash\n",
        "        allow-in:\n",
        "          - src/vendor/**\n"
    );

    assert!(
        violations(
            "src/vendor/shim.ts",
            "import x from \"lodash\";",
            configuration
        )
        .is_empty()
    );
    assert_eq!(
        violations("src/app.ts", "import x from \"lodash\";", configuration).len(),
        1
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  security/forbidden-dependency:\n",
        "    severity: off\n",
        "    packages:\n",
        "      - name: lodash\n"
    );

    assert!(violations("src/a.ts", "import x from \"lodash\";", configuration).is_empty());
}

#[test]
fn a_relative_or_absolute_specifier_yields_no_package_at_all() {
    assert!(
        violations("src/a.ts", "import x from \"./sibling\";", &forbidding(".")).is_empty(),
        "a leading dot is not a package name"
    );
    assert!(
        violations(
            "src/a.ts",
            "import x from \"../up/thing\";",
            &forbidding("..")
        )
        .is_empty()
    );
    assert!(
        violations(
            "src/a.ts",
            "import x from \"/rooted/thing\";",
            &forbidding("")
        )
        .is_empty(),
        "an absolute specifier has no first segment to name"
    );
    assert!(violations("src/a.py", "from .rel import x", &forbidding(".")).is_empty());
}

#[test]
fn a_protocol_qualified_specifier_is_never_a_package() {
    assert!(
        violations(
            "src/a.ts",
            "import { z } from \"node:fs\";",
            &forbidding("node:fs")
        )
        .is_empty(),
        "naming the specifier itself must not turn a builtin into a dependency"
    );
}

#[test]
fn a_scope_with_no_package_names_nothing() {
    assert!(
        violations("src/a.ts", "import x from \"@corp\";", &forbidding("@corp")).is_empty(),
        "a bare scope is not a package"
    );
}

#[test]
fn a_leading_path_separator_does_not_hide_a_crate() {
    assert_eq!(
        violations("src/a.rs", "use ::serde::de::Error;", &forbidding("serde")).len(),
        1,
        "::serde names the crate more explicitly, not less"
    );
    assert!(violations("src/a.rs", "use ::crate_like::thing;", &forbidding("serde")).is_empty());
}

#[test]
fn a_scope_with_a_trailing_separator_names_no_package() {
    for module in ["@corp/", "@corp//legacy"] {
        assert!(
            violations(
                "src/a.ts",
                &format!("import c from \"{module}\";"),
                &forbidding(module)
            )
            .is_empty(),
            "{module} has no package after the scope"
        );
    }
}

#[test]
fn a_bundler_alias_is_first_party_rather_than_a_scope() {
    for module in ["@/components/Button", "@/x", "@"] {
        assert!(
            violations(
                "src/a.ts",
                &format!("import f from \"{module}\";"),
                &forbidding(module)
            )
            .is_empty(),
            "{module} aliases first-party source rather than naming a registry scope"
        );
    }
}
