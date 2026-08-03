#!/usr/bin/env python3
"""Check that a release tag, the workspace version and the changelog name one version.

Publishing cannot be undone: a version may be yanked but never replaced. So the places a version
is written must agree before anything is built, and the changelog section for that version becomes
the release notes rather than being written twice.

One crate in the workspace depends on another, and crates.io requires that dependency to carry a
version rather than only a path. Cargo cannot inherit the workspace version into a dependency
requirement, so that number is written by hand and was missed once: the tag, the manifest and the
changelog all agreed on 0.2.0 while godlint-cli still asked for godlint-core 0.1.9, and the
workspace did not build. This checks it too, because a release that cannot build is worse than one
that was never tagged.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

CHANGELOG = Path("CHANGELOG.md")
INTERNAL = "Internal"
MANIFEST = Path("Cargo.toml")
CRATES = Path("crates")


def workspace_version() -> str:
    match = re.search(r'^version = "([^"]+)"', MANIFEST.read_text(encoding="utf-8"), re.MULTILINE)

    if match is None:
        raise SystemExit(f"{MANIFEST}: no workspace version found")

    return match.group(1)


def internal_requirements(version: str) -> list[str]:
    wrong = []

    for manifest in sorted(CRATES.glob("*/Cargo.toml")):
        for name, asked in re.findall(
            r'^(\S+) = \{[^}]*path = "[^"]+"[^}]*version = "([^"]+)"', 
            manifest.read_text(encoding="utf-8"),
            re.MULTILINE,
        ):
            if asked != version:
                wrong.append(f"{manifest} asks for {name} {asked}, not {version}")

    return wrong


def notes(version: str) -> str:
    text = CHANGELOG.read_text(encoding="utf-8")
    heading = re.compile(rf"^## \[{re.escape(version)}\][^\n]*$", re.MULTILINE)
    start = heading.search(text)

    if start is None:
        raise SystemExit(
            f"{CHANGELOG}: no section for {version}. "
            "Rename the Unreleased section before tagging, so the release notes exist."
        )

    rest = text[start.end() :]
    end = re.search(r"^## ", rest, re.MULTILINE)
    section = rest[: end.start()] if end else rest
    body = without_internal(section).strip()

    if not body:
        raise SystemExit(
            f"{CHANGELOG}: the section for {version} announces nothing. "
            f"{'Every entry is under Internal, which the release body leaves out' if section.strip() else 'The section is empty'}. "
            "A release needs a sentence a user can read."
        )

    return body


def without_internal(section: str) -> str:
    """The section a release announces, which is every category except Internal.

    The release body is this text verbatim, so it reaches people who will never read this
    repository. An entry saying that a refactor changed nothing observable belongs in the log
    and not in that announcement, and `Internal` is where it goes.
    """

    kept: list[str] = []
    internal = False

    for line in section.splitlines():
        if line.startswith("### "):
            internal = line[4:].strip() == INTERNAL

        if not internal:
            kept.append(line)

    return "\n".join(kept)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tag")
    parser.add_argument("--notes", action="store_true", help="print the release notes")
    arguments = parser.parse_args()

    version = arguments.tag.removeprefix("v")
    declared = workspace_version()

    if version != declared:
        raise SystemExit(
            f"tag {arguments.tag} does not match the workspace version {declared}. "
            "The tag decides the release, so the manifest must already say the same."
        )

    for problem in internal_requirements(declared):
        print(problem, file=sys.stderr)

    if internal_requirements(declared):
        raise SystemExit(
            "a crate in this workspace asks for another at the wrong version, so the release "
            "would not build. Cargo cannot inherit the workspace version here, so it is written "
            "by hand."
        )

    body = notes(version)

    if arguments.notes:
        print(body)
    else:
        print(f"{arguments.tag} matches version {declared} and has release notes")

    return 0


if __name__ == "__main__":
    sys.exit(main())
