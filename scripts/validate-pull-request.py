#!/usr/bin/env python3
"""Check the invariants a pull request template asks a contributor to confirm.

A checklist records an intention; this records a fact. Every check here is one a
reviewer would otherwise have to perform by hand, and each failure names the file to
edit rather than the box to tick.

Run with no arguments to check the working tree. Pass --release-line <ref> to additionally
apply the checks that depend on what a change touched.

Those checks measure the branch against the release line rather than against whichever
branch a pull request targets, because a changelog entry describes a change as a release
will present it. A pull request stacked on another carries the entry in the branch below
it, and asking about the target would demand a second entry for one change.
"""

from __future__ import annotations

import argparse
import functools
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

RULES_DIR = Path("crates/godlint-core/src/rules")
RULE_TESTS_DIR = Path("crates/godlint-core/tests/rules")
RULE_TESTS_INDEX = Path("crates/godlint-core/tests/rules.rs")
REGISTRY = RULES_DIR / "mod.rs"
REGISTRATIONS = RULES_DIR / "registry.rs"
FIXTURES_DIR = Path("crates/godlint-cli/tests/fixtures/rules")
E2E = Path("crates/godlint-cli/tests/e2e.rs")
CONFIG = Path("crates/godlint-core/src/config/mod.rs")
DOGFOOD = Path("godlint.yaml")
WORKFLOWS = Path(".github/workflows")
MUTANTS = Path(".cargo/mutants.toml")

ROADMAP = Path("docs/rule-roadmap.md")
RULES = Path("docs/rules.md")
CHANGELOG = Path("CHANGELOG.md")

# Behaviour a user can observe. Scan and path discovery belong here because a change to
# either can stop reporting files that were reported before, which is exactly the kind of
# change a reader of the changelog is looking for.
NEEDS_CHANGELOG = (
    f"{RULES_DIR}/",
    "crates/godlint-core/src/analyzers/",
    "crates/godlint-core/src/discovery.rs",
    "crates/godlint-core/src/paths.rs",
    str(CONFIG),
)


@dataclass
class Report:
    failures: list[str] = field(default_factory=list)
    checked: int = 0

    def check(self, condition: bool, message: str) -> bool:
        self.checked += 1
        if not condition:
            self.failures.append(message)
        return condition


