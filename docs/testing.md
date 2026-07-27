# Testing strategy

Godlint is fixture-first and end-to-end biased. The primary proof of rule behavior is:

```text
source fixture + godlint.yaml -> normalized expected diagnostics
```

Every rule needs valid, invalid, configuration, and suppression fixtures. Shared rules
need equivalent Rust, TypeScript/JavaScript, and Python cases when the concept applies.
Repository rules need miniature realistic repositories rather than mocked dependency
graphs.

Use unit tests for small deterministic invariants that are hard to diagnose through
end-to-end tests: configuration merging, glob behavior, source ranges, fingerprints,
cache keys, diff parsing, and graph algorithms.

The validation stack is:

1. Unit tests for deterministic primitives.
2. Rule fixtures for behavior and false-positive boundaries.
3. CLI/repository integration tests for product contracts.
4. A pinned real-world corpus for runtime and false-positive regression measurement.
