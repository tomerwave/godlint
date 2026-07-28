# Dogfooding policy

Godlint must enforce its own shipped policy against the Godlint repository. This is a
release requirement.

- The first rule-capable vertical slice adds a versioned root `godlint.yaml`.
- CI runs the binary built from the current commit against the current repository.
- A new blocking rule is enabled for Godlint in the same pull request that adds it.
- Exceptions require a reason, owner, issue reference, and expiry; no silent global
  exclusion of product code, workers, or tests is allowed.
- Every CI run scans the whole repository, on pull requests and on the main branch
  alike.

The workspace foundation proves only build and CI health. The first user-value slice
must make `godlint check .` pass before it is considered shipped.

## Planned, not yet policy

The following are intended additions to this policy. None of them is implemented, so
none of them is a gate today.

- Changed-files checks on pull requests, with the full scan reserved for the main
  branch. This needs a diff-aware mode that does not exist yet, so pull requests
  currently run the same full scan as main.
- A release workflow that scans a clean checkout of the exact tagged source.
- Retaining machine-readable reports as build artifacts, which needs the JSON and SARIF
  reporters scheduled in Phase 6 of the [rule roadmap](rule-roadmap.md).
