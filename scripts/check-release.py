#!/usr/bin/env python3
"""Check that a release tag, the workspace version and the changelog name one version.

Publishing cannot be undone: a version may be yanked but never replaced. So the three places
a version is written must agree before anything is built, and the changelog section for that
version becomes the release notes rather than being written twice.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

CHANGELOG = Path("CHANGELOG.md")
MANIFEST = Path("Cargo.toml")


def workspace_version() -> str:
    match = re.search(r'^version = "([^"]+)"', MANIFEST.read_text(encoding="utf-8"), re.MULTILINE)

    if match is None:
        raise SystemExit(f"{MANIFEST}: no workspace version found")

    return match.group(1)


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

    return rest[: end.start()].strip() if end else rest.strip()


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

    body = notes(version)

    if arguments.notes:
        print(body)
    else:
        print(f"{arguments.tag} matches version {declared} and has release notes")

    return 0


if __name__ == "__main__":
    sys.exit(main())
