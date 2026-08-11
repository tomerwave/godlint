# Changelog

All notable changes to Godlint will be documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). Before `1.0`, a `0.x` release may
change the `godlint-core` API; the command line and the configuration schema are what the version
speaks about.

## [0.8.0] - 2026-08-11

### Added

- `ci/untrusted-github-env` reports attacker-influenced expressions in scripts that write to
  `$GITHUB_ENV` or `$GITHUB_PATH`, where later workflow steps can inherit a changed value.
- Homebrew users can install Godlint with `brew install tomerwave/tap/godlint`. Releases update the
  formula from the verified macOS archives after publishing their GitHub release.

### Fixed

- The mutation gate now examines path handling, exclusion matching, and suppression dates instead
  of leaving those finding-affecting decisions in the unmutated list.
- Workflow facts now expose typed boolean policy values and unquoted run bodies, while JSON scan
  issues carry their dialect so consumers do not reimplement workflow classification.
- Rules now enforce their declared language support at the reporting boundary, so a finding cannot
  be emitted for a dialect marked as absent by `Rule::LANGUAGES`.
- `Violation::cap` now requires every violation variant to choose its severity cap explicitly, so
  adding a new diagnostic cannot silently inherit the uncapped error default.

### Internal

- Godlint's own repository now installs and self-updates the shared Godharness configuration.

## [0.7.0] - 2026-08-05

### Added

- `git/branch-naming` checks pull-request branches through `GITHUB_HEAD_REF` and local checked-out
  branches through Git. It replaces Godlint's bespoke branch-name script and separate pull-request job,
  runs in `recommended@1`, and allows repositories to replace the accepted types or admit automation
  branch patterns.

## [0.6.2] - 2026-08-05

### Fixed

- Project `exclude` patterns now extend Godlint's built-in exclusions instead of replacing them.
  Adding a repository-specific exclusion therefore continues to skip dependency directories, caches,
  and generated output by default.

## [0.6.1] - 2026-08-05

### Fixed

- `architecture/filename-case` now ignores framework-required dynamic route filenames beginning with
  `[name]`, `[...name]` or `[[...name]]`. Astro and Next.js use those segments as routing syntax, so
  renaming them changes the route rather than correcting a convention. Malformed bracketed names remain
  findings.

## [0.6.0] - 2026-08-04

### Added

- Rules take `only-in` and `allow-in`: the paths a rule applies to, and the exemptions inside them.
  `allow-in` existed on eleven of fifty rules, each implementing it itself, and `only-in` did not exist
  at all — so a rule that is inherently about one part of a tree could not say so. The only way to say
  "this rule does not belong here" was `exclude`, which drops a path for **every** rule at once: one
  misplaced rule cost you every other rule in that directory. This repository is the evidence, having
  excluded `scripts` and `packaging` wholesale to silence one rule, hiding 21 unrelated findings to do
  it. This change does not lift that exclusion, and it does remove the reason for it: every one of
  those rules can now be declared per path, so what keeps the two directories out of the scan is the
  21 findings underneath, each wanting its own change.

  The narrower setting decides, so `allow-in` carves exceptions out of `only-in`, and both empty means
  every file, which is what a rule naming no paths wants.

  It is one implementation rather than fifty because the check sits in `rules::report`, the single
  function that turns a violation into a finding. A rule cannot forget to honour its own scope, because
  no rule consults it. The eleven hand-written `allow-in` checks are gone, matching the same globs
  against the same path as before. `Rule::Configuration` now requires `Scoped`, so a new rule does not
  compile until its configuration can say where its rule applies — the property is held by the compiler
  rather than by a checklist.

  One rule needed more than that, and it is the interesting one. `ci/stale-action-refs` reports a
  contradiction *between* files: one commit labelled `# v3` in one workflow and `# v4` in another. Scope
  there has to gate what the rule *reads*, not only what it reports — otherwise excluding a workflow
  still produces a finding in the file that was not excluded, caused by the one that was. Its own test
  said so in its name, `allow_in_removes_a_workflow_from_reporting_and_repository_evidence`, and it
  failed the moment the central check replaced its own. A rule whose verdict depends on more than the
  file it reports in must scope its evidence. Review found the other half of that untested — evidence
  honouring `allow-in` while ignoring `only-in` passed all 881 tests — so the mirror case is pinned
  too — in both directions. Review then found a third shape: both scope tests assert that nothing is
  reported, so an evidence filter that *over*-drops was unpinned, and dropping every workflow the
  moment `only-in` is non-empty passed all 882 tests. That is the worse failure of the two, because
  `only-in` is the setting whose purpose is to point a rule *at* something. The fixture that already
  demands five findings now sets `only-in` as well, so those five expectations pin the positive
  direction and the same block pins the interaction — `allow-in` still carves `allowed.yml` out of an
  `only-in` that includes it. Its three-pattern list is load-bearing for a second reason found while
  reviewing it: it is the only multi-pattern `only-in` anywhere in the suite, so it is the only thing
  pinning that `only-in` matches as a disjunction over every pattern rather than checking the first.
  A fourth shape came out of the same review — no test had a non-empty `only-in`, a file outside it,
  *and* findings expected from the files inside, so narrowing that must still report was unpinned and
  a guard written at the wrong granularity silenced the rule whenever anything was out of scope. Two
  workflows in scope contradicting each other, one outside, now pin it. Four directions each for
  `only-in` and `allow-in`: reports inside, silent outside, the interaction between them, and the
  evidence path separately from the reporting path.

  One sharp edge, documented rather than fixed: `only-in` narrows, so a pattern matching nothing
  leaves a rule with nowhere to apply and it reports nothing, anywhere, without saying so. A typo in
  `exclude` or `allow-in` fails safe because a pattern matching nothing changes nothing; a typo here
  fails open. A misspelled key is still caught by validation, only a misspelled path is not.

### Changed

- `policy/unused-suppression` reports a directive that silences nothing whatever the reason, where it
  previously excused a rule set to `off`. Three things can make a directive dead — the finding was
  fixed, the rule is `off`, or the rule is scoped away from that path by `only-in` or `allow-in` — and
  the rule reported the first and third while treating the second as dormant. That split was not a
  policy, it was two answers to one question: `off` was excused deliberately so a gradual adoption
  would not turn the inactive parts of a policy into failures, and scope arrived later, took the
  opposite answer, and nobody reconciled them.

  Reported in all three now, because a dormant exemption and a dead one are indistinguishable from
  outside, and only one is harmless. A directive nobody is watching silences a real finding the day
  the severity or the scope changes, un-reviewed — the same reasoning that made a stale drift
  declaration fail rather than notice. The gradual-adoption cost is real and now written down in
  `docs/suppressions.md` along with the way to pay it: this rule takes a severity like any other, so
  `warning` covers a cleanup in progress.

  A fourth case belongs in that list and was missing from the first draft of this entry: a rule the
  configuration never mentions at all never runs, so a directive naming it is as dead as one for a
  rule set to `off`. Reported too, and tested.

- `policy/unused-suppression` cannot switch itself off. `severity: off` is now rejected as invalid
  configuration, and so are `only-in` and `allow-in` on that rule — scoping a rule to nothing is
  switching it off by another name, and `only-in`/`allow-in` reached every rule two releases ago,
  which quietly gave the one rule meant to be undefeatable two new ways to be defeated. A rule able
  to retire itself could retire every exemption it audits. `warning` is still accepted and is how to
  absorb a cleanup without failing the build.

  Scope of that claim, because a wider one would be false: it is the rule's own configuration that
  cannot retire it. A top-level `exclude` still drops a path from the scan for every rule including
  this one — Godlint's own `godlint.yaml` relies on it, and doing so hides 19 dead directives in the
  CLI fixture tree on purpose — and so does naming paths on the command line. And the check lives in
  configuration validation, which runs when the CLI loads a file, so a `godlint-core` consumer
  deserialising a `Config` directly is not bound by it. Making the shape unrepresentable rather than
  rejected is the version that would bind both, and is its own change.

  The message it prints changed with it, because the old one became false the moment the rule stopped
  requiring the target to be enabled: `Suppression does not silence an enabled finding; remove it or
  narrow the rule` said *enabled* when enablement is no longer the criterion, and offered *narrow the
  rule* as a remedy that does not exist when the rule is off everywhere. It now reads `Suppression
  silences nothing; remove it, or restore the rule it names to this path.` — a statement that is true
  in all four cases, and advice that does not tell a reader to delete a reviewed exemption they may
  want when the rule returns.

- A declaration in `.github/accepted-drift.md` that the released binary does not report fails the
  released-agreement check instead of printing a notice nobody reads. `docs/releasing.md` said the
  quiet part out loud — "deleting it is remembered rather than enforced" — and remembering is not a
  gate. What makes it worth failing is what a declaration is: it stands ready to accept disagreement
  in one named rule, so a line left behind after the drift it described is resolved will silently
  accept the *next* drift in that rule, which is the one case this check exists to catch. A stale
  declaration is not untidiness, it is an exemption nobody is watching. The pressure lands where the
  release process already expected it: the first pull request after a release either deletes the file
  or goes red naming each line to remove.

  Three things look like a stale declaration and are not, and calling any of them stale would have
  been worse than the notice it replaces, because the instruction is *delete the line*. Staleness
  reads "not among the rules the release reported", so it is only sound where that list is the whole
  list. It is not when the release could not read this repository's configuration — it reported
  nothing about any rule, so failing there would fail every pull request that adds a configuration
  key, for declarations that are perfectly good. It is not when a finding's rule id could not be
  parsed, because the unreadable one may *be* the declared rule. And it is not when the release
  claimed findings and the annotations hold none, which is no record rather than a record of nothing.
  None of the three is called stale. The configuration case reports its declarations *unexercised*
  and passes; the other two report them *not examined* and leave the run red on the findings they
  could not read, unless a drift label declares those, which is the escape a label has always been.
  A stale declaration also does not short-circuit the undeclared-finding report: a run with both
  says both,
  and names every stale line with its own kind rather than the first — the first attempt got the
  short-circuit wrong and the suite caught it, and review caught the rest.
- The drift gate reads the status the released binary exited with instead of matching a sentence in
  its output. It decided whether the release could read the configuration by grepping for
  `Configuration is invalid`, and the binary being grepped is a *past* release — so no test in this
  repository could hold that wording still, and rewording it in a later version would have silently
  reclassified an unreadable configuration as drift, the one failure the gate exists to prevent. It
  now reads the action's `status` output, which has to be fixed to exist at all for a run with
  findings (below), and when the status says the check did not finish it asks the release itself:
  `config validate` answers *can you read this configuration* with an exit status, and has since the
  first release, so every binary the gate can run understands the question. Two things follow beyond
  the plumbing. A release that cannot parse a file exits 2 having still reported what it did reach,
  and those partial findings were read as drift with a choice of labels offered; a verdict on part of
  a tree is not a verdict, so that now fails and says so. And the guidance the gate printed said
  adding a *rule* lands there, which stopped being true when a release started ignoring an unknown
  rule key with a notice — only a configuration key, a suite or a configuration version reaches it.
  A tree with no `godlint.yaml` at all is now reported rather than waved through: the release cannot
  read a file that is not there, so with no check for one a repository stating no policy read as a
  release too old to understand it, which is the same silent pass from the other direction.
  The step's own conclusion is still read, for the one question the status cannot answer: a step
  after the check failing leaves Godlint's own status honest while the action failed for its own
  reason. That the outcome is read at all was `ci/no-silenced-failure` reporting this repository's
  drift job the moment nothing did. `docs/ci.md` documents the `status` output, which existed
  undocumented, and what each status means.

