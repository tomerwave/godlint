---
name: propose-a-rule
description: Use when proposing a new Godlint rule, or reconsidering a rule previously marked "do not build". Applies the three decidability filters and the standard issue shape from docs/skills/propose-a-rule.md.
---

Read `docs/skills/propose-a-rule.md` in full and follow it. In short:

1. Run the candidate through the three filters in order: decidable without types, has a
   specific false-positive case configuration can exempt, has a measurable threshold (see
   `docs/skills/proposing-a-threshold.md` if it has a number in it).
2. File it with `gh issue create`, using the `rule_proposal.yml` template's shape: policy
   problem, blocked, examples, scope, remediation, false positive, source, definition of done.
3. Verify every cited URL actually resolves before it goes in the issue.
4. Label it: `rule-proposal` plus a priority (`P1`/`P2`/`P3`) and, if something blocks it,
   `needs-a-fact` / `needs-a-subsystem` / `needs-a-language`. Only use `good first issue` if
   nothing new needs to exist first.

Do not skip straight to writing code — this skill is for the proposal, not the implementation.
See `add-a-rule` for that.
