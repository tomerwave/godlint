# Enforceable practices research

Status: proposed rule direction, reviewed 2026-08-01.

This document applies Godlint's decidability, false-positive, and threshold filters to four
sources of engineering guidance. The result is seven candidate policies. Each has a specific
fact or research gate; none should bypass the ordinary proposal process and go straight to
implementation.

The sources are inputs, not authorities. A practice belongs in Godlint only when repository
syntax can prove a violation deterministically and the exception can be configured precisely.

## Recommended candidates

| Candidate | Priority | Confidence | Blocked by |
| --- | --- | --- | --- |
| `architecture/no-module-load-side-effect` | P1 | High for configured calls | Call-scope fact |
| `reliability/no-control-flow-in-finally` | P1 | High | A fact for exits that escape a `finally` suite |
| `policy/accountable-tool-suppression` | P2 | High after syntax is specified | A native-suppression fact for comments and Rust attributes |
| `security/lockfile-required` | P2 | High when explicitly enabled | Manifest and repository-file facts |
| `security/pinned-container-image` | P2 | High | A Dockerfile reader |
| `security/dockerignore-required` | P2 | High for declared build contexts | Dockerfile, ignore-file, and build-context facts |
| `security/no-dynamic-module-path` | P3 | Medium | Corpus calibration and dynamic-import call coverage |

### `architecture/no-module-load-side-effect`

Report a configured direct call when it executes at module load rather than inside a function.
The Node guidance identifies network and database calls at module scope as hidden startup work;
the same policy is useful in Python when a repository names the calls that have effects.

- **Detection:** a configured callee whose call fact has no enclosing function.
- **Languages:** JavaScript, TypeScript, and Python. Ordinary runtime calls at Rust module scope
  are already rejected by the language.
- **Configuration:** callee names and `allow-in` path globs; ship with no guessed effect catalogue.
- **Remediation:** move the call behind an explicit function or exempt an intentional bootstrap
  module.
- **False-positive boundary:** registration, tracing, and dependency-injection setup can be
  intentional module initialization. Path exemptions express that policy without guessing which
  calls are pure.

This is preferable to a general "no side effects" rule. Syntax can prove where a call occurs; it
cannot prove whether an arbitrary function is pure.

### `reliability/no-control-flow-in-finally`

Report `return`, `break`, or `continue` when it leaves a JavaScript, TypeScript, or Python
`finally` suite. That control flow can replace an active return value or exception, which makes
failures disappear.

- **Detection:** an exit statement whose target is outside the nearest `finally` suite.
- **Languages:** JavaScript, TypeScript, and Python. Rust has no `finally` construct.
- **Configuration:** severity only; an intentional override uses an accountable inline
  suppression.
- **Remediation:** calculate cleanup state inside `finally`, then return or branch after it.
- **False-positive boundary:** a deliberately overriding return is legal, but its risk is the
  policy. The existing suppression mechanism is the precise exception.

PEP 8 states this recommendation directly. Unlike most PEP 8 guidance, it is a reliability
property shared by two supported dialects rather than a Python formatting convention.

### `policy/accountable-tool-suppression`

Report a native linter or type-checker suppression that is broader or less accountable than its
ecosystem permits. Coding agents frequently silence a diagnostic instead of fixing it; that
decision should remain specific and reviewable.

The first supported forms should be:

| Dialect | Report | Prefer |
| --- | --- | --- |
| Rust | `#[allow(...)]` without a reason | `#[expect(..., reason = "...")]` for an expected lint |
| JavaScript/TypeScript | `eslint-disable` without rule names or a description | A scoped directive naming rules and why |
| Python | bare `# noqa` or `# type: ignore` | A directive naming diagnostic codes and why |

The language adapters must define what counts as a diagnostic code and a reason. Do not search raw
source in the rule. Expiry and ownership can follow only after one metadata syntax works across all
three dialects; the initial rule should not invent incompatible per-language fields.

The Rust source specifically recommends fixing warnings, using `expect` rather than permanent
`allow`, and recording why. This candidate generalizes that accountability principle without
pretending the directive syntax is identical across languages.

### `security/lockfile-required`

Require an enabled application manifest to have exactly one recognized lockfile in its declared
repository scope. Lockfiles make the dependency graph reviewed and tested in CI the graph used
elsewhere.

- **Detection:** configured manifest kinds and their accepted lockfile names.
- **Configuration:** application roots, manifest kind, accepted lockfiles, and path exemptions.
- **Remediation:** generate and commit the ecosystem lockfile.
- **False-positive boundary:** publishable Rust and Python libraries may intentionally test broad
  dependency ranges. The rule must be opt-in or know that a configured root is an application; file
  presence alone cannot decide that.

The rule should not validate lockfile freshness. Package managers already do that more accurately.

### `security/pinned-container-image`

Report a Docker `FROM` reference that uses `latest`, omits a tag, or lacks a digest when the
configured policy requires immutable images.

- **Detection:** parsed `FROM` references, including every stage and `ARG` indirection that resolves
  to a literal in the same Dockerfile.
- **Configuration:** `ban-latest`, `require-tag`, or `require-digest`; trusted registries and path
  exemptions.
- **Remediation:** use an explicit version tag or digest at the configured strength.
- **False-positive boundary:** teams differ on whether a version tag is sufficient. The rule must
  encode that choice rather than treating every mutable tag as equally wrong.

