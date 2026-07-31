# Godlint agent guide

This file is the repository navigation index for coding agents and contributors.
Read the linked documents relevant to the task before changing code or policy.

## Project documents

[The documentation index](docs/README.md) lists every document. The ones that most often decide a
change:

- [Product scope](docs/product-scope.md) — product promise, MVP boundary, and non-goals.
- [Architecture](docs/architecture.md) — system boundaries, crate ownership, and delivery order.
- [Testing strategy](docs/testing.md) — fixture-first testing and required validation layers.
- [Dogfooding](docs/dogfooding.md) — how Godlint enforces policy on its own repository.
- [Rule reference](docs/rules.md) — every implemented rule and what it cannot see yet.
- [Configuration](docs/configuration.md) — the `godlint.yaml` schema.
- [Local development](docs/local-development.md) — the build and the checks CI runs.
- [Releasing](docs/releasing.md) — the tag-driven release and the registries.
- [Contributing](CONTRIBUTING.md) — change conventions, branch naming, pull request templates, labels.

## Skills

Step-by-step procedures for the recurring tasks in this repository. Follow the linked document in
full before starting the task — these are not summaries of it.

- [Propose a rule](docs/skills/propose-a-rule.md) — turn a candidate practice into a filed issue,
  including the three decidability filters and the standard issue shape.
- [Add a rule](docs/skills/add-a-rule.md) — implement an approved proposal: the ten places one
  rule touches, mirrored from `scripts/validate-pull-request.py`.
- [Propose a threshold](docs/skills/proposing-a-threshold.md) — measure a numeric limit against
  this repository rather than borrowing one from another linter.
- [Releasing](docs/releasing.md) — the tag-driven release process.
- [Opening a pull request](CONTRIBUTING.md) — branch naming, templates, and the drift labels.

Claude Code additionally reads these as `.claude/skills/<name>/SKILL.md`; Cursor reads them as
`.cursor/rules/<name>.mdc`. All three point at the same document in `docs/skills/` — there is one
copy of each procedure, not three.

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
  rule breaks a test. Run `git diff origin/main...HEAD > changed.diff && cargo mutants
  --in-diff changed.diff` and leave no surviving mutant in what you touched. `--file` does
  not narrow a run: `examine_globs` in `.cargo/mutants.toml` decides, and it covers the
  analysers as well as the rules, because an adapter that stops emitting a fact reports
  nothing and a passing suite looks exactly like that.
- Every line of a rule must be reached by a test. `cargo llvm-cov --workspace --json
  --output-path coverage.json && python3 scripts/check-rule-coverage.py coverage.json`.
- Run `python3 scripts/validate-pull-request.py` before opening a pull request. It checks
  that a rule is registered, configurable, fixtured, tested, documented, and dogfooded,
  and names the file to edit for anything missing.
- Do not commit `.omx/`; it contains local planning/runtime state and is git-ignored.

## Current implementation status

The workspace, CLI, configuration validation and discovery, source discovery, and
thirty-seven rules are implemented across eight families; [the rule reference](docs/rules.md)
lists every one, and its support matrix records which languages each covers. CI dogfoods all of
them against this repository through `godlint check .`. Function, comment, call, access, import,
condition, test and assertion facts exist; the repository dependency graph does not. Do not add
semantic workers or new crate boundaries without a proven need.

A function means the same thing in every language, and rules depend on that: Rust `fn`
items and closures, Python `def` functions and lambdas, and JavaScript/TypeScript
function declarations, function expressions, methods, and arrow functions. Do not add a
function-shaped fact for one language without its equivalents in the other two.

A rule that cannot apply to a language declares it in `Rule::LANGUAGES` with a reason, never
in a match arm the caller cannot see. Every language a rule claims needs a fixture that reports
it there; `scripts/validate-pull-request.py` fails in both directions.
