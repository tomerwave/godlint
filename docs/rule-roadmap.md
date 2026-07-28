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

Each suite is opt-in except `recommended@1`. A repository may then set thresholds and
path-scoped overrides, for example a looser file-size limit for generated code. Rules
must never silently exclude production, test, or worker code.

## Threshold profiles

Thresholds are policy choices, not language semantics. Godlint ships no hidden
universal number. The following profiles are the starting point for documented suites:

| Rule | Recommended | Strict | Source of the initial policy |
| --- | ---: | ---: | --- |
| `maintainability/file-size` | 500 effective lines | 300 effective lines | User policy: file-size ceiling requested at 500; existing TypeScript policy uses 300 |
| `maintainability/function-size` | 50 effective lines | 30 effective lines | Existing TypeScript `max-lines-per-function` policy |
| `maintainability/function-nesting` | 3 | 2 | Existing Godlint rule; lower is intentionally stricter |
| `maintainability/parameter-count` | 6 | 4 | Common design-lint threshold; tune per repository |
| `maintainability/cyclomatic-complexity` | 10 | 8 | Existing TypeScript `complexity: 10` policy |
| `maintainability/return-count` | 5 | 3 | Pylint's `too-many-return-statements` design metric. Counting `?` and implicit tail expressions raises Rust counts, so Rust-heavy repositories may need a looser threshold than the profile — this one uses 8 |
| `maintainability/function-statements` | 30 | 20 | Derived from the `maintainability/function-size` profile: a function sitting at its effective-line ceiling should not be almost entirely statements, so each profile allows about two thirds of its line budget as statements |

“Effective lines” exclude blank lines and comment-only lines when configured, matching
the current function-size contract. ESLint likewise makes blank-line and comment
handling explicit for function-size metrics. Both options default to enabled, because
a policy about function length is a policy about code, not about documentation.

The complexity threshold needs one caveat. Godlint counts language-defined branch
points — `if`, `else if`, loops, `match` and `switch` arms, `catch` and `except`
handlers, conditional expressions, and the Rust `?` operator — but deliberately does
not count short-circuit `&&`, `||`, `and`, or `or`. A boolean guard is one decision a
reader makes at one place, and counting its operands penalizes writing the condition
plainly. JavaScript and TypeScript `?.` and `??` are excluded on the same grounds and
this is a decision, not an oversight: they read as one access with a fallback rather
than as a branch a reader has to trace. The recommended threshold of 10 was borrowed from an existing ESLint
`complexity` setting, and tools differ on whether logical operators contribute, so a
threshold migrated from another linter should be re-checked against Godlint's own metric
rather than assumed equivalent.

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
`FunctionFact`; they need no semantic or repository-graph capability.

| Rule | Status | Confidence | Languages | Configuration | Dogfood default |
| --- | --- | --- | --- | --- | --- |
| `maintainability/function-size` | Shipped | High | All eleven supported extensions | `max-lines`, blank/comment policy | Error, 300 while Godlint is young |
| `maintainability/function-nesting` | Shipped | High | All eleven supported extensions | `max-depth` | Error, 3 |
| `maintainability/file-size` | Shipped | High | All eleven supported extensions | `max-lines`, blank/comment policy | Warning, 500 |
| `maintainability/empty-function` | Shipped | High | All eleven supported extensions except `.pyi` interface stubs | `allow-names` | Warning |
| `policy/todo-requires-reference` | Shipped | High | All comment syntaxes and Python docstrings | `markers`, `reference-prefixes` | Warning |
| `style/no-comments` | Shipped | High, but opinionated | All comment syntaxes and Python docstrings | `allow-doc-comments` | Error, documentation not permitted |

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

| New field or fact | Rules unlocked | Status | Confidence | Languages | Configuration | Dogfood default | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `parameter_count` | `maintainability/parameter-count` | Shipped | High | All eleven supported extensions | `max-parameters` | Warning, 6 | Count declared parameters only, excluding a method receiver (`self`, `&self`, `cls`); do not infer types or defaults initially |
| `decision_points` | `maintainability/cyclomatic-complexity` | Shipped | High | All eleven supported extensions | `max-complexity` | Warning, 10 | Count language-defined branch points, including the Rust `?` operator, but not short-circuit boolean operators; fixture each language explicitly |
| `return_count` | `maintainability/return-count` | Shipped | Medium | All eleven supported extensions | `max-returns` | Warning, 8 | Count every exit path: explicit `return`, the Rust `?` operator, and an implicit trailing expression. Keep opt-in because early returns are often clearer |
| `statement_count` | `maintainability/function-statements` | Shipped | Medium | All eleven supported extensions | `max-statements` | Warning, 30 | Count statements through nested blocks but not into nested functions, which are measured as functions in their own right; comments are not statements, and an expression-bodied arrow or lambda is one |

Phase 2 is complete. Its fact additions stay small and reusable for future policy.

### Accountable exceptions

