# Dogfooding policy

Godlint must enforce its own shipped policy against the Godlint repository. This is a
release requirement.

- The first rule-capable vertical slice adds a versioned root `godlint.yaml`.
- CI runs the binary built from the current commit against the current repository.
- A new blocking rule is enabled for Godlint in the same pull request that adds it.
- Exceptions require a reason, owner, issue reference, and expiry; no silent global
  exclusion of product code, workers, or tests is allowed.
- Pull requests run changed-files checks; the main branch runs a full scan.
- Release candidates scan a clean checkout of the exact tagged source and retain JSON
  and SARIF reports as artifacts.

The workspace foundation proves only build and CI health. The first user-value slice
must make `godlint check .` pass before it is considered shipped.
