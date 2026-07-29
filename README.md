<p align="center">
  <img src="assets/godlint-icon.svg" width="168" alt="Godlint logo: code brackets and a V mark inside a broken circle">
</p>

<h1 align="center">Godlint</h1>

<p align="center">
  <strong>One policy engine for every language in your repository.</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/godlint-cli"><img src="https://img.shields.io/crates/v/godlint-cli?label=crates.io" alt="crates.io"></a>
  <a href="https://www.npmjs.com/package/@godlint/cli"><img src="https://img.shields.io/npm/v/@godlint/cli?label=npm" alt="npm"></a>
  <a href="https://pypi.org/project/godlint/"><img src="https://img.shields.io/pypi/v/godlint?label=PyPI" alt="PyPI"></a>
  <a href="https://github.com/tomerwave/godlint/actions/workflows/test.yml"><img src="https://github.com/tomerwave/godlint/actions/workflows/test.yml/badge.svg" alt="Tests"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT License"></a>
</p>

<p align="center">
  <a href="docs/README.md">Documentation</a> ·
  <a href="docs/rules.md">Rules</a> ·
  <a href="docs/configuration.md">Configuration</a> ·
  <a href="docs/ci.md">CI</a> ·
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

> **Pre-alpha:** Godlint enforces twenty-one rules across three languages and installs from four
> channels, but its configuration format, rule identifiers and suite contents may still change
> before `1.0`.

Godlint is a deterministic code-policy engine for polyglot repositories. It lets a team state an
engineering standard once and enforce it across Rust, TypeScript, JavaScript and Python, in one
command, with the same thresholds and the same wording everywhere.

It exists for the policies a single-language linter structurally cannot hold: that no layer imports
upward, that a suppression names an owner and an expiry, that a threshold is the same number in every
language. It complements Clippy, ESLint, Ruff and Pyright rather than replacing them — nothing here
duplicates a compiler, a formatter or a type checker.

## What it does today

- Twenty-one rules over Rust, TypeScript, JavaScript and Python, from function size to import
  boundaries. See [the rule reference](docs/rules.md).
- One suite, `recommended@1`, so a repository adopts a standard in a line rather than twenty-one.
- Exceptions that expire: every inline suppression carries a reason, an owner and a date, and a rule
  fails the build when one outlives it.
- Terminal, JSON, SARIF and GitHub-annotation output, so the same run serves a person, a dashboard
  and a pull request diff.
- A GitHub Action that puts each finding on the line it belongs to, with no token and no permissions.

Deliberately not here: no LLM decides whether CI passes, and there are no third-party plugins yet.
[The rule roadmap](docs/rule-roadmap.md) records what is coming and why each threshold is the number
it is. Baselines and diff-aware enforcement — the two things that make adoption gradual in a large
repository — are on that roadmap rather than in the list above.

## Install

No Rust toolchain is needed on three of the four channels, which is the point: Godlint lints
JavaScript, TypeScript and Python, and most people working in those languages do not have one.

| Channel | Command |
| --- | --- |
| npm | `npm install --save-dev @godlint/cli` |
| PyPI | `pip install godlint` |
| Cargo | `cargo install godlint-cli` |
| Binary | [latest release](https://github.com/tomerwave/godlint/releases/latest) — Linux, macOS, Windows, plus a static musl build |

Every channel installs the same binary and the same command, `godlint`. The npm package is scoped
because npm holds the bare name too close to an existing one. Release archives ship a `.sha256`
beside them:

```bash
tar -xzf godlint-x86_64-unknown-linux-gnu.tar.gz
shasum -a 256 -c godlint-x86_64-unknown-linux-gnu.tar.gz.sha256
install -m 755 godlint /usr/local/bin/
```

## Quickstart

Godlint enforces nothing until a configuration asks it to. Write `godlint.yaml` at the repository
root, adopt the suite, and run it:

```yaml
version: 1
suites: [recommended@1]
```

```bash
godlint check
```

`check` reads the current directory when given no paths, and exits non-zero when a finding is at or
above `fail-on`. That is what makes enforcement one line in any CI system:

```yaml
- run: godlint check
```

A threshold can be loosened, tightened or declined without abandoning the suite — see
[configuration](docs/configuration.md) for the whole schema, and
[inline suppression](docs/suppressions.md) for exempting a single site.

## On GitHub

The action installs the binary and annotates every finding on the line it belongs to:

```yaml
- uses: tomerwave/godlint@v1
  with:
    version: 0.1.9
```

Findings appear in Files changed and disappear when they are fixed. It needs no token and no
permissions, which is why it also works on a pull request from a fork. Pin `version`: the default is
the latest release, and a floating one changes what a pull request is held to without a commit saying
so. [Using Godlint in CI](docs/ci.md) covers the inputs, the output formats and the job summary.

## Documentation

| | |
| --- | --- |
| [Rule reference](docs/rules.md) | Every rule, what it measures, and what it cannot see yet |
| [Configuration](docs/configuration.md) | `godlint.yaml`: suites, thresholds, severities, exclusions |
| [Using Godlint in CI](docs/ci.md) | The action, output formats, annotations |
| [Inline suppression](docs/suppressions.md) | Exempting a site, accountably |
| [Rule roadmap](docs/rule-roadmap.md) | What is shipped, what is next, and every threshold's reasoning |
| [Product scope](docs/product-scope.md) | The promise and the non-goals |
| [Architecture](docs/architecture.md) | Crate boundaries and how a language stays behind one |
| [Local development](docs/local-development.md) | Building, testing and running Godlint on itself |
| [Contributing](CONTRIBUTING.md) | Proposing a rule, opening a pull request |

## Contributing

Early design feedback, rule ideas backed by concrete examples, and documentation improvements are all
useful. Read [CONTRIBUTING.md](CONTRIBUTING.md) and abide by the
[Code of Conduct](CODE_OF_CONDUCT.md). Please do not file security vulnerabilities in public issues;
use the process in [SECURITY.md](SECURITY.md).

## License

Godlint is released under the [MIT License](LICENSE).
