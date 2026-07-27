# Architecture

Godlint has a Rust core that coordinates configuration, file discovery, language
analysis, rules, repository facts, and reporting.

```text
CLI -> config + discovery -> language analyzers -> shared facts
    -> rules + repository graph -> normalized diagnostics -> reports
```

Language adapters retain native AST and parser details. They emit a small shared fact
model (`Function`, `Import`, `Call`, `EnvironmentRead`, `ErrorHandler`, `Assertion`,
`Mock`, and `DependencyEdge`) that rules can consume without a universal AST.

Source files are identified with repository-relative paths and a shared language enum.
Ranges use byte offsets internally; the source contract validates them and derives
one-based line and Unicode-scalar-column positions only at reporting boundaries.

Start with only `godlint-cli` and `godlint-core`. Add fixture-test support,
configuration, diagnostics, analyzers, rules, graph, cache, SARIF, and external tools
as dedicated crates only after their ownership boundaries are proven by real code.

Semantic workers and external ecosystem-tool adapters are post-MVP capabilities.
