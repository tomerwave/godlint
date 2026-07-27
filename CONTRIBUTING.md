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

When the Rust workspace is introduced, the required local checks will be:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

Until then, documentation and community changes should be checked for working links,
accurate scope, and respectful language.

## Code of Conduct

By participating, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).
