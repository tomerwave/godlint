#!/usr/bin/env python3
"""Check the invariants a pull request template asks a contributor to confirm.

A checklist records an intention; this records a fact. Every check here is one a
reviewer would otherwise have to perform by hand, and each failure names the file to
edit rather than the box to tick.

Run with no arguments to check the working tree. Pass --base <ref> to additionally
apply the checks that depend on what a change touched.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

RULES_DIR = Path("crates/godlint-core/src/rules")
RULE_TESTS_DIR = Path("crates/godlint-core/tests/rules")
RULE_TESTS_INDEX = Path("crates/godlint-core/tests/rules.rs")
REGISTRY = RULES_DIR / "mod.rs"
FIXTURES_DIR = Path("crates/godlint-cli/tests/fixtures/rules")
E2E = Path("crates/godlint-cli/tests/e2e.rs")
CONFIG = Path("crates/godlint-core/src/config.rs")
DOGFOOD = Path("godlint.yaml")
WORKFLOWS = Path(".github/workflows")

ROADMAP = Path("docs/rule-roadmap.md")
README = Path("README.md")
CHANGELOG = Path("CHANGELOG.md")

NOT_A_RULE = {"mod.rs", "line_count.rs"}

BEHAVIOUR_PATHS = ("crates/godlint-core/src/rules/", "crates/godlint-core/src/analyzers/")
SCHEMA_PATHS = ("crates/godlint-core/src/config.rs",)


@dataclass
class Report:
    failures: list[str] = field(default_factory=list)
    checked: int = 0

    def check(self, condition: bool, message: str) -> None:
        self.checked += 1
        if not condition:
            self.failures.append(message)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.exists() else ""


def rule_modules() -> list[Path]:
    return sorted(p for p in RULES_DIR.glob("*.rs") if p.name not in NOT_A_RULE)


def rule_id(module: Path) -> str | None:
    match = re.search(r'const ID: &\'static str = "([^"]+)"', read(module))
    return match.group(1) if match else None


def check_rule(report: Report, module: Path) -> None:
    name = module.stem
    identifier = rule_id(module)

    report.check(
        identifier is not None,
        f"{module}: no `const ID` found, so the rule has no stable identifier",
    )
    if identifier is None:
        return

    slug = identifier.split("/", 1)[-1]
    registry = read(REGISTRY)

    report.check(
        f"pub mod {name};" in registry,
        f"{REGISTRY}: `pub mod {name};` is missing",
    )
    report.check(
        f"{name}::evaluate" in registry,
        f"{REGISTRY}: `{name}::evaluate` is not in EVALUATORS, so the rule never runs",
    )
    report.check(
        f'rename = "{identifier}"' in read(CONFIG),
        f'{CONFIG}: no field renamed "{identifier}", so the rule cannot be configured',
    )

    fixture = FIXTURES_DIR / slug
    report.check(
        (fixture / "godlint.yaml").exists() and (fixture / "expected.yaml").exists(),
        f"{fixture}: a fixture with godlint.yaml and expected.yaml is required",
    )
    report.check(
        f'"{slug}"' in read(E2E),
        f"{E2E}: fixture {slug!r} is not declared, so no test runs it",
    )

    unit_tests = RULE_TESTS_DIR / f"{name}.rs"
    report.check(unit_tests.exists(), f"{unit_tests}: unit tests are required")
    report.check(
        f'"rules/{name}.rs"' in read(RULE_TESTS_INDEX),
        f"{RULE_TESTS_INDEX}: `rules/{name}.rs` is not declared, so its tests never run",
    )

    for document in (ROADMAP, README, CHANGELOG):
        report.check(
            identifier in read(document),
            f"{document}: does not mention {identifier}",
        )

    report.check(
        identifier in read(DOGFOOD),
        f"{DOGFOOD}: {identifier} is not enabled, so Godlint does not dogfood it",
    )


def check_fixtures(report: Report) -> None:
    declared = read(E2E)

    for fixture in sorted(p for p in FIXTURES_DIR.iterdir() if p.is_dir()):
        report.check(
            (fixture / "godlint.yaml").exists(),
            f"{fixture}: godlint.yaml is missing",
        )
        expected = fixture / "expected.yaml"
        report.check(expected.exists(), f"{expected}: is missing")
        report.check(
            "exit-code:" in read(expected),
            f"{expected}: no exit-code, so the fixture asserts nothing about failure",
        )
        report.check(
            f'"{fixture.name}"' in declared,
            f"{E2E}: fixture {fixture.name!r} is not declared, so no test runs it",
        )


def check_workflows(report: Report) -> None:
    toolchains: dict[str, set[str]] = {}

    for workflow in sorted(WORKFLOWS.glob("*.yml")):
        body = read(workflow)
        report.check(
            "permissions:" in body,
            f"{workflow}: declares no `permissions`",
        )
        found = set(re.findall(r"rustup toolchain install (\S+)", body))
        if found:
            toolchains[str(workflow)] = found

    versions = {version for found in toolchains.values() for version in found}
    report.check(
        len(versions) <= 1,
        f"workflows pin different Rust toolchains: {sorted(versions)}",
    )


def changed_files(base: str) -> list[str]:
    try:
        diff = subprocess.run(
            ["git", "diff", "--name-only", f"{base}...HEAD"],
            capture_output=True,
            text=True,
            check=True,
        )
    except subprocess.CalledProcessError as error:
        print(f"unable to diff against {base}: {error.stderr.strip()}", file=sys.stderr)
        return []

    return [line for line in diff.stdout.splitlines() if line]


def check_change(report: Report, base: str) -> None:
    changed = changed_files(base)
    if not changed:
        return

    touches_behaviour = any(
        path.startswith(BEHAVIOUR_PATHS) or path.startswith(SCHEMA_PATHS)
        for path in changed
    )
    report.check(
        not touches_behaviour or "CHANGELOG.md" in changed,
        "CHANGELOG.md: rule behaviour or configuration schema changed without an entry",
    )

    new_rules = [
        path
        for path in changed
        if path.startswith(str(RULES_DIR)) and Path(path).name not in NOT_A_RULE
    ]
    report.check(
        not new_rules or any(path.startswith(str(FIXTURES_DIR)) for path in changed),
        f"{FIXTURES_DIR}: rule modules changed without touching any fixture",
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", help="git ref to diff against for change-scoped checks")
    arguments = parser.parse_args()

    report = Report()

    for module in rule_modules():
        check_rule(report, module)

    check_fixtures(report)
    check_workflows(report)

    if arguments.base:
        check_change(report, arguments.base)

    if report.failures:
        print(f"{len(report.failures)} of {report.checked} checks failed:\n")
        for failure in report.failures:
            print(f"  - {failure}")
        print("\nSee .github/PULL_REQUEST_TEMPLATE/ for what each check is asking for.")
        return 1

    print(f"all {report.checked} checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
