# Product scope

Godlint is a deterministic, local-first code-policy engine for polyglot repositories.
It will analyze Rust, TypeScript/JavaScript, and Python and enforce shared engineering
standards across architecture, reliability, maintainability, testing, and security.

## MVP

The MVP provides one CLI, configuration and composable suites, syntax analysis for all
three initial languages, at least twelve useful high-confidence rules, exceptions,
baseline/no-new-violations behavior, and terminal/JSON/SARIF output.

## Non-goals

- Replacing compilers, formatters, ESLint, Ruff, Clippy, or type checkers.
- LLM-based CI pass/fail decisions.
- A universal cross-language AST.
- Arbitrary third-party plugins in early releases.
- Whole-program formal verification or large automatic architecture rewrites.

## Product principles

- Define policy once; detect it appropriately in each language.
- Explain findings with evidence, confidence, and remediation guidance.
- Support incremental adoption through baselines, diff-aware checks, and accountable
  exceptions.
- Optimize for repeatable local and CI execution.
