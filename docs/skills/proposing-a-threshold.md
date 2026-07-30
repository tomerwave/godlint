# Skill: propose a threshold

Godlint's rule is that a number is **measured, not borrowed**. Every threshold in
`recommended@1` was set by measuring this repository's own source, not by copying another
linter's default — see `docs/rule-roadmap.md` for the reasoning behind each existing one.

## Why borrowing a default is the wrong move

Another linter's default answers a different question: what passes *its* userbase's average
code. It says nothing about whether the number is tight enough to catch what Godlint's own
convention actually cares about, and it says nothing about whether this repository — which is
supposed to dogfood its own policy — can pass it today. A borrowed number that turns out too
loose ships a rule that never fires; one that's too tight fails the dogfood check on day one for
reasons unrelated to the rule's design.

## The method

1. **Write a short script that measures the metric across this repository's real source**,
   excluding `crates/godlint-cli/tests/fixtures/` and `.github/fixtures/` — fixtures are
   deliberately extreme and would skew the numbers.
2. **Look at the distribution, not the average.** p50, p90, p95, p99, and the max. A single
   outlier at the top tells you less than where the bulk of real code sits.
3. **Compare against what the established catalogues default to** — not to copy them, but to
   know whether Godlint's number is meant to be tighter (its existing thresholds mostly are:
   nesting 2 against ESLint's 4, complexity 5 against Ruff's 10) or whether there's a real reason
   this one should sit elsewhere.
4. **Pick the number that's tight but passes on this repository as it stands.** If it doesn't
   pass, that's informative too — either the threshold is wrong, or the repository has a real
   violation to fix before the rule ships.
5. **Say so in the proposal or the changelog.** State the measurement, not just the number — the
   next person tightening it needs to know what was actually true when this was chosen.

## Worked example

Proposing `maintainability/condition-complexity` ([#95](https://github.com/tomerwave/godlint/issues/95)),
a rough single-line scan of this repository's Rust sources for `&&`/`||` per `if`/`while`
condition found:

| operators per condition | count |
| --- | --- |
| 0 | 98 |
| 1 | 14 |
| 2 | 3 |
| 3+ | 0 |

That's a floor, not an exact count — `rustfmt` wraps long conditions across lines, and a
single-line scan misses those. It was enough to conclude a threshold of 3 would pass today, and
consistent with Sonar defaulting to 3 for the same metric while Ruff's `max-bool-expr` defaults to
5 — Godlint's other numbers already run tighter than the field, so 3 was the reasonable choice,
recorded as such in the issue rather than asserted without evidence.

## When you can't measure exactly

Say what you couldn't verify, and why. "This is a floor, because X" is honest and useful; silently
presenting an approximation as an exact count is not. The condition-complexity example above does
exactly this — the wrapped-line gap is stated, not hidden.
