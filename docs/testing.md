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

Run it over the rules layer with:

```bash
cargo install cargo-mutants
cargo mutants --file 'crates/godlint-core/src/rules/*.rs'
```

A pull request that touches a rule runs this over the lines it changed, with
`--in-diff`, so the check stays proportionate to the change and asks the question that
matters for review: are the decisions this change introduced exercised by anything? A
weekly run covers the whole rules layer, which is where coverage that has rotted rather
than newly arrived is caught.

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
5. A pinned real-world corpus for runtime and false-positive regression measurement.
