# Testing strategy

Godlint is fixture-first and end-to-end biased. The primary proof of rule behavior is:

```text
source fixture + godlint.yaml -> normalized expected diagnostics
```

Every rule needs valid, invalid, and configuration fixtures, plus coverage of scoped
exclusion: a rule must stay silent on a path the top-level `exclude` globs remove from
the scan. Shared rules need equivalent Rust, TypeScript/JavaScript, and Python cases
when the concept applies. Repository rules need miniature realistic repositories rather
than mocked dependency graphs.

Inline suppression is a fourth fixture class, and it is intent rather than an enforced
gate today. The `suppression-applies` fixture shows a directive silencing
`maintainability/function-nesting`, `maintainability/empty-function` and
`policy/todo-requires-reference`; the remaining function rules are covered by the shared
mechanism rather than one fixture each, and `scripts/validate-pull-request.py` does not
require a fourth class. Extending it per rule is worth doing, but claiming it is required
while nine rules lack it would make this document the thing that is wrong.
`policy/unused-suppression` is what keeps a directive from outliving the finding it hid.
File-level findings remain unsuppressible inline because their location has no preceding
declaration; test their configured path exclusion instead.

Keep test code outside production `src/` modules. Public crate contracts belong in
`crates/<crate>/tests/`. Rule fixtures belong in
`crates/godlint-cli/tests/fixtures/rules/<rule-id>/`, each with its own `godlint.yaml`
and `expected.yaml` covering valid, invalid, configuration, and exclusion behavior. This
keeps source modules focused on the shipped implementation and makes the rule contract
easy to inspect.

Use focused integration tests for small deterministic invariants that are hard to
diagnose through rule fixtures: configuration merging, glob behavior, source ranges,
fingerprints, cache keys, diff parsing, and graph algorithms.

A test that needs scratch files on disk takes them from `TemporaryDirectory` in each
crate's `tests/support/temporary.rs`, never by building a name under the shared temporary
root. A name built from a clock and a process-local counter is not unique across
processes: `SystemTime::now` advances in microsecond steps rather than per call, and every
process allocates the same first counter value, so two concurrent test processes can agree
on a path. `TemporaryDirectory` claims its directory with a single `create_dir`, which
fails rather than sharing, and retries on the one collision a recycled process id can
still produce. It removes the tree on drop, so a suite leaves nothing behind for the next
run to read.

## The real-world corpus

Nine repositories pinned to a commit, listed in `corpus/repositories.json` and checked by
`scripts/check-real-world.py`. Four are single-language and small, and read cleanly. Five are there
because they are awkward: Deno mixes Rust and TypeScript, Sentry is a Python and TSX monolith,
Home Assistant is eighteen thousand Python files, and VS Code is the largest TypeScript tree.

The gate is **unreadable files, never findings.** Findings change whenever a rule changes, so
gating on them would fail on every rule this repository ships and would be switched off within a
week. A file Godlint cannot read is a defect whatever the rules say, and it is the failure that
hides: the file contributes nothing and the loss leaves no trace in a findings count.

Each repository carries a budget rather than a list of paths, because one of them is at four
hundred and enumerating those would bury the reason under the data. The budget fails in both
directions, exactly as the rule-coverage one does: above it is a regression, and below it means a
grammar learned the syntax and the number is now reserving silence for the next failure.

Writing it down found three grammar gaps that no fixture would have, because a fixture is written
by someone who already knows the syntax:

| syntax | grammar | seen in |
| --- | --- | --- |
| `interface A<in T>`, TypeScript 4.7 variance | `tree-sitter-typescript` | 4 files in Zod |
| ``styled('a')<{x?: boolean}>`css` `` | `tree-sitter-typescript` (tsx) | 408 files in Sentry |
| `class A[T = int]`, PEP 696 defaults | `tree-sitter-python` | 11 in Home Assistant, 3 in Sentry |

Each is a construct real projects compile and ship today. Before Godlint judged the part of a file
that parsed, every one of those files contributed nothing at all.

## Proving the fixtures are adequate

A fixture proves a rule fires. Nothing about a passing suite proves the suite would have
noticed had the rule changed, and that is the property that matters: a rule whose
decisions no test exercises can be altered silently.

Two mechanisms establish it.

Every rule must have a fixture that reports it and a fixture that configures it without
reporting it. The first proves the rule fires; the second proves it stays quiet on
conforming code, which is the half a rule that always fires would also satisfy.
`scripts/validate-pull-request.py` enforces both.

