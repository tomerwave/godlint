## Rule

<!-- The rule ID, for example maintainability/parameter-count. -->

## Policy this enforces

<!--
State the organization-level decision the rule makes, in one paragraph. A rule earns its
place by encoding a decision a team makes once, not by matching a name another linter
uses. See docs/rule-roadmap.md for the product boundary.
-->

## What counts, per language

<!--
Name the construct this rule measures in Rust, in Python, and in JavaScript/TypeScript,
and say explicitly where a language has no counterpart. One threshold across three
languages only means something if the thing being counted means the same in each.
-->

| Language | Counted | Not counted |
| --- | --- | --- |
| Rust | | |
| Python | | |
| JavaScript/TypeScript | | |

## Threshold and its source

<!--
Where does the recommended number come from? "Godlint ships no hidden universal number."
If it was borrowed from another tool, confirm that tool measures the same thing.
-->

## False positives considered

<!--
Which idiomatic constructs would a naive implementation report wrongly, and what does
this one do about them? Name the ones you tested.
-->

## Validation

<!-- The checks you ran and their results. -->

## Checklist

- [ ] The fact is derived from the analyzer, not re-lexed in the rules layer.
- [ ] Node-kind knowledge lives in the language vocabulary, not in shared code.
- [ ] Unit tests analyze real source rather than injecting metric values.
- [ ] Tests cover the constructs where the languages diverge, not only where they agree.
- [ ] A test pins the exact threshold boundary.
- [ ] Fixtures cover valid, invalid, and configuration cases, plus scoped exclusion.
- [ ] The rule is enabled for Godlint in `godlint.yaml`, or the omission is justified here.
- [ ] `docs/rule-roadmap.md`, `README.md`, and `CHANGELOG.md` record the rule.
- [ ] Output is deterministic and ordering does not depend on message wording.
