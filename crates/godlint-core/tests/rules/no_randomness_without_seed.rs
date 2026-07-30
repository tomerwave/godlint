use godlint_core::rules::{Violation, no_randomness_without_seed};

use super::support::rule_violations;

const ENABLED: &str =
    "version: 1\nrules:\n  testing/no-randomness-without-seed:\n    severity: error\n";

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(
        no_randomness_without_seed::evaluate,
        path,
        source,
        configuration,
    )
}

fn reported(path: &str, source: &str) -> Vec<Violation> {
    violations(path, source, ENABLED)
}

#[test]
fn reports_an_unseeded_generator_in_each_language() {
    let cases = [
        (
            "tests/test_basket.py",
            "def test_price():\n    items = random.sample(pool, 3)\n",
        ),
        (
            "tests/basket.spec.js",
            "it('prices', () => {\n  const n = Math.random();\n});\n",
        ),
        (
            "tests/basket.spec.ts",
            "it('prices', () => {\n  const n = Math.random();\n});\n",
        ),
        (
            "tests/basket.rs",
            "#[test]\nfn prices() {\n    let n: usize = rand::random();\n}\n",
        ),
        (
            "tests/basket.rs",
            "#[test]\nfn prices() {\n    let mut rng = rand::thread_rng();\n}\n",
        ),
    ];

    for (path, source) in cases {
        assert_eq!(
            reported(path, source).len(),
            1,
            "unseeded randomness in a test is the finding: {path} {source}"
        );
    }
}

#[test]
fn names_the_generator_and_the_fix() {
    let violations = reported(
        "tests/test_basket.py",
        "def test_price():\n    items = random.sample(pool, 3)\n",
    );
    let message = violations
        .first()
        .expect("reports the generator")
        .to_string();

    assert!(
        message.starts_with("random.sample "),
        "the message must name the generator: {message}"
    );
    assert!(
        message.contains("reproduced") && message.contains("seed"),
        "the message must say why it matters and what to do: {message}"
    );
}

#[test]
fn keeps_a_file_that_seeds_its_generator() {
    assert!(
        reported(
            "tests/test_basket.py",
            "def test_price():\n    random.seed(1)\n    items = random.sample(pool, 3)\n"
        )
        .is_empty()
    );
    assert!(
        reported(
            "tests/test_basket.py",
            "def test_price():\n    rng = random.Random(1)\n    items = random.sample(pool, 3)\n"
        )
        .is_empty()
    );
    assert!(
        reported(
            "tests/basket.spec.js",
            "seedrandom('fixed');\nit('prices', () => {\n  const n = Math.random();\n});\n"
        )
        .is_empty()
    );
}

#[test]
fn seeds_the_whole_file_rather_than_one_test() {
    assert!(
        reported(
            "tests/test_basket.py",
            concat!(
                "def test_seeded():\n    random.seed(1)\n\n",
                "def test_unseeded():\n    items = random.sample(pool, 3)\n"
            )
        )
        .is_empty(),
        "a file-wide exemption under-reports rather than over-reports, which is the safe direction"
    );
}

#[test]
fn keeps_randomness_outside_a_test() {
    assert!(
        reported(
            "tests/test_basket.py",
            "def sample_pool(pool):\n    return random.sample(pool, 3)\n"
        )
        .is_empty()
    );
}

#[test]
fn binds_a_generator_to_the_language_that_spells_it() {
    assert!(
        reported(
            "tests/test_basket.py",
            "def test_price():\n    n = Math.random()\n"
        )
        .is_empty()
    );
    assert!(
        reported(
            "tests/basket.spec.js",
            "it('prices', () => {\n  random.sample(pool, 3);\n});\n"
        )
        .is_empty()
    );
}

#[test]
fn keeps_a_seed_spelled_for_another_language() {
    assert_eq!(
        reported(
            "tests/basket.spec.js",
            "random.seed(1);\nit('prices', () => {\n  const n = Math.random();\n});\n"
        )
        .len(),
        1,
        "a Python seed call does not seed a JavaScript generator"
    );
}

#[test]
fn permits_a_generator_inside_an_approved_path() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  testing/no-randomness-without-seed:\n",
        "    severity: error\n",
        "    allow-in:\n",
        "      - tests/property/**\n"
    );

    assert!(
        violations(
            "tests/property/test_basket.py",
            "def test_price():\n    items = random.sample(pool, 3)\n",
            configuration,
        )
        .is_empty(),
        "a property-based suite generates randomness deliberately and reports its own seed"
    );
    assert_eq!(
        violations(
            "tests/unit/test_basket.py",
            "def test_price():\n    items = random.sample(pool, 3)\n",
            configuration,
        )
        .len(),
        1
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration =
        "version: 1\nrules:\n  testing/no-randomness-without-seed:\n    severity: off\n";

    assert!(
        violations(
            "tests/test_basket.py",
            "def test_price():\n    items = random.sample(pool, 3)\n",
            configuration,
        )
        .is_empty()
    );
}
