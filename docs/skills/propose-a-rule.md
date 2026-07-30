# Skill: propose a rule

Turns a candidate practice into an issue using the `rule_proposal.yml` template, with the
`rule-proposal` label plus a priority (`P1`/`P2`/`P3`) and, if relevant, a blocker
(`needs-a-fact` / `needs-a-subsystem` / `needs-a-language`).

## Before writing anything: three filters

Apply these in order, from [#84](https://github.com/tomerwave/godlint/issues/84). A practice
that fails one belongs in a different place than a rule proposal.

1. **Is it decidable without types?** Godlint has syntax and facts, not resolution. If the
   practice needs knowing what a value *is* rather than how it is *spelled*, it is a semantic-phase
   candidate or a "do not build" — write the research question instead, don't write a rule
   proposal that can't be built.
2. **What is the specific false-positive case, and can configuration express the exemption?**
   Not "is this possible" but the exact input where the rule is wrong, and whether a repository
   can turn that exemption off. If the exemption cannot be expressed, the rule will be disabled
   wholesale within a week of shipping — see [`network-timeout-required`](https://github.com/tomerwave/godlint/issues/99)
   for how Ruff's `S113` learned this the hard way.
3. **Is the threshold measurable?** If the rule has a number in it, it must be
   [measured against this repository](proposing-a-threshold.md), not borrowed from another
   linter's default.

## The issue shape

Every filed proposal in this backlog follows the same sections, because a reviewer reads dozens
of these and a consistent shape is what lets them scan instead of re-read:

| Section | What goes there |
| --- | --- |
| Policy problem | What defect this catches, and why it matters *here* — not a restatement of what the rule does |
| Blocked | Either "nothing — every fact and engine this needs already exists" or the specific fact/subsystem it needs, linked as its own issue if it unlocks more than one rule |
| Valid and invalid examples | Real code, not pseudocode, for every language the rule covers |
| Analysis scope | Which languages, and the scope note: what's spelling-based, what a fixture needs to prove |
| Diagnostic and remediation | The exact message format, and what a person does next |
| The false positive to design for | The answer from filter 2, stated as a scenario |
| Source | A verified link — see below |
| Definition of done | Copy verbatim from an existing filed issue; it does not change per rule |

## Verify every citation before it goes in the issue

A broken link in a proposal is worse than no citation — it looks authoritative and isn't. Before
filing:

```bash
curl -sS -o /dev/null -w '%{http_code}' -L -A 'Mozilla/5.0' "$URL"
```

Every source cited in this backlog was checked this way; three were wrong on the first pass and
were caught before filing, not after.

## Labels

- **Kind:** `rule-proposal` (this skill), `enhancement` (a fact or engine), `research` (an open
  question), `tech-debt`.
- **Priority:** `P1` nothing blocks it and it's high value; `P2` clear value, blocked on an
  enabler; `P3` worth having, lower value or needs careful config.
- **Blocker:** `needs-a-fact`, `needs-a-subsystem`, `needs-a-language`, or none.
- **`good first issue`:** only if *everything* needed already exists — no new fact, no design
  question left open. Not a synonym for "easy to read."

## When an enabler unlocks more than one rule

File it separately (`enhancement`), and link every rule that needs it. When it unlocks exactly
one rule, fold it into that rule's issue under "Blocked" instead — a fact nobody else needs isn't
its own line item on the roadmap.

## Reconsidering an existing verdict

A rule already marked "do not build" can be reopened if the reasoning was wrong, not just because
it's tempting. State exactly what changed — a new fact exists, a false-positive concern was
overstated, a stricter reading of scope turns out to be enough. Several rules in this backlog
were reclassified this way; each says so in its own issue.
