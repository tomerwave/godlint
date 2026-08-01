# Deterministic quality gates

Status: product direction, reviewed 2026-08-01.

Godlint should make engineering intent executable without rebuilding every language ecosystem.
It owns repository policy and composes deterministic specialist tools for language, framework,
artifact, and supply-chain checks.

## Ownership rule

| Godlint owns | Companion tools own |
| --- | --- |
| Cross-language architecture, shared maintainability limits, restricted APIs, accountable exceptions, stable findings, and one policy configuration | Formatting, type checking, framework semantics, unused-code discovery, artifact schemas, vulnerability databases, secret signatures, and language-specific bug catalogues |

A companion gate is first-class when Godlint can pin its version, validate that it ran, preserve
its native rule IDs and locations, and expose one deterministic result. Godlint should not copy a
specialist rule merely to put its name under the Godlint namespace.

## Enforcement priorities

### Abstractions and inheritance

Static analysis cannot decide that a new abstraction is needed: that depends on expected change,
domain responsibility, and contracts not present in syntax. Enforce the consequences of a sound
abstraction instead:

- dependency direction with `architecture/dependency-boundary`;
- sibling isolation with `architecture/module-independence`;
- public surfaces with `architecture/no-internal-import`;
- configured infrastructure access with restricted imports and calls;
- cycles once the repository graph exists; and
- complexity and similarity as evidence that a boundary may be missing.

Inheritance depth, abstract-class shape, and subtype contracts require resolved types and differ by
language. Keep them in Clippy, Ruff/Pylint, ESLint/type-aware plugins, or Java tools until Godlint has
a semantic worker. Godlint must never recommend inheritance over composition from a numeric proxy.

### Similar code

Add experimental `maintainability/similar-block` with normalized token edit similarity, an 85%
default threshold, and minimum size gates. It must report the two ranges and exact score, preserve
identifiers and literals by default, and exclude only configured generated or fixture paths.

Adoption sequence:

1. Report exact large clones.
2. Enable 85% fuzzy matching as an opt-in warning.
3. Measure precision and runtime on Rust, Python, JavaScript, and TypeScript repositories.
4. Ratchet new duplication without forcing a repository-wide cleanup.

PMD CPD and jscpd are useful companion choices today. Godlint should still own a future
cross-language metric so the threshold and exception contract are identical across supported
languages.

### Tests that observe business behavior

An assertion count proves syntax, and line coverage proves execution. Neither proves that a test
would notice broken behavior. Mutation testing is the deterministic gate: change an operator,
branch, return value, or call; the relevant test suite must fail.

Godlint should add a companion-gate contract rather than implement mutation engines:

- map production paths to a pinned mutation command and machine-readable report;
- require changed production logic to be mutation-tested;
- fail on surviving or untested mutations, with explicit exclusions for equivalent mutants;
- preserve mutant location, mutation, test command, and engine version in output; and
- use a ratchet or changed-code threshold before enforcing a repository-wide score.

Use cargo-mutants for Rust, Stryker for JavaScript/TypeScript, mutmut or Cosmic Ray for Python, and
PIT for Java. `testing/assertion-required` remains useful hygiene, but it is not the business-logic
quality claim.

### Readability

Godlint should deepen readable structure, not whitespace taste. Existing size, nesting, parameter,
statement, return, condition, decision, and cognitive-complexity rules already cover the strongest
cross-language signals. Next candidates are:

| Candidate | Deterministic boundary |
| --- | --- |
| `style/identifier-terms` | Configured forbidden abbreviations and preferred domain terms; opt-in because vocabulary is repository-specific |
| `maintainability/similar-block` | Large token-similar blocks, never short phrases |
| `architecture/no-cycle` | Complete import-cycle edge chain, once repository graph facts exist |
| `policy/accountable-tool-suppression` | Native suppression must name diagnostics and include a reason |

Naming grammar, import order, expression simplification, and formatting remain language-linter or
formatter work.

### Errors and propagation

The default policy is: catch only to recover, translate, retry, add context, clean up, or cross an
application boundary. Otherwise let the error propagate.

| Policy | Direction |
| --- | --- |
| Empty handlers | Shipped as `reliability/empty-error-handler` |
| Catch and immediately rethrow the same error | Add `reliability/redundant-catch-rethrow` at high confidence |
| Return or branch out of `finally` | Add `reliability/no-control-flow-in-finally` at high confidence |
| Throw strings or unrelated values | Delegate today; revisit with type facts for language-correct error values |
| Domain-specific error hierarchy | Semantic phase: configured domain base types and inheritance/type resolution |
| Domain-to-HTTP mapping | Framework boundary: verify configured middleware owns translation; domain code must not depend on transport errors |
| Preserve an underlying cause while translating | Semantic phase: JavaScript `cause`, Python exception chaining, Rust error sources, and Java causes |

