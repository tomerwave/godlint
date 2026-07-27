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

Start with only `godlint-cli`, `godlint-core`, and `godlint-test-support`. Split
configuration, diagnostics, analyzers, rules, graph, cache, SARIF, and external tools
into dedicated crates only after their ownership boundaries are proven by real code.

Semantic workers and external ecosystem-tool adapters are post-MVP capabilities.
