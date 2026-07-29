# Local development

Godlint is a Rust workspace: `godlint-core` holds the analysis and `godlint-cli` builds the `godlint`
binary. It requires Rust `1.97.1`, which `rust-toolchain.toml` pins, so
[rustup](https://rustup.rs/) installs the right one on first build.

## The checks CI runs

Run these before opening a pull request. CI runs the same ones, and a failure here is a failure there:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
python3 scripts/validate-pull-request.py
cargo run -p godlint-cli -- check .
```

The last line is Godlint checking Godlint. The repository adopts `recommended@1` and holds itself to
it, so a change that makes the code exceed a threshold fails its own policy — see
[dogfooding](dogfooding.md) for what that commits the project to.

`scripts/validate-pull-request.py` enforces the parts of the pull request templates that can be
checked mechanically. Its change-scoped checks measure the branch against `origin/main` rather than the
pull request's target, so a stack of pull requests needs one changelog entry for the change rather than
one per pull request.

## Running the binary you just built

```bash
cargo run -p godlint-cli -- --version
cargo run -p godlint-cli -- check
cargo run -p godlint-cli -- check crates
cargo run -p godlint-cli -- config validate
```

`check` reads the current directory when given no paths. [Configuration](configuration.md) covers what
it reads, and [the rule reference](rules.md) covers what it reports.

## Tests

Rule behaviour lives in fixtures rather than in Rust assertions, so a rule's contract is readable as
input and expected output. Crate contracts live in `crates/<crate>/tests/`, and rule fixtures in
`crates/godlint-cli/tests/fixtures/rules/<rule-id>/`. No test code belongs in `src/`.

[The testing strategy](testing.md) explains the layers and which one a given change belongs in. Two
gates go beyond the ordinary test run:

```bash
cargo llvm-cov --workspace --json --output-path coverage.json
python3 scripts/check-rule-coverage.py coverage.json
cargo mutants
```

`check-rule-coverage.py` fails in both directions: a rule below its line-coverage budget fails, and so
does one comfortably above it, because a budget nobody lowers stops meaning anything.

## Two conventions that surprise people

**No comments in `src/`.** `style/no-comments` runs at `error` over this repository's own source.
Reasoning that would have been a comment belongs in `docs/`, and a name that needs a comment to be
understood should be renamed. [The architecture guide](architecture.md) is where the *why* lives.

**A language's node kinds never leak.** Shared code must not name a tree-sitter node kind; each
language exposes its own vocabulary and the rules are written against facts. That is what keeps one
threshold meaning the same thing in four languages, and [the architecture guide](architecture.md)
describes the boundary.

## Packaging

The release builds the binaries; the packaging scripts wrap them:

```bash
python3 packaging/build-npm.py --only x86_64-apple-darwin
python3 packaging/build_wheels.py --only x86_64-apple-darwin
```

Both take `--only` for the same reason: one platform can be built and installed on one machine to
check that the command it installs actually runs, which is a class of bug that no dry run catches.
[Releasing](releasing.md) covers the rest.
