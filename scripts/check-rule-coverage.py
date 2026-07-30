#!/usr/bin/env python3
"""Require every line of the rules layer to be executed by a test.

A rule whose decisions no test reaches can be changed without anything noticing, and
mutation testing does not close that hole: altering an unexercised line often breaks
behaviour that other tests do cover, which marks the mutant caught while the line stays
untested. Coverage answers the narrower question directly.

The budget below counts lines, not percentages, because a percentage large enough to
tolerate the known residue is also large enough to hide a newly added branch.

Usage:
    cargo llvm-cov --workspace --json --output-path coverage.json
    python3 scripts/check-rule-coverage.py coverage.json
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

RULES = "/src/rules/"

# Lines that cannot be executed by any test, with the reason they cannot.
#
# There used to be nine, eight of them the same thing: a `?` propagating a location error
# that could not occur, because every fact validated its range on construction without the
# type recording that it had. A range is now built only by the file it indexes, so locating
# one cannot fail, rule evaluation reports no error at all, and those eight lines are gone
# rather than documented. The budget is fixed in both directions: a newly uncovered line
# pushes the count over, and a budget left above reality is reported too, which is how that
# collapse showed up here rather than as silent slack.
BUDGET = {
    # The marker range is derived from a comment range that is already valid and can only
    # narrow it, so asking the file for it cannot fail.
    "src/rules/todo_requires_reference.rs": 1,
}


def uncovered(report: Path) -> dict[str, list[int]]:
    data = json.loads(report.read_text(encoding="utf-8"))
    found: dict[str, list[int]] = {}

    for entry in data["data"][0]["files"]:
        name = entry["filename"]

        if RULES not in name:
            continue

        lines = sorted(
            {segment[0] for segment in entry["segments"] if segment[2] == 0 and segment[3]}
        )

        if lines:
            found[f"src/rules/{name.split(RULES)[1]}"] = lines

    return found


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__)
        return 1

    found = uncovered(Path(sys.argv[1]))
    failures = []

    for name, lines in sorted(found.items()):
        allowed = BUDGET.get(name, 0)

        if len(lines) > allowed:
            failures.append(
                f"{name}: {len(lines)} uncovered lines, budget {allowed}: {lines}"
            )

    for name, allowed in sorted(BUDGET.items()):
        actual = len(found.get(name, []))

        if actual < allowed:
            failures.append(
                f"{name}: only {actual} uncovered lines against a budget of {allowed}; "
                "lower the budget so it keeps its grip"
            )

    if failures:
        print("rule coverage is not sufficient:\n")

        for failure in failures:
            print(f"  - {failure}")

        print(
            "\nEvery line of a rule must be reached by a test. Add the case, or if the "
            "line genuinely cannot be executed, raise the budget with the reason beside it."
        )
        return 1

    total = sum(len(lines) for lines in found.values())
    print(f"rule coverage is complete except {total} documented unreachable lines")
    return 0


if __name__ == "__main__":
    sys.exit(main())
