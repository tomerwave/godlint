<p align="center">
  <img src="assets/godlint-icon.svg" width="168" alt="Godlint logo: code brackets and a V mark inside a broken circle">
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

> **Pre-alpha:** Godlint has an early local CLI and its first cross-language rules.
> Its public API, configuration format, and rule suites are not stable yet.

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

See the [rule roadmap](docs/rule-roadmap.md) for the rule families, thresholds, and
delivery sequence. The implementation sequence is:

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
cargo run -p godlint-cli -- check .
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

The `check` command evaluates the configured rules across Rust,
TypeScript/JavaScript, and Python source files. Eleven rules are implemented:

- `maintainability/file-size` — effective lines in a file.
- `maintainability/function-size` — effective lines in a function.
- `maintainability/function-nesting` — how deeply control-flow blocks nest inside a
  function.
- `maintainability/parameter-count` — declared parameters, excluding a method receiver.
- `maintainability/decision-complexity` — branch points in a function. A `match` or
  `switch` counts once rather than once per arm, and a guard on an arm counts.
- `maintainability/return-count` — exit paths from a function, explicit or implicit.
- `maintainability/function-statements` — statements in a function, through nested
  blocks but not into nested functions.
- `maintainability/empty-function` — function bodies that appear unintentionally empty.
- `policy/todo-requires-reference` — TODO-style markers that need an issue reference.
- `style/no-comments` — commentary where the code should speak for itself.
- `policy/accountable-suppression` — inline suppressions that cannot account for
  themselves.
- `policy/unused-suppression` — inline suppressions that no longer silence an enabled
  finding.

A function means the same thing in every language: Rust `fn` items and closures,
Python `def` functions and lambdas, and JavaScript/TypeScript function declarations,
function expressions, methods, and arrow functions. Findings below the configured
`fail-on` severity are reported without failing the command.

```bash
godlint check
godlint check crates
```

## Accountable exceptions

A single site can be exempted from a rule by a comment that says why, who owns it, and
when the exemption lapses:

```rust
// godlint-ignore-next-line maintainability/function-size owner=tomer expires=2026-12-31 -- splitting this in #482
fn long_function() {
    // ...
}
```

`godlint-ignore-enclosing` applies to the whole function containing it. There is no
file-wide form — that is what `exclude` is for. `policy/accountable-suppression` reports
a directive with no reason, an unknown rule, or an expiry in the past; and
`policy/unused-suppression` reports one that no longer hides an enabled finding. Neither
policy rule can be suppressed. List every exemption in the repository with:

```bash
godlint suppressions
```

See [inline suppression](docs/suppressions.md) for the full syntax and semantics.

## Contributing

We welcome early design feedback, rule ideas backed by concrete examples, parser and
performance research, documentation improvements, and eventually implementation
contributions. Please read [CONTRIBUTING.md](CONTRIBUTING.md) and abide by the
[Code of Conduct](CODE_OF_CONDUCT.md).

Please do not file security vulnerabilities in public issues; use the process in
[SECURITY.md](SECURITY.md).

## License

Godlint is released under the [MIT License](LICENSE).
