use godlint_core::rules::{Violation, no_sleep_in_test};

use super::support::rule_violations;

const ENABLED: &str = "version: 1\nrules:\n  testing/no-sleep-in-test:\n    severity: error\n";

fn violations(path: &str, source: &str) -> Vec<Violation> {
    rule_violations(no_sleep_in_test::evaluate, path, source, ENABLED)
}

#[test]
fn reports_a_blocking_sleep_in_each_language() {
    let cases = [
        (
            "tests/test_worker.py",
            "def test_drains():\n    time.sleep(2)\n",
        ),
        (
            "tests/test_worker.py",
            "async def test_drains():\n    await asyncio.sleep(2)\n",
        ),
        (
            "tests/worker.rs",
            "#[test]\nfn drains() {\n    std::thread::sleep(delay);\n}\n",
        ),
        (
            "tests/worker.rs",
            "#[test]\nfn drains() {\n    thread::sleep(delay);\n}\n",
        ),
        (
            "tests/worker.rs",
            "#[test]\nfn drains() {\n    tokio::time::sleep(delay).await;\n}\n",
        ),
        (
            "tests/worker.spec.js",
            "it('drains', async () => {\n  await page.waitForTimeout(500);\n});\n",
        ),
        (
            "tests/worker.spec.ts",
            "it('drains', async () => {\n  await browser.pause(500);\n});\n",
        ),
        (
            "worker_test.go",
            "package worker\n\nimport \"testing\"\nimport \"time\"\n\nfunc TestDrains(t *testing.T) {\n\ttime.Sleep(time.Second)\n}\n",
        ),
    ];

    for (path, source) in cases {
        assert_eq!(
            violations(path, source).len(),
            1,
            "a sleep inside a test is the finding: {path} {source}"
        );
    }
}

#[test]
fn names_the_call_it_reports() {
    let reported = violations(
        "tests/test_worker.py",
        "def test_drains():\n    time.sleep(2)\n",
    );
    let message = reported.first().expect("reports the sleep").to_string();

    assert!(
        message.starts_with("time.sleep "),
        "the message must name the call: {message}"
    );
    assert!(
        message.contains("condition"),
        "the message must name the fix: {message}"
    );
}

#[test]
fn reads_a_sleep_however_deeply_it_is_nested_in_the_test() {
    assert_eq!(
        violations(
            "tests/worker.rs",
            "#[test]\nfn drains() {\n    for _ in 0..3 {\n        thread::sleep(delay);\n    }\n}\n"
        )
        .len(),
        1,
        "containment is what decides, so a loop body is still inside the test"
    );
    assert_eq!(
        violations(
            "tests/worker.rs",
            "#[test]\nfn drains() {\n    let f = || {\n        thread::sleep(delay);\n    };\n}\n"
        )
        .len(),
        1
    );
}

#[test]
fn reads_a_test_declared_by_another_runners_attribute() {
    assert_eq!(
        violations(
            "tests/worker.rs",
            "#[tokio::test]\nasync fn drains() {\n    tokio::time::sleep(delay).await;\n}\n"
        )
        .len(),
        1,
        "an attribute path ending in test is a test, whichever runner owns it"
    );
}

#[test]
fn keeps_a_sleep_reached_through_an_alias() {
    assert!(
        violations(
            "tests/test_worker.py",
            "from time import sleep\n\n\ndef test_drains():\n    sleep(2)\n"
        )
        .is_empty(),
        "documented gap: matching the written spelling is what stops this reporting anything named \
         sleep, and closing it takes import resolution"
    );
}

#[test]
fn reads_a_timer_wrapped_in_a_promise() {
    assert_eq!(
        violations(
            "tests/worker.spec.js",
            "it('drains', async () => {\n  await new Promise((r) => setTimeout(r, 500));\n});\n"
        )
        .len(),
        1,
        "this is JavaScript's commonest test sleep, and no name match reaches it"
    );
    assert_eq!(
        violations(
            "tests/worker.spec.ts",
            "it('drains', async () => {\n  await new Promise((r) => setInterval(r, 500));\n});\n"
        )
        .len(),
        1
    );
}

#[test]
fn keeps_a_timer_that_is_not_the_promise() {
    assert!(
        violations(
            "tests/worker.spec.js",
            "it('debounce', () => {\n  jest.useFakeTimers();\n  setTimeout(fn, 100);\n});\n"
        )
        .is_empty(),
        "a bare timer under fake timers waits on nothing; the promise wrapper is what makes a sleep"
    );
    assert!(
        violations(
            "tests/worker.spec.js",
            concat!(
                "it('guards', async () => {\n",
                "  await new Promise((resolve, reject) => {\n",
                "    server.on('ready', resolve);\n",
                "    setTimeout(() => reject(new Error('timeout')), 5000);\n",
                "  });\n});\n"
            )
        )
        .is_empty(),
        "a timer guarding an event wait is the fix this rule asks for, not the defect"
    );
    assert!(
        violations(
            "tests/worker.spec.js",
            "const sleep = (ms) => new Promise((r) => setTimeout(r, ms));\n"
        )
        .is_empty(),
        "a sleep helper outside a test is still outside a test"
    );
}

#[test]
fn keeps_a_sleep_outside_a_test() {
    assert!(
        violations(
            "tests/test_worker.py",
            "def wait_for_shutdown():\n    time.sleep(5)\n"
        )
        .is_empty(),
        "a helper is free to sleep; only a test's own timing dependency is the smell"
    );
    assert!(
        violations(
            "src/worker.rs",
            "fn drain() {\n    thread::sleep(delay);\n}\n"
        )
        .is_empty()
    );
}

#[test]
fn keeps_a_test_that_waits_on_a_condition() {
    assert!(
        violations(
            "tests/test_worker.py",
            "def test_drains():\n    assert eventually(queue_is_empty)\n"
        )
        .is_empty()
    );
}

#[test]
fn binds_a_sleep_to_the_language_that_spells_it() {
    assert!(
        violations(
            "tests/worker.spec.js",
            "it('drains', () => {\n  time.sleep(2);\n});\n"
        )
        .is_empty()
    );
    assert!(
        violations(
            "tests/test_worker.py",
            "def test_drains():\n    page.waitForTimeout(500)\n"
        )
        .is_empty()
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration = "version: 1\nrules:\n  testing/no-sleep-in-test:\n    severity: off\n";

    assert!(
        rule_violations(
            no_sleep_in_test::evaluate,
            "tests/test_worker.py",
            "def test_drains():\n    time.sleep(2)\n",
            configuration,
        )
        .is_empty()
    );
}
