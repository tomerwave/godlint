# godlint

Command-line interface for [Godlint](https://github.com/tomerwave/godlint), a deterministic
code-policy engine for Rust, TypeScript, JavaScript and Python.

```sh
cargo install godlint-cli
```

```yaml
# godlint.yaml
version: 1
suites: [recommended@1]
```

```sh
godlint check
```

`check` exits non-zero when a finding is at or above `fail-on`, so enforcing the policy is one
line in CI. See the [repository README](https://github.com/tomerwave/godlint#readme) for the rule
list and configuration reference.
