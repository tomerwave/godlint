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
  rule behavior lives in fixtures under
  `crates/godlint-cli/tests/fixtures/rules/<rule-id>/`.
- Do not add a public API or crate boundary before a real implementation need exists.
- Never commit `.omx/` planning/runtime files.
- Pick the pull request template that matches the change: `new-rule` when adding or
  changing a rule, `infrastructure` for build, CI, tooling, or documentation work. Append
  `?template=new-rule.md` or `?template=infrastructure.md` to the pull request URL.
- `python3 scripts/validate-pull-request.py` enforces the parts of those templates that
  can be checked. Run it locally; CI runs it too.
