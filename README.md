<p align="center">
  <img src="assets/godlint-icon.svg" width="168" alt="Godlint logo: code brackets and a check mark inside a circular mark">
</p>

<h1 align="center">Godlint</h1>

<p align="center">
  <strong>The executable engineering constitution for repositories maintained by humans and coding agents.</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/godlint-cli"><img src="https://img.shields.io/crates/v/godlint-cli?label=crates.io" alt="crates.io"></a>
  <a href="https://www.npmjs.com/package/@godlint/cli"><img src="https://img.shields.io/npm/v/@godlint/cli?label=npm" alt="npm"></a>
  <a href="https://pypi.org/project/godlint/"><img src="https://img.shields.io/pypi/v/godlint?label=PyPI" alt="PyPI"></a>
  <a href="https://github.com/tomerwave/godlint/actions/workflows/test.yml"><img src="https://github.com/tomerwave/godlint/actions/workflows/test.yml/badge.svg" alt="Tests"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT License"></a>
</p>

<p align="center">
  <a href="#quickstart">Quickstart</a> ·
  <a href="docs/rules.md">Rules</a> ·
  <a href="docs/README.md">Documentation</a> ·
  <a href="CONTRIBUTING.md">Contribute</a>
</p>

AI coding agents increase output. They do not automatically preserve your architecture, security
boundaries or engineering standards.

Godlint turns those decisions into deterministic policy that every contributor runs locally and in
CI. Define a rule once; enforce it across Rust, TypeScript, JavaScript and Python with the same
result for humans and agents.

```text
Human or agent writes code
        ↓
Godlint checks repository policy
        ↓
Clear finding → fix → deterministic CI
```

## Why Godlint

- **Guardrails for agents.** Catch architectural drift before generated code reaches review.
- **One policy across languages.** Share boundaries and thresholds across a polyglot repository.
- **Deterministic enforcement.** No LLM decides whether CI passes.
- **Accountable exceptions.** Suppressions carry a reason, owner and expiry.
- **Local by default.** Source code does not leave your machine.

Godlint complements Clippy, ESLint, Ruff and Pyright. They understand one language deeply; Godlint
enforces the repository-level decisions that must hold across all of them.

## Quickstart

Create `godlint.yaml`:

```yaml
version: 1
suites: [recommended@1]
```

Run:

```bash
npx @godlint/cli check
# or: uvx godlint check
# or: cargo install godlint-cli && godlint check
```

That is enough to enforce the recommended policy locally and in CI.

## What it enforces

Godlint ships 25 rules for:

- architecture and module boundaries;
- restricted imports, calls and dependencies;
- centralized configuration and safer execution;
- complexity, nesting, function and file size;
- production logging and error handling;
- accountable, expiring suppressions.

See the [rule reference](docs/rules.md) for every rule and its limits.

## GitHub

Annotate findings directly on pull requests:

```yaml
- uses: tomerwave/godlint@v1
  with:
    version: 0.1.9
```

The action needs no token or write permissions. Godlint also emits terminal, GitHub, JSON and SARIF
output. See [Using Godlint in CI](docs/ci.md).

## Install

Use the package manager already in your project:

| Ecosystem | Command |
| --- | --- |
| npm | `npm install --save-dev @godlint/cli` |
| pnpm | `pnpm add -D @godlint/cli` |
| Yarn | `yarn add -D @godlint/cli` |
| Bun | `bun add -d @godlint/cli` |
| uv | `uv add --dev godlint` |
| pip | `pip install godlint` |
| Cargo | `cargo install godlint-cli` |

Prebuilt binaries for Linux, macOS and Windows are available from the
[latest release](https://github.com/tomerwave/godlint/releases/latest).

## Contribute

Godlint is pre-alpha. This is the best time to shape it.

- **Use it:** try `recommended@1` on a real repository and report false positives.
- **Propose a rule:** bring valid and invalid examples across the relevant languages.
- **Improve it:** code, fixtures, documentation and design feedback are welcome.

Start with [CONTRIBUTING.md](CONTRIBUTING.md) or open a
[rule proposal](https://github.com/tomerwave/godlint/issues/new?template=rule_proposal.yml).
Report security issues through [SECURITY.md](SECURITY.md), not a public issue.

## Status

Godlint is pre-alpha. Rule identifiers, configuration and suite contents may change before `1.0`.
The project is deterministic and useful today; the [roadmap](docs/rule-roadmap.md) records what
comes next.

## License

[MIT](LICENSE)
