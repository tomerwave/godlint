#!/usr/bin/env python3
"""Check the invariants a pull request template asks a contributor to confirm.

A checklist records an intention; this records a fact. Every check here is one a
reviewer would otherwise have to perform by hand, and each failure names the file to
edit rather than the box to tick.

The checks that depend on what a change touched always run: --release-line <ref> names the
ref to compare against, and without it the script discovers one. A run that cannot find a
release line fails rather than quietly reporting fewer checks than it did in CI.

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
MUTANTS_WORKFLOW = WORKFLOWS / "mutants.yml"

CORE_SOURCE = Path("crates/godlint-core/src")

# A file here is outside the mutation gate on purpose, with the reason it is outside. The gate's
# scope was previously checked only against the workflow's trigger paths — two of our own lists,
# compared with each other and never with the tree — so a file named in neither was silently
# unmutated. Everything in this table except lib.rs meets .cargo/mutants.toml's own stated
# criterion, that a surviving mutant there hides findings rather than changing one, and #245
# carries the measured plan for bringing each one in. Adding a file to godlint-core now demands a
# decision rather than nothing.
UNMUTATED = {
    "crates/godlint-core/src/lib.rs": "declares modules and re-exports; no decision to mutate",
    "crates/godlint-core/src/glob.rs": "#245, first in: decides whether an exclude pattern matches",
    "crates/godlint-core/src/date.rs": "#245: decides whether a suppression has expired",
    "crates/godlint-core/src/paths.rs": "#245: path handling underneath discovery",
    "crates/godlint-core/src/discovery.rs": "#245: decides which files are scanned",
    "crates/godlint-core/src/scan.rs": "#245: drives the scan",
    "crates/godlint-core/src/suites.rs": "#245: decides what recommended@1 enables, and at what severity",
    "crates/godlint-core/src/source.rs": "#245: position math; larger, and more of its mutants will be equivalent",
    "crates/godlint-core/src/config/mod.rs": "#245: configuration parsing",
    "crates/godlint-core/src/config/rules.rs": "#245: configuration parsing",
    "crates/godlint-core/src/config/validate.rs": "#245: configuration validation",
    "crates/godlint-core/src/config/error.rs": "#245: shapes the message a rejected configuration prints",
}

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


DIALECTS = {
    "cjs": "JS/TS",
    "js": "JS/TS",
    "jsx": "JS/TS",
    "mjs": "JS/TS",
    "cts": "JS/TS",
    "mts": "JS/TS",
    "ts": "JS/TS",
    "tsx": "JS/TS",
    "py": "Python",
    "pyi": "Python",
    "rs": "Rust",
    "yml": "Workflow",
    "yaml": "Workflow",
}

ANALYSED = "✓"

# The columns the matrix must carry, in order, so that no other table in the document can
# be read as the matrix.
DIALECT_COLUMNS = ("JS/TS", "Python", "Rust", "Workflow")

REPORTED = re.compile(r"^\s*(\S+?):\d+:\d+: \w+\[([a-z-]+/[a-z-]+)\]")


def documented_languages() -> dict[str, dict[str, str]]:
    """Read the support matrix in docs/rules.md, keyed by rule.

    The matrix is asserted against `Rule::LANGUAGES` by
    crates/godlint-core/tests/languages.rs, so reading the document here reads the
    declarations without this script needing to parse Rust.
    """

    header = ("", "Rule", *DIALECT_COLUMNS, "")
    rows: dict[str, dict[str, str]] = {}
    found = False

    for line in read(RULES).splitlines():
        cells = tuple(cell.strip() for cell in line.split("|"))

        if not found:
            found = cells == header
            continue

        if not line.startswith("|"):
            break

        rule = cells[1].strip("`")
        if "/" in rule and len(cells) == len(header):
            rows[rule] = dict(zip(DIALECT_COLUMNS, cells[2:], strict=False))

    return rows


def reported_dialects() -> dict[str, set[str]]:
    """Which dialect each rule is proven to report in, taken from the fixture corpus."""

    evidence: dict[str, set[str]] = {}

    for fixture in fixture_directories():
        for line in read(fixture / "expected.yaml").splitlines():
            match = REPORTED.match(line)
            if match is None:
                continue

            path, identifier = match.groups()
            dialect = DIALECTS.get(path.rsplit(".", 1)[-1])
            if dialect is not None:
                evidence.setdefault(identifier, set()).add(dialect)

    return evidence


def check_language_matrix(report: Report) -> None:
    """A language a rule claims needs a fixture that reports it in that language.

    This fails in both directions, as the coverage budget does. A ✓ with no fixture
    behind it is a claim nothing has tested, and a fixture reporting a rule in a
    language the matrix marks absent means the matrix is telling a reader the rule
    does not apply to code it does in fact judge.
    """

    documented = documented_languages()
    evidence = reported_dialects()

    report.check(
        len(documented) > 0,
        f"{RULES}: no language support matrix found, so nothing records which "
        "languages a rule covers",
    )

    for rule, marks in documented.items():
        reported = evidence.get(rule, set())

        for dialect, mark in marks.items():
            if mark == ANALYSED:
                report.check(
                    dialect in reported,
                    f"{FIXTURES_DIR}: no fixture reports {rule} in {dialect}, so "
                    f"nothing proves the {ANALYSED} {RULES} claims for it",
                )
            else:
                report.check(
                    dialect not in reported,
                    f"{RULES}: marks {rule} as {mark} for {dialect}, but a fixture "
                    "reports it there",
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

    exclusions = block(body, "exclude_re")
    entries = [line for line in exclusions if line.startswith('"')]
    commented = [line for line in exclusions if line.startswith("#")]

    report.check(
        len(commented) >= len(entries),
        f"{MUTANTS}: every exclusion needs a stated reason; found {len(entries)} "
        f"exclusions and {len(commented)} comment lines beside them",
    )


def block(body: str, key: str) -> list[str]:
    """The lines inside a TOML array, stripped, without its brackets."""

    lines: list[str] = []
    inside = False

    for line in body.splitlines():
        if line.startswith(f"{key} = ["):
            inside = True
        elif inside and line.startswith("]"):
            break
        elif inside:
            lines.append(line.strip())

    return lines


def examined_globs() -> list[str]:
    return [line.strip('",') for line in block(read(MUTANTS), "examine_globs") if line.strip('",')]


def examined_paths() -> set[str]:
    """The trigger path each examined glob needs, as a workflow would spell it."""

    return {glob.replace("/**/*.rs", "/**").replace("/*.rs", "/**") for glob in examined_globs()}


def triggered_paths() -> set[str]:
    lines = block(read(MUTANTS_WORKFLOW).replace("    paths:", "paths = ["), "paths")

    return {line.lstrip("- ").strip("'\"") for line in lines if line.startswith("- ")}


def glob_matcher(pattern: str) -> re.Pattern[str]:
    """A mutants.toml glob as a regular expression, with `**/` matching zero or more directories.

    `fnmatch` is wrong for this: its `*` crosses `/`, so `rules/**/*.rs` fails to match
    `rules/mod.rs` and every rule module reads as unexamined. Getting that backwards makes a
    scope check report the opposite of the truth, so the glob is translated by hand.
    """

    expression = ""
    index = 0

    while index < len(pattern):
        if pattern.startswith("**/", index):
            expression += "(?:[^/]+/)*"
            index += 3
        elif pattern[index] == "*":
            expression += "[^/]*"
            index += 1
        else:
            expression += re.escape(pattern[index])
            index += 1

    return re.compile(f"^{expression}$")


def check_mutation_coverage(report: Report) -> None:
    """Every Rust file in godlint-core is examined by the mutation gate, or explained.

    Comparing the gate's globs with the workflow's trigger paths says nothing about a file that
    appears in neither, which is how the layer deciding what is *seen* stayed outside the gate.
    This compares the globs with the tree instead.
    """

    matchers = [glob_matcher(glob) for glob in examined_globs()]
    examined = lambda path: any(matcher.match(path) for matcher in matchers)

    for source in sorted(CORE_SOURCE.rglob("*.rs")):
        path = source.as_posix()
        report.check(
            examined(path) or path in UNMUTATED,
            f"{MUTANTS}: {path} is outside the mutation gate with no reason given; examine it, "
            "or name it in UNMUTATED in this script with why it is outside",
        )

    for path in sorted(UNMUTATED):
        report.check(
            Path(path).exists(),
            f"UNMUTATED names {path}, which no longer exists; delete the entry",
        )
        report.check(
            not examined(path),
            f"UNMUTATED names {path}, which {MUTANTS} now examines; delete the entry",
        )


def check_mutation_scope(report: Report) -> None:
    """A file the mutation config examines must trigger the pull-request mutation job.

    The two lists lived apart, and #88 is what that cost: the analysers were examined by
    the weekly sweep and triggered no pull-request run, so the layer that decides what is
    seen had no gate on the day it changed. A path in one list and not the other is now a
    failed check rather than a silence.
    """

    triggered = triggered_paths()

    # A single-level glob examines one directory and silently drops every module a split
    # moves into a subdirectory of it. That is how 477 lines of workflow reader escaped the
    # gate on the day it was written.
    for glob in examined_globs():
        report.check(
            not glob.endswith("/*.rs") or glob.endswith("/**/*.rs"),
            f"{MUTANTS}: {glob} examines one directory only; write it as **/*.rs so a module "
            "moved into a subdirectory stays mutated",
        )

    report.check(
        len(triggered) > 0,
        f"{MUTANTS_WORKFLOW}: no pull_request `paths:` found, so the job either never "
        "runs or runs on every change",
    )

    for path in sorted(examined_paths()):
        report.check(
            path in triggered,
            f"{MUTANTS_WORKFLOW}: {MUTANTS} examines {path} and no pull_request path "
            "triggers on it, so a change there is merged unmutated",
        )


def check_conflict_markers(report: Report) -> None:
    """No tracked file carries a merge-conflict marker.

    `git rebase --continue` accepts a staged file whose conflict is unresolved, so a failed
    resolution lands as a commit that looks deliberate. Nothing else here reads a file for the
    markers, and a changelog full of them passed every one of 1046 checks.
    """

    markers = ("<<<<<<< ", ">>>>>>> ")

    for path in tracked_text_files():
        lines = read(path).splitlines()
        found = [
            number
            for number, line in enumerate(lines, start=1)
            if line.startswith(markers) or line == "======="
        ]
        report.check(
            not found,
            f"{path}: unresolved conflict marker on line {found[0] if found else 0}",
        )


def tracked_text_files() -> list[Path]:
    listed = git(["ls-files"], "cannot list tracked files").splitlines()

    return [
        Path(name)
        for name in listed
        if Path(name).suffix in (".md", ".rs", ".py", ".toml", ".yml", ".yaml", ".sh", ".json")
        and Path(name).is_file()
    ]


def check_workflows(report: Report) -> None:
    """One toolchain across every workflow.

    `permissions` used to be checked here by looking for the string, which counted a match
    inside a comment or a `run:` line. `ci/explicit-workflow-permissions` replaced it: the
    rule reads the YAML, knows a declaration from a comment about one, and runs against this
    repository in CI like every other rule Godlint dogfoods.
    """

    versions: set[str] = set()

    for workflow in sorted(WORKFLOWS.glob("*.yml")):
        versions.update(re.findall(r"rustup toolchain install (\S+)", read(workflow)))

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
    committed = git(
        ["diff", "--name-only", f"{base}...HEAD"],
        f"cannot diff against {base}",
    )

    return sorted(set(committed.splitlines()) | uncommitted_files())


def uncommitted_files() -> set[str]:
    """Paths changed in the working tree, staged or not, including untracked files.

    A local run happens before the commit as often as after it, and a change the diff
    against the release line cannot see is a change these checks silently skip.
    """

    paths: set[str] = set()

    for line in git(
        ["status", "--porcelain=1", "--untracked-files=all"],
        "cannot read the working tree",
    ).splitlines():
        path = line[3:] if len(line) > 3 else ""
        _, _, renamed_to = path.partition(" -> ")
        paths.add(renamed_to or path)

    return {path for path in paths if path}


def git(arguments: list[str], failure: str) -> str:
    result = subprocess.run(
        ["git", *arguments],
        capture_output=True,
        text=True,
        check=False,
    )

    if result.returncode != 0:
        raise SystemExit(
            f"{failure}: {result.stderr.strip()}\n"
            "The change-scoped checks did not run, which is a failure and not a pass."
        )

    return result.stdout


def discovered_release_line() -> str:
    for candidate in ("origin/main", "main"):
        resolved = subprocess.run(
            ["git", "rev-parse", "--verify", "--quiet", candidate],
            capture_output=True,
            text=True,
            check=False,
        )
        if resolved.returncode == 0:
            return candidate

    raise SystemExit(
        "no release line to compare against: pass --release-line.\n"
        "The change-scoped checks did not run, which is a failure and not a pass."
    )


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
    check_language_matrix(report)
    check_mutation_config(report)
    check_mutation_coverage(report)
    check_mutation_scope(report)
    check_conflict_markers(report)
    check_workflows(report)
    check_documentation_links(report)

    check_change(report, arguments.release_line or discovered_release_line())

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
