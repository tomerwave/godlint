# Changelog

All notable changes to Godlint will be documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). Before `1.0`, a `0.x` release may
change the `godlint-core` API; the command line and the configuration schema are what the version
speaks about.

## [Unreleased]

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