### `security/dockerignore-required`

Require each declared Docker build context to contain a `.dockerignore` and exclude configured
sensitive paths such as `.env`, `.git`, `.aws`, and package-manager credentials.

- **Detection:** evaluate ignore semantics relative to each configured build context.
- **Configuration:** build contexts and required exclusion patterns.
- **Remediation:** create or strengthen the context's `.dockerignore`.
- **False-positive boundary:** a Dockerfile's directory is not necessarily its build context. The
  rule must require declared contexts rather than infer them from file location.

### `security/no-dynamic-module-path`

Report a non-literal path passed to JavaScript `require` or dynamic `import`, or to configured
Python dynamic-import APIs. A value-derived module path expands the executable surface and can be
dangerous when input is not trusted.

- **Detection:** a known dynamic loader whose module argument is not a literal.
- **Configuration:** loader names and `allow-in` paths; start at warning.
- **Remediation:** use an explicit allowlist that maps input to literal module paths.
- **False-positive boundary:** plugin hosts and dependency-injection containers load modules by
  design. Without taint analysis, a variable is not proof of attacker control, so this cannot enter
  `recommended@1` until a corpus shows that the opt-in policy is useful.

## Already covered

These sources also validate policy Godlint already ships. No duplicate rule is needed.

| Source concept | Existing Godlint policy |
| --- | --- |
| Business components, layers, dependency inversion | `architecture/dependency-boundary` and `architecture/module-independence` |
| Explicit public surfaces and no deep coupling | `architecture/no-internal-import` and `architecture/restricted-import` |
| Centralized, environment-aware configuration | `security/direct-environment-read` |
| Ban dynamic execution | `security/no-dynamic-execution` |
| Treat child processes carefully | `security/no-shell-command` and configurable `architecture/restricted-call` |
| Keep debug output out of production | `logging/no-production-log` |
| Avoid empty error handling | `reliability/empty-error-handler` |
| KISS and the measurable edge of single responsibility | Function size, nesting, statement, parameter, condition, decision, and cognitive-complexity rules |
| Stable module filenames | `architecture/filename-case` |
| Restrict an ecosystem-specific dependency such as `anyhow` to binary paths | `security/forbidden-dependency` with scoped exceptions |

## Delegate or do not build

| Guidance | Decision |
| --- | --- |
| PEP 8 layout, whitespace, import grouping, naming, singleton comparison, lambda assignment, and Python idioms | Delegate to Ruff or another Python linter. These are language conventions, not repository policy. |
| Rust cloning, borrowing, iterator selection, `unwrap`, `expect`, panic use, and Clippy lint groups | Delegate to Clippy. Several require type information and all are Rust-specific. Godlint may enforce a repository-specific call restriction, but should not duplicate Clippy's catalogue. |
| JavaScript strict equality, `const`, arrow functions, callback style, and import placement | Delegate to ESLint or Biome. |
| SRP, OCP, LSP, ISP, composition over inheritance, and YAGNI | Do not claim direct enforcement. Their violation depends on responsibilities, contracts, or product need, none of which syntax proves. Enforce concrete dependency boundaries and complexity ceilings instead. |
| DRY as a duplicate-block rule | Do not build from this evidence. Text duplication is not the same as duplicated knowledge, and generated code, tests, and deliberate parallel implementations create noisy matches. Reconsider only with a precise corpus and exemption model. |
| Law of Demeter as a member-chain limit | Do not build as a shared default. Fluent APIs and Rust iterator chains make length a poor proxy for coupling. |
| Promise handling and EventEmitter error listeners | Semantic phase. Correctness depends on knowing that an expression is a promise or an object is an EventEmitter. |
| Error-path tests and per-test database data | Validate through coverage and integration-test design. Source syntax cannot prove that the failure behavior was exercised or that records are isolated. |
| Secret detection and dependency vulnerability scanning | Delegate to dedicated scanners. Godlint can enforce use boundaries, but should not maintain secret signatures or advisory databases. |

## Delivery order

1. Add call scope and implement `architecture/no-module-load-side-effect`.
2. Add the focused `finally` exit fact and implement `reliability/no-control-flow-in-finally`.
3. Specify native suppression forms with fixtures before implementing
   `policy/accountable-tool-suppression`.
4. Add repository manifest facts, then implement `security/lockfile-required`.
5. Add one Dockerfile/build-context subsystem, then deliver both container rules from it.
6. Measure dynamic module loading in real repositories before promoting its candidate beyond
   warning.

Each rule still needs its own proposal, valid and invalid examples, configuration cases,
suppression fixture, corpus calibration where relevant, and dogfooding before implementation.

## Sources

- [Node.js Best Practices, reviewed at `dc3d60c`](https://github.com/goldbergyoni/nodebestpractices/tree/dc3d60c29d5483d9ea99cf261bbd6203516a2ba7)
- [SOLID, DRY, KISS, and related principles](https://scalastic.io/en/solid-dry-kiss/)
- [Apollo Rust Best Practices, reviewed at `f388056`](https://github.com/apollographql/rust-best-practices/tree/f38805670e426da744b6e46f14bc11ad255c7e88)
- [PEP 8](https://peps.python.org/pep-0008/)
