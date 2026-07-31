use godlint_core::{
    config::Severity,
    rules::{Violation, no_internal_import},
};

use super::support::{rule_findings, rule_violations};

const ENABLED: &str =
    "version: 1\nrules:\n  architecture/no-internal-import:\n    severity: error\n";

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(no_internal_import::evaluate, path, source, configuration)
}

fn reported(path: &str, source: &str) -> Vec<Violation> {
    violations(path, source, ENABLED)
}

fn marker(path: &str, source: &str) -> String {
    match reported(path, source).first() {
        Some(Violation::InternalImport { marker, .. }) => marker.clone(),
        other => panic!("expected an internal import for {source}, got {other:?}"),
    }
}

#[test]
fn reports_a_hidden_segment_in_a_package_path() {
    let cases = [
        (
            "src/app.ts",
            "import { p } from \"some-lib/internal/parser\";",
        ),
        (
            "src/app.ts",
            "import { p } from \"some-lib/private/parser\";",
        ),
        ("src/app.ts", "import { p } from \"some-lib/impl/parser\";"),
        (
            "src/app.ts",
            "import { p } from \"some-lib/_internal/parser\";",
        ),
        ("src/app.py", "from package._private.helpers import munge"),
        ("src/app.py", "from package.impl.engine import run"),
    ];

    for (path, source) in cases {
        assert_eq!(reported(path, source).len(), 1, "{source}");
    }
}

#[test]
fn reports_a_build_output_segment_at_warning() {
    for segment in ["dist", "src", "build"] {
        let source = format!("import {{ p }} from \"some-lib/{segment}/parser\";");
        let findings = rule_findings(no_internal_import::evaluate, "src/app.ts", &source, ENABLED);

        assert_eq!(
            findings.first().map(|finding| finding.severity),
            Some(Severity::Warning),
            "some packages publish their build output as the documented entry: {source}"
        );
    }
}

#[test]
fn reports_a_hidden_segment_at_error_even_beside_a_build_one() {
    let findings = rule_findings(
        no_internal_import::evaluate,
        "src/app.ts",
        "import { p } from \"some-lib/dist/internal/parser\";",
        ENABLED,
    );

    assert_eq!(
        findings.first().map(|finding| finding.severity),
        Some(Severity::Error),
        "a path naming both is certain, so the certain marker decides"
    );
    assert_eq!(
        marker(
            "src/app.ts",
            "import { p } from \"some-lib/dist/internal/parser\";"
        ),
        "internal",
        "and the message names the marker that decided it"
    );
}

#[test]
fn keeps_a_package_entry_point() {
    assert!(reported("src/app.ts", "import { p } from \"some-lib\";").is_empty());
    assert!(reported("src/app.ts", "import { p } from \"some-lib/parser\";").is_empty());
    assert!(reported("src/app.py", "from package.public import ok").is_empty());
    assert!(reported("src/app.py", "import os.path").is_empty());
}

#[test]
fn keeps_a_marker_that_is_the_first_segment() {
    assert!(
        reported("src/app.ts", "import { a } from \"src/utils\";").is_empty(),
        "a bare src is this project's own path alias, not a reach into a package"
    );
    assert!(
        reported("src/app.py", "from __future__ import annotations").is_empty(),
        "a leading underscore names the module itself"
    );
    assert!(
        reported("src/app.ts", "import { t } from \"internal-tool\";").is_empty(),
        "a package whose name merely begins with internal is a package"
    );
}

#[test]
fn keeps_a_scoped_packages_own_name() {
    assert!(
        reported("src/app.ts", "import x from \"@scope/internal\";").is_empty(),
        "a scoped package name spans two segments, so its second one is still the package"
    );
    for marker in ["src", "dist", "impl", "private"] {
        let source = format!("import x from \"@scope/{marker}\";");

        assert!(reported("src/app.ts", &source).is_empty(), "{source}");
    }
    assert_eq!(
        reported("src/app.ts", "import x from \"@scope/pkg/src/deep\";").len(),
        1,
        "reaching past a scoped package is still reaching past it"
    );
}

#[test]
fn keeps_a_python_language_protocol_module() {
    assert!(
        reported("src/app.py", "import package.__main__").is_empty(),
        "__main__ is the interpreter's entry-point protocol, not an author's private module"
    );
    assert!(reported("src/app.py", "from package.__init__ import x").is_empty());
    assert_eq!(
        reported("src/app.py", "from package._private.helpers import munge").len(),
        1,
        "a single leading underscore is still the author saying keep out"
    );
    assert_eq!(
        reported("src/app.py", "from package.__private import x").len(),
        1,
        "a leading double underscore without a trailing one is name mangling, not a protocol"
    );
    assert_eq!(
        reported("src/app.py", "from package._helpers__ import x").len(),
        1,
        "and a trailing double underscore alone is not one either"
    );
}

#[test]
fn keeps_a_relative_import_into_your_own_private_module() {
    assert!(
        reported("src/app.ts", "import { h } from \"./internal/helper\";").is_empty(),
        "your own internals are yours to reach into"
    );
    assert!(reported("src/app.ts", "import { h } from \"../src/helper\";").is_empty());
    assert!(reported("src/app.py", "from ._private import local").is_empty());
    assert!(reported("src/app.ts", "import { h } from \"#internal/helper\";").is_empty());
    assert!(
        reported("src/app.ts", "import { h } from \"/internal/helper\";").is_empty(),
        "a rooted path names this project, not a package"
    );
}

#[test]
fn leaves_rust_alone() {
    assert!(
        reported("src/app.rs", "use other_crate::internal::thing;").is_empty(),
        "Rust module privacy is enforced by the compiler, so a module you can import is public"
    );
    assert!(reported("src/app.rs", "use crate::internal::other;").is_empty());
}

#[test]
fn permits_a_module_the_project_has_allowed() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/no-internal-import:\n",
        "    severity: error\n",
        "    allow:\n",
        "      - vendor-lib/**\n"
    );

    assert!(
        violations(
            "src/app.ts",
            "import { p } from \"vendor-lib/dist/patched\";",
            configuration
        )
        .is_empty()
    );
    assert_eq!(
        violations(
            "src/app.ts",
            "import { p } from \"other-lib/dist/patched\";",
            configuration
        )
        .len(),
        1
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration =
        "version: 1\nrules:\n  architecture/no-internal-import:\n    severity: off\n";

    assert!(
        violations(
            "src/app.py",
            "from package._private.helpers import munge",
            configuration
        )
        .is_empty()
    );
}
