# Contributing to Godlint

Godlint is pre-alpha, so clear problem statements, fixture-quality examples and design feedback are as
useful as code.

Start with [local development](docs/local-development.md) for the build and the checks CI runs.

## Before opening an issue or pull request

- Search existing issues and discussions first.
- For a proposed rule, include representative valid and invalid examples **for each relevant
  language**, the desired diagnostic, likely false-positive cases, and whether the rule is syntax-,
  semantic- or repository-based.
- Discuss substantial design changes before implementing them, so the public rule and configuration
  contracts stay coherent.
- For a security issue, follow [SECURITY.md](SECURITY.md) instead of opening a public issue.

## Design principles

- Only deterministic analysis decides CI pass or fail.
- Prefer high-confidence, explainable findings over broad heuristics.
- Keep native parser details inside language adapters; share small, versioned facts.
- Preserve source privacy: analysis is local by default.
- Treat suppressions as visible, expiring policy debt.
- Do not add a dependency without a documented need and a maintenance rationale.
- Do not add a public API or crate boundary before a real implementation need exists.

## Change conventions

- Keep changes focused, reviewable and reversible.
- Maintain deterministic diagnostic ordering and stable finding fingerprints.
- Use explicit argument arrays for any external command.
- Keep test code out of `src/`: crate contracts live in `crates/<crate>/tests/`, and rule behaviour in
  fixtures under `crates/godlint-cli/tests/fixtures/rules/<rule-id>/`.
- A rule change needs valid, invalid, configuration and suppression fixtures; a safe fix also needs an
  expected-output fixture.
- Update documentation when public behaviour, configuration, suite defaults or rule semantics change,
  and add release-note text for anything user-visible.

## Branches and pull requests

Branch from `main` and name the branch with a Conventional Commits type, a slash, and a lower-case
description — `feat/import-fact`. The accepted types are `feat`, `fix`, `perf`, `docs`, `style`,
`refactor`, `test`, `build`, `ci`, `chore` and `revert`. Further slashes are allowed. A required check
enforces this; it cannot refuse the push, only the merge, because GitHub's own branch-name rule is not
available on this repository's plan.

Pick the template that matches the change by appending `?template=new-rule.md` or
`?template=infrastructure.md` to the pull request URL: `new-rule` for adding or changing a rule,
`infrastructure` for build, CI, tooling or documentation work.

`main` takes no direct push, no force-push and no deletion, and every required check must be green
before a merge. Merges are merge commits and the branch is deleted afterwards. An administrator can
bypass, which exists to unstick a broken check rather than to skip review.

## Labels

Most labels are applied for you from the paths a pull request touches — `documentation`,
`rule-proposal`, `tech-debt`, `packaging`. Nothing to do.

Two are yours to apply, and only you can know which fits. `The released Godlint agrees with this tree`
runs the *published* binary against your branch, so it goes red whenever a change makes the release
report something this tree no longer considers a finding. That is legitimate in exactly two cases:

- **`fixes-false-positive`** — the rule was reporting something it should not have. The release still
  reports it because it predates the fix.
- **`relaxes-a-rule`** — the rule itself was narrowed, or a threshold loosened.

Applying either makes the check pass and records which of the two happened; the explanation is printed
either way, and the check returns to green on its own after the next release.

Neither label belongs on a pull request where the repository has genuinely drifted from the standard it
publishes — there the findings are real. Applying one to hide a regression defeats the only check that
compares what Godlint ships against what Godlint demands.

Adding a rule or tightening a threshold never needs a label: the released binary is the more permissive
one, so the check stays green.

## Code of Conduct

By participating, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).