@functools.cache
def read(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.exists() else ""


def rule_id(module: Path) -> str | None:
    match = re.search(r'const ID: &\'static str = "([^"]+)"', read(module))
    return match.group(1) if match else None


def struct_name(module: Path) -> str:
    match = re.search(r"pub struct (\w+);", read(module))
    return match.group(1) if match else ""


def rule_modules() -> list[Path]:
    return sorted(p for p in RULES_DIR.glob("*.rs") if rule_id(p) is not None)


def check_rule(report: Report, module: Path, identifier: str) -> None:
    name = module.stem
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

    # The registry is what a suppression directive is validated against, and what tells
    # policy/unused-suppression whether a named rule is enabled. A rule missing from it is
    # reported as a typo when a directive names it, and its suppressions never count as
    # used, so both failures are silent.
    report.check(
        f'id: {struct_name(module)}::ID' in read(REGISTRATIONS),
        f"{REGISTRATIONS}: {name} is not registered, so a suppression cannot name {identifier}",
    )

    fixture = FIXTURES_DIR / slug
    report.check(
        fixture.is_dir(),
        f"{fixture}: a fixture directory is required",
    )

    unit_tests = RULE_TESTS_DIR / f"{name}.rs"
    report.check(unit_tests.exists(), f"{unit_tests}: unit tests are required")
    report.check(
        f'"rules/{name}.rs"' in read(RULE_TESTS_INDEX),
        f"{RULE_TESTS_INDEX}: `rules/{name}.rs` is not declared, so its tests never run",
    )

    for document in (ROADMAP, RULES, CHANGELOG):
        report.check(
            identifier in read(document),
            f"{document}: does not mention {identifier}",
        )

    # A repository adopts rules by naming them or by naming a suite. Godlint names a suite,
    # and which rules a suite sets is a question about configuration rather than about
    # source text: recommended_enables_every_rule_at_error in tests/suites.rs answers it
    # from the expanded configuration, where it cannot drift. Reading the suite's Rust for
    # a call spelling would fail on any refactor of it that changed no behaviour.
    report.check(
        identifier in read(DOGFOOD) or "suites:" in read(DOGFOOD),
        f"{DOGFOOD}: {identifier} is enabled by neither a rules entry nor an adopted suite, "
        "so Godlint does not dogfood it",
    )

    check_rule_coverage(report, identifier)


def fixture_directories() -> list[Path]:
    return sorted(path for path in FIXTURES_DIR.iterdir() if path.is_dir())


def check_rule_coverage(report: Report, identifier: str) -> None:
    reported = f"[{identifier}]"
    fires = False
    stays_silent = False

    for fixture in fixture_directories():
        expected = read(fixture / "expected.yaml")
        configured = identifier in read(fixture / "godlint.yaml")

        fires = fires or reported in expected
        stays_silent = stays_silent or (configured and reported not in expected)

    report.check(
        fires,
        f"{FIXTURES_DIR}: no fixture reports {identifier}, so nothing proves it fires",
    )
    report.check(
        stays_silent,
        f"{FIXTURES_DIR}: no fixture configures {identifier} without reporting it, "
        "so nothing proves it stays silent on conforming code",
    )


def check_fixtures(report: Report) -> None:
    for fixture in fixture_directories():
        report.check(
            (fixture / "godlint.yaml").exists(),
            f"{fixture}: godlint.yaml is missing, so the fixture inherits the root config",
        )


def check_mutation_config(report: Report) -> None:
    body = read(MUTANTS)

    report.check(
        "test_workspace = true" in body,
        f"{MUTANTS}: must set test_workspace, or the fixture corpus does not decide "
        "whether a mutant was caught",
    )

    exclusions = [
        line.strip()
        for line in body.splitlines()
        if line.strip().startswith('"') and line.strip().endswith('",')
    ]
    commented = body.count("    #")

    report.check(
        commented >= len(exclusions),
        f"{MUTANTS}: every exclusion needs a stated reason; found {len(exclusions)} "
        f"exclusions and {commented} comment lines beside them",
    )


def check_workflows(report: Report) -> None:
    versions: set[str] = set()

    for workflow in sorted(WORKFLOWS.glob("*.yml")):
        body = read(workflow)
        report.check(
            "permissions:" in body,
            f"{workflow}: declares no `permissions`",
        )
        versions.update(re.findall(r"rustup toolchain install (\S+)", body))

    report.check(
        len(versions) <= 1,
        f"workflows pin different Rust toolchains: {sorted(versions)}",
    )


def heading_slugs(document: Path) -> set[str]:
    headings = re.findall(r"^#{1,6} (.+)$", read(document), re.MULTILINE)
    return {
        re.sub(r"[^a-z0-9 -]", "", heading.lower()).replace(" ", "-")
        for heading in headings
    }


def check_documentation_links(report: Report) -> None:
    """A moved document leaves the links to it pointing nowhere, and nothing else notices.

    Only relative links are followed. A broken external URL is a fact about the internet
    rather than about this commit, and a check that needs the network cannot gate a merge.
    """

    documents = sorted(
        path
        for path in Path(".").rglob("*.md")
        if not any(part in {".git", "target", "node_modules"} for part in path.parts)
    )

    for document in documents:
        for target, fragment in re.findall(
            r"\[[^\]]*\]\(([^)#\s]+)(#[^)\s]*)?\)", read(document)
        ):
            if target.startswith(("http://", "https://", "mailto:")):
                continue

            destination = (document.parent / target).resolve()

            if not report.check(
                destination.exists(),
                f"{document}: links to {target}, which does not exist",
            ):
                continue

            if fragment and destination.suffix == ".md":
                report.check(
                    fragment[1:] in heading_slugs(destination),
                    f"{document}: links to {target}{fragment}, and that heading is gone",
                )


def changed_files(base: str) -> list[str]:
    diff = subprocess.run(
        ["git", "diff", "--name-only", f"{base}...HEAD"],
        capture_output=True,
        text=True,
        check=False,
    )

    if diff.returncode != 0:
        raise SystemExit(
            f"cannot diff against {base}: {diff.stderr.strip()}\n"
            "The change-scoped checks did not run, which is a failure and not a pass."
        )

    return [line for line in diff.stdout.splitlines() if line]


def check_change(report: Report, release_line: str) -> None:
    changed = changed_files(release_line)

    if not changed:
        return

    report.check(
        not any(path.startswith(NEEDS_CHANGELOG) for path in changed)
        or str(CHANGELOG) in changed,
        f"{CHANGELOG}: observable behaviour — a rule, the configuration schema, or which files are scanned — changed without an entry",
    )



def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--release-line",
        help="git ref for the release line this change will land on, such as origin/main",
    )
    arguments = parser.parse_args()

    report = Report()

    for module in rule_modules():
        identifier = rule_id(module)
        if identifier is not None:
            check_rule(report, module, identifier)

    check_fixtures(report)
    check_mutation_config(report)
    check_workflows(report)
    check_documentation_links(report)

    if arguments.release_line:
        check_change(report, arguments.release_line)

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
