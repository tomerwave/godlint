# Go support roadmap

This document records the feasibility audit and delivery roadmap for Go source support in Godlint.
The first analyzer tranche is implemented in PR #306; the table remains the source of truth for
what is supported, what still needs Go-specific facts, and what should not be ported unchanged.

Godlint currently has 60 registered rules. A Go repository can already use repository, Git,
dependency, and CI rules because those rules inspect metadata or workflow YAML rather than source
language syntax. The work below separates the delivered analyzer coverage from the remaining
language-specific work.

## Delivery status

PR #306 delivers Go discovery, parsing, shared facts, fixtures, and the direct structural, policy,
logging, environment, hash, shell, and basic testing rules marked `Yes` below. The remaining `Adapt`
items are deliberately future work: each needs a narrowly defined Go fact or policy decision rather
than a superficial syntax match. The `No` items stay excluded because their source-language
constructs do not exist in Go.

## Decision legend

| Mark | Meaning |
| --- | --- |
| Yes | The existing rule can consume shared facts once a Go analyzer emits them. |
| Adapt | The rule needs Go-specific facts, catalogues, or policy decisions. |
| No | Go has no corresponding construct, so implementing the rule for Go would be misleading. |
| Existing | The rule already applies to Go repositories without Go parser support. |

## Go source rules

| Rule | Go | Decision and reason |
| --- | --- | --- |
| `maintainability/function-size` | Yes | Go functions and methods map directly to function facts. |
| `maintainability/function-nesting` | Yes | Go blocks and control flow provide the required nesting facts. |
| `maintainability/file-size` | Yes | Effective line counting is language-independent. |
| `maintainability/empty-function` | Yes | Empty Go bodies are directly decidable. |
| `policy/todo-requires-reference` | Yes | Go comments use the existing marker/reference model. |
| `maintainability/parameter-count` | Yes | Go functions and methods expose parameters directly. |
| `maintainability/decision-complexity` | Yes | Count Go `if`, `switch`, `select`, and guarded branches. |
| `maintainability/condition-complexity` | Yes | Go boolean expressions provide the required operators. |
| `maintainability/cognitive-complexity` | Yes | Go control-flow facts fit the shared metric. |
| `maintainability/return-count` | Yes | Go `return` statements are explicit syntax facts. |
| `maintainability/function-statements` | Yes | Go statements can be counted without semantic typing. |
| `style/no-comments` | Yes | Go line and block comments are parser facts. |
| `style/no-commented-code` | Yes | It consumes comment text, independent of Go semantics. |
| `maintainability/no-duplicate-string` | Yes | Go string literals can feed the existing literal analysis. |
| `policy/accountable-suppression` | Yes | Go suppression directives are comments. |
| `policy/unused-suppression` | Yes | Go ranges and suppression comments can be tracked. |
| `architecture/restricted-call` | Yes | Go call expressions can be matched against policy catalogues. |
| `architecture/restricted-import` | Yes | Go import declarations are explicit and deterministic. |
| `architecture/dependency-boundary` | Adapt | Requires Go package/import-path facts rather than file-local imports. |
| `architecture/module-independence` | Adapt | Requires a Go package graph and stable package identity. |
| `architecture/filename-case` | Yes | Paths are language-independent. |
| `security/direct-environment-read` | Yes | Detect `os.Getenv`, `os.LookupEnv`, and configured equivalents. |
| `security/forbidden-dependency` | Adapt | Read dependency identities from `go.mod`. |
| `security/no-shell-command` | Yes | Detect `os/exec` calls and configured wrappers. |
| `security/no-weak-hash` | Yes | Detect Go MD5/SHA-1 packages and constructors. |
| `security/no-insecure-random` | Yes | Distinguish `math/rand` from `crypto/rand`. |
| `logging/no-production-log` | Yes | Detect `fmt.Print*`, `log.Print*`, and configured loggers. |
| `testing/no-empty-test` | Yes | Detect `TestXxx(*testing.T)` functions with empty bodies. |
| `testing/no-skipped-test` | Yes | Detect `t.Skip`, `t.Skipf`, and `t.SkipNow`. |
| `testing/no-sleep-in-test` | Yes | Detect `time.Sleep` in test functions. |
| `testing/no-test-helper-in-production` | Yes | Use `_test.go` boundaries and `t.Helper` facts. |
| `testing/no-network-in-unit-test` | Adapt | Requires a Go network-call catalogue. |
| `testing/no-randomness-without-seed` | Adapt | Must account for Go-version-specific `math/rand` behavior. |
| `testing/assertion-required` | Adapt | Go’s standard library has no assertion primitive; decide whether `t.Error`/`t.Fatal` count. |
| `reliability/explicit-timer-delay` | Adapt | Map the policy to `time.AfterFunc`, timers, tickers, and goroutine scheduling. |
| `reliability/network-timeout-required` | Adapt | Requires HTTP client timeout, request-context, and transport-deadline facts. |
| `reliability/no-control-flow-in-finally` | No | Go has no `finally`; `defer` has different semantics and needs a separate rule. |
| `reliability/redundant-catch-rethrow` | No | Go has no catch/rethrow handlers; error wrapping is a different rule. |
| `reliability/empty-error-handler` | No | Go has no exception-handler construct. |
| `testing/no-focused-test` | No | Go’s test framework has no focused-test marker such as `.only`. |
| `security/no-dynamic-execution` | No | Go has no native `eval`-style execution construct; reflection/plugins would be a different policy. |