Mutation testing then asks the question directly. `cargo mutants` alters a rule — inverts
a comparison, drops a negation, returns a fixed value — and reports any alteration the
suite still accepts. A surviving mutant is a decision no test depends on, so it names the
missing case rather than the missing file. This is how the threshold boundaries were
confirmed to be genuinely covered, and how an `allow-names` comparison was found to be
loosenable from exact match to prefix match with the whole suite still green.

Run it with:

```bash
cargo install cargo-mutants
cargo mutants
```

`examine_globs` in `.cargo/mutants.toml` decides which files carry mutants, so `--file` is
not how to narrow a run — the configuration wins, and a `--file` glob quietly does nothing.
`-F <regex>`, matched against the names `--list` prints, does narrow one. To ask only about a
change, ask about its diff:

```bash
git diff origin/main...HEAD > changed.diff
cargo mutants --in-diff changed.diff
```

That is what a pull request runs, so the check stays proportionate to the change and asks
the question that matters for review: are the decisions this change introduced exercised by
anything? A weekly run covers every examined file, which is where coverage that has rotted
rather than newly arrived is caught.

Three layers are examined, and the third is the one worth arguing about. The rules layer
decides what is reported and `suppression.rs` decides what is silenced. The analysers decide
what is *seen*, and that is where a false negative hides best: a rule that reports the wrong
thing fails a fixture loudly, while an adapter that never emits the fact produces silence,
and silence is what a passing suite looks like. `reliability/empty-error-handler` was
mutation-clean for its whole life while the adapter beneath it resolved a Python `except`
body by position, so the two forms real code writes reported nothing.

The pull-request job used to narrow the diff to `rules/` as well, which meant an analyser
change was mutated by the weekly sweep and by nothing that could block a merge — the gate
existed on the day it was needed and was pointed elsewhere. The job now diffs the whole tree
and lets `examine_globs` decide, and `check_mutation_scope` in
`scripts/validate-pull-request.py` fails if an examined path stops triggering the job. One
list, checked, rather than two that agree by habit.

`cargo-mutants` is a development tool rather than a dependency. Nothing in `Cargo.toml`
requires it, and it does not appear in anything Godlint builds or ships.

Three things about reading its output. A mutant reported as unviable did not compile,
which is not a gap. A mutant that survives is usually a decision no test depends on, and
the answer is to write the case. Occasionally it is an equivalent mutant: an alteration
that cannot change results, such as failing to advance a cursor that only skips work.
No test can distinguish one of those, so it belongs in `exclude_re` with the reason beside
it, and `scripts/validate-pull-request.py` requires every exclusion to carry one.

The first run of this over the rules layer found four survivors in code that had passed
review and a full suite: two in error reporting that no test constructed, one genuine
off-by-one where code begins at the byte a block comment ends, and one equivalent mutant.
That is the argument for the tool in one sentence.

It is not a proof of coverage, and should not be read as one. A newly added branch can
still be untested while every mutant of it is caught, because altering the line can break
behaviour that other tests do cover, and that failure is enough to mark the mutant caught.
This was measured: adding an unexercised exemption to `empty-function` produced three
mutants, all caught, and the exemption itself remained untested. Coverage closes that particular hole, because it asks the narrower question: was this
line ever executed? An unexercised branch is an uncovered line however well the rest of
the file is tested.

```bash
cargo llvm-cov --workspace --json --output-path coverage.json
python3 scripts/check-rule-coverage.py coverage.json
```

The budget is a count of lines rather than a percentage, because a percentage loose enough
to tolerate the known residue is also loose enough to hide a new two-line branch. It stood at
nine, eight of them error propagation from `SourceFile::location` that could not fire; making
a range valid by construction deleted that plumbing rather than documenting it, and the
budget is now two. Both remaining lines are named in the script beside the number: one match
arm kept for exhaustiveness, and one range that can only narrow a range already checked.

That collapse is also the argument for the budget failing in both directions. Removing the
plumbing left the numbers above reality, and the script reported it — a budget that only
catches new uncovered lines would have kept nine lines of slack available for the next
untested branch to hide in.

The two mechanisms answer different questions and neither replaces the other. Coverage
asks whether a line ran; mutation testing asks whether anything would notice if it changed.
Deciding that a new case deserves a fixture is still a reviewer's job, which is why the
rule template asks for it.

The validation stack is:

1. Focused crate-contract tests for deterministic primitives.
2. Rule fixtures for behavior and false-positive boundaries.
3. CLI/repository integration tests for product contracts.
4. Mutation testing over the rules layer, to establish that the fixtures and unit tests
   exercise the decisions a rule makes.
5. A pinned real-world corpus, which asks whether Godlint can still read code that ships.