The desired architecture is a named domain error at the failure site, natural propagation through
application code, and one configured boundary that maps it to a stable user-facing response.
Godlint can enforce that only after it can resolve types and configured framework boundaries.

## Language profiles

Godlint should publish versioned reference profiles rather than install hidden tools. Repositories
choose the tools, pin versions, commit configuration, and run the same commands locally and in CI.

| Surface | Default companion | Godlint's role |
| --- | --- | --- |
| Rust | rustfmt, Clippy, cargo-mutants | Shared policy, architecture, and mutation-gate evidence |
| Python | Ruff formatter/linter, a type checker, mutmut or Cosmic Ray | Shared policy and repository boundaries |
| JavaScript/TypeScript | one formatter; ESLint plus typescript-eslint for typed rules | Shared policy and cross-language architecture |
| React | React Doctor | Do not copy React-specific state, effect, accessibility, or performance rules |
| Markdown | markdownlint-cli2 | Require a pinned config and accountable suppressions |
| GitHub Actions | actionlint plus zizmor | Godlint owns organization workflow policy; delegate syntax/types and security catalogue |
| Dockerfile | Hadolint | Keep only cross-repository container policy in Godlint |
| Kubernetes | kubeconform | Delegate schema validation, including configured custom-resource schemas |

Do not run two formatters over the same files. Prefer native structured output; use reviewdog only
as presentation, never as the source of pass/fail semantics.

## Deterministic tool map

| Tool | Keep it for | Do not duplicate in Godlint |
| --- | --- | --- |
| Knip | Unused files, exports, dependencies, binaries, unresolved imports, and configured dependency cycles in JS/TS | Entry-point and framework-plugin discovery |
| React Doctor | React/Next.js framework diagnostics | React-specific state, effect, performance, security, and accessibility catalogue |
| dependency-cruiser | Configured JS/TS dependency rules, cycles, orphans, and dependency classifications | JS/TS graph implementation while Godlint's cross-language graph is incomplete |
| actionlint | GitHub Actions schema, expression types, action inputs/outputs, reusable workflows, embedded ShellCheck/Pyflakes, and workflow semantics | Workflow language correctness |
| zizmor | GitHub Actions security, permissions, credential exposure, and reference attacks | CI security vulnerability catalogue |
| markdownlint | CommonMark/GFM consistency and custom Markdown rules | Markdown formatting and parser compatibility |
| Hadolint | Dockerfile AST and embedded ShellCheck rules | Dockerfile language best practices |
| kubeconform | Kubernetes and custom-resource schema validation | Manifest schema catalogue |
| OSV-Scanner | Dependency and image vulnerability matching against OSV data | Advisory database |
| Gitleaks | Secret signatures and history scanning | Secret detector catalogue |
| Semgrep | Organization patterns and proposed-rule prototyping | A generic user-defined pattern runtime in the MVP |
| PMD CPD or jscpd | Immediate clone detection while Godlint's similarity fact is researched | A permanent opaque replacement for Godlint's shared 85% policy |

Every external gate should be pinned, run with committed configuration, fail closed on scanner
errors, and upload machine-readable results. Any suppression must name the finding and carry owner,
reason, and expiry metadata where the tool permits it.

## Delivery order

1. Implement high-confidence error-flow syntax rules.
2. Prototype the 85% similarity fact and establish a multilingual corpus.
3. Define the companion-gate report contract and prove it with mutation testing.
4. Publish minimal per-language reference profiles with no overlapping formatter ownership.
5. Add tool-presence and pinned-configuration policy only after the contract is stable.
6. Add type-resolved error hierarchy and boundary rules in the semantic phase.

## Sources

- [Knip: how analysis works](https://knip.dev/explanations/how-knip-works) and [issue types](https://knip.dev/reference/issue-types)
- [React Doctor](https://github.com/millionco/react-doctor)
- [dependency-cruiser](https://github.com/sverweij/dependency-cruiser)
- [actionlint](https://github.com/rhysd/actionlint)
- [zizmor](https://github.com/zizmorcore/zizmor)
- [markdownlint](https://github.com/DavidAnson/markdownlint)
- [Hadolint](https://github.com/hadolint/hadolint)
- [kubeconform](https://github.com/yannh/kubeconform)
- [OSV-Scanner](https://github.com/google/osv-scanner)
- [Gitleaks](https://github.com/gitleaks/gitleaks)
- [Semgrep documentation](https://semgrep.dev/docs/)
- [PMD CPD](https://pmd.github.io/pmd/pmd_userdocs_cpd.html) and [jscpd](https://github.com/kucherenko/jscpd)
- [Stryker mutation testing](https://stryker-mutator.io/docs/), [cargo-mutants](https://mutants.rs/), [mutmut](https://mutmut.readthedocs.io/), and [PIT](https://pitest.org/)
