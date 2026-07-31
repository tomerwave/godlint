use godlint_core::rules::{Violation, no_test_helper_in_production};

use super::support::rule_violations;

const ENABLED: &str =
    "version: 1\nrules:\n  testing/no-test-helper-in-production:\n    severity: error\n";

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(
        no_test_helper_in_production::evaluate,
        path,
        source,
        configuration,
    )
}

fn reported(path: &str, source: &str) -> Vec<Violation> {
    violations(path, source, ENABLED)
}

#[test]
fn reports_a_test_tree_import_from_production_in_each_language() {
    let cases = [
        (
            "src/refund.ts",
            "import { fake } from \"../../tests/helpers/gateway\";",
        ),
        ("src/refund.js", "import { fake } from \"./__mocks__/api\";"),
        ("src/refund.py", "from ..tests.helpers import fake"),
        ("src/refund.py", "from .conftest import fixture"),
        ("src/refund.rs", "use crate::tests::helpers::fake;"),
        ("src/refund.rs", "use super::fixtures::order;"),
    ];

    for (path, source) in cases {
        assert_eq!(
            reported(path, source).len(),
            1,
            "production reaching into the test tree is the finding: {path} {source}"
        );
    }
}

#[test]
fn names_the_module_and_the_segment_that_decided_it() {
    let reported = reported(
        "src/refund.ts",
        "import { fake } from \"../../tests/helpers/gateway\";",
    );
    let message = reported.first().expect("reports the import").to_string();

    assert!(
        message.starts_with("../../tests/helpers/gateway names tests,"),
        "the message must name both the module and the segment: {message}"
    );
    assert!(
        message.contains("interface"),
        "the message must name the fix: {message}"
    );
}

#[test]
fn keeps_an_import_from_a_test_file() {
    let cases = [
        (
            "tests/refund.test.ts",
            "import { fake } from \"../helpers/gateway\";",
        ),
        (
            "src/__tests__/refund.ts",
            "import { fake } from \"../fixtures/order\";",
        ),
        ("tests/test_refund.py", "from .conftest import fixture"),
        (
            "src/refund.spec.ts",
            "import { fake } from \"../tests/helpers\";",
        ),
    ];

    for (path, source) in cases {
        assert!(
            reported(path, source).is_empty(),
            "a test may use its own helpers: {path} {source}"
        );
    }
}

#[test]
fn keeps_a_production_import_that_names_no_test_tree() {
    assert!(reported("src/refund.ts", "import { g } from \"./gateway\";").is_empty());
    assert!(reported("src/refund.py", "from .gateway import settle").is_empty());
    assert!(reported("src/refund.py", "import os.path").is_empty());
    assert!(reported("src/refund.rs", "use crate::gateway::settle;").is_empty());
}

#[test]
fn keeps_a_third_party_package_that_happens_to_ship_tests() {
    assert!(
        reported(
            "src/refund.ts",
            "import { u } from \"some-lib/tests/util\";"
        )
        .is_empty(),
        "only a local import reaches into this repository's own test tree"
    );
    assert!(reported("src/refund.py", "from testing.helpers import fake").is_empty());
    assert!(
        reported("src/refund.rs", "use other_crate::tests::fake;").is_empty(),
        "another crate's modules are its own business"
    );
}

#[test]
fn matches_a_segment_whatever_its_case() {
    assert_eq!(
        reported("src/refund.ts", "import { f } from \"../Tests/helpers\";").len(),
        1
    );
}

#[test]
fn keeps_a_segment_that_merely_contains_a_helper_name() {
    assert!(
        reported(
            "src/refund.ts",
            "import { f } from \"./testing-utils/fake\";"
        )
        .is_empty(),
        "a segment is matched whole, so testing-utils is not tests"
    );
    assert!(reported("src/refund.ts", "import { f } from \"./contest/rules\";").is_empty());
}

#[test]
fn reads_the_configured_test_paths_and_helpers() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  testing/no-test-helper-in-production:\n",
        "    severity: error\n",
        "    test-paths:\n",
        "      - spec/**\n",
        "    helpers:\n",
        "      - doubles\n"
    );

    assert_eq!(
        violations(
            "src/refund.ts",
            "import { f } from \"./doubles/gateway\";",
            configuration
        )
        .len(),
        1,
        "a repository names its own scaffolding directory"
    );
    assert!(
        violations(
            "spec/refund.ts",
            "import { f } from \"./doubles/gateway\";",
            configuration
        )
        .is_empty(),
        "and its own test paths"
    );
    assert!(
        violations(
            "src/refund.ts",
            "import { f } from \"../../tests/helpers\";",
            configuration
        )
        .is_empty(),
        "replacing the defaults replaces them, rather than adding to them"
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration =
        "version: 1\nrules:\n  testing/no-test-helper-in-production:\n    severity: off\n";

    assert!(
        violations(
            "src/refund.ts",
            "import { f } from \"../../tests/helpers\";",
            configuration
        )
        .is_empty()
    );
}
