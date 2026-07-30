#!/usr/bin/env python3
"""Run Godlint over pinned real repositories and require that it can still read them.

The gate is unreadable files, never findings. Findings change whenever a rule changes, so gating
on them would fail on every rule this repository ships and would be switched off within a week.
A file Godlint cannot read is a defect whatever the rules say: it contributes nothing, and the
loss leaves no trace in a findings count.

Each repository carries a budget rather than a list of paths, because one of them is at four
hundred and enumerating those would bury the reason under the data. The budget fails in both
directions, like the rule-coverage one: over it is a regression, and under it means a grammar
learned the syntax and the budget is now reserving silence for the next failure.

Repositories are pinned to a commit so a budget describes a fixed tree. Bumping a pin and
bumping a budget are the same review conversation.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

MANIFEST = ROOT / "corpus" / "repositories.json"

CONFIGURATION = "version: 1\n\nsuites:\n  - recommended@1\n"


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: check-real-world.py <godlint binary> [name ...]", file=sys.stderr)

        return 2

    binary = Path(sys.argv[1]).resolve()
    wanted = set(sys.argv[2:])
    repositories = json.loads(MANIFEST.read_text(encoding="utf-8"))["repositories"]
    selected = [one for one in repositories if not wanted or one["name"] in wanted]
    failures: list[str] = []

    for repository in selected:
        failures.extend(check(binary, repository))

    return report(failures, len(selected))


def report(failures: list[str], selected: int) -> int:
    if not failures:
        print(f"\n{selected} repositories are within their unreadable-file budgets")

        return 0

    print("")

    for failure in failures:
        print(f"  - {failure}")

    print(
        f"\n{len(failures)} budgets disagree with reality. Either Godlint stopped reading code "
        f"that ships, or {MANIFEST.name} needs the new number with the reason beside it.",
    )

    return 1


def check(binary: Path, repository: dict) -> list[str]:
    name = repository["name"]

    with tempfile.TemporaryDirectory() as directory:
        tree = Path(directory) / name

        clone(repository, tree)
        (tree / "godlint.yaml").write_text(CONFIGURATION, encoding="utf-8")
        report = scan(binary, tree)

    return compared(repository, report)


def clone(repository: dict, tree: Path) -> None:
    tree.mkdir(parents=True)
    run(["git", "init", "--quiet"], tree)
    run(["git", "remote", "add", "origin", repository["url"]], tree)
    run(["git", "fetch", "--quiet", "--depth", "1", "origin", repository["commit"]], tree)
    run(["git", "checkout", "--quiet", "FETCH_HEAD"], tree)


def run(command: list[str], tree: Path) -> None:
    finished = subprocess.run(command, cwd=tree, capture_output=True, text=True, check=False)

    if finished.returncode != 0:
        raise SystemExit(f"{' '.join(command)} failed in {tree}: {finished.stderr.strip()}")


def scan(binary: Path, tree: Path) -> dict:
    finished = subprocess.run(
        [str(binary), "check", "--format", "json", "."],
        cwd=tree,
        capture_output=True,
        text=True,
        check=False,
    )

    if not finished.stdout:
        raise SystemExit(f"godlint reported nothing for {tree}: {finished.stderr.strip()}")

    return json.loads(finished.stdout)


def compared(repository: dict, report: dict) -> list[str]:
    name = repository["name"]
    budget = repository["unreadable-budget"]
    issues = report["issues"]

    print(f"{name}: {len(report['findings'])} findings, {len(issues)} unreadable, budget {budget}")

    for cause, count in causes(issues).most_common(3):
        print(f"    {count:>4}  {cause}")

    if len(issues) > budget:
        return [
            f"{name}: {len(issues)} unreadable files against a budget of {budget}. "
            f"{worst(issues)}",
        ]

    if len(issues) < budget:
        return [
            f"{name}: only {len(issues)} unreadable files against a budget of {budget}; "
            f"lower the budget so it keeps its grip",
        ]

    return []


def causes(issues: list[dict]) -> Counter:
    return Counter(re.sub(r"\d+", "N", issue["message"]).split(";")[0] for issue in issues)


def worst(issues: list[dict]) -> str:
    first = min(issues, key=lambda issue: issue["path"])

    return f"For example {first['path']}: {first['message']}"


if __name__ == "__main__":
    sys.exit(main())