- `validate-pull-request.py` asks for a changelog entry when any shipped source file changes, rather
  than when one of five hand-listed paths does. What the old list omitted decided the policy:
  `config/rules.rs` holds every default threshold, so **raising one — the most user-visible change
  this project can make — needed no entry at all**, and neither did `suites.rs`, which decides what a
  suite enables and at what severity. Exemptions are now named with a reason instead of inferred from
  a list of what someone remembered to include, and an entry that says nothing a user can observe
  changed is a valid entry, which is the sentence a refactor should be made to write. Those entries go
  under a new `### Internal` category that `check-release.py` leaves out of the release body, because
  the body is the section verbatim and reaches people who will never read this repository — so the log
  keeps the refactor and the announcement does not carry it. A file leaving `crates/*/src/` counts as
  a change to it: `git diff --name-only` resolves a rename to its destination, so moving a shipped
  module into `tests/` reported nothing at all until the diff was taken with `--no-renames`. The
  category names are constrained too, because the release body is name-sensitive: `### internal`
  would have shipped a refactor to users while looking right in the file.
- Suppression matching is grouped by file instead of comparing every finding with every suppression.
  `apply` scanned the whole suppression list for each finding and `policy/unused-suppression` scanned
  the whole finding list for each suppression, so the cost grew with the *product* of two
  repository-sized numbers — the only step in the pipeline that did — and nearly every comparison
  established that two unrelated files are not the same file. On 3,000 files carrying 2,000
  suppressions `godlint check` goes from 468ms to 339ms; doubling that corpus takes the pairwise path
  to 1,093ms and the grouped path to 616ms, so the gap widens as a repository grows. Grouping cannot
  change which suppression matches: `covers` already required the paths to be equal, and `Ord` on a
  path agrees with `==` on it, so the map admits and rejects exactly the pairs the scan did.
- Reading and parsing files runs on every core. The scan walked discovered files one at a time while
  read, parse and fact collection are independent per file and share nothing mutable — 85% of the run
  on one core. Chunks are merged in chunk order, so the facts arrive in the same order they did
  sequentially and the output does not depend on how the work was divided. Measured on a 2,104-file
  tree with ten cores: 1,244ms to 467ms, and eight consecutive runs are byte-identical. A tree of 32
  files or fewer stays sequential, and a second thread appears at 33 — measured at that boundary, the
  difference is inside the noise either way, because a run of that size is dominated by the 69ms it
  takes to start and read the configuration. The win is 2.66× on a large repository and nothing at all
  on a small one. Where a machine reports one core, `available_parallelism` returns 1 and the
  sequential path is taken.
- Deciding that a path is not excluded no longer allocates. `glob::segment_matches` built two
  `Vec<char>`s and a table per segment comparison before comparing anything, including for the
  literal patterns every `exclude:` list is made of — `target`, `node_modules`, `.venv`. Measured on
  a 2,104-file tree, `godlint check` goes from 1.61s to 1.50s, and the output is byte-identical over
  3,712 findings. Every rule's `allow-in` and `test-paths` matching takes the same path, so it is
  faster too. A pattern holding `*` or `?` still goes through the matcher unchanged.
- `validate-pull-request.py` refuses a changelog that names a release twice, lists a category twice,
  or holds an entry under no category. A conflict resolution that keeps both sides leaves a second
  `## [Unreleased]` behind; it renders, and it passed every other check, which is how two of them
  reached `main` in one night of rebases. The section they damaged is now one heading per category.
- `validate-pull-request.py` refuses a tracked file carrying a merge-conflict marker. `git rebase
  --continue` accepts a staged file whose conflict was never resolved, so a botched resolution lands
  as a commit that looks deliberate — which is exactly what happened while rebasing this branch, and
  all 1046 checks passed over a changelog full of `<<<<<<<`.
- `validate-pull-request.py` compares the mutation gate's scope with the tree rather than only with
  the mutation workflow's trigger paths. Twelve files in `godlint-core` — including the ones that
  decide which files are scanned, whether an `exclude` pattern matches, and whether a suppression
  has expired — generate no mutants at all, and nothing said so. Each is now named with the reason
  it is outside, a file that is neither examined nor named fails the check, and #245 carries the
  plan for bringing them in.
- `maintainability/cognitive-complexity` counts a Rust `let … else` as a branch, weighted by the
  nesting it sits at, the way every other branching form is counted. `decision-complexity` already
  counted it, so the two metrics disagreed about whether a refutable binding is a decision; a
  `let Some(value) = option else { return; }` now costs 1 at the top level and 3 inside an `if`.
  Nothing in this repository crosses the threshold of 15 as a result.
- Every function's metrics come from one walk of its syntax tree instead of five. Decision points,
  cognitive score, return paths, statement count and block depth each recursed the same subtree
  separately, re-reading every node's kind each time; on a 10,160-file tree that was 29% of the whole
  run, more than parsing. One traversal carries the nesting level, block depth and else position that
  the five walks each tracked alone. Measured on a 2,104-file tree, `godlint check` goes from 1.53s to
  1.27s, with identical output over 16,896 findings when every metric's limit is set to 1 so each
  function reports all five of its measured values — with the single deliberate exception below.

### Fixed

- The action's job summary appears when there are findings, which is the only time it was ever for.
  GitHub invokes a `shell: bash` step as `bash -e`, and a script's own `set -uo pipefail` cannot undo
  the `-e` it was invoked with, so the step ended at `godlint check | tee` the moment the check
  reported anything — before writing the findings count, before writing the status, and before the
  two steps after it, which a composite action skips once one fails. So `summary`, whose whole reason
  for existing is that GitHub renders only so many annotations and a first adoption produces
  hundreds, ran only against trees that had nothing to summarise, where it printed `No findings.`
  The `findings` and `status` outputs were empty for the same reason, and `docs/ci.md` explained that
  with the wrong cause — a composite action withholding its outputs when it fails, which measurably
  it does not — and a wrong explanation is why the real one went unexamined this long. Found by
  asserting in the `dirty` workflow job that the action fails *with findings*, where it had asserted
  only that it fails: an aborting step and findings failing a run look identical from outside, so the
  failure that job proved all along was the abort. Turning `-e` off covers the `tee` that writes the
  annotations as well, so its status is now checked rather than assumed — a half-written annotations
  file would otherwise have every count and every later step describing a shorter run than happened.
  The step now has a test that runs its own body, extracted from `action.yml` so the test cannot
  drift from the shipped step, under the shell GitHub uses. It was written after this step broke a
  second time in the same pull request, for the neighbouring reason: `PIPESTATUS` describes the last
  pipeline and an assignment is a command, so reading it on two lines read the second from the
  assignment. Both breakages were invisible from outside — the step failed, GitHub skipped the rest
  of the action, and the job failed the way findings fail. The test catches both, and catches
  `| tee "$output" || true`, the obvious fix that silently reports every run as status 0.
- A blank `helpers` or `test-paths` entry for `testing/no-test-helper-in-production` is rejected as
  invalid configuration. The two were broken differently: `helpers: [""]` matched the empty segments
  that splitting a Rust `::` path on one colon produced and reported `crate::tests::helper` with the
  message `names , which is test scaffolding`, while a blank `test-paths` entry matched nothing at all,
  so the option looked configured and did nothing. Every other list-valued option in the schema
  already refused a blank entry; these two were missed.
- Four decisions in the analysers had no test depending on them, which a full mutation sweep of
  `main` found: a Rust `use {std, core};` brace list must contribute no import, an ordinary `let`
  must not count as a branch where a `let … else` does, and `.tsx` must be parsed by the TSX grammar
  while `.ts` is parsed by the TypeScript one. That last pair reject each other's syntax in both
  directions — JSX under TypeScript and an angle-bracket type assertion under TSX — so swapping them
  broke nothing any test noticed until now.
- `validate-pull-request.py`'s change-scoped checks see the working tree, not only what is committed.
  They read `git diff <release line>...HEAD`, so a local run before the commit — which is most of them —
  found no changed files, skipped the checks, and printed that every check passed. Staged, unstaged and
  untracked paths all count now, and the changelog check consequently fails locally at the point the
  entry is missing rather than in CI.
- `ci/no-silenced-failure` reports `continue-on-error: True` and `TRUE`, not only the lowercase
  spelling. YAML's core schema calls all three true and GitHub honours each, so a capital letter
  silenced a step and the rule said nothing — a false negative in the one rule whose whole job is
  noticing a check that cannot fail. `yes`, `on` and a quoted `"true"` stay silent: they are not
  booleans in the core schema, and reporting them would rest on a guess about GitHub's coercion that
  cannot be checked without a network. Found by probing the built binary while reviewing the rule,
  not by reading it.
  plan for bringing them in. The walk covers every crate: `godlint-cli` was outside the gate in
  its entirety, including the module that decides the JSON, SARIF and annotation shapes three
  other gates parse.
- Two more gates in `validate-pull-request.py` stopped taking a proxy for the thing. The workflow
  toolchain check globbed `*.yml` while `source.rs` reads both `yaml` and `yml`, so a workflow named
  `.yaml` was scanned by Godlint and invisible to the gate; and "every mutation exclusion needs a
  reason" counted comment lines against exclusion lines, which passed one exclusion with a five-line
  essay beside four with none. Each exclusion is now paired with the line above it.
- The lists `recommended@1` enforces by default are pinned by tests. Nothing asserted them: every
  test passed its own markers, test paths and helpers, so deleting `XXX` from the marker defaults —
  which silently stops `policy/todo-requires-reference` asking for a reference on an `XXX:` comment
  in every repository using the suite — passed all 1,860 checks. This repository writes no comments
  in Rust, so its own dogfooding could not notice either. Found when a one-line pull request proposed
  exactly that change under a title claiming to add a marker.
- `maintainability/function-nesting` no longer charges a function for the blocks inside a closure it
  returns. A curried `a => b => { … }` reported the *outer* function's depth as the inner closure's,
  while `decision-complexity`, `cognitive-complexity`, `return-count` and `function-statements` all
  reported the outer function as empty — so one metric contradicted the other four and the rule
  reference, which says a closure's own complexity belongs to the closure. The inner closure still
  gets its own finding at its own depth. Found by review of the walk consolidation, which made the
  inconsistency visible; across 453,807 functions in a 26,404-file corpus this changes 127 functions
  in 51 files, all of them curried, and none in this repository.

### Internal

- Nothing a user can observe: the per-language module separator is defined once. Two rules kept their
  own copy — one returning a `char`, so Rust's `::` was halved to `:`, and one that used `/` for every
  language except Python, so a Rust path was a single segment. Neither was observable: splitting
  `crate::tests::helper` on one colon still yields `tests` between two empty segments, and the rule
  whose separator was wrong for Rust returns before consulting it, because `is_own` treats every Rust
  module as the file's own. That is why both survived. They now call `rules::module_path`, which has
  been right all along.