Godlint can currently narrow a rule two ways, and neither can carry accountability. The
`exclude` globs remove a path from the scan, which suits generated code and deliberately
non-conforming test data. `allow-names` on `maintainability/empty-function` names a
function, and it applies repository-wide.

Neither expresses "this one site is a known exception". That matters because
[dogfooding policy](dogfooding.md) requires every exception to record a reason, an owner,
an issue reference, and an expiry, and none of those can be attached to a glob or to an
entry in a name list. The gap has a cost already visible in practice: an unavoidable
exception forces a rule to be weakened for the whole repository, which is how a
fixture-shaped allow-list entry can end up load-bearing for CI.

Inline suppression closes it. A comment at the site names the rule and carries the
justification, for example:

```text
godlint-ignore-next-line maintainability/function-size -- splitting this in #482
```

Requirements, before implementation:

1. A stable directive syntax in every supported comment syntax, including Python
   docstrings, resolved from `CommentFact` rather than by re-scanning text.
2. A required justification, so an unexplained suppression is itself a finding.
3. Scope limited to the following line or the enclosing declaration; never a whole file,
   because a file-wide directive is an `exclude` entry with less visibility.
4. Optional owner and expiry, with an expired suppression reported so exceptions cannot
   accumulate silently.
5. A report of every active suppression, so the total is auditable rather than discovered
   one grep at a time.

This precedes the strict suites and the baseline work: promoting a rule to blocking is
only reasonable once a project has an accountable way to record the cases it cannot fix
yet. It also unblocks the fourth fixture class that
[the testing strategy](testing.md) currently has to defer, and the accountable-exception
row in the policy mapping below.

### Phase 3 — Calls and organization policy

Introduce `CallFact` with direct callee path, source range, enclosing function, and
literal arguments where unambiguous. Do not claim alias or type resolution until a
semantic capability exists.

| Rule | Confidence | Initial detection | Example policy |
| --- | --- | --- | --- |
| `architecture/restricted-call` | High | Direct configured callee match | `loadConfig` only in `config.ts`-like modules |
| `security/direct-environment-read` | High | Direct platform API match | Require a single configuration boundary |
| `reliability/explicit-timer-delay` | High | Direct timer calls with omitted delay | Require an intentional delay value |
| `logging/no-production-log` | Medium | Direct configured logging calls | Ban `console.log` / `print` outside approved paths |
| `reliability/network-timeout-required` | Medium | Configured known client calls | Require explicit timeout argument |

The first rule in this phase should be `architecture/restricted-call`, because it is
the language-neutral form of the supplied `no-restricted-syntax` policy around
`loadConfig`.

### Phase 4 — Imports and repository graph

Introduce `ImportFact` first, then a repository graph only when an import-local rule
cannot answer the policy.

| Rule | Required capability | Confidence | Notes |
| --- | --- | --- | --- |
| `architecture/restricted-import` | Direct import fact | High | Ban direct imports of internal or risky modules |
| `architecture/dependency-boundary` | Import fact plus configured path layers | High | Enforce UI → application → domain → infrastructure direction |
| `architecture/no-cycle` | Repository graph | High | Report the complete cycle edge chain |
| `security/forbidden-dependency` | Package/import mapping | High | Block dependencies by explicit policy |
| `architecture/filename-case` | Repository path fact | Medium | Support scoped case conventions and generated-file exceptions |

### Phase 5 — Error handling and testing

These rules need explicit framework or language configuration. They start as warnings
until real-project fixtures establish precision.

| Rule | Required fact | Confidence | Initial scope |
| --- | --- | --- | --- |
| `reliability/empty-error-handler` | Error-handler fact | High | Empty catch / except bodies with documented exclusions |
| `reliability/ignored-error` | Error result/exception fact | Medium | Direct discard patterns only |
| `testing/no-focused-test` | Test fact | High | Configured test framework names and attributes |
| `testing/no-skipped-test` | Test fact | High | Configured skip APIs and decorators |
| `testing/assertion-required` | Test and assertion facts | Medium | Configured framework assertion APIs |
| `testing/no-mock-production-module` | Mock and import facts | Medium | Explicit configured production boundaries |

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
| `complexity: 10` | Implement after `decision_points` fact; recommended threshold 10 |
| `curly: all` | Delegate to ESLint, Biome, or formatter/language style tooling; Rust and Python do not share this syntax policy |
| `no-restricted-syntax(loadConfig)` | Implement as `architecture/restricted-call` with configured allowed paths |
| `unicorn/prevent-abbreviations` | Future opt-in `style/identifier-terms`; medium confidence because domain vocabulary needs project-specific allow-lists |
| `unicorn/filename-case` | Future `architecture/filename-case`; scoped paths and generated-file exemptions required |
| `padding-line-between-statements` | Delegate to formatting/style tooling; do not add a fragile cross-language whitespace rule |
| Type-aware `typescript-eslint` rules | Defer to the TypeScript semantic phase; Godlint must never claim type-aware parity from syntax alone |
| Per-file exceptions | Implement first-class scoped overrides and accountable exceptions before strict suites |

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
