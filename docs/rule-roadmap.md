# Rule roadmap

Godlint is a cross-language policy engine. It should enforce the rules that need one
organization-level decision across Rust, JavaScript/TypeScript, and Python. It should
not reimplement formatters, compilers, or each language's general-purpose linter.

## Product boundary

| Own in Godlint | Delegate to existing tools |
| --- | --- |
| Repository policy, architectural boundaries, organization-specific restricted APIs, shared maintainability limits, accountable exceptions, and cross-language rule configuration | Whitespace, indentation, import layout, quote style, semicolon style, compiler/type errors, borrow checking, language-idiomatic micro-optimizations, and broad bug-finding catalogs |

Use Godlint for a policy such as “a source file may not exceed 500 effective lines”
or “only configuration modules may read environment variables.” Use formatters for
spacing and blank-line layout, and retain Rust, JavaScript/TypeScript, and Python
linters for language-specific correctness.

## Rule policy

Every rule has four attributes before implementation:

1. A stable ID, category, configuration schema, and exact output contract.
2. A required fact and language coverage; parser nodes never cross that boundary.
3. A confidence level: `high` can be enabled by a suite, `medium` starts as warning,
   and `low` remains experimental.
4. A fixture mini-repository with `godlint.yaml`, `expected.yaml`, valid cases,
   invalid cases, configuration cases, and later suppression cases.

Rules are not “recommended” merely because another tool has a similarly named rule.
They become recommended after the rule is deterministic, cross-language coverage is
intentional, Godlint dogfoods it, and its fixture corpus demonstrates acceptable
false-positive behavior.

## Policy suites

```text
recommended@1  High-confidence rules safe for most repositories.
strict@1       Stronger maintainability limits and naming conventions.
security@1     Explicit risky API and dependency restrictions.
testing@1      Test hygiene rules with framework configuration.
architecture@1 Repository layout, dependency, and ownership rules.
```

`recommended@1` is shipped. It names every rule at `error`, because a standard a repository
can partly ignore is a suggestion. Every suite including `recommended@1` is opt-in and must
be named in `suites:`; a configuration that names none enforces nothing, which keeps a rule
silent until a repository asks for it.

A `rules:` entry overrides the suite for that rule, in either direction — a looser threshold
for generated code, a tighter one, or `severity: off` to decline one rule without abandoning
the suite. That is what the confidence ladder needs: a rule can be adopted as a warning
first. Rules must never silently exclude production, test, or worker code.

The thresholds in `recommended@1` are measured against this repository rather than borrowed:

| Rule | `recommended@1` | Why not the roadmap's strict number |
| --- | ---: | --- |
| `maintainability/file-size` | 500 | 300 would demand splitting files whose length is a table, not a tangle |
| `maintainability/function-size` | 50 | 30 fights test bodies built from `concat!` blocks |
| `maintainability/function-nesting` | 2 | the strict number, adopted |
| `maintainability/parameter-count` | 4 | the strict number, adopted |
| `maintainability/decision-complexity` | 5 | the strict number, adopted |
| `maintainability/condition-complexity` | 3 | measured against this repository: a single-line scan of Rust conditions found 98 with zero operators, 14 with one, 3 with two, none above |
| `maintainability/return-count` | 6 | see below; 3 is a Python number |
| `maintainability/function-statements` | 14 | tighter than the strict profile's 20, measured |

`return-count` is the one place the strict profile is wrong rather than ambitious. At 3 it
reported 21 functions here, and every one was Rust idiom: 14 dominated by `?` propagation, 6
by guard clauses, and the last was three `.ok()?` conversions. Pylint's
`too-many-return-statements` counts `return` statements in a language without `?` or tail
expressions, so its number does not transfer — the same mistake as the ESLint `complexity`
threshold this project already re-derived. Getting a three-guard-clause function under 3
means nesting it, which the nesting rule then reports.

## Threshold profiles

Thresholds are policy choices, not language semantics. Godlint ships no hidden
universal number. The following profiles are the starting point for documented suites:

