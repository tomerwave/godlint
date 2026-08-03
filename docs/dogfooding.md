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

## What the scan excludes, and what that costs

An `exclude` entry is the widest instrument the configuration has: it drops a path for **every rule
at once**, so unlike a suppression it leaves no trace at the site and `godlint check` cannot report
what it hid. That is why the policy above forbids a silent one. Every entry in `godlint.yaml` names
its reason, and the two that hold back real findings are itemised here — measured by scanning with
them removed, not asserted.

Most entries are build output, caches or installed dependencies, and two more are the fixture trees
that exist in order to contain violations: scanning `crates/godlint-cli/tests/fixtures` or
`.github/fixtures` would report each fixture's deliberate finding as a finding against Godlint.

`scripts` and `packaging` are different, because they are this repository's own code. Scanning them
reports **127 findings**:

| Rule | Findings | What it is |
| --- | ---: | --- |
| `style/no-comments` | 78 | Python and JavaScript whose comments carry the reasoning. Not debt. |
| `logging/no-production-log` | 21 | A gate script's `print` is its interface, not logging. Not debt. |
| `architecture/restricted-call` | 7 | `sys.exit` and `process.exit` in files whose job is to exit. Not debt. |
| `maintainability/function-statements` | 8 | Debt. |
| `architecture/filename-case` | 5 | Debt: four scripts and one wrapper are kebab-case where the rule asks Python for snake_case. |
| `maintainability/function-nesting` | 4 | Debt. |
| `maintainability/decision-complexity` | 3 | Debt. |
| `maintainability/parameter-count` | 1 | Debt. |

So 106 of the 127 are the rules meeting code they were not written for, and **21 are real**.

The reason this is an exclusion rather than three lines of rule configuration is a gap in the
product. `logging/no-production-log` and `architecture/restricted-call` both take `allow-in` path
globs, so those 28 could be declared per rule and stay enforced everywhere else. `style/no-comments`
takes a severity and nothing else — with 78 findings it decides the outcome on its own, and there is
no way to spell "not in these paths" for it short of removing the paths. Giving that rule `allow-in`
would let this exclusion shrink to the two directories' real debt, and would serve any repository
that wants prose-free product code and commented build scripts.

## Planned, not yet policy

The following are intended additions to this policy. None of them is implemented, so
none of them is a gate today.

- Changed-files checks on pull requests, with the full scan reserved for the main
  branch. This needs a diff-aware mode that does not exist yet, so pull requests
  currently run the same full scan as main.
- A release workflow that scans a clean checkout of the exact tagged source.
- Retaining machine-readable reports as build artifacts, which needs the JSON and SARIF
  reporters scheduled in Phase 6 of the [rule roadmap](rule-roadmap.md).
