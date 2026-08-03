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
what it hid. That is what the policy above means by no *silent* global exclusion. Every entry in
`godlint.yaml` names its reason, and the two that hold back findings are itemised here — measured by
scanning with them removed, not asserted.

Most entries are build output, caches or installed dependencies. They cannot be shortened to the
ones this repository happens to produce: `Config::excludes()` falls back to the built-in list only
when `exclude` is empty, so naming any path at all means naming all twelve.

Two more are the fixture trees that exist in order to contain violations. Scanning
`crates/godlint-cli/tests/fixtures` and `.github/fixtures` reports 465 findings, each one a
fixture's deliberate violation attributed to Godlint. This is the entry closest to what the policy
literally prohibits — `tests` — and the reason it is allowed is that the alternative is a rule
suite that reports its own test data.

`scripts` and `packaging` are the two that matter, because they are this repository's own code, and
`packaging/npm/shim.js` is more than that: it ships to every npm user. Scanning both reports
**127 findings**.

**106 are rules meeting code they were not written for.**

| Rule | Findings | Why it fires |
| --- | ---: | --- |
| `style/no-comments` | 78 | Python and JavaScript that explains policy. 56 are ordinary comments and 22 are docstrings — the second group reachable by `allow-doc-comments`, which the `recommended@1` suite deliberately sets to `false`. |
| `logging/no-production-log` | 21 | A gate script's `print` is its interface, not logging. |
| `architecture/restricted-call` | 7 | `sys.exit` and `process.exit`, in files whose job is to exit. |

**5 are a rule meeting the same argument, from the other side.**

| Rule | Findings | The cost of fixing it |
| --- | ---: | --- |
| `architecture/filename-case` | 5 | `scripts/check-real-world.py`, `check-release.py`, `check-rule-coverage.py`, `validate-pull-request.py` and `packaging/build-npm.py` are kebab-case where the rule asks Python for snake_case. But a script's *name* is its interface exactly as its `print` is: these are named in four workflows, `CONTRIBUTING.md` and five documents, and `.github/workflows/real-world.yml` matches one as a **path trigger**, so renaming it silently changes when that workflow runs. Renaming would also leave `scripts/` mixed, since the four `.sh` files are correctly kebab-case. Worth doing deliberately, in a change that can be reviewed as an interface change — not as tidying. |

**16 are debt with no argument for them.**

| Rule | Findings |
| --- | ---: |
| `maintainability/function-statements` | 8 |
| `maintainability/function-nesting` | 4 |
| `maintainability/decision-complexity` | 3 |
| `maintainability/parameter-count` | 1 |

## Why this is an exclusion and not rule configuration

`logging/no-production-log` takes rule-level `allow-in` path globs, and
`architecture/restricted-call` takes them per call entry, with the built-in catalogue treated as
additive — so those 28 findings could be declared per rule and every other path would stay enforced.
That is four or five lines of configuration, not an exclusion.

`style/no-comments` is the one that decides it. Its two settings are a severity and
`allow-doc-comments`, and neither is a path. Nor is there a way around it elsewhere: the top-level
configuration is `version`, `fail-on`, `exclude`, `suites` and `rules` with unknown keys rejected, so
there is no `overrides` block; a suite is an opaque name with no options; and a nested
`scripts/godlint.yaml` is never read, because discovery descends into a directory unless it is the
root of a git repository. With 78 findings the rule decides the outcome by itself, and the only ways
to silence it here are removing the paths or writing 78 inline suppressions.

Giving `style/no-comments` an `allow-in` would shrink this exclusion to the 21 findings above and
leave every other rule enforced on both directories. It would also serve any repository that wants
prose-free product code and commented build scripts, which is not an unusual thing to want.

## Planned, not yet policy

The following are intended additions to this policy. None of them is implemented, so
none of them is a gate today.

- Changed-files checks on pull requests, with the full scan reserved for the main
  branch. This needs a diff-aware mode that does not exist yet, so pull requests
  currently run the same full scan as main.
- A release workflow that scans a clean checkout of the exact tagged source.
- Retaining machine-readable reports as build artifacts, which needs the JSON and SARIF
  reporters scheduled in Phase 6 of the [rule roadmap](rule-roadmap.md).
