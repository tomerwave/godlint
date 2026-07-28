# Godlint agent guide

This file is the repository navigation index for coding agents and contributors.
Read the linked documents relevant to the task before changing code or policy.

## Project documents

- [Product scope](docs/product-scope.md) — product promise, MVP boundary, and non-goals.
- [Architecture](docs/architecture.md) — system boundaries, crate ownership, and delivery order.
- [Testing strategy](docs/testing.md) — fixture-first testing and required validation layers.
- [Dogfooding](docs/dogfooding.md) — how Godlint enforces policy on its own repository.
- [Contribution conventions](docs/contributing.md) — code, documentation, dependency, and change expectations.

## Operating rules

- Keep pass/fail enforcement deterministic; an LLM must never decide CI status.
- Keep analysis local by default and never expose source code without explicit user
  authorization.
- Prefer high-confidence, explainable diagnostics over broad heuristic coverage.
- Keep language-specific parser details inside their analyzer boundaries; rules consume
  small, language-neutral facts.
- Add a rule only with valid, invalid, configuration, and suppression fixtures.
- Dogfood every shipped rule: Godlint must run it against this repository in CI.
- Do not add dependencies, public APIs, configuration schema, or crate boundaries
  without updating the relevant documentation and tests.
- Do not commit `.omx/`; it contains local planning/runtime state and is git-ignored.

## Current implementation status

The workspace, CLI shell, configuration validation, source discovery, and the shared
function-size rule are implemented. The current slice connects language extractors and
the CLI so Godlint can dogfood the rule against this repository. Do not add semantic
workers or new crate boundaries without a proven need.
