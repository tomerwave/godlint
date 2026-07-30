---
name: opening-a-pull-request
description: Use when opening a pull request against Godlint — branch naming, template choice, labels, and how the changelog check is measured against the release line rather than the target branch. See CONTRIBUTING.md.
---

Read `CONTRIBUTING.md` in full and follow it — branch prefix (Conventional Commits type, no
`codex` prefix), which pull request template to pick, and what the two drift labels
(`fixes-false-positive` / `relaxes-a-rule`) mean and when they are and aren't honest to apply.

Two things easy to get wrong: `validate-pull-request.py`'s change-scoped checks measure against
`origin/main` (or the given release line), not the branch's own target — a stack of pull requests
needs one changelog entry for the whole change, not one per pull request in the stack. And most
labels (`documentation`, `rule-proposal`, `tech-debt`, `packaging`) are applied automatically from
the paths touched — only the two drift labels are ever applied by hand.