- Nothing a user can observe: `TextFile` hands out the text a range covers, so a rule no longer indexes
  into the file itself. Four rules and two fact accessors sliced `file.text()` with raw offsets, which
  is the position math `TextFile` exists to own. Two of them also did arithmetic on those offsets to
  strip an expression's braces; the scanner guarantees the shape that makes the arithmetic safe, so
  this is not a fix for a live underflow, it is one fewer place that depends on the guarantee.
  `slice` indexes directly, so a range built by another file panics rather than returning a plausible
  empty string — a rule that quietly declines to fire is worse than a crash, and `docs/architecture.md`
  now records that reasoning where the range guarantee is stated.
- Nothing a user can observe: a fact stores the details it was built from instead of copying them
  across field by field. `SourceFacts` re-declared all ten of `Collected`'s vectors and then moved
  them one at a time, and `FunctionFact`, `JobFact` and `StepFact` each re-declared their `Details`
  struct in full and copied every field. `TestFact` and `AssertionFact` already did the right thing,
  so the file was arguing with itself. 76 lines shorter, and adding an eleventh fact kind is one edit
  rather than three that have to agree. Nothing a user can observe means the command line: a
  `godlint-core` consumer printing one of these four types with `{:?}` now sees the fields nested one
  level deeper, under a name that is private to the crate. `Debug` is not a stability surface and no
  reporter, test or snapshot in this repository reads it, but it is not literally unchanged.

- Nothing a user can observe: a rule is registered in one row instead of three coordinated places.
  `registry.rs` held fifty six-line `Registration` literals, each naming a severity function that
  `registry/severity.rs` generated from a second macro, with the thirty-line import list written out
  in both files. A `registrations!` row now names the rule's type, its configuration field and
  whether it can be suppressed, and the identifier and language support are read off the type rather
  than restated — so those two cannot disagree with the rule at all, and the field is declared once
  instead of in two files. 628 lines across two files become 171 in one, and `severity.rs` is gone.
  Adding a rule was three coordinated edits plus two import lists, which is why
  `docs/skills/add-a-rule.md` never mentioned `severity.rs` and every new rule carried an
  undocumented obligation.

- Nothing a user can observe: twenty-one rule configuration structs are declared by two macros
  instead of by hand. Thirteen were exactly `{ severity }` and eight exactly `{ severity, allow-in }`,
  and `config/rules.rs` already had the parameterised form for this situation in `count_limit_rules!`.
  The file is 89 lines shorter, every type name survives, and the schema is unchanged — an unknown
  field still reports `` unknown field `allow_in`, expected `severity` or `allow-in` ``.
- Nothing a user can observe: `Display for Violation` states the four test violations as four arms.
  They shared one arm containing a four-way `if matches!(self, …)` chain that re-tested `self` to
  recover what the match had already decided — the only line in a fifty-arm table a reader could not
  scan. Exhaustiveness is unchanged, which is the property that arm exists to keep.
- Nothing a user can observe: five rules asked a shared question in their own private spelling.
  `security/forbidden-dependency`, `architecture/restricted-import` and `architecture/filename-case`
  each carried a copy of the path-glob match that `rules::catalogue` already provides, and
  `architecture/dependency-boundary` and `architecture/module-independence` carried byte-identical
  helpers for which declared set holds a file and which names an imported module. That pair is now
  `scoped::endpoints`, so a change to how a set claims a file happens once instead of twice.
- Also nothing observable: `logical_operator`, `opens_operator_sequence`, `is_else_if` and the
  comment-prefix lookup were byte-identical in the Rust and ECMAScript analysers, so they live in
  `analyzers::vocabulary` now, where the shared analyser helpers already are and where naming a
  grammar node kind is still in bounds. Python's versions genuinely differ and are untouched. A
  second `JobFact` constructor with no callers is gone; it filled defaults a workflow never has and
  would have silently set a job's body to its own first line.

## [0.5.0] - 2026-08-01

### Added

- `ci/stale-action-refs` — makes full commit pins reviewable without network access. It reports a pin
  without an inline version label at warning, and reports repository-proven contradictions at the
  configured severity when the same action and SHA carry different labels or the same action and label
  name different SHAs. A single leading `v` is normalised before comparing, because `v4.6.2` and
  `4.6.2` name the same release and reporting them as a contradiction would spend the rule's only
  asset — that it speaks when a label lies. `allow-in` removes paths from reporting and comparison.
  The rule deliberately cannot verify that a label names the pinned commit; zizmor's online
  `stale-action-refs` and `ref-version-mismatch` audits cover that external check.
- `ci/no-silenced-failure` — reports checks that cannot make a workflow fail: literal
  `continue-on-error: true` settings and scripts ending `|| true`, `; exit 0`, or `|| exit 0`. A
  same-job read of `steps.<id>.outcome` or `.conclusion` proves a deliberate soft step and stays
  silent. Corpus-common `continue-on-error` and `|| true` findings are capped at warning; explicit
  exit-zero endings stay at the configured severity.

### Changed

- `ci/no-monolithic-job` raises the `recommended@1` step limit from 7 to 20, the p90 of 231 jobs
  across 94 workflows in nine widely used repositories. The former limit came from this repository
  alone and reported 36% of real jobs. Raising it also made this repository's own `release.yml`
  exemption unnecessary, which is what confirmed the number was wrong rather than the workflow.
- `ci/no-inline-script` keeps its limit of 8 after the same measurement — across 981 real scripts it
  is p85, and 15% exceed it — so the number is now corpus-backed rather than repository-derived.
- A condition written without braces is read as an expression by every rule that reads conditions.
  GitHub treats an `if:` as an expression whether or not it is wrapped in `${{ … }}`, so
  `ci/bot-conditions` was missing the idiomatic spelling entirely and `ci/no-silenced-failure`'s
  escape hatch did not open for it. Both now share one reader.

### Fixed

- `style/no-comments` no longer reports a returned string literal as a docstring. The check confirmed
  the string was the first thing in a block but never that the string *was* the statement, so a Python
  function whose first statement returned a literal — `return "active"` — was reported as a comment at
  error under `recommended@1`. A real docstring, including a module docstring, still reports. Found by
  writing deliberately bad Python to probe for rules Godlint is missing.
- A declared drift in `.github/accepted-drift.md` survives the merge that lands it, so a pull request
  that deliberately relaxes a rule no longer has to be re-declared on every subsequent branch.
- `validate-pull-request.py` runs its change-scoped checks whether or not `--release-line` is passed,
  and fails when it cannot find a release line to compare against. Only CI passed the flag, so a local
  run reported one fewer check than CI ran and printed that all of them passed — including, on one
  branch, the check that then failed the pull request.

## [0.4.0] - 2026-07-31

### Added

- Godlint reads `.github/workflows/*.yml` as well as source, through a new `tree-sitter-yaml`
  grammar and `analyzers::workflow`. `WorkflowFacts` exposes every `uses:` reference with its
  owner, name and version and whether that version is a commit rather than a movable tag; which
  jobs a workflow declares; and whether `permissions` and `concurrency` are declared, at the
  workflow level or per job. `ci/pin-third-party-actions` is the first rule to read them and
  `ci/explicit-workflow-permissions` is next. A workflow is discovered by the same walk
  as source, skipped by the same `exclude` globs, and bounded by the same maximum file size.
  Reading the syntax rather than the text is what lets a `uses:` inside a comment, a string, or a
  step named `uses:` be ignored, and what gives a finding a real line and column.
- Workflow rules can now ask about each step and its settings, expressions inside YAML values,
  comments, job dependencies and reusable-workflow secrets, and literal container or service
  credentials. Every site retains its source range, so a rule can relate an expression to a command or
  condition without mistaking an example in a comment for executable workflow policy.
- Job workflow facts now expose a job-level `if:` range, so policy rules can apply the same condition
  checks to privileged jobs and privileged steps.
- A workflow whose YAML Godlint cannot read now reports `syntax not recognised at line N`, the same
  issue a source file reports, instead of contributing nothing silently. This found two fixtures of
  its own that were invalid YAML — a plain scalar may not contain `: ` — one of them the fixture
  asserting that a `uses:` inside a string is ignored, which was resting partly on the file failing to
  parse at all. `every_workflow_fixture_is_yaml_that_github_would_accept` in `e2e.rs` now refuses a
  fixture the grammar cannot read.
- `ci/bot-conditions` — reports step and job conditions that compare `github.actor` or
  `github.triggering_actor` with a configured bot identity. Those fields are attacker-influenced on
  several triggers, so the check proves nothing about who opened the pull request; compare its author
  or verify the app instead. `bots` defaults to Dependabot, GitHub Actions, and Renovate identities.
- `ci/explicit-workflow-permissions` — reports a job that runs with whatever the repository grants by
  default. What it reports follows the fix: a workflow declaring nothing anywhere is one finding at the
  file, because one line at the top closes it, while a workflow whose *other* jobs are already narrowed
  is reported per job that is still open, at that job's line. `require-per-job` additionally asks each
  job to narrow a workflow-level block, and is off in `recommended@1` because inheriting one is a
  choice a repository may have made deliberately. This replaces the check in
  `scripts/validate-pull-request.py` that looked for the string `permissions:` in each workflow, and
  counted a match inside a comment or a `run:` line.
- `ci/hardcoded-container-credentials` — reports literal usernames and passwords in job container
  and service credential blocks while leaving GitHub-expression interpolation alone.
- `ci/no-comments` — reports comments in workflow YAML except version labels trailing `uses:` values.
  YAML has no doc-comment equivalent, so the rule has no option beyond `severity`.
- `ci/no-inline-script` — reports a workflow `run:` script above its configured effective-line
  limit. `recommended@1` deliberately keeps 8 after measuring 981 scripts in 94 corpus workflows:
  it is p85 and 15% exceed it. Blank and shell-comment-only lines do not consume the default budget.
  The measurement is source-based, so compressed one-line command chains remain a documented
  boundary rather than a shell parser hidden inside a line rule.
- `ci/no-monolithic-job` — reports a workflow job above its configured step limit.
  `recommended@1` adopts 7 after measuring 21 jobs: p50 was 3, p95 was 7 and the maximum was 11.
  It counts independently reviewable and retryable steps; command aggregation inside a step remains
  `no-inline-script`'s concern.
- `ci/overprovisioned-secrets` — reports a step input or environment variable set to the whole
  `${{ secrets }}` context, including `toJSON(secrets)`, while named secret members stay silent.
- `ci/pin-third-party-actions` — reports a workflow step using a third-party action at a ref that can
  move. A tag, a branch or a version string can be repointed by whoever owns the action, and what they
  point at next runs in your workflow with your token; only a full forty-character commit SHA counts as
  pinned, and a short SHA does not, because it is neither what GitHub resolves nor collision-resistant.
  A local `./path` action, a `docker://` image, and any owner in `trusted-owners` are silent.
  `trusted-owners` defaults to `actions` and `github` — the accounts GitHub publishes from — because
  pinning those too is a policy decision rather than closing a hole; set it to `[]` to require every
  action to be pinned. Enabling it on this repository found five unpinned third-party uses across four
  workflows, now pinned. A workflow finding cannot be suppressed inline, because comment facts come
  from source and not from YAML; an `exclude` glob is the way to scope it.
- `ci/secrets-inherit` — reports `secrets: inherit` on a reusable-workflow call because it gives the
  callee every secret available to the caller; name the required secrets instead. Named secrets and
  no `secrets:` declaration are silent, and `allow-in` path globs scope trusted callers.
