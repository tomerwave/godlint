---
name: proposing-a-threshold
description: Use when a rule proposal or an existing rule needs a numeric threshold (a max, a limit, a count). Measures the metric against this repository's own source rather than borrowing another linter's default. See docs/skills/proposing-a-threshold.md.
---

Read `docs/skills/proposing-a-threshold.md` in full and follow it. In short:

1. Write a short script measuring the metric across this repository's real source, excluding
   `crates/godlint-cli/tests/fixtures/` and `.github/fixtures/`.
2. Look at the distribution — p50, p90, p95, p99, max — not just the average.
3. Compare against what established catalogues default to, to know whether this number should
   be tighter (most of Godlint's existing thresholds are) or has a real reason to differ.
4. Pick the number that's tight but passes on this repository today, or record that it doesn't
   and why.
5. State the measurement in the proposal or changelog — not just the chosen number.

If the measurement can't be exact (e.g. formatting wraps lines the scan can't see), say so
explicitly rather than presenting an approximation as a precise count.
