<p align="center">
  <img src="assets/godlint-icon.svg" width="168" alt="Godlint logo: code brackets and a check mark inside a broken circle">
</p>

<h1 align="center">Godlint</h1>

<p align="center">
  <strong>One policy engine for every language in your repository.</strong>
</p>

<p align="center">
  <a href="LICENSE">MIT License</a> ·
  <a href="CONTRIBUTING.md">Contributing</a> ·
  <a href="SECURITY.md">Security</a> ·
  <a href="CODE_OF_CONDUCT.md">Code of Conduct</a>
</p>

> **Pre-alpha:** Godlint is in public preparation. There is no installable CLI or
> stable API yet. We are establishing the project’s product, contribution, and
> release foundations before implementation begins.

Godlint is an open-source, deterministic code-policy engine for polyglot
repositories. It will help teams define engineering standards once and enforce them
consistently across Rust, TypeScript/JavaScript, and Python.

Godlint is designed for architecture, reliability, test quality, security, and
maintainability policies that single-language linters cannot enforce across a whole
repository. It will complement established tools such as Clippy, ESLint, Ruff, and
Pyright—not replace them.

## What Godlint will provide

- One local-first CLI with deterministic pass/fail results.
- Shared policy concepts with language-aware detection.
- Repository and cross-language architecture checks.
- Accountable exceptions: scope, reason, owner, issue, and expiry.
- Gradual adoption through baselines and diff-aware enforcement.
- Terminal, JSON, and SARIF reports for local development and CI.

## Initial scope

The first release will focus on Rust, TypeScript/JavaScript, and Python. The planned
MVP emphasizes high-confidence rules: file and function size, complexity,
centralized configuration, swallowed errors, timeouts, test assertions, policy
hygiene, and import cycles.

The project will not use an LLM to decide whether CI passes, replace compilers or
formatters, or support arbitrary third-party plugins in its early releases.

## Status and roadmap

The detailed product and technical direction is maintained in the project planning
materials while the repository is pre-alpha. The implementation sequence is:

1. Workspace, CLI, configuration, diagnostics, fixtures, and documentation.
2. Syntax analysis for all three initial languages and common facts.
3. High-confidence file and repository rules, exceptions, baseline, and SARIF.
4. Caching, architecture graph, and GitHub Actions integration.
5. Optional semantic workers and ecosystem-tool adapters.

## Local development

Godlint currently requires Rust `1.97.1`. After installing Rust with
[rustup](https://rustup.rs/), run the same checks used by CI:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

The initial command shell is available with:

```bash
cargo run -p godlint-cli -- --version
```

Configuration validation is the first implemented product capability:

```bash
godlint config validate
godlint config validate --config path/to/godlint.yaml
```

The initial `check` command evaluates the configured function-size rule across Rust,
TypeScript/JavaScript, and Python source files:

```bash
godlint check
godlint check crates
```

## Contributing

We welcome early design feedback, rule ideas backed by concrete examples, parser and
performance research, documentation improvements, and eventually implementation
contributions. Please read [CONTRIBUTING.md](CONTRIBUTING.md) and abide by the
[Code of Conduct](CODE_OF_CONDUCT.md).

Please do not file security vulnerabilities in public issues; use the process in
[SECURITY.md](SECURITY.md).

## License

Godlint is released under the [MIT License](LICENSE).
