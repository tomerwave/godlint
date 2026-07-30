---
name: add-a-rule
description: Use when implementing a Godlint rule that already has an approved proposal — writing the rule, its fixtures, tests and documentation. Follows the ten-point checklist in docs/skills/add-a-rule.md that mirrors scripts/validate-pull-request.py.
---

Read `docs/skills/add-a-rule.md` in full and follow it. In short, one rule touches ten places:
`rules/mod.rs` (module + evaluator registration), `config/mod.rs` (the configurable field),
`rules/registry.rs` (the suppression-visible id), a fixture directory under
`crates/godlint-cli/tests/fixtures/rules/`, unit tests plus their declaration in
`tests/rules.rs`, and three documents (`docs/rule-roadmap.md`, `docs/rules.md`,
`CHANGELOG.md`) that must all name the identifier — plus dogfooding it in `godlint.yaml`.

Before opening a pull request:

```bash
python3 scripts/validate-pull-request.py
cargo llvm-cov --workspace --json --output-path coverage.json
python3 scripts/check-rule-coverage.py coverage.json
```

Write a fixture for every syntactic form that produces the fact the rule reads, not one fixture
per rule — see the #88 note in `docs/skills/add-a-rule.md` for why a rule can be mutation-clean
and still miss its main case.

No comments in the rule's own source; put the "why" in `docs/rules.md` or
`docs/architecture.md` instead.