## Repository, Git, dependency, and CI rules

These rules already apply to Go repositories. They do not require Go parser support.

| Rule | Go | Decision and reason |
| --- | --- | --- |
| `git/branch-naming` | Existing | Reads branch metadata. |
| `repository/no-committed-secret-file` | Existing | Reads repository paths. |
| `dependencies/lockfile-version-drift` | Existing, with a boundary | Go’s `go.mod` has no own package version to compare with `go.sum`; the current own-version rule should not invent one. |
| `ci/bot-conditions` | Existing | Reads workflow YAML. |
| `ci/explicit-workflow-permissions` | Existing | Reads workflow permissions. |
| `ci/hardcoded-container-credentials` | Existing | Reads workflow YAML. |
| `ci/no-comments` | Existing | Reads workflow comments. |
| `ci/no-inline-script` | Existing | Reads workflow steps. |
| `ci/frozen-lockfile-install` | Adapt | Add Go workflow patterns such as `go mod download` and `go build -mod=readonly`. |
| `ci/no-monolithic-job` | Existing | Reads workflow structure. |
| `ci/no-silenced-failure` | Existing | Reads workflow failure controls. |
| `ci/overprovisioned-secrets` | Existing | Reads workflow permissions and secrets. |
| `ci/pin-third-party-actions` | Existing | Reads action references. |
| `ci/secrets-inherit` | Existing | Reads workflow inheritance. |
| `ci/stale-action-refs` | Existing | Reads workflow action references. |
| `ci/template-injection` | Existing | Reads workflow expressions. |
| `ci/unredacted-secrets` | Existing | Reads workflow scripts and environment files. |
| `ci/untrusted-github-env` | Existing | Reads workflow expressions. |

## Recommended delivery order for the remaining work

1. Add Go package and module facts for imports, package identity, `go.mod`, and dependency graphs.
2. Add Go-specific reliability facts for HTTP timeouts, contexts, timers, and randomness.
3. Decide the assertion policy before implementing `testing/assertion-required`.
4. Keep the three exception/focused-test rules out of Go unless a distinct Go-specific rule is
   proposed for their underlying concern.

## Repository follow-up

The implementation is tracked in [#302](https://github.com/tomerwave/godlint/issues/302),
[#303](https://github.com/tomerwave/godlint/issues/303), and [PR #306](https://github.com/tomerwave/godlint/pull/306).
Future `Adapt` items should be proposed as separate rule issues with valid, invalid, configuration,
and scoped-exclusion fixtures before implementation.
