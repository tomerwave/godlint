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

- Be concise, always. Report the finding, the evidence, and the decision — not the
  narration. Prefer a table or a short list to prose, name the file and line rather than
  describing where it is, and cut any sentence that restates the previous one. This applies
  to pull request descriptions, review comments, commit bodies, and answers to the user.
  Brevity is not the same as vagueness: keep the reasoning that changes a decision and drop
  the rest.
- Keep pass/fail enforcement deterministic; an LLM must never decide CI status.
- Keep analysis local by default and never expose source code without explicit user
  authorization.
- Prefer high-confidence, explainable diagnostics over broad heuristic coverage.
- Keep language-specific parser details inside their analyzer boundaries; rules consume
  small, language-neutral facts.
- Add a rule only with valid, invalid, and configuration fixtures plus scoped-exclusion
  coverage. [Inline suppression](docs/suppressions.md) is implemented; a suppression
  fixture is intent rather than an enforced gate, and `docs/testing.md` records which rules
  have one.
- Dogfood every shipped rule: Godlint must run it against this repository in CI.
- Write no comments in Rust source, including documentation comments; `style/no-comments`
  enforces this and a comment will fail CI. Put the reasoning a comment would carry into
  [the architecture guide](docs/architecture.md) instead, and name things so the code
  reads without it. Comments inside test fixtures are input data and are exempt.
- Do not add dependencies, public APIs, configuration schema, or crate boundaries
  without updating the relevant documentation and tests.
- A rule change is not covered because a fixture exists; it is covered when altering the
  rule breaks a test. Run `cargo mutants --file 'crates/godlint-core/src/rules/*.rs'` and
  leave no surviving mutant in a rule you touched.
- Every line of a rule must be reached by a test. `cargo llvm-cov --workspace --json
  --output-path coverage.json && python3 scripts/check-rule-coverage.py coverage.json`.
- Run `python3 scripts/validate-pull-request.py` before opening a pull request. It checks
  that a rule is registered, configurable, fixtured, tested, documented, and dogfooded,
  and names the file to edit for anything missing.
- Do not commit `.omx/`; it contains local planning/runtime state and is git-ignored.

## Current implementation status

The workspace, CLI, configuration validation and discovery, source discovery, and ten
rules are implemented: `maintainability/file-size`, `function-size`, `function-nesting`,
`parameter-count`, `decision-complexity`, `return-count`, `function-statements`,
`empty-function`, `policy/todo-requires-reference`, and `style/no-comments`. CI dogfoods
all ten against this repository through `godlint check .`. Phases 1 and 2 of the
[rule roadmap](docs/rule-roadmap.md) are complete; call facts, imports, and the
repository graph are not. Do not add semantic workers or new crate boundaries without a
proven need.

A function means the same thing in every language, and rules depend on that: Rust `fn`
items and closures, Python `def` functions and lambdas, and JavaScript/TypeScript
function declarations, function expressions, methods, and arrow functions. Do not add a
function-shaped fact for one language without its equivalents in the other two.
