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
`crates/godlint-cli/tests/fixtures` and `.github/fixtures` reports 467 findings, 465 and 2,
each one a fixture's deliberate violation attributed to Godlint. This is the entry closest to
what the policy literally prohibits — `tests` — and the reason it is allowed is that the
alternative is a rule suite that reports its own test data.

`scripts` and `packaging` are the two that matter, because they are this repository's own code, and
`packaging/npm/shim.js` is more than that: it ships to every npm user. Scanning both reports **131
findings** — measured by deleting those two lines from `godlint.yaml` and running
`cargo run --release -- check .`, which is how every number below was arrived at and how the next
reader should check whether they have drifted.

**110 are rules meeting code they were not written for.**

| Rule | Findings | Why it fires |
| --- | ---: | --- |
| `style/no-comments` | 82 | Python and JavaScript that explains policy. 60 are ordinary comments and 22 are docstrings — the second group reachable by `allow-doc-comments`, which the `recommended@1` suite deliberately sets to `false`. |
| `logging/no-production-log` | 21 | A gate script's `print` is its interface, not logging. |
| `architecture/restricted-call` | 7 | `sys.exit` and `process.exit`, in files whose job is to exit. |

**5 are a rule meeting the same argument, from the other side.**

| Rule | Findings | The cost of fixing it |
| --- | ---: | --- |
| `architecture/filename-case` | 5 | `scripts/check-real-world.py`, `check-release.py`, `check-rule-coverage.py`, `validate-pull-request.py` and `packaging/build-npm.py` are kebab-case where the rule asks Python for snake_case. But a script's *name* is its interface exactly as its `print` is: these five names appear in four workflows and nineteen other files — seven documents under `docs/`, `AGENTS.md`, `CONTRIBUTING.md`, two pull-request templates, three agent skill definitions and `.cargo/mutants.toml` — and `.github/workflows/real-world.yml` matches one as a **path trigger**, so renaming it silently changes when that workflow runs. `validate-pull-request.py` alone is named in eighteen files. Renaming would also leave `scripts/` mixed, since the four `.sh` files are correctly kebab-case. Worth doing deliberately, in a change that can be reviewed as an interface change — not as tidying. |

**16 are debt with no argument for them.**

| Rule | Findings |
| --- | ---: |
| `maintainability/function-statements` | 8 |
| `maintainability/function-nesting` | 4 |
| `maintainability/decision-complexity` | 3 |
| `maintainability/parameter-count` | 1 |

## This exclusion is debt now, not a missing mechanism

It used to be a mechanism gap. `style/no-comments` took a severity and `allow-doc-comments`,
neither of which is a path, so with 82 of the 131 findings coming from that one rule the only ways
to say "not in these two directories" were removing them from the scan or writing 82 inline
suppressions. That is no longer true: every rule takes `only-in` and `allow-in`. The exclusion
survives for a different reason now, and the distinction matters, because *cannot* and *have not*
call for different things.

Replacing it takes about a dozen lines — `allow-in` on `style/no-comments` and
`logging/no-production-log`, and per-call `allow-in` on `architecture/restricted-call`, whose
built-in catalogue is additive so declaring `calls` does not clobber the other dialects. Measured by
writing it out and running `check`:

| configuration | findings in `scripts` and `packaging` |
| --- | ---: |
| the two paths excluded, as today | 0, none of them visible |
| the two paths scanned, nothing configured | 131 |
| `allow-in` on `style/no-comments` alone | 49 |
| `allow-in` on all three by-design rules | **21** |

Those 21 are exactly the debt itemised above — 5 `architecture/filename-case` and 16
maintainability findings — which is the arithmetic working out, not a coincidence.

So un-excluding is no longer blocked on the product. It is blocked on 21 findings that would fail
the build, and each wants a different answer: the filename-case five are an interface change
touching four workflows and nineteen other files, thirteen of the sixteen are real complexity in
the gate scripts, and three are in the package wrappers. None of it is tidying, all of it is
reviewable on its own, and until some of it happens the exclusion is a deliberate deferral rather
than a limitation.

## Planned, not yet policy

The following are intended additions to this policy. None of them is implemented, so
none of them is a gate today.

- Changed-files checks on pull requests, with the full scan reserved for the main
  branch. This needs a diff-aware mode that does not exist yet, so pull requests
  currently run the same full scan as main.
- A release workflow that scans a clean checkout of the exact tagged source.
- Retaining machine-readable reports as build artifacts, which needs the JSON and SARIF
  reporters scheduled in Phase 6 of the [rule roadmap](rule-roadmap.md).
