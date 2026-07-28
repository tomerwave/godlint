# Changelog

All notable changes to Godlint will be documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
project will follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html) once
releases begin.

## [Unreleased]

### Added

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
  they configure. Listing one callee twice is a configuration error rather than a silent
  choice between the two entries.
- `security/no-dynamic-execution` — reports JavaScript `eval`, `Function`, and
  `new Function`, plus Python `eval` and `exec`.
- `security/direct-environment-read` — reports direct JavaScript, Python, and Rust
  environment reads outside a configuration boundary. `**/config.*` and `**/config/**` are
  allowed without configuration, and `allow-in` widens that set rather than replacing it.
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

### Fixed

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
