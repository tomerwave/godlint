# Skill: add a rule

Implements a rule that already has an approved proposal (see
[propose a rule](propose-a-rule.md) if it doesn't yet). This checklist mirrors
`scripts/validate-pull-request.py` exactly — every item here is a real check that runs in CI, not
a style preference. If you do all of these, the validator passes; if you skip one, it tells you
which.

## The places one rule touches

| # | File | What's required, and why a miss is silent |
| --- | --- | --- |
| 1 | `crates/godlint-core/src/rules/mod.rs` | `pub mod <name>;` |
| 2 | `crates/godlint-core/src/rules/mod.rs` | `<name>::evaluate` listed in `EVALUATORS` — without this the rule compiles and never runs |
| 3 | `crates/godlint-core/src/config/mod.rs` | A field on `Rules` renamed to the identifier via `#[serde(rename = "family/name")]` — without this the rule cannot be configured |
| 4 | `crates/godlint-core/src/rules/registry.rs` | `id: <Struct>::ID` in `REGISTRATIONS` — without this a suppression naming the rule is silently treated as a typo, and `policy/unused-suppression` can never count its suppressions as used |
| 5 | `crates/godlint-cli/tests/fixtures/rules/<slug>/` | A fixture directory: per-language `example.*` files, `godlint.yaml`, `expected.yaml` |
| 5b | `crates/godlint-cli/tests/e2e.rs` | Each new fixture directory declared, or `every_fixture_directory_is_declared` fails with two long sorted sets and no hint which name is missing |
| 6 | `crates/godlint-core/tests/rules/<name>.rs` | Unit tests |
| 7 | `crates/godlint-core/tests/rules.rs` | The unit test file declared, or it never runs |
| 8 | `docs/rule-roadmap.md`, `docs/rules.md`, `CHANGELOG.md` | All three must mention the identifier |
| 9 | `godlint.yaml` | The rule dogfooded — named directly, or covered by an adopted suite |
| 10 | fixture set | At least one fixture where it **fires**, and at least one where it is **configured and stays silent** — proving both directions, not just the positive case |
| 11 | `docs/rules.md` support matrix | A row marking each dialect the rule covers. A dialect it cannot cover needs `Rule::LANGUAGES` to say so and why, or the row and the declaration disagree and `tests/languages.rs` fails |
| 12 | fixture set | A fixture that reports the rule in **every dialect the row claims** — a `✓` nothing fires in fails, and so does a fixture firing in a dialect the row marks absent |

Existing fixture directories run 3–12 files each; look at a rule in the same family before
guessing the shape.

## Two more places, only if the rule carries a numeric threshold into `recommended@1`

If the new rule has a `max-*` field and you add it to a suite, two hand-maintained tables outside
the places above also need the new entry, or the build fails with a message that does not look
like it is about your rule at all:

- `crates/godlint-core/tests/suites.rs`, the `configured_line_limits`/`configured_count_limits`
  functions — asserted against `docs/rule-roadmap.md`'s `recommended@1` table with the message
  *"the table is the source"*. Miss this and `recommended_carries_the_documented_thresholds` fails.
- `crates/godlint-core/tests/rules/registry.rs`, the `limits()` function — a minimal
  `key: 1` snippet used to prove every registered rule reads only its own configuration. Miss this
  and `every_registered_rule_reads_its_own_configuration` fails with a YAML parse error that names
  the missing field, not the missing table entry.

Neither is optional, and neither is discoverable from the checklist above — both were
found by running the full test suite, not by reading a spec.

## Two different things are both called "coverage" — don't confuse them

- **Fixture coverage** (item 10 above, checked by `validate-pull-request.py`): does a fixture
  *exist* proving the rule fires and proving it stays silent. This is presence, not percentage.
- **Line coverage** (`scripts/check-rule-coverage.py`, run against `cargo llvm-cov` output): what
  percentage of the rule's own source lines executed during the whole test run. This one **fails
  in both directions** — under budget is undertested, comfortably over budget means the budget
  stopped meaning anything and should come down.

Run both before opening a pull request:

```bash
python3 scripts/validate-pull-request.py
cargo llvm-cov --workspace --json --output-path coverage.json
python3 scripts/check-rule-coverage.py coverage.json
```

## The fixture obligation from #88

`reliability/empty-error-handler` shipped mutation-clean, passed every gate above, and still
missed its main case — `except ValueError: pass` — because every fixture and every unit test used
only a bare `except:`. The tests were written from the same blind spot as the code.

**Write a fixture for every syntactic form that produces the underlying fact, not one fixture per
rule.** If the rule reads a Python `except` clause, write one with no exception named, one with a
named exception, one with `as` binding, one with a tuple of exceptions. If it reads a JS `catch`,
write one with and without the bound parameter. The rule is only as good as the shapes it was
checked against.

## Deciding what to put in the support matrix

The default is that a rule covers every dialect, so the honest question is where it does not, and
that answer belongs in `Rule::LANGUAGES` rather than in a match arm inside the rule. Two reasons are
distinguished because a reader needs them to be: `Absence::NoSuchConstruct` for a language that
cannot express what the rule reports — Rust has no `catch` block to find empty — and
`Absence::NotImplemented` for a construct that exists and has not been taught yet, which is a gap to
close.

Do not answer from the rule's source alone. `architecture/no-internal-import` reads as though it
handles Rust, and returns early for every Rust path six lines in. Item 12 is what settles it: write
the fixture, and either it reports or the claim was wrong.

## No comments in `src/`

`style/no-comments` runs at `error` over this repository's own source, including new rule code.
Reasoning that would have been a comment belongs in a doc under `docs/`, linked from the
architecture guide if it explains a shared mechanism, or from the rule's own row in
`docs/rules.md` if it's specific to that rule.

## Order that avoids rework

1. Confirm the fact this rule needs already exists (`crates/godlint-core/src/facts.rs`). If not,
   that's a different skill — a fact is its own change, reviewed on its own.
2. Write the fixtures first: valid, invalid, and — where the rule can be misconfigured — a
   fixture proving that too. Fixtures are the spec; write them before the rule reads them wrong.
3. Implement the rule and wire in items 1–4.
4. Write unit tests (items 6–7), covering every syntactic form per the fixture obligation above.
5. Dogfood it (item 9) and run `godlint check .` against this repository — if it fires here,
   fix the finding or reconsider the rule, don't suppress it to make the number look clean.
6. Documentation and changelog (item 8) last, once the rule's actual behaviour is settled — not
   before, or the docs describe an intention rather than what shipped.
