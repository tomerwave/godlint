# Contribution conventions

Read [CONTRIBUTING.md](../CONTRIBUTING.md) for the public contribution process.

For repository changes:

- Keep changes focused, reviewable, and reversible.
- Document and justify new dependencies.
- Keep source analysis local; use explicit argument arrays for any external command.
- Maintain deterministic diagnostic ordering and stable finding fingerprints.
- Update documentation when public behavior, configuration, suite defaults, or rule
  semantics change.
- Keep test code outside `src/`: crate contracts live in `crates/<crate>/tests/`, and
  rule behavior lives in repository fixtures.
- Do not add a public API or crate boundary before a real implementation need exists.
- Never commit `.omx/` planning/runtime files.
