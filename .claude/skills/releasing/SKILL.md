---
name: releasing
description: Use when preparing or verifying a Godlint release — the tag-driven process, crates.io/npm/PyPI publishing order, and the floating v1 action tag. See docs/releasing.md.
---

Read `docs/releasing.md` in full and follow it — it is the release process itself, not a
summary of it.

Two things worth stating up front: only a repository administrator can create the release tag
(exact version tags are protected by a ruleset), and the GitHub release is published **last**,
after crates.io, npm and PyPI have all succeeded — never before. If you are not that
administrator, prepare everything up to `python3 scripts/check-release.py v<version>` passing
and hand off the tag creation.
