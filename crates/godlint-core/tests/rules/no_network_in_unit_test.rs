use godlint_core::rules::{Violation, no_network_in_unit_test};

use super::support::rule_violations;

const SCOPED: &str = concat!(
    "version: 1\n",
    "rules:\n",
    "  testing/no-network-in-unit-test:\n",
    "    severity: error\n",
    "    unit-paths:\n",
    "      - tests/unit/**\n"
);

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(
        no_network_in_unit_test::evaluate,
        path,
        source,
        configuration,
    )
}

fn scoped(path: &str, source: &str) -> Vec<Violation> {
    violations(path, source, SCOPED)
}

#[test]
fn reports_a_client_call_in_each_language() {
    let cases = [
        (
            "tests/unit/test_rates.py",
            "def test_rates():\n    requests.get(url)\n",
        ),
        (
            "tests/unit/test_rates.py",
            "def test_rates():\n    urllib.request.urlopen(url)\n",
        ),
        (
            "tests/unit/rates.spec.js",
            "it('rates', async () => {\n  await fetch(url);\n});\n",
        ),
        (
            "tests/unit/rates.spec.ts",
            "it('rates', async () => {\n  await axios.get(url);\n});\n",
        ),
        (
            "tests/unit/rates.rs",
            "#[test]\nfn rates() {\n    reqwest::blocking::get(url);\n}\n",
        ),
        (
            "tests/unit/rates.rs",
            "#[test]\nfn rates() {\n    TcpStream::connect(address);\n}\n",
        ),
    ];

    for (path, source) in cases {
        assert_eq!(
            scoped(path, source).len(),
            1,
            "a client call in a declared unit test is the finding: {path} {source}"
        );
    }
}

#[test]
fn names_the_client_and_the_fix() {
    let reported = scoped(
        "tests/unit/test_rates.py",
        "def test_rates():\n    requests.get(url)\n",
    );
    let message = reported.first().expect("reports the client").to_string();

    assert!(
        message.starts_with("requests.get "),
        "the message must name the client: {message}"
    );
    assert!(
        message.contains("inject"),
        "the message must name the fix: {message}"
    );
}

#[test]
fn stays_silent_until_the_repository_declares_its_unit_paths() {
    let unscoped = "version: 1\nrules:\n  testing/no-network-in-unit-test:\n    severity: error\n";

    assert!(
        violations(
            "tests/unit/test_rates.py",
            "def test_rates():\n    requests.get(url)\n",
            unscoped,
        )
        .is_empty(),
        "which test is a unit test is a repository fact; with none declared the rule has nothing to \
         scope to"
    );
}

#[test]
fn covers_each_client_verb_its_library_offers() {
    let cases = [
        ("tests/unit/a.py", "def test_a():\n    httpx.put(url)\n"),
        ("tests/unit/a.py", "def test_a():\n    httpx.delete(url)\n"),
        ("tests/unit/a.py", "def test_a():\n    httpx.head(url)\n"),
        (
            "tests/unit/a.js",
            "it('a', () => {\n  http.get(url);\n});\n",
        ),
        (
            "tests/unit/a.rs",
            "#[test]\nfn a() {\n    ureq::post(url);\n}\n",
        ),
    ];

    for (path, source) in cases {
        assert_eq!(
            scoped(path, source).len(),
            1,
            "a catalogue that covers one verb of a library and not its sibling is a silent gap: \
             {path} {source}"
        );
    }
}

#[test]
fn permits_a_client_inside_an_exempted_path() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  testing/no-network-in-unit-test:\n",
        "    severity: error\n",
        "    unit-paths:\n",
        "      - tests/unit/**\n",
        "    allow-in:\n",
        "      - tests/unit/contract/**\n"
    );

    assert!(
        violations(
            "tests/unit/contract/test_rates.py",
            "def test_rates():\n    requests.get(url)\n",
            configuration,
        )
        .is_empty(),
        "a mocked client follows this rule's own advice, and a callee match cannot tell them apart"
    );
    assert_eq!(
        violations(
            "tests/unit/test_rates.py",
            "def test_rates():\n    requests.get(url)\n",
            configuration,
        )
        .len(),
        1
    );
}

#[test]
fn keeps_a_client_call_outside_the_declared_unit_paths() {
    assert!(
        scoped(
            "tests/integration/test_rates.py",
            "def test_rates():\n    requests.get(url)\n"
        )
        .is_empty(),
        "an integration test reaching the real service is the point of it"
    );
}

#[test]
fn keeps_a_client_call_outside_a_test() {
    assert!(
        scoped(
            "tests/unit/test_rates.py",
            "def read_rates():\n    return requests.get(url)\n"
        )
        .is_empty()
    );
}

#[test]
fn keeps_a_faked_client() {
    assert!(
        scoped(
            "tests/unit/test_rates.py",
            "def test_rates():\n    rates = Rates(client=FakeClient())\n    assert rates.of('EUR')\n"
        )
        .is_empty()
    );
}

#[test]
fn binds_a_client_to_the_language_that_spells_it() {
    assert!(
        scoped(
            "tests/unit/rates.spec.js",
            "it('rates', () => {\n  requests.get(url);\n});\n"
        )
        .is_empty()
    );
    assert!(
        scoped(
            "tests/unit/test_rates.py",
            "def test_rates():\n    fetch(url)\n"
        )
        .is_empty()
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  testing/no-network-in-unit-test:\n",
        "    severity: off\n",
        "    unit-paths:\n",
        "      - tests/unit/**\n"
    );

    assert!(
        violations(
            "tests/unit/test_rates.py",
            "def test_rates():\n    requests.get(url)\n",
            configuration,
        )
        .is_empty()
    );
}
