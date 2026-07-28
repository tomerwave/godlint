# Testing strategy

Godlint is fixture-first and end-to-end biased. The primary proof of rule behavior is:

```text
source fixture + godlint.yaml -> normalized expected diagnostics
```

Every rule needs valid, invalid, and configuration fixtures, plus coverage of scoped
exclusion: a rule must stay silent on a path the top-level `exclude` globs remove from
the scan. Shared rules need equivalent Rust, TypeScript/JavaScript, and Python cases
when the concept applies. Repository rules need miniature realistic repositories rather
than mocked dependency graphs.

Inline suppression is planned but not implemented, so no rule can carry a suppression
fixture yet. Its requirements are recorded under accountable exceptions in the
[rule roadmap](rule-roadmap.md). When it lands, suppression cases become a fourth
required fixture class and every shipped rule is backfilled in the same change.

Keep test code outside production `src/` modules. Public crate contracts belong in
`crates/<crate>/tests/`. Rule fixtures belong in
`crates/godlint-cli/tests/fixtures/rules/<rule-id>/`, each with its own `godlint.yaml`
and `expected.yaml` covering valid, invalid, configuration, and exclusion behavior. This
keeps source modules focused on the shipped implementation and makes the rule contract
easy to inspect.

Use focused integration tests for small deterministic invariants that are hard to
diagnose through rule fixtures: configuration merging, glob behavior, source ranges,
fingerprints, cache keys, diff parsing, and graph algorithms.

The validation stack is:

1. Focused crate-contract tests for deterministic primitives.
2. Rule fixtures for behavior and false-positive boundaries.
3. CLI/repository integration tests for product contracts.
4. A pinned real-world corpus for runtime and false-positive regression measurement.
