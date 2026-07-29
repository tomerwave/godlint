# Contributing to Godlint

Thank you for considering a contribution to Godlint. The project is pre-alpha, so
clear problem statements, fixture-quality examples, and design feedback are as useful
as code.

## Before opening an issue or pull request

- Search existing issues and discussions first.
- For a proposed rule, include representative valid and invalid examples for each
  relevant language, the desired diagnostic, likely false-positive cases, and whether
  the rule is syntax-, semantic-, or repository-based.
- For a security issue, follow [SECURITY.md](SECURITY.md) instead of opening a public
  issue.
- Discuss substantial design changes before implementing them, so the public rule and
  configuration contracts remain coherent.

## Development principles

- Deterministic analysis only decides CI pass/fail status.
- Prefer high-confidence, explainable findings over broad heuristics.
- Keep native parser details inside language adapters; share small, versioned facts.
- Preserve source privacy: analysis is local by default.
- Treat suppressions as visible, expiring policy debt.
- Do not add dependencies without a documented need and maintenance rationale.

## Pull requests

Keep pull requests narrow and explain the problem being solved. Changes to a rule
should include valid, invalid, configuration, and suppression fixtures; a safe fix
also needs an expected-output fixture. Public behavior changes need release-note text
and documentation updates.

The required local checks are:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
python3 scripts/validate-pull-request.py
```

## Labels

Most labels are applied for you from the paths a pull request touches: `documentation`,
`rule-proposal`, `tech-debt`, `packaging`. Nothing to do.

Two are yours to apply, and only you can know which fits. The `Action` workflow runs the
*released* Godlint against the branch, so it goes red whenever a change makes the release
report something this tree no longer considers a finding. That is legitimate twice:

- `fixes-false-positive` - the rule was reporting something it should not have. The
  release still reports it because it predates the fix.
- `relaxes-a-rule` - the rule was deliberately narrowed, or a threshold loosened.

Applying either makes the check pass and records which of the two happened; the
explanation is printed either way. Applying one to hide a genuine regression defeats the
only check that compares what Godlint ships against what Godlint demands, so do not.

Adding a rule or tightening a threshold does not need a label. The released binary reports
less than this tree, not more, and the check stays green.

## Code of Conduct

By participating, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).
