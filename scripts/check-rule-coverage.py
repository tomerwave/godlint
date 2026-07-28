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
# Every entry is error propagation from `SourceFile::location`, which fails only for a
# range that is out of bounds or off a character boundary. Function, comment, call, and
# access facts validate their ranges when they are constructed, so no fact can carry such
# a range and the `?` never fires.
# Removing that plumbing would take this budget to zero; until then it is fixed, so a
# newly uncovered line pushes the count over and fails.
BUDGET = {
    "src/rules/mod.rs": 7,
    # A rule that evaluates two fact kinds propagates a location error from each, and the
    # shared reference driver propagates one of its own. CallFact and AccessFact validate
    # their ranges on construction, so none of these can execute through the fact contract.
    # Two are the rule's own `?` propagations. The third is `Language::Rust => false` in
    # `is_environment_access`: Rust states in its vocabulary that it has no member-access
    # form, so no Rust file produces an access fact and the arm exists to make the compiler
    # demand a decision when a language is added.
    "src/rules/direct_environment_read.rs": 3,
    "src/rules/reference.rs": 1,
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