What `recommended@1` ships is stated once, under [Policy suites](#policy-suites), and a test
holds that table to the code. This table records where each number came from and what
`strict@1` would tighten it to; it deliberately does not restate the shipped values, because
two tables of the same numbers is how the two disagreed before.

| Rule | `strict@1` intent | Source of the initial policy |
| --- | ---: | --- |
| `maintainability/file-size` | 300 effective lines | User policy: file-size ceiling requested at 500; existing TypeScript policy uses 300 |
| `maintainability/function-size` | 30 effective lines | Existing TypeScript `max-lines-per-function` policy |
| `maintainability/function-nesting` | 2 | Existing Godlint rule; lower is intentionally stricter. `recommended@1` adopted it |
| `maintainability/parameter-count` | 4 | Common design-lint threshold; tune per repository. `recommended@1` adopted it |
| `maintainability/decision-complexity` | 5 | Measured against this repository under Godlint's own metric, and adopted by `recommended@1`. The former 10 came from an ESLint `complexity: 10` setting, which does not transfer: ESLint counts every `case`, Godlint counts a multiway branch once |
| `maintainability/condition-complexity` | 3 | Measured against this repository: `&&`/`\|\|`/ternary operators per `if`/`while` condition, and adopted by `recommended@1`. Sonar's `S1067` defaults to 3 for the same metric; Ruff's `max-bool-expr` defaults to 5, but only counts `if` statements and not ternaries or `while` |
| `maintainability/return-count` | 3 | Pylint's `too-many-return-statements` design metric. Counting `?` and implicit tail expressions raises Rust counts, so this one does not transfer at all — see the note under Policy suites |
| `maintainability/function-statements` | 20 | Derived from the `maintainability/function-size` profile: a function sitting at its effective-line ceiling should not be almost entirely statements, so each profile allows about two thirds of its line budget as statements |

“Effective lines” exclude blank lines and comment-only lines when configured, matching
the current function-size contract. ESLint likewise makes blank-line and comment
handling explicit for function-size metrics. Both options default to enabled, because
a policy about function length is a policy about code, not about documentation.

`maintainability/decision-complexity` is named for what it counts rather than for
McCabe's cyclomatic complexity, because it deliberately differs from it in one way that
matters.

Godlint counts language-defined branch points: `if`, `else if`, loops, `catch` and
`except` handlers, conditional expressions, the Rust `?` operator, a `match` or `case`
guard, and a multiway branch — `match` or `switch` — **once**, not once per arm.

The arm rule is the interesting one. Cyclomatic complexity counts each arm, because each
is a distinct path. But an exhaustive `match` over an enum is a dispatch table: the reader
looks up one variant and reads one arm, and the compiler guarantees no case is missing.
Ten one-line arms is one decision, not ten. Counting per arm also produced a perverse
ordering — a `match` with three guards scored the same as the same `match` with none,
because guards were not counted at all, and both scored higher than a `match` with real
nested branching. Counting the construct once and counting guards fixes the ordering.

The cost is honesty about naming and about the threshold. This is closer to Cognitive
Complexity, which treats a `switch` as a single structure for the same reason, than to
cyclomatic complexity. And the recommended threshold could not come across from the
ESLint `complexity: 10` it was originally borrowed from, because ESLint counts every
`case`: under Godlint's metric this repository's worst function measures 7 rather than 11,
so 10 constrained nothing. The profiles above are measured against this repository, which
is what the general warning here always asked for — a threshold migrated from another
linter must be re-checked against Godlint's own metric rather than assumed equivalent.

Short-circuit `&&`, `||`, `and`, and `or` are deliberately not counted. A boolean guard is
one decision a reader makes at one place, and counting its operands penalizes writing the
condition plainly. JavaScript and TypeScript `?.` and `??` are excluded on the same grounds
and this is a decision, not an oversight: they read as one access with a fallback rather
than as a branch a reader has to trace. A Python comprehension filter is likewise not
counted; only a `case` guard is.

The justification is that a reader looks up one arm rather than tracing every arm, which
holds in all three languages. It is *not* exhaustiveness: only Rust's `match` is checked
for that. A JavaScript `switch` has fallthrough and no exhaustiveness check, and a Python
`match` without a `case _` falls through silently in the same way, so both are scored
slightly generously — a six-way Python `if`/`elif` chain measures 7 while the same six
branches written as a `match` with no `case _` measure 2. Counting per arm for those two
languages alone would make one threshold mean different things in different languages,
which is the failure this project exists to avoid, so the uniform rule wins and the
generosity is accepted knowingly.

One multiway construct is still counted per arm, deliberately: a Python `except` chain.
Four handlers measure 5 where a Rust `match` over an error enum measures 1. Handlers are
ordered, are not exhaustive, and are tried in sequence until one matches, so each is a
decision a reader traces — the shape of an `else if` chain rather than of a dispatch
table. A `try`/`catch` in JavaScript has one handler and so cannot diverge.

A refutable `let` — Rust's `let … else` — counts as one decision, like the `if let` it
replaces. It was counted as zero until the arm rule landed, which made the idiomatic form
measure less than the nested form it is meant to replace.

Nesting depth is likewise a property of control flow, not of declarations.
`maintainability/function-nesting` measures how deeply `if`, `for`, `while`, `match`,
`with`, `try`, and `switch` blocks nest inside one function; an `else if` chain is one
level, because that is how it reads. Enclosing functions do not contribute depth, so a
closure is measured on its own body.

### What counts as a function

One shared threshold across three languages only means something if “a function” means
the same thing in each of them. Otherwise the same code, written idiomatically, scores
differently per language: a 40-line Rust closure would be charged to its enclosing
`fn`, while the equivalent JavaScript arrow function would be measured on its own. Every
function rule — size, nesting, complexity, return count, statement count, empty body,
parameter count — consumes the same `FunctionFact`, so the fact must cover every
construct a reader would call a function.

| Language | Counted as a function |
| --- | --- |
| Rust | `fn` items, including methods and associated functions, and closures |
| Python | `def` functions, including methods, and lambdas |
| JavaScript/TypeScript | Function declarations, function expressions, methods, and arrow functions |

A nested function is measured on its own body and is not folded into the function that
encloses it. This is what makes a closure-heavy Rust or JavaScript file comparable with
a Python file that would express the same logic as named helpers.

## Delivery roadmap

### Phase 1 — Existing facts and file metrics

These rules use source-level `CommentFact` and `SourceFile` data alongside
`FunctionFact`; they need no semantic or repository-graph capability. Severity and
thresholds are not listed per rule: a repository adopts them through a suite, and
`recommended@1` names every rule at `error`.

| Rule | Status | Confidence | Languages | Configuration |
| --- | --- | --- | --- | --- |
| `maintainability/function-size` | Shipped | High | All eleven supported extensions | `max-lines`, blank/comment policy |
| `maintainability/function-nesting` | Shipped | High | All eleven supported extensions | `max-depth` |
| `maintainability/file-size` | Shipped | High | All eleven supported extensions | `max-lines`, blank/comment policy |
| `maintainability/empty-function` | Shipped | High | All eleven supported extensions except `.pyi` interface stubs | `allow-names` |
| `policy/todo-requires-reference` | Shipped | High | All comment syntaxes and Python docstrings | `markers`, `reference-prefixes` |
| `style/no-comments` | Shipped | High, but opinionated | All comment syntaxes and Python docstrings | `allow-doc-comments` |
| `policy/accountable-suppression` | Shipped | High | All comment syntaxes and Python docstrings | `require-owner`, `require-expiry` |

`style/no-comments` is a policy rather than a defect check, which is why it sits in the
`style` category and is off unless a repository opts in. It states that code should carry
its own explanation; whether that is right is a decision a team makes, not something
Godlint asserts. Two interactions are worth knowing before enabling it: a marker comment
is reported by `policy/todo-requires-reference` as well, and
`maintainability/empty-function` treats a comment in an otherwise empty body as the
author documenting intent, so the two rules disagree about that body on purpose.

`file-size` establishes that Godlint can evaluate file-level facts alongside
function-level facts. It directly reflects the requested 500-line policy without
requiring a new parser capability.

The eleven supported extensions are `.rs`; `.py` and `.pyi`; `.js`, `.jsx`, `.mjs`, and
`.cjs`; and `.ts`, `.tsx`, `.mts`, and `.cts`. Scanning skips the paths in the top-level
`exclude` list, which replaces the built-in defaults when set. Findings below the
top-level `fail-on` severity are reported without failing `godlint check`, which is the
mechanism the confidence ladder depends on: a medium-confidence rule can be adopted as a
warning and observed before it blocks anyone.

### Phase 2 — Richer function facts

Extend `FunctionFact` only when the same data will serve multiple rules.

| New field or fact | Rules unlocked | Status | Confidence | Languages | Configuration | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `parameter_count` | `maintainability/parameter-count` | Shipped | High | All eleven supported extensions | `max-parameters` | Count declared parameters only, excluding a method receiver (`self`, `&self`, `cls`); do not infer types or defaults initially |
| `decision_points` | `maintainability/decision-complexity` | Shipped | High | All eleven supported extensions | `max-complexity` | Count language-defined branch points, including the Rust `?` operator and a `match`/`case` guard, but not short-circuit boolean operators and not one per arm of a multiway branch; fixture each language explicitly |
| `return_count` | `maintainability/return-count` | Shipped | Medium | All eleven supported extensions | `max-returns` | Count every exit path: explicit `return`, the Rust `?` operator, and an implicit trailing expression. Keep opt-in because early returns are often clearer |
| `statement_count` | `maintainability/function-statements` | Shipped | Medium | All eleven supported extensions | `max-statements` | Count statements through nested blocks but not into nested functions, which are measured as functions in their own right; comments are not statements, and an expression-bodied arrow or lambda is one |

Phase 2 is complete. Its fact additions stay small and reusable for future policy.

### Accountable exceptions

Shipped. [Inline suppression](suppressions.md) is the reference; this section records what
was required and how each requirement was met.

Godlint could previously narrow a rule two ways, and neither could carry accountability.
The `exclude` globs remove a path from the scan, which suits generated code and
deliberately non-conforming test data. `allow-names` on
`maintainability/empty-function` names a function, and it applies repository-wide.

Neither expresses "this one site is a known exception". That matters because
[dogfooding policy](dogfooding.md) requires every exception to record a reason, an owner,
an issue reference, and an expiry, and none of those can be attached to a glob or to an
entry in a name list. The gap had a cost visible in practice: an unavoidable exception
forced a rule to be weakened for the whole repository, which is how a fixture-shaped
allow-list entry can end up load-bearing for CI.

A comment at the site now names the rules and carries the justification:

```text
godlint-ignore-next-line maintainability/function-size owner=tomer expires=2026-12-31 -- splitting this in #482
```

| Requirement | How it is met |
| --- | --- |
| A stable directive syntax in every supported comment syntax, including Python docstrings, resolved from `CommentFact` rather than by re-scanning text | `godlint-ignore-next-line` and `godlint-ignore-enclosing`, parsed from comment facts. A directive must open its line, so prose mentioning one is not one |
| A required justification, so an unexplained suppression is itself a finding | `policy/accountable-suppression` reports a directive with no `-- <reason>` |
| Scope limited to the following line or the enclosing declaration; never a whole file | The two directives above. There is no file-wide form, and `godlint-ignore-enclosing` at the top level is reported rather than ignored |
| Optional owner and expiry, with an expired suppression reported so exceptions cannot accumulate silently | `owner=` and `expires=`, enforceable with `require-owner` and `require-expiry`; an expiry in the past is reported |
| A report of every active suppression, so the total is auditable rather than discovered one grep at a time | `godlint suppressions` |

Two consequences are deliberate and documented in the reference: a defective directive
still suppresses and is reported against itself, so an expiry does not detonate into
unrelated findings; and `policy/accountable-suppression` cannot itself be suppressed.

This precedes the strict suites and the baseline work: promoting a rule to blocking is
only reasonable once a project has an accountable way to record the cases it cannot fix
yet. It also unblocks the fourth fixture class that
[the testing strategy](testing.md) previously had to defer, and the accountable-exception
row in the policy mapping below.

`policy/unused-suppression` is shipped. It reports a directive that names an enabled,
suppressible rule but silences no finding. A directive for a disabled rule is dormant,
not unused, so projects can adopt a rule gradually without manufacturing exception debt.

### Phase 3 — Calls and organization policy

Introduce `CallFact` with direct callee path, source range, enclosing function, and
literal arguments where unambiguous. Do not claim alias or type resolution until a
semantic capability exists.

| Rule | Confidence | Initial detection | Example policy |
| --- | --- | --- | --- |
| `architecture/restricted-call` | Shipped | High | Direct callee and macro match | Block direct process exits and debug output by default; configure calls such as `loadConfig` with `allow-in` path globs |
| `security/no-dynamic-execution` | Shipped | High | Direct JavaScript/Python callee match | Block JavaScript `eval`/`Function` and Python `eval`/`exec` |
| `security/direct-environment-read` | Shipped | High | Direct platform API match | Require a single configuration boundary |
| `reliability/explicit-timer-delay` | Shipped | High | Direct JavaScript/TypeScript timer calls with fewer than two arguments | Require an intentional delay value |
| `logging/no-production-log` | Shipped | High | Direct logging callee match | Ban debug logging outside approved paths |
| `reliability/network-timeout-required` | Medium | Configured known client calls | Require explicit timeout argument |

`architecture/restricted-call` establishes the direct-call boundary. It detects only
spelled, direct callee paths: aliases, computed properties, and type-mediated calls wait
for semantic analysis. Its built-in policy is deliberately narrow: JavaScript
`process.exit`, Python `sys.exit` and `os._exit`, and Rust `std::process::exit`. It does not ban file or
network I/O, subprocesses, or `unwrap`, because those need repository context before a default
can remain high-confidence.

Debug logging used to sit in that list and now has its own rule, because the two policies have
different shapes. A process exit is banned outright and excused per call site;
logging is permitted wherever a repository says it belongs, so
`logging/no-production-log` takes one `allow-in` for the whole class rather than an entry per
callee. Keeping both would also have reported the same call twice. Its defaults are the calls
that exist to be read during development — `console.log`, `console.debug`, `console.info`,
`console.trace`, Python `print` and `pprint.pprint`, and Rust `dbg!` — and each is bound to the
dialect that spells it, so a TypeScript function named `print` is not a Python built-in. It
deliberately leaves `console.error`, `console.warn`, `println!` and the `logging` module alone:
those are how a program talks to its user or its logger, not leftover debugging.

`security/no-dynamic-execution` and `security/direct-environment-read` ship with this
same call fact. The former is an AI-safety default; the latter centralizes global
configuration dependencies. A project can add its own architecture boundary with:

```yaml
rules:
  architecture/restricted-call:
    severity: error
    calls:
      - name: loadConfig
        allow-in:
          - "**/config.ts"
          - "**/config/**"
```

`reliability/explicit-timer-delay` covers JavaScript and TypeScript only. Omitting the
second argument from `setTimeout` or `setInterval` is valid code that schedules immediate
execution; Python and Rust timer APIs require their delay argument, so treating their
invalid calls as a shared policy finding would duplicate the language checker rather than
enforce an organization policy.

It reads the timer under its bare name and under a global receiver — `window`, `globalThis`
and `self` — because those spell the same function rather than a different one. A receiver
it does not know, such as `timers.setTimeout`, is left alone: that is the point at which
the callee stops being decidable without semantic analysis.

A comment is not an argument. `setTimeout(work /*, 100 */)` is reported, because the shape a
reader most wants caught is the one where the delay was commented out, and a rule that went
quiet exactly then would be worse than no rule.

Two limits remain, both consequences of counting spelled arguments rather than resolving
values. A spread, `setTimeout(...args)`, is reported even though the delay may travel inside
it — one argument is what the call site spells. An aliased timer, `const t = setTimeout`
followed by `t(work)`, is not reported, the same boundary
`architecture/restricted-call` draws. Both wait for semantic analysis rather than for a
heuristic that would guess.

### Phase 4 — Imports and repository graph

Introduce `ImportFact` first, then a repository graph only when an import-local rule
cannot answer the policy.

`ImportFact` is shipped. It carries the range that spells the module and reads the module from
it, the way every other fact reads its text, so the two cannot disagree. What counts as the
module is decided per language behind the vocabulary: a Rust `use` path or `extern crate` name,
the module of a Python `import` or `from ... import` with an alias seen through, and the source
string of a JavaScript or TypeScript `import` — including a re-export, which is an import edge
whatever it is spelled as.

It resolves nothing, the same boundary the call rules draw. `use a::{b, c}` is the module `a`
rather than two modules; `import a, b` in Python records the first name only; and a
`require()` call is a call fact rather than an import. A restricted module covers what lies
beneath it — `crate::internal` catches `crate::internal::deep` — by matching a whole segment,
so `crate::internals` is a different module and not a longer spelling of the same one.

A Rust alias is seen through — `use crate::internal as inner` is the module
`crate::internal` — matching what Python already did for `import x as y`. A brace list with no
common path, `use {crate::a, std::env}`, spells no single module and so yields no fact at all,
rather than a module named after the braces. What counts as a segment separator is the one the
language spells: `::` in Rust, `.` in Python, `/` in JavaScript and TypeScript. That is why a
restriction on `lodash` leaves `lodash.merge` alone — a separately published package rather
than a submodule — while catching `lodash/merge`.

`architecture/dependency-boundary` reads the same fact and takes an ordered list of layers.
Position in the list *is* the policy: a layer may depend on itself and on anything below it, and
a dependency that runs upward is reported. Because nothing is resolved, a layer declares both
sides of the question — the `paths` it contains and the `modules` that name it — since a file
path and an import spelling are not the same string and neither can be derived from the other.
A configuration that gives a layer only one of the two is rejected rather than silently
enforcing half a policy. A file no layer contains, or a module no layer names, is outside the
policy and reported by neither.

`security/forbidden-dependency` reads the same fact but asks a different question, which is why
it is a separate rule rather than a second spelling of `architecture/restricted-import`. That
rule matches a module path by prefix, for a boundary inside the repository. This one maps an
import to the *package* it comes from and matches that name exactly, so naming `lodash` once
catches `lodash`, `lodash/merge` and anything deeper, while leaving `lodash-es` alone — a
separately published package whose name merely starts the same way.

What counts as the package is per language: the first path segment in JavaScript and
TypeScript, or the first two when the name is scoped, so `@corp/legacy/deep` is `@corp/legacy`
and `@corp/allowed` is a different dependency; the first dotted segment in Python; and the
first `::` segment in Rust, which also covers `extern crate`. First-party code is not a
dependency and yields no package at all: a relative import in any language, and `crate`, `self`
or `super` in Rust, after a leading `::` is stripped so that `::serde` is read as the crate
`serde` rather than as nothing. Neither is a specifier containing a colon — which covers a
platform builtin such as `node:fs`, a URL, and a Windows path alike — nor one rooted at `/`.

Only a static `import`, `export ... from`, or the equivalent declaration in each language
produces an import fact. A `require()` call, a dynamic `import()`, and TypeScript's
`import x = require(...)` are calls rather than declarations, so neither import rule sees them.
That is the same boundary `architecture/restricted-call` covers from the other side, and it is
the first gap to close if these rules are relied on for dependency policy.

Declared order carries one thing only: the direction a dependency may run. Which layer a file
or a module belongs to is decided by the most specific declaration that covers it, not by
whichever was declared first. Those are two different orderings, and conflating them made a
layer that also matched a broader earlier layer report itself as crossing a boundary, and hid a
real violation inside a nested layer. So `src/app/api/**` wins over `src/app/**` for a file
beneath it, and `crate::app::api` wins over `crate::app` for a module beneath that, wherever
each appears in the list.

| Rule | Status | Required capability | Confidence | Notes |
| --- | --- | --- | --- | --- |
| `architecture/restricted-import` | Shipped | Direct import fact | High | Ban direct imports of internal or risky modules |
| `architecture/dependency-boundary` | Shipped | Import fact plus configured path layers | High | Enforce UI → application → domain → infrastructure direction |
| `architecture/no-cycle` | Planned | Repository graph | High | Report the complete cycle edge chain |
| `security/forbidden-dependency` | Shipped | Package/import mapping | High | Block dependencies by explicit policy |
| `architecture/filename-case` | Shipped | Repository path fact | Medium | Support scoped case conventions and generated-file exceptions |

`architecture/filename-case` reads no syntax at all — a path is the whole input — which makes it
the one rule that behaves identically in every language.

The convention comes from the **extension**, not the language, because that is where the
distinction actually lives. `.tsx` and `.jsx` are components and take `PascalCase`; `.ts`, `.js`,
`.mjs`, `.cjs`, `.mts` and `.cts` are modules and take `kebab-case`; `.rs`, `.py` and `.pyi` take
`snake_case`, which each ecosystem already enforces. So `Button.tsx` and `use-button.ts` are both
correct in the same directory, and a language-level default could not express that.

| Extension | Case |
| --- | --- |
| `.jsx`, `.tsx` | `PascalCase` |
| `.cjs`, `.cts`, `.js`, `.mjs`, `.mts`, `.ts` | `kebab-case` |
| `.py`, `.pyi`, `.rs` | `snake_case` |

A `scopes` entry declares the case for the paths it names and wins over the extension default, for
a repository whose layout says something the extension does not. Where two scopes name the same
path, the most specific declaration decides, the same rule
`architecture/dependency-boundary` follows for layers — so `ui/legacy/**` beats `ui/**` wherever
each sits in the list, and a narrower scope cannot become dead configuration by being declared
second. `allow` is checked before any scope and exempts a path outright, which is what a generated
file needs.

A leading or trailing separator is not part of the name, which is what `__init__.py`,
`__main__.py` and `_private.py` need: PEP 8's snake_case includes them, and `__init__.py` is a file
Python requires rather than one a project chose. A separator doubled *inside* a name still leaves
an empty segment and is reported, and a name that is only separators is not a name.

Every case is judged in ASCII in every position, so `Éclair.tsx` is reported rather than accepted
on its leading character alone. The name judged is what precedes the first dot, so
`widget.test.ts` is judged as `widget` — a compound extension is a suffix, not part of the name. A
file whose name begins with the dot has no name to judge and is left alone.

A finding is suppressible, the same as a file-size finding: both are reported against the whole
file, and neither is one of the two rules that hold suppressions to account.

### Phase 5 — Error handling and testing

The test rules need explicit framework configuration. Error-handler policy is syntax-defined and
ships at error through `recommended@1`.

| Rule | Status | Required fact | Confidence | Initial scope |
| --- | --- | --- | --- | --- |
| `reliability/empty-error-handler` | Shipped | Error-handler fact | High | Empty JavaScript/TypeScript `catch` bodies and Python `except` bodies containing only `pass` |
| `reliability/ignored-error` | Planned | Error result/exception fact | Medium | Direct discard patterns only |
| `testing/no-focused-test` | Planned | Test fact | High | Configured test framework names and attributes |
| `testing/no-skipped-test` | Planned | Test fact | High | Configured skip APIs and decorators |
| `testing/assertion-required` | Planned | Test and assertion facts | Medium | Configured framework assertion APIs |
| `testing/no-mock-production-module` | Planned | Mock and import facts | Medium | Explicit configured production boundaries |

### Phase 6 — Semantic and external capabilities

Only after the syntax/fact rules are stable:

- TypeScript type-aware checks through a pinned compiler worker.
- Optional Pyright, Clippy, and Cargo integrations with explicit capabilities.
- Alias-aware restricted API rules, floating promise checks, and type-based security
  policy.
- Baselines, changed-files mode, JSON/SARIF reporters, and GitHub annotations before
  promoting stricter rules broadly.

## Mapping the supplied TypeScript policy

| Supplied policy | Godlint decision |
| --- | --- |
| `max-lines: 300` | Implement `maintainability/file-size`; strict profile 300, recommended profile 500 |
| `max-lines-per-function: 30` | Existing `function-size`; strict profile 30 |
| `complexity: 10` | Shipped as `maintainability/decision-complexity`; recommended threshold 8, because Godlint counts a multiway branch once rather than once per `case` and the ESLint number does not transfer |
| `curly: all` | Delegate to ESLint, Biome, or formatter/language style tooling; Rust and Python do not share this syntax policy |
| `no-restricted-syntax(loadConfig)` | Implement as `architecture/restricted-call` with configured allowed paths |
| `unicorn/prevent-abbreviations` | Future opt-in `style/identifier-terms`; medium confidence because domain vocabulary needs project-specific allow-lists |
| `unicorn/filename-case` | Future `architecture/filename-case`; scoped paths and generated-file exemptions required |
| `padding-line-between-statements` | Delegate to formatting/style tooling; do not add a fragile cross-language whitespace rule |
| Type-aware `typescript-eslint` rules | Defer to the TypeScript semantic phase; Godlint must never claim type-aware parity from syntax alone |
| Per-file exceptions | [Inline suppression](suppressions.md) is shipped; scoped path overrides remain before strict suites |

## Ecosystem positioning

### JavaScript and TypeScript

Keep ESLint plus `typescript-eslint` for JavaScript/TypeScript correctness and typed
rules. The supplied `strictTypeChecked` and `stylisticTypeChecked` configuration is a
strong strict TypeScript policy, but type-aware linting requires TypeScript project
analysis and is intentionally slower. Godlint should complement it with shared
policy, not attempt to duplicate it before the semantic worker phase.

Unicorn is a strong source of policy ideas: filename case, required error messages,
expiring TODOs, explicit timer delays, and identifier conventions. Import only the
ideas that have a clear cross-language or organization-policy meaning. Do not copy its
entire recommended preset into Godlint.

Biome is useful as a benchmark for rule taxonomy and as a JavaScript formatter/linter
option. Its complexity, suspicious, style, and accessibility categories demonstrate
useful separation, but its JavaScript-specific rules should remain delegated.

### Python

Keep Ruff as the primary Python formatter/linter. Its default rules focus on broad
correctness and its ecosystem includes pyflakes, pycodestyle, pyupgrade, bugbear,
simplify, and import ordering. Pylint's design checks are useful inspiration for
parameter, branch, and nesting metrics. Godlint should own only the shared policy
counterparts and repository architecture rules.

### Rust

Keep `rustfmt` and Clippy mandatory. Clippy explicitly separates correctness,
suspicious, style, complexity, performance, pedantic, restriction, nursery, and Cargo
categories, and cautions against enabling its whole restriction group. Godlint should
not duplicate those broad Rust-only checks; it should apply the same organization
policy to Rust that it applies to JavaScript/TypeScript and Python.

### Semgrep

Use Semgrep as a research and prototyping reference for organization-specific pattern
rules. Its declarative patterns are valuable when validating a proposed policy, but
Godlint should first implement direct high-confidence facts so its diagnostics and
cross-language behavior remain deterministic and explainable. A user-defined pattern
SDK is post-MVP.

## Definition of done for each rule

Before merging a rule PR:

1. Add or reuse a fact without exposing parser nodes.
2. Add strict configuration validation and deterministic finding order.
3. Add a fixture mini-repository under
   `crates/godlint-cli/tests/fixtures/rules/<rule-id>/` with `godlint.yaml` and
   `expected.yaml`.
4. Cover every claimed source extension; omit unsupported languages explicitly.
5. Enable the rule for Godlint in the same PR, or record a temporary accountable
   exception.
6. Run format, Clippy, workspace tests, the E2E harness, and `godlint check .`.
7. Promote to a suite only after real-repository calibration.

## Sources

- [ESLint rule reference](https://eslint.org/docs/latest/rules/) and
  [function-size options](https://eslint.org/docs/latest/rules/max-lines-per-function)
- [typescript-eslint shared configurations](https://typescript-eslint.io/users/configs/)
  and [typed linting guidance](https://typescript-eslint.io/getting-started/typed-linting/)
- [eslint-plugin-unicorn rule catalog](https://www.npmjs.com/package/eslint-plugin-unicorn)
- [Ruff rule catalog](https://docs.astral.sh/ruff/rules/) and
  [configuration guidance](https://docs.astral.sh/ruff/linter/)
- [Clippy lint categories](https://doc.rust-lang.org/stable/clippy/index.html)
  and [rustfmt import configuration](https://doc.rust-lang.org/beta/nightly-rustc/src/rustfmt_nightly/config/mod.rs.html)
- [Biome rule catalog](https://biomejs.dev/linter/rules)
- [Pylint branch metric](https://pylint.readthedocs.io/en/v3.1.1/user_guide/messages/refactor/too-many-branches.html)
  and its sibling design metrics for return statements and statement count
- [ESLint `complexity` rule](https://eslint.org/docs/latest/rules/complexity), whose
  threshold Godlint borrows while counting a slightly different metric
- [Semgrep rule-writing ideas](https://semgrep.dev/docs/writing-rules/rule-ideas)