- `ci/template-injection` — reports attacker-influenced GitHub expressions interpolated directly
  into a workflow `run:` script, where the runner expands them before the shell sees the command.
  Expressions passed through `env:` or `with:` stay silent; binding the value to an environment
  variable and referencing it quoted is the documented fix. `allow-in` path globs scope exceptions.
- `ci/unredacted-secrets` — reports a `run:` script that combines a direct `secrets.*` expression
  with `$GITHUB_ENV` or `$GITHUB_OUTPUT`, where GitHub's masking no longer follows the value. It
  deliberately does not infer data flow through variables or earlier steps.
- A language support matrix in [the rule reference](docs/rules.md#language-support), recording for
  every rule which of the three dialects it covers, and distinguishing a language that has no such
  construct from one Godlint has not taught the rule yet. Each rule declares this as
  `Rule::LANGUAGES`; the matrix is asserted against the declarations, and
  `scripts/validate-pull-request.py` requires a fixture that reports the rule in each language it
  claims. Writing it down found `architecture/no-internal-import` claiming Rust while exempting every
  Rust path, which is now declared as the language having no such construct: `rustc` already refuses
  an import that reaches past another crate's public surface.

### Changed

- A rule name under `rules:` that this version of Godlint does not know is now reported and ignored
  rather than refusing the whole file. One configuration often has to be read by two versions at once
  — a pinned one in CI and a newer one locally — and a hard stop made adopting a rule an atomic
  upgrade across every consumer. The notice names the key, and the nearest rule it knows when there is
  one, because ignoring a *misspelling* is the case this makes dangerous. Everything else is still
  refused, and [the configuration guide](docs/configuration.md#what-a-version-may-read) states where
  the line is and why: an unknown rule key can only subtract the rule it names, while an unknown
  option, top-level key or suite name could make a run mean something other than the file says.
- `rules::evaluate` takes the workflow facts alongside the source facts. A caller inside this
  repository passes `&report.workflows`; a caller outside it that analyses no workflows passes `&[]`.

### Fixed

- `architecture/dependency-boundary`, `architecture/module-independence`,
  `policy/accountable-suppression`, `architecture/filename-case` and `security/forbidden-dependency`
  now have fixtures in every language they cover; each was previously proven in one language only. No
  rule behaviour changed — the gap was in what the corpus proved, and the new gate is what found it.
## [0.3.0] - 2026-07-31

### Added

- `testing/no-empty-test` — reports a test whose body does nothing, so it cannot fail. It reads the
  test's own body rather than any function inside it, so a test that registers an empty callback is not
  itself empty, and a test with no body to read at all such as `it.todo('later')` is left to
  `no-skipped-test`.
- `testing/no-sleep-in-test` — reports a test that waits on the clock instead of on the condition,
  which is the usual reason a suite passes locally and fails in CI. Python `time.sleep` and
  `asyncio.sleep`, Rust `thread::sleep` and `tokio::time::sleep`, and the JavaScript runners' own
  waits, `page.waitForTimeout` and `browser.pause`, plus JavaScript's commonest test sleep, which is a
  *shape* rather than a name: a `setTimeout` or `setInterval` inside a `Promise` whose only call it is.
  That condition separates a sleep from a timeout guard, where the promise also waits on an event — and no
  other linter appears to catch the idiom, since Cypress, Playwright and `no-hard-wait` all match a
  framework's wait API by name. The call must fall inside a test, so a helper in
  the same file may still sleep — and so may a `pytest.fixture` or a `beforeEach`, which is the more
  tempting hiding place and needs a fixture fact to see. A sleep reached through an alias is not reported,
  because that takes import resolution; and a mocked sleep under
  `patch("time.sleep")` is reported although it is instant, for the same reason the alias escapes.

- `testing/no-network-in-unit-test` — reports a test calling an HTTP or socket client from a path the
  repository has declared as unit tests. Such a test is slow, fails when a service is down, and cannot
  run offline; it usually also means the seam that should have been injected was not. Which test is a
  unit test is a fact about the repository rather than about the file, so the rule reports nothing until
  `unit-paths` names them, and `allow-in` carves exemptions back out of those paths for a mocked client.
  Being silent until configured puts it in an established category rather than a new one: six rules
  already ship in the suite at error with an empty list. `recommended@1` enables it at error and it stays silent
  until then, because guessing is worse in both directions: Rust's own convention puts integration tests
  in `tests/`, where reaching the real service is the point, and a repository with no such split would
  see every test reported. The fixture directory is the worked example; this repository cannot name the
  rule in its own `godlint.yaml` until the next release, because the configuration schema rejects an
  unknown rule key and the released-agreement check runs the published binary against this tree.
- `testing/no-randomness-without-seed` — reports a test drawing from a general-purpose generator in a
  file that never seeds one. A failure there cannot be reproduced, so the report is not actionable. The
  catalogue is shared with `security/no-insecure-random`, because the same call is unpredictable to an
  attacker and unreproducible to a reader. Seeding is read per file rather than per call, since
  `random.seed(1)` and `random.sample(...)` are separate calls: any seeding call exempts the file. That
  under-reports rather than over-reports, and `allow-in` covers a property-based suite that draws from
  the standard library on purpose. Rust gets its own remedy, because `rand::random` and `rand::thread_rng`
  cannot be seeded: there the message asks for a seeded `StdRng`, and a file that builds one is exempt.
  `rand::rng` is covered, being what `thread_rng` became in rand 0.9, and numpy is covered on both sides —
  it previously knew `np.random.seed` without knowing `np.random.rand`.
- Rules can now ask about a call that falls inside a test. `CallInTestRule` reads the call facts of a
  file, keeps only those a test's range encloses, and hands the rule the whole file's facts beside the
  call, so a rule can also ask what else the file does. That is the shape shared by `no-sleep-in-test`,
  `no-randomness-without-seed` and `no-network-in-unit-test`.
- `testing/no-focused-test` — reports a test or suite marked to run on its own, `it.only` and
  `describe.only` and the other runners' `.only`. A focused test that passes proves almost nothing,
  because nothing else ran.
- `testing/no-skipped-test` — reports a test that does not run: `.skip` or `.todo` in JavaScript and
  TypeScript, `#[ignore]` beside `#[test]` in Rust in either order, and a `pytest.mark.skip` or
  `unittest.skip` decorator in Python. A skipped test rots without anything noticing, so the rule asks
  for it to be deleted, fixed, or suppressed with an owner and an expiry.
- `architecture/no-internal-import` — reports an import that reaches past a package's public surface,
  coupling you to something nobody promised to keep. It reads the import path and nothing else, so a
  marker counts only after the first segment: `some-lib/src/deep` is reported and `src/utils`, a path
  alias to your own code, is not; `package._private.helpers` is reported and `from __future__ import
  annotations` is not. A relative import is always silent, because your own internals are yours to reach
  into. Two tiers: `internal`, `private`, `impl` and a Python `_` prefix say the author did not mean this
  for you and report at error, while `dist`, `src` and `build` merely name build output that some
  packages publish as their documented entry, so they report at warning; a path naming both is certain.
  Rust is out of scope — module privacy there is enforced by the compiler, so a module you can import is
  one its author made public. Two segment shapes are exempt for reasons that are not conventions: a scoped
  package's name spans two segments, so `@scope/internal` may be the whole package, and a Python
  `__dunder__` is a language protocol rather than an author's decision, so `import package.__main__` is
  silent. `allow` exempts a module the project must reach into.
- `security/no-shell-command` — reports a command run through a shell, where any interpolated value
  becomes executable. The three languages put the defect in three different places, so the rule reads
  three signals. Python's callee is innocent and the argument is the finding, so the check is callee-blind — any call
  passing a truthy `shell=` reports, which is what sees `sp.run(...)` and `run(...)` after an aliased or
  `from` import without listing either — and `shell=False` is read rather than merely looked for. JavaScript's callee is the
  finding — `exec` shells out, `execFile` does not — but the common spelling destructures it, so a bare
  `exec` counts only where the file imports `child_process` by `import` or `require`; without that
  import the same name is a regular expression's `exec`, so `pattern.exec(reference)` is silent. Python's
  bare names are read the same way, so `from os import system` then `system(cmd)` is reported, gated on the
  file importing `os`, `commands` or `subprocess` — and in both languages a name the file **declares
  itself** is never the module's, so a local `def system(x)` or `function exec(p)` is silent. Rust's
  program is the finding, so `Command::new("sh")` is reported and `Command::new("git")` is not. A literal
  command with nothing interpolated is reported too: it is not injectable today, but the argument-array
  form is no harder to write, and reporting only interpolated strings would mean deciding what
  interpolation looks like inside an f-string. `allow-in` exempts a release script.
- `testing/no-test-helper-in-production` — reports a production file importing its own test tree. That
  ships test scaffolding to users and inverts the dependency, so production depends on the tests, and it
  breaks any build that excludes them. Only a **local** import counts — `./`, `../`, a bare `.`, or Rust's
  `crate::`/`super::` — which is what keeps `some-lib/tests/util` silent, since a third-party package's
  test tree is its own business and cannot be shipped by you. Segments match whole and
  case-insensitively, so `Tests/` counts and `testing-utils/` does not. A file that is itself a test is
  exempt, because a test using its own helpers is the arrangement being protected; `test-paths` decides
  that and defaults to the conventions of all four languages, and `helpers` names the scaffolding
  segments. Setting either replaces the default rather than adding to it.
- `testing/assertion-required` — reports a test that asserts nothing. Such a test verifies only that the
  code does not raise, so it passes when the behaviour is wrong, which is the failure a test exists to
  prevent. It reports at **warning** whatever severity is configured, including inside `recommended@1`,
  because whether a test asserts through a helper is not decidable without resolution. It reuses
  `Violation::cap()`, though not in the same shape as `security/no-weak-hash`, which caps one of its two
  violations and keeps the other sharp; this rule has one violation and caps it, because there is no
  subcase where it can prove a test asserts nothing. `fail-on: warning` still buys a hard gate, but a
  repository-wide one rather than a per-rule one. Three shapes that look
  assertion-free are silent: `pytest.raises` and `#[should_panic]`, because asserting that something
  raises is asserting; a `describe` or other suite, because it asserts through the tests inside it; and an
  empty test, which is `no-empty-test`'s finding. For the helper case, `extra-assertions` names the
  functions a repository asserts through, so it configures the rule rather than turning it off.
- Assertion facts. A rule can ask which calls in a file are assertions, what each is called, and how
  many operands it took. Which calls count is a framework question rather than a language one, so each
  language module answers it, and each answers a different shape: Python has assertion syntax, so
  `assert value == 1` is a statement no call fact would have seen; Rust's assertions are macros, matched
  against the six names exactly; JavaScript has neither, so the fact reads the callee for `expect` and
  the `assert` module, including the type assertions `expectTypeOf` and `assertType`, which a typed suite
  may use to the exclusion of every other kind. Rust's `#[should_panic]` is recorded too — the attribute is the assertion, and
  without it every `should_panic` test would look assertion-free — at the function's range rather than
  the attribute's, so it falls inside the test that owns it. The names are explicit sets rather than an
  `assert` prefix, because a prefix
  claims a domain helper called `assert_invariant`. `expect(value).toBe(1)` is one assertion, not two —
  the matcher is a second call on the same chain — but its *range* spans the whole chain, because the
  matcher is what the assertion checks and without it `expect(v).toBe(1)` and `expect(v).toBeGreaterThan(0)`
  are indistinguishable. An assertion also carries its own text, so
  `no-duplicate-assertion` can compare two of them. Whether an operand was the *message* is not
  recorded: that needs a per-name arity table for three ecosystems, and a wrong one would demand a
  message from Jest's `expect`, which has none. Three boundaries are deliberate: a path-qualified macro
  such as `static_assertions::assert_eq!` is not recorded, nor is `should`-style JavaScript, nor
  `raises(...)` reached through an aliased import. This unblocks `testing/assertion-required`,
  `no-conditional-test-logic`, `no-duplicate-assertion` and, with that table, `assertion-message-required`.
- Test facts. A rule can ask whether a declaration is a test, what its name is, which marker made it
  one, and whether that marker carried focus or skipping. What counts as a test is a framework
  question rather than a language one, so each language module answers it: Rust reads the attributes
  preceding a function, which stack, so `#[test]` and `#[ignore]` in either order describe the same
  test; Python reads a `test_` prefix or a `pytest.mark` decorator; JavaScript and TypeScript read a
  runner call and its member, so `it.only` and `describe.skip` carry focus in the name. The fact stops
  at syntax: a rule that wants to treat a path as a test directory combines the fact with a glob,
  because an analyzer sees no configuration. This unblocks `testing/no-focused-test`,
  `no-skipped-test`, `no-empty-test`, `no-sleep-in-test`, `no-randomness-without-seed` and
  `no-network-in-unit-test`, each of which asks about other facts falling inside a test's range.
  `no-conditional-test-logic` needs more than this fact and is not among them: the problem is an
  assertion reachable on only one path, and knowing what an assertion is takes a fact that does not
  exist yet.

## [0.2.0] - 2026-07-30

### Added

- A real-world corpus gate: `corpus/repositories.json` pins nine repositories to a commit and
  `scripts/check-real-world.py` requires that Godlint can still read them. The gate is unreadable
  files, never findings, because findings change whenever a rule changes while a file Godlint cannot
  read is a defect whatever the rules say. Each repository carries a budget that fails in both
  directions, as the rule-coverage one does. Four of the nine are awkward on purpose: Deno mixes Rust
  and TypeScript, Sentry is a Python and TSX monolith, Home Assistant is eighteen thousand Python
  files, and VS Code is the largest TypeScript tree. Writing it found three grammar gaps that no
  fixture would have: TypeScript 4.7 variance annotations, a generic type argument on a tagged
  template as `styled('a')<{x?: boolean}>` (408 files in Sentry alone), and PEP 696 type-parameter
  defaults.

- Call facts carry their arguments: whether each one was positional or keyword-named, and its literal
  value where the value is a literal. A value that is not a literal reads as present with an unknown
  value and is never guessed at, which is what lets a rule stay silent instead of reporting a maybe.
  Quoting and string syntax are per-language judgements, so each language module decides what a literal
  is and what its value is; the extractor names no node kind.
- A rule can report one finding below the severity it is configured at, when it is sure something is
  worth saying but not sure enough to block. The configured severity stays a ceiling the repository
  sets: a cap can only lower a finding, never raise it, so a rule configured at `info` still reports
  `info`. One shared line applies it, and every existing rule is unchanged.
- `security/no-weak-hash` now also reports a broken algorithm named by a literal argument to a hash
  factory: `crypto.createHash("md5")`, `crypto.createHmac("sha1", …)`, and `hashlib.new("md5")`,
  case- and separator-insensitive so `MD5` and `sha-1` count. It covers JavaScript and TypeScript as a
  result, which it previously did not. `crypto.createHash(algorithm)` reports at **warning** with a
  message saying the algorithm could not be read — SonarJS reports the same case as an ordinary finding
  and is wrong whenever the value is SHA-256, so the severity carries the uncertainty instead.
- `security/no-insecure-random` — reports a general-purpose random generator, which is predictable by
  design: JavaScript `Math.random` and `crypto.pseudoRandomBytes`, Python's `random` module, and Rust
  `rand::random`/`rand::thread_rng`.
  The message names the secure generator of the language it reports in — `crypto.getRandomValues`,
  `secrets`, or `rand::rngs::OsRng` — which is the thing a configured call list cannot do. `allow-in`
  exempts a path where unpredictability is not the point, such as jitter or a test fixture. The first
  rule built on the shared call catalogue: a table, a message, and a per-language remedy.
- `architecture/module-independence` — reports a dependency between modules a repository has declared
  independent of each other. `architecture/dependency-boundary` orders layers, so a dependency is wrong in
  one direction only; sibling isolation is wrong in both, and that is the constraint that keeps two feature
  modules from quietly becoming one. A member declares the same two halves a layer does, so the existing
  path and module matching is reused rather than reinvented, and `recommended@1` adopts it with no sets
  configured — like the other architecture rules it enforces nothing until a repository names something.
  A member importing its own internals, a file outside the set importing a member, and a member importing
  something the set does not name are all deliberately silent.
- `maintainability/cognitive-complexity` — measures how hard a function is to follow rather than how
  many paths run through it, weighting nested control flow: every branch costs one plus the nesting depth
  it sits at, so four flat guard clauses cost 4 while four nested branches cost 10. `decision-complexity`
  scores those two the same, which is the gap this closes. Three discounts come from the metric's
  specification and are each fixtured: a `switch` costs one however many arms it has, an `else if` costs
  one because the reader already paid for the `if`, and a run of one logical operator costs one however
  long it is. A closure's complexity belongs to the closure rather than its host, which deviates from
  Sonar's specification and matches every other function metric here. `recommended@1` adopts 15, Sonar's
  published default; across this repository's 1387 functions the highest score is 6, so the threshold is a
  ceiling against regression rather than a description of current practice.
- `maintainability/condition-complexity` — reports a single `if` or `while` condition that combines
  more `&&`, `||`, or ternary operators than the configured limit (3 in `recommended@1`, measured
  against this repository). `decision-complexity` deliberately does not count these operators, so a
  five-part boolean condition and a one-part one score identically today; this rule closes that gap.
  Counting is flat — three operators cost three, whichever operators they are — and a standalone
  ternary not attached to an `if`/`while` is out of scope.
- The documentation is a set of documents rather than one README. `docs/rules.md` holds the rule
  reference, `docs/configuration.md` the `godlint.yaml` schema, `docs/ci.md` the action and the output
  formats, `docs/local-development.md` the build, `docs/releasing.md` the release, and `docs/README.md`
  indexes them. The README is what a reader needs to decide whether to try Godlint and how to start;
  it had grown to hold the full rule reference inside a section called Local development, because a
  validator check demanded every rule identifier appear in it. That check now asks the rule reference
  for the same thing, so the invariant survives the move.
- `validate-pull-request.py` follows every relative link in every Markdown file, and fails on one that
  points at a missing file or a heading that no longer exists. Nothing caught that before, which is how
  a documentation split becomes a set of dead links.
- `tomerwave/godlint@v1` resolves, and each release moves it. The tag is the action's interface
  version rather than the binary's: the inputs are what it promises and they have not changed, while
  the command line is still `0.1.x`. It advances only after every registry and every archive has
  succeeded, so it never points at a half-published release, and a break in the inputs means a `v2`
  rather than a bump. Exact version tags stay immutable; the floating tag is deliberately outside
  that rule, since a tag whose purpose is to move cannot be protected against moving.
- `reliability/empty-error-handler` — reports an error handler that discards the error: an empty
  JavaScript or TypeScript `catch` body, and a Python `except` body holding nothing but a
  placeholder. `pass`, `...` and a lone `;` are placeholders, and so is a comment, in both
  languages: a comment neither handles the error nor re-raises it, and Godlint already has an
  accountable way to say a swallow is deliberate — a suppression with an owner and an expiry, which
  a comment cannot be held to. The exception clause is read for its body wherever that body sits, so
  `except ValueError as error:` is held to the same standard as a bare `except:`. Rust is out of
  scope: it has no `catch`, and discarding a `Result` is a separate rule on the roadmap.
- Two labels make the drift check pass, and which one records what happened: `fixes-false-positive`
  when a rule was reporting something it should not have, and `relaxes-a-rule` when the rule was
  narrowed or a threshold loosened. Both are cases where the released binary still reports what the
  change removed, so it cannot approve the tree; one label could say a drift was expected but not
  which kind, and the kind is the part a reader of the history wants. The explanation is printed
  either way, and neither label belongs on a pull request where the repository has genuinely drifted
  from the standard it publishes.

### Changed

- Whether a curated call list ships as configuration or as a named rule is now decided and recorded in
  the architecture guide: ship the opinion as a named rule, keep `architecture/restricted-call` for policy
  Godlint has no opinion about. A configured list cannot say why it exists, so its message cannot say what
  to do instead, and it cannot carry a stable identifier for a suppression to survive a configuration edit.
  `rules::catalogue` now owns the machinery all four call-matching rules shared — the dialect table, the
  dialect a language speaks, the macro-aware spelling of a callee, and the path allowance — so a new named
  rule costs a table and a message rather than a copy of the engine. `architecture/restricted-call` went
  from 96 lines to 58 and `logging/no-production-log` from 85 to 50. No rule changes behaviour: output is
  byte-identical across all 52 fixtures. Replacing a per-language match with a catalogue lookup also
  removed an arm that could never execute, so the rule-coverage budget drops from two documented
  unreachable lines to one.

### Fixed

- A file the grammar could only partly parse contributed nothing at all. One construct it did not
  recognise discarded the whole file, and the loss left no trace in a findings count — only a scan issue
  on stderr. Godlint now judges every node whose subtree parsed and skips the rest, so a function whose
  body failed to parse is still never reported. Measured on Zod, where four files use TypeScript 4.7
  variance annotations that `tree-sitter-typescript` does not implement: 905 functions and 1726 findings
  were being thrown away by 21 error nodes inside interface declarations.

- A source file was read whole with no size bound, so one very large file in a scanned tree could
  exhaust memory before anything inspected it. Files are now read through a bounded reader with a
  four-mebibyte ceiling, and a file above it is reported as a scan issue naming the limit instead of
  being loaded. The bound is on the read rather than on a size checked beforehand, so the allocation
  is limited by construction.
- `security/no-dynamic-execution` missed a built-in reached through the global object, so
  `globalThis.eval(code)` was silent while `eval(code)` reported. The rule now strips a known global
  prefix before matching: `globalThis`, `window`, `self`, and `global` in JavaScript and TypeScript,
  and `builtins` in Python. Python's `self` is deliberately not on that list, because there it names
  the instance a method was called on rather than the global scope. A finding still reports the
  spelling the file used. An alias such as `const e = globalThis.eval` still escapes, which needs
  value tracking rather than a longer list.
- `security/direct-environment-read` missed `process?.env.PORT` while reporting
  `process.env?.PORT`. Optional member access denotes the same read, so which spelling an author
  chose decided whether the policy applied. A callee and an access target are now resolved when the
  fact is built rather than read back out of the source range, and the language module decides which
  spellings name one path. Optional calls resolve the same way, so `outer?.parse(input)` and
  `outer.parse?.(input)` both reach `architecture/restricted-call` as `outer.parse`.
- A filename could rewrite the report about it. Godlint printed repository paths, messages, and
  arguments unescaped, so an escape sequence in a name repainted the surrounding output and a newline
  turned one finding into what read as two. Every diagnostic now goes through one escaping boundary:
  a control character is rendered readably in the terminal and GitHub formats, and as a `\u` escape in
  JSON and SARIF. The machine-readable formats also escape the control characters above `0x7f`, which
  they previously passed through.
- A directory Godlint could not read discarded every finding from the rest of the run. Discovery now
  reports the files it found alongside the failures it hit: a path named on the command line is still
  fatal, because a partial answer to an explicit request is a wrong answer, while anything reached
  below such a path becomes a scan issue and costs its own contents only. The exit code still says
  something went wrong, so degrading never turns into passing.
- The README described Godlint in the future tense - what it *would* provide, what the first release
  *would* focus on, GitHub Actions integration as an unreached phase - while it was shipping on four
  channels. It now says what the tool does today, and the one claim that was genuinely unshipped,
  gradual adoption through baselines and diff-aware enforcement, is named as roadmap rather than as a
  feature.
- `CONTRIBUTING.md` and `docs/contributing.md` were two documents with one name and no stated
  difference. The public contribution process is now `CONTRIBUTING.md` alone, and the release process
  it had absorbed is `docs/releasing.md`.

## [0.1.9] - 2026-07-29

### Fixed

- Godlint works on Windows. A repository-relative path is spelled with forward slashes wherever a
  policy sees it, so a glob written with `/` matches, and a file name is the last segment rather than
  the whole path. On Windows every `exclude` pattern silently matched nothing — so excluded
  directories were scanned — and `architecture/filename-case` reported
  `crates\godlint-cli\src\main` as a name that is not snake_case. Both were found by running the
  action against this repository on a Windows runner rather than against a fixture.

- A release is published whether or not one already exists for the tag. Listing an action on the
  Marketplace is done by editing a release, so a release created by hand first made the workflow fail
  at `gh release create` and the archives never attached — leaving the release that the action
  resolves as `latest` carrying no binary at all. The archives are uploaded separately from creating
  the release, and their count is asserted afterwards, because an archive that never attached is
  invisible until someone tries to install.

- The action's own check runs against this repository rather than a fixture, so a released binary
  that disagrees with the code here is visible instead of surprising whoever installs it. It fails
  when they disagree and prints which of the two reasons it is: a false positive fixed here and not
  released yet, or this repository having drifted from the standard it publishes. Adding a rule or
  tightening a threshold does not land there, because the released binary is always the more
  permissive one. The check is not required, so it never blocks a merge. Whether the action *works* —
  install, checksum, annotate, fail — is gated separately by a tree with findings in it, which does
  not depend on the release agreeing with this one.

## [0.1.8] - 2026-07-29

### Added

- A GitHub action. `uses: tomerwave/godlint@v1` installs the released binary, verifies it against
  the checksum published beside it, and reports each finding as an annotation on its line. No
  toolchain is installed and no token is used, which is what lets it work on a pull request from a
  fork. The job summary carries a count per rule, because GitHub renders only so many annotations per
  run and a repository with more findings than that would otherwise lose the rest silently. `version`
  defaults to the latest release and says so; pin it for a check that cannot change under you. The
  action is listed as `Run Godlint`, since a Marketplace name has to be unique across every action,
  user and organisation, and `Godlint` was taken. It changes no `uses:` line and no package name.

  The action is exercised by its own pull requests on Linux, macOS and Windows, against one tree it
  must pass and one it must fail — so a change to it is tested before release rather than after. That
  test found three things on its first run: a checksum file written with a carriage return, an
  unauthenticated call to `api.github.com` answering 403 from a rate-limited runner, and that the
  action cannot be tested against a release older than the flag it depends on. The version is now
  resolved from the redirect the releases page serves, and a download is verified by comparing the
  published hash rather than by handing the file to `shasum -c`, which reads a filename out of it.


## [0.1.7] - 2026-07-29

### Fixed

- The Windows checksum file ends its line with a newline rather than a carriage return and newline.
  PowerShell's `Out-File` writes CRLF, so every Unix checksum tool read the filename as ending in a
  carriage return and refused to verify a correct download.


### Added

- `check --format <github|json|sarif|terminal>`. `terminal` stays the default. `github` emits
  workflow-command annotations, so a finding lands on the exact line of a pull request diff without a
  token and without permissions, which is also what makes it work on a pull request from a fork.
  `json` and `sarif` are documents for another tool to read, and both are emitted even when there is
  nothing to report, because a consumer parses a document rather than prose. A format a person reads
  still says `No findings.`, since silence reads as the tool not having run.

  The annotation format escapes a property and a message differently, which is not cosmetic: `:` and
  `,` separate properties, so escaping them in the message turned `std::process::exit` into
  `std%3A%3Aprocess%3A%3Aexit`.

## [0.1.6] - 2026-07-29

### Added

- A PyPI package. `pip install godlint` installs a binary and needs no Rust toolchain, the same
  reasoning as the npm package. Wheels repackage the binaries the release already built and
  version-checked rather than compiling them again, so every channel ships the same file per
  platform. pip distinguishes glibc from musl by wheel tag, unlike npm, so both get their own
  wheel: seven in total across macOS, Linux and Windows. Published by trusted publishing, so no
  PyPI token exists.


## [0.1.5] - 2026-07-29

### Changed

- npm is reached by trusted publishing, so no registry token is stored for either registry.

- The GitHub release waits for every registry rather than only for crates.io. A release announces
  that a version is available, and it is not available while a registry is missing it. Binaries
  survive a registry failure as workflow artifacts either way, so nothing is lost by waiting.


## [0.1.4] - 2026-07-29

### Fixed

- The npm packages are published from a list the assembler writes, dependencies first, so the check
  that packs them and the step that publishes them walk the same list. Naming the paths twice in the
  workflow is what published five packages and then failed on the sixth, whose directory had been
  renamed — the same shape as the `./` mistake before it: the check and the action were not the same
  command.


## [0.1.3] - 2026-07-29

### Fixed

- The npm front door is `@godlint/cli`. npm refuses the bare name `godlint` as too similar to
  `oxlint`, which only surfaced on upload — a dry run does not check name policy. The command it
  installs is still `godlint`, because the executable a package provides is named independently of
  the package, so `npx godlint check` reads the same either way.

- A failed npm publish no longer costs the release its GitHub release and binaries. The two now run
  in parallel after the builds rather than in a chain, so one registry cannot withhold artifacts
  that are already correct.


## [0.1.2] - 2026-07-29

### Fixed

- The npm packages are published from paths prefixed with `./`. npm reads a bare `owner/name` as a
  GitHub shorthand, so `packages/cli-darwin-arm64` was fetched from git rather than published from
  disk, and the release failed after the crates were already published. A dry run now packs every
  package before any upload, so a packaging mistake costs nothing.


## [0.1.1] - 2026-07-29

### Added

- An npm package. `npm install --save-dev godlint` installs a binary and needs no Rust
  toolchain — the audience for a JavaScript, TypeScript and Python linter mostly does not have one.
  The bare `godlint` package carries no binary and declares one optional platform package per
  platform; npm installs only the one matching `os` and `cpu`, and the package runs it. Nothing is
  downloaded during install, so it works with `--ignore-scripts` and without a network. Linux ships
  the statically linked build, so one binary per architecture runs against either libc. Packages are
  published by trusted publishing rather than a stored token, and each carries a provenance
  attestation tying it to the commit and workflow that built it.

- Release binaries for Windows (`x86_64-pc-windows-msvc`) and for Linux without glibc
  (`x86_64-unknown-linux-musl`, statically linked), alongside the Linux and macOS builds for both
  architectures. The musl build is what a container image needs: a glibc binary fails there with a
  loader error rather than a useful message, and a linter mostly runs in CI. The same build is what
  the npm packages ship for Linux.

## [0.1.0] - 2026-07-29

### Added

- Policy suites. `suites: [recommended@1]` adopts a named standard in one line instead of
  twenty-one rule blocks with hand-picked numbers. `recommended@1` enables every rule at
  `error`, because a standard a repository can partly ignore is a suggestion. Suites are
  opt-in — a configuration naming none enforces nothing — and a `rules:` entry overrides the
  suite for that rule in either direction, including `severity: off`, which is what lets a
  rule be adopted as a warning first. An unknown suite name is a configuration error that
  lists the available ones. Godlint's own configuration is now the suite and nothing else,
  since an override here would be this project exempting itself from its own standard.

- `architecture/filename-case` (`scopes`, `allow`) — reports a file name that does not follow the
  convention for its extension or for a declared scope. It reads no syntax: a path is the whole
  input. The convention comes from the extension rather than the language, because that is where
  the distinction lives: `PascalCase` for `.tsx` and `.jsx`, `kebab-case` for `.ts`, `.js`,
  `.mjs`, `.cjs`, `.mts` and `.cts`, and `snake_case` for `.rs`, `.py` and `.pyi`. So `Button.tsx`
  and `use-button.ts` are both correct in the same directory. A `scopes` entry declares the case
  for the paths it names and wins over the extension default, with the most specific scope
  deciding rather than the first declared; `allow` is checked first and exempts a path outright. A
  leading or trailing separator is not part of the name, so `__init__.py`, `__main__.py` and
  `_private.py` are snake_case as PEP 8 has them. Every case is judged in ASCII in every position.
  The name judged is what precedes the first dot, so `widget.test.ts` is judged as `widget`.

- `security/forbidden-dependency` (`packages`, each with `allow-in`) — reports an import of a
  package the project has ruled out. It maps an import to its package and matches that name
  exactly, so naming `lodash` catches `lodash/merge` and anything deeper while leaving
  `lodash-es` alone. The package is the first path segment in JavaScript and TypeScript, or the
  first two when scoped, the first dotted segment in Python, and the first `::` segment in Rust
  including `extern crate`. A relative import, `crate`, `self`, `super`, and a builtin reached
  through a protocol name no package and are never dependencies — in fact any specifier
  containing a colon is rejected, which covers `node:fs`, a URL and a Windows path alike, as is
  any specifier rooted at `/`. A leading `::` in Rust is stripped first, so `::serde` is the
  crate `serde`.

- `architecture/dependency-boundary` (`layers`) — reports a dependency that runs against a
  declared layer order. Position in the list is the policy: a layer may depend on itself and on
  anything below it. Each layer declares both the `paths` it contains and the `modules` that
  name it, because nothing is resolved and neither string can be derived from the other; a layer
  given only one of the two is a configuration error rather than half a policy.

- `architecture/restricted-import` (`modules`, each with `allow-in`) — reports an import of a
  module a repository puts behind a boundary. A restricted name covers what lies beneath it by
  whole segment, so `crate::internal` catches `crate::internal::deep` while `crate::internals`
  is a different module. Built on a new `ImportFact`, which reads the module from the range that
  spells it: a Rust `use` path or `extern crate` name, a Python `import` or `from ... import`
  module with an alias seen through, and the source string of a JavaScript or TypeScript
  `import` or re-export. It resolves nothing, the same boundary the call rules draw.

- `logging/no-production-log` (`allow-in`) — reports debug logging outside the paths a
  repository approves. Defaults to the calls that exist to be read during development:
  `console.log`, `console.debug`, `console.info`, `console.trace`, Python `print` and
  `pprint.pprint`, and Rust `dbg!`, each bound to the dialect that spells it. `console.error`,
  `console.warn`, `println!` and the `logging` module are left alone, being how a program talks
  to its user rather than leftover debugging.

- `reliability/explicit-timer-delay` — requires an explicit delay for JavaScript and
  TypeScript `setTimeout` and `setInterval` calls, where omission silently defaults to
  immediate execution. Reads the timer under a global receiver (`window`, `globalThis`,
  `self`) as the same timer, and does not count a comment as an argument, so a
  commented-out delay is reported rather than mistaken for one.

- Open-source project foundations, community guidance, security reporting guidance,
  and project brand assets.
- A local CLI with `godlint config validate` and `godlint check`, source discovery for
  Rust, TypeScript/JavaScript, and Python, and a versioned root `godlint.yaml`.
- `maintainability/function-size` (`max-lines`) — reports a function longer than a
  configured effective-line ceiling.
- `maintainability/file-size` (`max-lines`) — the same ceiling applied to a whole file.
- `maintainability/function-nesting` (`max-depth`) — reports control-flow blocks
  nested too deeply inside a function.
- `maintainability/parameter-count` (`max-parameters`) — reports declared parameters
  above a ceiling, excluding a method receiver (`self`, `&self`, `cls`).
- `maintainability/decision-complexity` (`max-complexity`) — reports a function with
  too many branch points.
- `maintainability/return-count` (`max-returns`) — reports a function with too many
  exit paths.
- `maintainability/function-statements` (`max-statements`) — reports a function with
  too many statements.
- `maintainability/empty-function` (`allow-names`) — reports a function body that
  appears unintentionally empty.
- `policy/todo-requires-reference` (`markers`, `reference-prefixes`) — requires an
  issue reference beside a TODO-style marker.
- Top-level `fail-on` (default `error`) — the lowest severity that makes `godlint
  check` fail. Findings below it are reported without failing the command, which is
  what makes adopting a rule as a warning first a real option rather than a promise.
- Top-level `exclude` — a list of path globs supporting `*`, `?`, and `**` that
  replaces the built-in defaults when set. The defaults are `.git`, `.mypy_cache`,
  `.next`, `.tox`, `.venv`, `__pycache__`, `build`, `coverage`, `dist`,
  `node_modules`, `target`, and `vendor`. Previously only `.git`, `node_modules`, and
  `target` were skipped, and the list was not configurable.
- Support for the `.mjs` and `.cjs` JavaScript extensions and the `.mts` and `.cts`
  TypeScript extensions.
- `style/no-comments` (`allow-doc-comments`) — requires code to explain itself rather
  than leaning on prose beside it. Documentation comments are permitted by default,
  because a published contract is written for a reader who cannot see the
  implementation, which is a different job from explaining a line to someone already
  reading it. An interpreter shebang is always exempt. Enable it alongside
  `policy/todo-requires-reference` only deliberately: a marker comment will be reported
  by both.

- Inline suppression. `godlint-ignore-next-line` and `godlint-ignore-enclosing` exempt a
  single site from named rules, carrying a required reason and an optional `owner=` and
  `expires=`. Both are parsed from comment facts, so they work in every comment syntax
  Godlint reads including Python docstrings, and a directive must open its line so prose
  that mentions one is not one. There is deliberately no file-wide form: that is an
  `exclude` entry with less visibility. See [inline suppression](docs/suppressions.md).
- `policy/accountable-suppression` (`require-owner`, `require-expiry`) — reports a
  suppression that cannot account for itself: no reason, no rule named, an unknown rule,
  an unrecognised option, an expiry that is not a calendar date or has passed, a missing
  owner or expiry when required, or a `godlint-ignore-enclosing` with nothing to enclose.
  It cannot itself be suppressed, since nothing else would then hold suppressions to
  account. A defective directive still silences what it names and is reported against
  itself, so a lapsed expiry fails the build with one clear finding rather than an
  avalanche of unrelated ones.
- `policy/unused-suppression` — reports an inline directive that does not silence an
  enabled finding. A directive for an off rule is dormant rather than unused, so staged
  rule adoption does not manufacture exception debt.
- `architecture/restricted-call` — restricts direct process exits and debug-only output
  once enabled, and restricts configured direct callees to approved path globs. Naming a
  built-in restriction under `calls` lets its `allow-in` boundary apply to it, which is how
  a CLI permits `console.log` in its entry point. A Rust macro is named with its `!` —
  `dbg!` restricts the macro, `dbg` restricts a function of that name — which is both how
  Rust spells them and how a finding reports them, so the name a reader sees is the name
  they configure. A built-in name stays bound to the language that defines it, so scoping
  Python's `print` does not restrict a TypeScript function of that name, while a name the
  project invents applies wherever it is called. The consequence is that a callee of your own
  whose name a built-in already claims cannot be restricted: naming `print` reaches Python's
  and not a TypeScript function of that name, and there is no language key yet to say which
  was meant, so such a policy is silent rather than wrong. Listing one callee twice is a
  configuration error rather than a silent choice between the two entries.
- `security/no-dynamic-execution` — reports JavaScript `eval`, `Function`, and
  `new Function`, plus Python `eval` and `exec`.
- `security/direct-environment-read` — reports direct JavaScript, Python, and Rust
  environment reads outside a configuration boundary. `allow-in` defaults to `**/config.*`
  and `**/config/**`, and setting it replaces that default rather than adding to it, so a
  repository can narrow the exemption as well as widen it.
- All three read a callee exactly as spelled, and are off until a repository configures
  them, like every other rule. `std::env::var` is matched where the aliased `env::var` after
  `use std::env` is not, because knowing they name the same function needs resolution that
  [the rule roadmap](docs/rule-roadmap.md) defers to a semantic phase. There is no scope
  analysis either, so a local binding shadowing a restricted name is reported — a Python
  parameter called `exec`, or a TypeScript `const process`. A Rust macro and a function that
  share a name are distinguished, so `dbg!(x)` is restricted where a `fn dbg` is not.
- `godlint suppressions [paths...]` — lists every suppression in scope with its location,
  scope, rules, owner, expiry, and reason, then the total. A directive with no reason is
  listed as `(no justification)` rather than omitted.
- `godlint-ignore-next-line` skips the remainder of its own comment, so a directive on its
  own line inside a block comment reaches the code after the comment rather than the
  closing delimiter. Read literally, the next line after such a directive is `*/`, which
  would make it silence nothing and report nothing. A justification likewise no longer
  absorbs the closing delimiter, so `-- awaiting #485 */` records `awaiting #485`.
- `godlint_core::date::Date` — a calendar date with no new dependency, used for
  suppression expiry. `godlint check` now reads the current date, which is passed
  explicitly into rule evaluation rather than read inside a rule; it is the only
  time-dependent input, and fixtures pin dates far in the past and future.

### Changed

- A blank path pattern is reported as `<rule> path patterns must not be blank` rather than naming
  `allow-in`, since `architecture/filename-case` has no `allow-in` key — it has `allow` and
  `scopes`, and the old message named a field that rule does not have.

- A scoped JavaScript specifier of exactly `@` is no longer read as a package. `@/components` is a
  bundler alias for first-party source rather than a registry scope, and first-party code names no
  dependency.

- One registry is the single list of rules. `rules::registry` already had to know every
  rule to answer "is this rule enabled", which `policy/unused-suppression` needs, and
  `RULE_IDS` was a second list of the same rules used to answer "does this rule exist".
  Keeping both invited a silent disagreement: a rule present in `RULE_IDS` but missing from
  the registry made every suppression naming it report `NotSuppressible`, and the reverse
  made one report `UnknownRule`. `RULE_IDS` is gone; `is_known_rule`, `is_suppressible_rule`,
  `configured_severity` and `rule_ids` all read the registry, and
  `scripts/validate-pull-request.py` now requires a new rule to appear there.
- `policy/unused-suppression` builds its finding with the shared helper the other rules use
  rather than its own copy, which also takes the rule-coverage budget back down by one.

- `style/no-comments` no longer reports a comment that is **only** suppression directives.
  A directive is machine-readable policy metadata rather than prose beside the code, and
  reporting it would make suppression unusable in any repository that adopts the policy —
  including this one. The exemption deliberately requires the whole comment to be
  directives: exempting any comment that merely *contains* one would let a single valid
  directive launder arbitrary prose past a rule set to `error`. Blank lines and the
  comment's own delimiters do not count against it.
- `maintainability/function-nesting` measures how deeply control-flow blocks nest
  inside a function — `if`, `for`, `while`, `match`, `with`, `try`, `switch` — rather
  than how many functions enclose it. An `else if` chain counts as one level. The
  message is now "Function nests blocks N levels deep (max M)."
- Rust closures and Python lambdas are functions for every function rule, as
  JavaScript/TypeScript arrow functions already were. Size, nesting, complexity,
  return paths, and statement count are attributed to the closure itself rather than
  to the function enclosing it, so one shared threshold means the same thing in all
  three languages.
- `maintainability/cyclomatic-complexity` is renamed `maintainability/decision-complexity`,
  and a `match` or `switch` now counts **once** rather than once per arm, while a guard on
  an arm counts for the first time. An exhaustive `match` over an enum is a dispatch table:
  the reader looks up one variant and reads one arm, and the compiler guarantees no case is
  missing, so ten one-line arms is one decision rather than ten. Counting per arm also
  produced a perverse ordering, since a `match` with three guards scored identically to the
  same `match` with none — guards were not counted at all — and both outscored a `match`
  with real nested branching. The rule is renamed because counting a multiway branch once
  is no longer McCabe's cyclomatic complexity; it is closer to Cognitive Complexity, which
  treats a `switch` as one structure for the same reason. A Python comprehension filter is
  still not counted; only a `case` guard is.
- The recommended `max-complexity` drops from 10 to 8 and the strict profile from 8 to 5,
  measured against this repository rather than inherited. The 10 came from an ESLint
  `complexity: 10` setting and could not travel with the metric, because ESLint counts
  every `case`: under the new definition this repository's worst function measures 7 rather
  than 11, so 10 constrained nothing. Short-circuit `&&`, `||`, `and`, and `or` remain
  uncounted.
- Counting a multiway branch uniformly is slightly generous to JavaScript, and knowingly so.
  "One decision" rests on exhaustiveness, which Rust's `match` has and a JavaScript
  `switch` — with fallthrough and no exhaustiveness check — does not. Counting `switch` per
  case in JavaScript alone would make one threshold mean different things in different
  languages, which is the failure this project exists to avoid.
- Godlint's first accountable exception is deleted rather than renewed. The suppression on
  `impl fmt::Display for SuppressionDefect` existed only because the metric counted its
  arms; answering the question it recorded removed the need for it, and
  `godlint suppressions` now reports none.
- `maintainability/return-count` counts every exit path: an explicit `return`, the
  Rust `?` operator, and an implicit trailing expression such as a Rust tail
  expression or a concise arrow or lambda body. The message is now "Function has N
  return paths (max M)." This keeps Rust comparable with languages that must write
  `return`.
- `maintainability/function-statements` counts statements through nested blocks but
  not into nested functions, and no longer counts comments as statements. An
  expression-bodied arrow or lambda counts as one statement.
- `maintainability/empty-function` no longer reports a body containing only a comment,
  an abstract or overload declaration (`@abstractmethod`, `@overload`), a TypeScript
  constructor that assigns parameter properties, or any declaration in a `.pyi`
  interface stub. Python `pass` and `...` both count as empty.
- `policy/todo-requires-reference` takes a configurable `markers` list, defaulting to
  `TODO`, `FIXME`, `HACK`, and `XXX`. Each marker is reported at its own position and
  owns only the text up to the next marker, so one reference can no longer excuse
  several markers. A reference is a configured prefix followed by digits that end the
  token, and a prefix must start a word, so `NOTJIRA-42` does not satisfy `JIRA-`.
  Purely numeric `reference-prefixes` are rejected by configuration validation. Python
  docstrings are scanned alongside comments.
- `skip-blank-lines` and `skip-comments` default to `true` instead of being required.
- Godlint's own source carries no comments, and enables `style/no-comments` with
  `allow-doc-comments: false` to keep it that way. The reasoning that used to sit in
  module and item documentation now lives in [the architecture guide](docs/architecture.md),
  which is the better home for it: a boundary is easier to explain once, in prose, than
  in fragments beside the code it constrains.
- A limit violation is one `Violation::Limit` carrying the metric, the measured value,
  and the ceiling, rather than one variant per metric. Rendering stays identical.
- Line and column derivation binary-searches a per-file line index instead of scanning to
  the offset, so reporting cost no longer grows with how far into a file a finding sits.
  With `style/no-comments` reporting once per comment this was measurable: a file with
  6,400 comments took 678 ms and now takes 161 ms.
- Effective-line counting is derived from the analyzer's comment facts instead of
  scanning text for `//` and `#`, so Python docstrings are skipped like JSDoc blocks
  and nested Rust block comments are handled correctly.
- `max-lines` and `max-complexity` reject `0`, because those metrics have a floor of 1.
  `max-depth`, `max-parameters`, `max-returns`, and `max-statements` accept `0`,
  because forbidding a construct outright is a real policy.

- Pull request templates for a new rule and for infrastructure work, each asking for
  what a reviewer would otherwise have to reconstruct: the policy a rule encodes, what
  it counts in each language, where its threshold came from, and which idiomatic
  constructs a naive implementation would misreport.
- `scripts/validate-pull-request.py`, run by CI, which checks the repository invariants
  those templates ask about rather than trusting a ticked box. A rule module must be
  registered, configurable, fixtured, unit-tested, documented in the roadmap, README and
  changelog, and enabled for Godlint itself; every fixture must be declared by a test and
  assert an exit code; every workflow must declare permissions and pin one toolchain.

- Effective-line counting walks its comment facts with a forward cursor and tests each
  line against only the comments that can overlap it, rather than filtering every comment
  per call and scanning the whole list per character. Counting a heavily annotated file no
  longer costs its line count times its comment count: a file with 6,400 comments took
  1,080 ms and now takes 177 ms.

- Mutation testing over the rules layer, replacing the removed requirement that a rule
  change touch a fixture. That check could be satisfied by editing any fixture at all and
  so never established what it appeared to; altering a rule and requiring a test to notice
  establishes it directly. A pull request mutates the lines it changed; a weekly run covers
  every rule. It raises the floor rather than proving coverage: a new branch can still be
  untested while every mutant of it is caught, so review still decides whether a case
  deserves a fixture.
- Three cases the first mutation run showed were unexercised: code that begins at the byte
  a block comment ends, and the two error-reporting paths of `RuleError`.
- `scripts` is excluded from Godlint's own scan. The no-comments policy was adopted for
  the Rust codebase, where the reasoning moved into the architecture guide; a maintenance
  script that encodes a budget should carry the reason beside the number.
- A coverage gate over the rules layer, budgeted in uncovered lines rather than as a
  percentage. This closes the hole mutation testing leaves: an unexercised branch shows up
  as an uncovered line even when every mutant of it is caught. It found four such cases on
  its first run, including all three severity gates and a reference prefix followed by a
  non-digit.
- Every rule must now have a fixture that reports it and a fixture that configures it
  without reporting it, so a rule cannot be left with nothing proving it stays silent on
  conforming code. `style/no-comments` was missing the second and now has it.
- A rule that compares one measurement against one ceiling now declares the metric it
  reports under, how to measure it, and how to read the ceiling, and no longer expresses
  the comparison. The comparison lives in the driver, so a rule cannot invert the test,
  and pairing the metric with the rule as an associated constant means it cannot report
  under another rule's metric. Behaviour and output are unchanged. The `FileRule` trait and
  its driver go with the change, having no implementors left once the one file rule became a
  limit rule; the coverage gate is what noticed.

- The pull request validator no longer requires a fixture change whenever a rule module
  changes. A behaviour-preserving refactor should leave every fixture untouched, and
  byte-identical output is the evidence that it is correct, so the check penalised exactly
  the change it should have welcomed. Whether a fixture is owed is a judgement, and the
  template asks for it there.

- `ConfigError` moved to `config/error.rs`, the configuration validators to
  `config/validate.rs`, and `Violation` to `rules/violation.rs`. Each file had crossed the
  500-line or 50-line ceiling this project holds itself to as rules were added, and the ceiling
  is not the thing to move. The three variants reporting an entry duplicated within an `allow-in`
  list also repeated one sentence, which is now written once.

- `architecture/restricted-call` no longer bans `console.log`, `console.debug`, Python `print`
  or Rust `dbg!` by default; `logging/no-production-log` owns them. Two consequences to know
  when upgrading. A suppression naming `architecture/restricted-call` over one of those calls
  now reports twice — once as an unused suppression and once as a production log — and must be
  renamed to the logging rule. And naming one of them under `calls` no longer scopes it to a
  language, because the language binding belonged to their being built-ins; the logging rule
  keeps that binding, so restrict debug output through it. A process exit is banned
  outright and excused per call site, while logging is permitted wherever a repository says it
  belongs, so one `allow-in` for the class fits it better than an entry per callee — and two
  rules reporting the same call reported it twice. Naming any of them under `calls` still
  restricts it.

- Every rule driver reports through one kernel, so the severity gate and the loop that turns
  a violation into a finding exist once rather than in each of five drivers. A file rule and a
  suppression rule had kept their own copies because their items are not a slice a
  `SourceFacts` owns.

- A source range is built only by the file it indexes, so a range that exists has already
  been checked against that file. Locating one is infallible, no fact re-validates a range
  its type already vouches for, and rule evaluation reports no error at all: the single
  error it could return was a location failure that could not occur. `SourceRange::new`,
  `SourceRangeError`, `RuleError`, and the three fact range errors are gone, and every rule
  evaluator returns findings rather than a `Result`. The count of lines no test can reach
  fell from nine to two.

### Fixed

- A suite name is checked by the same lookup that expands it, so a name can no longer be
  offered as available while enforcing nothing. The name was previously validated against one
  list and expanded by a separate comparison, which a second suite could satisfy in one place
  and not the other.

- A call or access fact reads its callee or target from the range it already carries instead
  of storing a second copy, and `SourceFile` holds its path behind an `Arc` so cloning a file
  into a fact no longer allocates one. Both cut the memory a scan holds, since every fact for
  every file is live at once, and the first removes state that could disagree with itself.

- `godlint-ignore-enclosing` covers a byte range rather than a line range, and excludes
  declarations nested inside the one it resolves to. It previously silenced the named rule
  for anything sharing the declaration's lines, so a second arrow function on the same line
  escaped a justification written for the first, and a closure inside a function was covered
  by a reason that described only its parent. Both were widening failures: they hid findings
  with no output saying so. A directive for a nested declaration goes inside that
  declaration, which resolution already handles by picking the innermost. `Finding` carries
  the source range it is derived from to make this possible; it was previously computed and
  discarded. Containment compares whole ranges rather than the position a finding starts at,
  so a declaration beginning where another ends is not inside it, and a file-level finding —
  which spans the whole file — remains unsuppressible inline as documented.
  Because the exclusion is by range rather than by declaration identity, it also drops
  findings that are not about a declaration: a comment inside a closure now escapes a
  directive on the enclosing function, for `style/no-comments` and
  `policy/todo-requires-reference`. This is a narrowing, so an existing directive may begin
  reporting findings it previously hid; move it to the declaration the finding sits in.

- Quoted prose is no longer a live suppression. `is_furniture` stripped `"` and `'` on every
  line of every comment so that a Python docstring's delimiter could open a directive, which
  also meant `// 'godlint-ignore-next-line maintainability/empty-function -- only prose'`
  parsed as a defect-free directive, silenced the rule, and was exempt from
  `style/no-comments`. A quote now opens a directive only on the line where a quote opens
  the comment. Closing a comment is a separate question: a docstring's final `"""` remains
  furniture wherever it falls, so a directive inside a multiline docstring still reaches the
  code after it rather than the closing delimiter.
- `maintainability/decision-complexity` counts a refutable `let` — Rust's `let … else`. It
  counted zero, so the idiomatic form measured less than the nested `if let` it replaces:
  three bindings measured 1 against the nested form's 4. Both now measure 4.
- A suppression stacked above another reaches the code rather than its neighbour. The first
  of two consecutive directives resolved its next line to the second directive, silencing
  nothing while the audit listed it as live — the same silent no-op that the closing
  delimiter used to cause. Directive lines are now skipped whether they sit in the same
  comment or in another.
- An option given twice keeps its first value and is reported as a defect. An expiry could
  previously be renewed by appending a second `expires=`, invisible to both `godlint check`
  and `godlint suppressions`.
- `owner=` with an empty value no longer satisfies `require-owner`. An empty option value
  reads as absent.
- `policy/unused-suppression` no longer reports a `godlint-ignore-enclosing` that has
  nothing to enclose. Such a directive is misplaced rather than stale, so "remove it" was
  the wrong remedy, and it was reported twice when both suppression rules were enabled.

- A Python docstring following a shebang is recognised as a docstring. The shebang counted
  as the block's first statement, so the docstring below it was read as an ordinary string:
  `skip-comments` counted it as code and `policy/todo-requires-reference` did not scan it.

- Configuration discovery stops at a repository boundary, meaning a directory
  containing `.git`. It previously walked to the filesystem root, so a stray
  `godlint.yaml` in a parent or home directory could silently govern an unrelated
  repository and relocate the reported path root.
- Scan discovery stops at the same boundary, so walking a directory no longer descends
  into an embedded repository or submodule. A parent repository previously reported
  findings inside a submodule it does not own, and a linked worktree created inside the
  repository was scanned twice. Both the configuration and the scan boundary now come from
  one predicate, `paths::is_repository_root`, rather than each hardcoding `.git`.
  **This can stop reporting files you were seeing findings for.** If a nested repository
  holds first-party code you want scanned under the parent's policy, name the parent
  first — `godlint check . nested` — because the first requested path decides the
  configuration root. `godlint check nested` on its own asks for a repository that must
  carry its own `godlint.yaml`. A skipped nested repository is silent, like an `exclude`
  entry.
- A file that cannot be read, for example because it is not valid UTF-8, is reported
  as an issue against that file. It previously aborted the whole run and discarded
  every other finding.
- `policy/todo-requires-reference` matches markers on word boundaries, so an
  identifier such as `AUTODOWNLOAD` is no longer reported as a `TODO`.
- `policy/todo-requires-reference` no longer accepts a colour literal such as
  `#3366ff` as an issue reference for the `#` prefix.
- A `//` inside a string literal is no longer mistaken for the start of a comment when
  counting effective lines.
- A UTF-8 byte-order mark is stripped before parsing, so it no longer changes line
  accounting or shifts reported columns.
