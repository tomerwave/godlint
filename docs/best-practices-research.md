# Best-practices research catalogue

This is the source-linked candidate backlog for Godlint. It is intentionally broader
than the [rule roadmap](rule-roadmap.md): a roadmap is a delivery commitment, while this
document records what we evaluated, which tool should own it, and why.

The sources include Clean Code and Clean Architecture, company standards from Airbnb
and Google, official language tooling, and security guidance. A cited practice is not
automatically a Godlint default. It becomes one only when its detection is
deterministic, remediation is clear, intended language coverage is explicit, and
Godlint can dogfood it without weakening the policy.

## Decision key

| Decision | Meaning |
| --- | --- |
| **Shipped** | Godlint implements it now. |
| **Default next** | High-confidence rule suitable for `recommended@1` when implemented. |
| **Suite** | Deterministic and valuable, but belongs in a stricter named suite. |
| **Configurable** | Useful organization policy, unsafe as a universal default. |
| **Delegate** | A formatter, compiler, type checker, security scanner, or dependency tool has necessary context. |
| **Later semantic** | Needs resolution, types, or data flow that Godlint must not pretend to have. |

Only one suite exists today: `recommended@1`, which enables every shipped rule at `error`. A
`strict`, `testing@1` or `security@1` named below is a proposal, not a thing a repository can adopt
yet, and so is any "warning first" severity.

The detection column is a contract. Until semantic workers exist, Godlint can reliably
use direct syntax spelling, literal arguments, comments, paths, imports, and a
repository graph—not aliases, types, reachability, taint, framework behavior, or
resolved dependency metadata.

## Product boundary

| Area | Godlint owns | Delegate | Evidence |
| --- | --- | --- | --- |
| Cross-language policy | Shared thresholds, path policy, accountable exceptions, direct dangerous APIs | — | [Clean Architecture](https://www.informit.com/articles/article.aspx?p=2832399) |
| Architecture | Import boundaries, module cycles, allowed dependencies, ownership paths | Manifest health | [Clean Architecture](https://www.informit.com/articles/article.aspx?p=2832399) |
| Formatting | — | Prettier, Ruff format, rustfmt | [Prettier](https://prettier.io/docs/why-prettier.html), [Ruff formatter](https://docs.astral.sh/ruff/formatter/) |
| Type correctness | — | `tsc`, typescript-eslint, Pyright, rustc/Clippy | [typescript-eslint](https://typescript-eslint.io/users/configs/) |
| Security data flow | Direct known-dangerous syntax only | CodeQL, Semgrep, secret scanners | [CodeQL](https://codeql.github.com/codeql-query-help/), [Semgrep](https://semgrep.dev/docs/writing-rules/rule-ideas) |
| Dependency governance | Explicit forbidden-dependency policy | cargo-deny, OSV/Dependabot, registry scanners | [cargo-deny](https://docs.rs/cargo-deny/latest/cargo_deny/) |

## Cross-language maintainability and policy

| Candidate rule | What it enforces | Detection | Decision | Source |
| --- | --- | --- | --- | --- |
| `maintainability/file-size` | Bounds effective source lines. | Lines minus configured blanks/comments. | **Shipped**; 500, at `error`. | [ESLint](https://eslint.org/docs/latest/rules/), [Clean Code](https://openlibrary.org/books/OL26222911M/Clean_Code) |
| `maintainability/function-size` | Bounds a function's effective lines. | Shared function fact. | **Shipped**; 50, at `error`. | [ESLint](https://eslint.org/docs/latest/rules/) |
| `maintainability/function-statements` | Bounds imperative statements. | AST statement count, excluding nested functions. | **Shipped**. | [ESLint](https://eslint.org/docs/latest/rules/) |
| `maintainability/function-nesting` | Bounds nested control flow. | AST control-flow depth. | **Shipped**; 2, at `error`. | [ESLint complexity](https://eslint.org/docs/latest/rules/complexity) |
| `maintainability/decision-complexity` | Bounds branching a reader traces. | Shared decision-point fact. | **Shipped**; 5, at `error`. | [ESLint complexity](https://eslint.org/docs/latest/rules/complexity) |
| `maintainability/cognitive-complexity` | Bounds how hard a function is to follow, weighting nesting rather than counting branches. | Same traversal as the decision-point fact, different accumulator: +1 per flow break **plus the current nesting level**; a `switch` counts once however many arms. | **Default next**; the highest-value addition in this document. | [Sonar white paper](https://www.sonarsource.com/docs/CognitiveComplexity.pdf), [Sonar S3776](https://github.com/SonarSource/sonar-python/blob/master/python-checks/src/main/resources/org/sonar/l10n/py/rules/python/S3776.html), [empirical validation](https://arxiv.org/pdf/2007.12520) |
| `maintainability/line-length` | Bounds columns per line. | Line width; the open question is whether a string literal's overflow counts. | **Default next**; [#82](https://github.com/tomerwave/godlint/issues/82) carries the measurement. | [pycodestyle E501](https://docs.astral.sh/ruff/rules/line-too-long/) |
| `maintainability/parameter-count` | Flags oversized APIs. | Declared parameters excluding receiver. | **Shipped**; 4, at `error`. | [Google Python](https://google.github.io/styleguide/pyguide.html) |
| `maintainability/return-count` | Flags many exit paths. | Explicit and defined implicit exits. | **Shipped**; 6, at `error`. | [Pylint](https://pylint.readthedocs.io/en/v2.17.7/user_guide/messages/refactor/too-many-return-statements.html) |
| `maintainability/empty-function` | Rejects unintentionally empty bodies. | No meaningful body construct. | **Shipped**. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `maintainability/no-boolean-positional-parameter` | Avoids unexplained `true`/`false` arguments. | Splits by evidence: a literal boolean **default** is syntactic today; a boolean **type hint** is not. | **Suite** for the literal-default half, **Later semantic** for the typed half. | [Ruff FBT001/FBT002](https://docs.astral.sh/ruff/rules/boolean-type-hint-positional-argument/), [Clean Code F3 Flag Arguments](https://github.com/janaipakos/Clean-Code-Smells-and-Heuristics) |
| `maintainability/no-deep-callback` | Prevents callback pyramids. | Nested JS function depth. | **Suite**, JS/TS only. | [Airbnb](https://github.com/airbnb/javascript) |
| `maintainability/no-magic-number` | Requires named domain values. | Configured literal policy. | **Configurable**; context is essential. | [ESLint](https://eslint.org/docs/latest/rules/) |
| `maintainability/no-duplicate-string` | Flags repeated nontrivial literals. | Per-file literal tally. | **Configurable**; not universal. | [Clean Code G5 Duplication](https://github.com/janaipakos/Clean-Code-Smells-and-Heuristics) |
| `maintainability/no-long-parameter-list-public-api` | Applies a tighter limit at public boundaries. | Export/public-declaration fact. | **Later semantic**. | [Rust API guidelines](https://rust-lang.github.io/api-guidelines/) |
| `maintainability/no-god-object` | Flags one class/module with too many responsibilities. | Requires semantic/repository metrics. | **Do not automate initially**. | [Clean Code](https://openlibrary.org/books/OL26222911M/Clean_Code) |
| `policy/todo-requires-reference` | Makes deferred work traceable. | Comment marker plus issue reference. | **Shipped**. | [Clippy](https://doc.rust-lang.org/clippy/index.html) |
| `policy/fixme-requires-owner-expiry` | Makes temporary fixes accountable. | Directive fields. | **Default next**. | [OWASP code review](https://cheatsheetseries.owasp.org/cheatsheets/Secure_Code_Review_Cheat_Sheet.html) |
| `policy/accountable-suppression` | Requires reason, owner, and expiry for exceptions. | Parsed inline directive. | **Shipped**. | [Semgrep](https://semgrep.dev/docs/writing-rules/rule-ideas) |
| `policy/unused-suppression` | Removes stale exceptions. | Suppression matches no enabled finding. | **Shipped**. | [Clippy configuration](https://doc.rust-lang.org/clippy/lint_configuration.html) |
| `policy/no-broad-source-exclusion` | Makes excluded source paths explainable. | Exclusion glob intersects source. | **Suite**. | [OWASP code review](https://cheatsheetseries.owasp.org/cheatsheets/Secure_Code_Review_Cheat_Sheet.html) |
| `style/no-comments` | Enforces a code-explains-itself policy. | Comment facts with explicit exceptions. | **Shipped**, opinionated. | [Clean Code](https://openlibrary.org/books/OL26222911M/Clean_Code) |
| `style/no-commented-code` | Blocks dead code in comments. | Heuristic source-like comment. | **Do not implement**; unreliable — though Ruff ships it, so the counter-evidence should be weighed rather than ignored. For this repository `style/no-comments` subsumes it. | [Clean Code C5](https://github.com/janaipakos/Clean-Code-Smells-and-Heuristics), [Ruff ERA001](https://docs.astral.sh/ruff/rules/commented-out-code/) |
| `style/no-abbreviations` | Requires expanded identifier words. | Identifier spelling. | **Do not default**; domain vocabulary matters. | [Unicorn](https://github.com/sindresorhus/eslint-plugin-unicorn) |
| `style/no-nested-ternary` | Avoids dense multi-branch expressions. | Nested conditional AST. | **Delegate** to ESLint/Biome. | [Biome](https://biomejs.dev/linter/javascript/rules/) |

## Cross-language reliability and direct-security policy

| Candidate rule | What it enforces | Detection | Decision | Source |
| --- | --- | --- | --- | --- |
| `reliability/empty-error-handler` | Rejects catches that discard errors. | Handler has only placeholders. | **Shipped**. | [Google JS](https://google.github.io/styleguide/jsguide.html), [Ruff S110](https://docs.astral.sh/ruff/rules/) |
| `reliability/ignored-error` | Rejects discarded fallible results. | Language-specific error/result fact. | **Default next**; deliberate contracts required. | [Rust API guidelines](https://rust-lang.github.io/api-guidelines/dependability.html) |
| `reliability/network-timeout-required` | Requires timeout/deadline on approved clients. | Direct known call missing configured argument. | **Default next**, warning first. | [Ruff S113](https://docs.astral.sh/ruff/rules/), [Semgrep](https://semgrep.dev/blog/2020/writing-semgrep-rules-a-methodology/) |
| `reliability/retry-requires-bound` | Stops unbounded retry loops. | Known retry call lacks attempt/deadline. | **Suite**, API-specific. | [OWASP](https://cheatsheetseries.owasp.org/cheatsheets/Secure_Code_Review_Cheat_Sheet.html) |
| `reliability/no-blind-catch` | Rejects catching everything and continuing. | Caught type spelled as written: absent, `Exception`, `BaseException`, or a bare `catch` that then tests the type. | **Default next**; needs an `allow-in` path escape for process boundaries — a request handler, a worker loop, `main`. | [Ruff BLE001](https://docs.astral.sh/ruff/rules/blind-except/), [Google Python §2.4](https://google.github.io/styleguide/pyguide.html) |
| `reliability/no-vanilla-error` | Requires a specific error type, not a bare one or a non-error value. | `raise`/`throw` argument shape. | **Suite**; cross-language, and `throw "string"` is the JS half. | [Ruff TRY002](https://docs.astral.sh/ruff/rules/raise-vanilla-class/), [Airbnb `no-throw-literal`](https://github.com/airbnb/javascript/blob/master/packages/eslint-config-airbnb-base/rules/best-practices.js) |
| `reliability/no-discarded-error-context` | Rejects handlers that drop the cause. | Closure or handler ignores its error parameter — `.map_err(\|_\| MyError)`, `except E: raise MyError`. | **Suite**. | [Clippy `map_err_ignore`](https://emschwartz.me/your-clippy-config-should-be-stricter/) |
| `reliability/no-verbose-reraise` | Preserves the traceback on re-raise. | `except E as e: raise e` rather than bare `raise`. | **Suite**, Python first. | [Ruff TRY201](https://docs.astral.sh/ruff/rules/verbose-raise/) |
| `reliability/no-await-in-loop` | Stops serialising work that should be concurrent, and hammering a service in sequence. | Await inside a loop body. | **Suite**, JS/TS and Python. | [ESLint no-await-in-loop](https://eslint.org/docs/latest/rules/no-await-in-loop) |
| `reliability/no-closure-over-loop-variable` | Prevents the captured-loop-variable bug. | Function defined inside a loop referencing the loop variable. | **Default next**; the same defect exists in all four languages. | [Ruff B023](https://docs.astral.sh/ruff/rules/function-uses-loop-variable/), [Airbnb `no-loop-func`](https://github.com/airbnb/javascript/blob/master/packages/eslint-config-airbnb-base/rules/best-practices.js) |
| `reliability/no-float-equality` | Rejects `==` on floating point. | Comparison operand literals/annotations. | **Suite**. | [Clippy `float_cmp`](https://emschwartz.me/your-clippy-config-should-be-stricter/) |
| `reliability/explicit-timer-delay` | Makes timer duration intentional. | JS/TS timer misses delay. | **Shipped**. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `reliability/no-infinite-loop` | Rejects literal endless loops without escape. | Literal condition only. | **Suite**; no heuristic claims. | [ESLint](https://eslint.org/docs/latest/rules/) |
| `security/no-dynamic-execution` | Bans eval/Function/eval/exec families. | Direct callee spelling. | **Shipped**. | [Semgrep](https://semgrep.dev/docs/writing-rules/rule-ideas) |
| `security/direct-environment-read` | Centralizes environment access. | Direct API outside configured paths. | **Shipped**. | [ESLint no-process-env](https://eslint.org/docs/latest/rules/no-process-env) |
| `architecture/restricted-call` | Bans abrupt exit and configured APIs. | Direct callee + allowed path. | **Shipped**. | [Clippy](https://doc.rust-lang.org/clippy/index.html) |
| `logging/no-production-log` | Stops debug residue in product paths. | Direct known logging calls. | **Shipped**. | [Clippy](https://doc.rust-lang.org/clippy/index.html) |
| `security/no-hardcoded-secret` | Flags credentials in clear names/literals. | Identifier plus string literal, **plus a Shannon entropy threshold** — entropy is a pure function of the string, so it stays deterministic, and it is what stops `AKIAIOSFODNN7EXAMPLE` being reported. | **Suite**, warning; use a secret scanner too. The exemption has to be per-literal, which points at inline suppression rather than configuration. | [OWASP](https://cheatsheetseries.owasp.org/cheatsheets/Secure_Code_Review_Cheat_Sheet.html) |
| `security/no-insecure-random` | Restricts weak random APIs in security paths. | Direct API + configured path. | **Configurable**. | [OWASP](https://cheatsheetseries.owasp.org/cheatsheets/Secure_Code_Review_Cheat_Sheet.html) |
| `security/no-weak-hash` | Bans MD5/SHA-1 in security contexts. | Direct constructor/call. | **Configurable**; checksums exist. | [Ruff](https://docs.astral.sh/ruff/rules/) |
| `security/no-shell-command` | Bans configured process launch APIs. | Direct call/import. | **Configurable**; CLIs differ. | [Ruff](https://docs.astral.sh/ruff/rules/) |
| `security/no-tls-verification-disable` | Rejects explicit certificate verification disablement. | Known literal option/field. | **Suite**. | [OWASP](https://cheatsheetseries.owasp.org/cheatsheets/Secure_Code_Review_Cheat_Sheet.html) |
| `security/no-dangerous-deserialization` | Bans unsafe loaders. | Direct known loader call. | **Suite**, Python first. | [Ruff](https://docs.astral.sh/ruff/rules/) |
| `security/no-dangerous-html-sink` | Bans raw HTML sinks. | Direct known property/attribute. | **Suite**, JS/TS first. | [Google JS](https://google.github.io/styleguide/jsguide.html), [CodeQL](https://codeql.github.com/codeql-query-help/) |
| `security/no-raw-sql` | Flags direct raw query construction. | Direct known API only. | **Later semantic**; use CodeQL/Semgrep. | [CodeQL](https://codeql.github.com/codeql-query-help/) |
| `security/no-path-traversal-sink` | Tracks untrusted paths reaching files. | Taint/data flow. | **Delegate**. | [CodeQL](https://codeql.github.com/codeql-query-help/) |
| `security/no-command-injection-sink` | Tracks untrusted strings into subprocesses. | Taint/data flow. | **Delegate**. | [CodeQL](https://codeql.github.com/codeql-query-help/) |

## JavaScript and TypeScript candidates

| Candidate rule | What it enforces | Detection | Decision | Source |
| --- | --- | --- | --- | --- |
| `js/no-loose-equality` | Requires `===`/`!==`, with a policy for `== null`. | Equality token. | **Delegate** to ESLint. | [Google TypeScript](https://google.github.io/styleguide/tsguide.html) |
| `js/no-throw-literal` | Requires `Error` objects, not arbitrary thrown values. | Throw expression shape. | **Delegate** to ESLint. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `js/no-new-array` | Avoids ambiguous `new Array(length)` behavior. | `new Array` call. | **Delegate**. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `js/no-prototype-mutation` | Bans direct prototype modification. | Assignment/call on `.prototype`. | **Suite**, framework-exempt. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `js/no-getter-setter` | Avoids surprising stateful property reads. | Getter/setter declaration. | **Configurable**. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `js/require-switch-default` | Requires a default switch arm. | Switch lacks default. | **Delegate**; typed exhaustiveness is better. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `js/no-switch-fallthrough` | Requires an intentional fall-through marker. | Non-terminating case sequence. | **Delegate** to ESLint. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `js/no-for-in-array` | Avoids inherited-key array iteration. | Needs target type. | **Delegate**. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `js/no-duplicate-import` | Combines imports from a module. | Normalized direct module strings. | **Delegate**; local style. | [Google JS](https://google.github.io/styleguide/jsguide.html), [Airbnb](https://github.com/airbnb/javascript) |
| `js/no-import-cycle` | Prevents cyclic ES modules. | Repository import graph. | **Default next** as `architecture/no-cycle`. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `js/no-wildcard-import` | Avoids opaque namespace imports. | `import * as`. | **Configurable**. | [Airbnb](https://github.com/airbnb/javascript) |
| `js/no-default-export` | Prefers named exports. | Export declaration. | **Do not default**; ecosystem preference. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `js/require-es-modules` | Requires ESM in configured paths. | `require`/`module.exports`. | **Configurable**; runtime policy. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `js/import-extension-policy` | Requires extension where runtime needs it. | Relative import string. | **Configurable**; bundler-specific. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `js/no-non-null-assertion` | Prevents unsound `value!`. | TypeScript assertion syntax. | **Delegate** to typescript-eslint. | [Google TypeScript](https://google.github.io/styleguide/tsguide.html) |
| `js/no-type-assertion` | Restricts `as Type` casts. | TypeScript assertion syntax. | **Delegate**. | [Google TypeScript](https://google.github.io/styleguide/tsguide.html) |
| `js/no-floating-promise` | Requires explicit promise handling. | Type-aware promise detection. | **Delegate** to typescript-eslint. | [typescript-eslint](https://typescript-eslint.io/rules/) |
| `js/strict-boolean-expressions` | Disallows ambiguous truthiness. | Type checker. | **Delegate**. | [typescript-eslint](https://typescript-eslint.io/rules/strict-boolean-expressions/) |
| `js/no-unnecessary-condition` | Finds statically redundant conditions. | Type checker. | **Delegate**. | [typescript-eslint](https://typescript-eslint.io/rules/) |
| `js/no-unnecessary-type-parameter` | Removes generic APIs that add no information. | Type checker. | **Delegate**. | [typescript-eslint](https://typescript-eslint.io/rules/no-unnecessary-type-parameters/) |
| `js/no-console-debug` | Blocks debug logs in product paths. | Direct configured call. | **Shipped** through logging policy. | [Unicorn](https://github.com/sindresorhus/eslint-plugin-unicorn) |
| `js/curly-control-flow` | Requires braces for control structures. | Single-statement body. | **Delegate** to linter/formatter. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `js/one-statement-per-line` | Preserves readable statement layout. | Token/newline style. | **Delegate** to formatter. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `js/no-implicit-coercion` | Avoids unreadable coercion. | Unary/binary expression patterns. | **Delegate** to ESLint/Biome. | [Biome](https://biomejs.dev/linter/javascript/rules/) |
| `js/no-restricted-syntax` | Applies team-defined AST restrictions such as `loadConfig`. | Configured direct syntax/call. | **Shipped** in portable restricted-call form. | [ESLint](https://eslint.org/docs/latest/rules/) |
| `js/no-dangerous-regexp` | Flags regular-expression denial-of-service patterns. | Requires regex analysis. | **Delegate** to dedicated security tooling. | [CodeQL](https://codeql.github.com/codeql-query-help/) |
| `js/no-inner-html` | Avoids `innerHTML` when an API boundary is available. | Direct property assignment. | **Suite**, with framework allow-list. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `js/no-date-string-parse` | Rejects non-ISO date parsing. | Direct constructor/parser literal. | **Configurable**; locale policy. | [Unicorn](https://github.com/sindresorhus/eslint-plugin-unicorn) |

## Python candidates

| Candidate rule | What it enforces | Detection | Decision | Source |
| --- | --- | --- | --- | --- |
| `python/no-mutable-default` | Rejects list/dict/set defaults. | Literal mutable default AST. | **Default next**. | [Google Python](https://google.github.io/styleguide/pyguide.html), [Ruff B006](https://docs.astral.sh/ruff/rules/) |
| `python/no-call-in-default` | Avoids definition-time execution. | Call in default expression. | **Suite**; configured immutable allow-list. | [Ruff B008](https://docs.astral.sh/ruff/rules/) |
| `python/no-blind-except` | Rejects bare/broad exception handling. | No type or `BaseException`. | **Default next**, warning first. | [Ruff BLE001](https://docs.astral.sh/ruff/rules/) |
| `python/no-except-pass` | Rejects handler `pass`. | Error handler fact. | **Shipped** as empty error handler. | [Ruff S110](https://docs.astral.sh/ruff/rules/) |
| `python/no-except-continue` | Rejects silent continuation after failure. | Handler is `continue` only. | **Suite**. | [Ruff S112](https://docs.astral.sh/ruff/rules/) |
| `python/require-request-timeout` | Requires timeouts on known HTTP APIs. | Direct call missing keyword. | **Default next**. | [Ruff S113](https://docs.astral.sh/ruff/rules/) |
| `python/no-shell-true` | Rejects `shell=True` subprocess use. | Direct call + truthy keyword. | **Default next**, security suite first. | [Ruff S604](https://docs.astral.sh/ruff/rules/call-with-shell-equals-true/) |
| `python/no-partial-executable-path` | Requires full executable paths for configured calls. | Literal bare command path. | **Suite**; CLI exception likely. | [Ruff S607](https://docs.astral.sh/ruff/rules/) |
| `python/no-unsafe-yaml-load` | Requires safe YAML loading. | Direct `yaml.load` form. | **Suite**. | [Ruff](https://docs.astral.sh/ruff/rules/) |
| `python/no-tar-extractall` | Prevents unsafe archive extraction. | Direct `extractall`. | **Suite**. | [Ruff S202](https://docs.astral.sh/ruff/rules/) |
| `python/no-flask-debug` | Rejects production debug mode. | Known literal app/config option. | **Suite**. | [Ruff S201](https://docs.astral.sh/ruff/rules/) |
| `python/no-autoescape-false` | Keeps template autoescaping on. | Known constructor literal. | **Suite**. | [Ruff S701](https://docs.astral.sh/ruff/rules/) |
| `python/no-raw-sql` | Flags raw query APIs. | Direct known call only. | **Later semantic**. | [Ruff](https://docs.astral.sh/ruff/rules/) |
| `python/no-assert-false` | Uses explicit assertion error. | `assert False`. | **Delegate** to Ruff. | [Ruff B011](https://docs.astral.sh/ruff/rules/) |
| `python/no-return-in-finally` | Avoids swallowing errors in `finally`. | Return/break/continue AST. | **Delegate** to Ruff. | [Ruff B012](https://docs.astral.sh/ruff/rules/) |
| `python/no-global-mutable-state` | Limits mutation of module global state. | Needs scope/alias analysis. | **Later semantic**. | [Google Python](https://google.github.io/styleguide/pyguide.html) |
| `python/no-lambda-assignment` | Prefers `def` for named functions. | Lambda assignment. | **Delegate**. | [Google Python](https://google.github.io/styleguide/pyguide.html) |
| `python/no-none-equality` | Requires `is None`. | Comparison AST. | **Delegate** to Ruff. | [Google Python](https://google.github.io/styleguide/pyguide.html) |
| `python/no-assert-in-production` | Stops removable assertions validating input. | Assert in configured source path. | **Suite**. | [Ruff B011](https://docs.astral.sh/ruff/rules/) |
| `python/main-guard-required` | Requires `if __name__ == '__main__'`. | Top-level behavior heuristic. | **Do not implement**; framework-specific. | [Google Python](https://google.github.io/styleguide/pyguide.html) |
| `python/no-wildcard-import` | Avoids unknown imported symbols. | `from x import *`. | **Configurable**. | [Ruff](https://docs.astral.sh/ruff/rules/) |
| `python/no-hardcoded-password` | Flags credential-like defaults/arguments. | Identifier + literal. | **Suite**, warning. | [Ruff](https://docs.astral.sh/ruff/rules/) |
| `python/no-temp-file-race` | Bans predictable temporary paths. | Direct known API. | **Suite**. | [Ruff](https://docs.astral.sh/ruff/rules/) |
| `python/no-insecure-ssl-context` | Bans TLS checks disabled in direct config. | Direct known setting. | **Suite**. | [Ruff](https://docs.astral.sh/ruff/rules/) |
| `python/no-django-extra` | Restricts known unsafe ORM extension APIs. | Direct known method. | **Configurable**, framework suite. | [Ruff](https://docs.astral.sh/ruff/rules/) |
| `python/no-mutable-contextvar-default` | Avoids shared mutable `ContextVar` state. | Literal mutable default. | **Suite**. | [Ruff B039](https://docs.astral.sh/ruff/rules/) |

## Rust candidates

| Candidate rule | What it enforces | Detection | Decision | Source |
| --- | --- | --- | --- | --- |
| `rust/no-unwrap-production` | Prevents panic-on-error in production paths. | Method spelling/path cannot prove aliases. | **Delegate** to selected Clippy lint. | [Clippy](https://doc.rust-lang.org/clippy/index.html), [Clippy config](https://doc.rust-lang.org/clippy/lint_configuration.html) |
| `rust/no-expect-production` | Restricts `expect` outside allowed paths. | Direct method spelling only. | **Delegate** to Clippy. | [Clippy config](https://doc.rust-lang.org/clippy/lint_configuration.html) |
| `rust/no-dbg` | Rejects `dbg!` residue. | Macro spelling. | **Default next** via logging policy. | [Clippy](https://doc.rust-lang.org/clippy/index.html) |
| `rust/no-todo-unimplemented` | Rejects `todo!`/`unimplemented!`. | Macro spelling. | **Suite**; fixtures may suppress. | [Clippy](https://doc.rust-lang.org/clippy/index.html) |
| `rust/no-panic-production` | Restricts `panic!` in libraries/helpers. | Macro spelling/path scope. | **Configurable**. | [Clippy](https://doc.rust-lang.org/clippy/index.html) |
| `rust/no-process-exit` | Prevents arbitrary helper termination. | Direct `std::process::exit`. | **Shipped** through restricted calls. | [Clippy](https://doc.rust-lang.org/clippy/index.html) |
| `rust/no-unsafe-without-policy` | Requires accountable unsafe blocks. | Unsafe block/item fact. | **Later semantic**; lean on Clippy/review. | [Clippy configuration](https://doc.rust-lang.org/clippy/configuration.html) |
| `rust/no-wildcard-import` | Avoids opaque `use x::*`. | Import AST. | **Configurable**; prelude/re-export exception. | [Clippy configuration](https://doc.rust-lang.org/clippy/lint_configuration.html) |
| `rust/public-error-contract` | Documents errors on public fallible APIs. | Requires public/doc/error semantics. | **Delegate** to rustdoc/Clippy. | [Rust API guidelines](https://rust-lang.github.io/api-guidelines/) |
| `rust/validate-arguments` | Validates input at boundaries. | Semantic domain design. | **Do not automate generally**. | [Rust C-VALIDATE](https://rust-lang.github.io/api-guidelines/dependability.html) |
| `rust/newtype-for-invariants` | Uses types for validated domain values. | Semantic design. | **Do not automate**. | [Rust C-VALIDATE](https://rust-lang.github.io/api-guidelines/dependability.html) |
| `rust/no-unnecessary-clone` | Avoids needless copies. | Borrow/type/performance analysis. | **Delegate** to Clippy/profiling. | [Clippy](https://doc.rust-lang.org/clippy/index.html) |
| `rust/no-large-future` | Prevents oversized async state machines. | Compiler layout data. | **Delegate** to Clippy. | [Clippy config](https://doc.rust-lang.org/clippy/lint_configuration.html) |
| `rust/no-manual-memory-mistake` | Finds ownership/unsafe anti-patterns. | Compiler facts. | **Delegate** to rustc/Clippy. | [Clippy](https://doc.rust-lang.org/clippy/index.html) |
| `rust/forbidden-crate` | Bans an organization-disallowed crate. | Cargo manifest/resolution needed. | **Later semantic**; imports are already covered. | [cargo-deny](https://docs.rs/cargo-deny/latest/cargo_deny/) |
| `rust/license-policy` | Enforces approved crate licenses. | Registry/resolved lockfile data. | **Delegate** to cargo-deny. | [cargo-deny](https://docs.rs/cargo-deny/latest/cargo_deny/) |
| `rust/advisory-policy` | Blocks vulnerable/yanked crates. | Advisory database. | **Delegate** to cargo-deny/audit. | [cargo-deny](https://docs.rs/cargo-deny/latest/cargo_deny/) |
| `rust/msrv-policy` | Preserves declared minimum Rust support. | Compiler/toolchain metadata. | **Delegate** to Cargo/Clippy. | [Clippy configuration](https://doc.rust-lang.org/clippy/configuration.html) |
| `rust/no-unsafe-macro-metavars` | Restricts unsafe macro expansion patterns. | Macro expansion context. | **Delegate** to Clippy. | [Clippy configuration](https://doc.rust-lang.org/clippy/lint_configuration.html) |
| `rust/no-duplicate-crate` | Avoids duplicate dependency versions. | Resolved dependency graph. | **Delegate** to cargo-deny. | [cargo-deny](https://docs.rs/cargo-deny/latest/cargo_deny/) |

## Tests, architecture, repository, and CI

| Candidate rule | What it enforces | Detection | Decision | Source |
| --- | --- | --- | --- | --- |
| `testing/no-focused-test` | Rejects committed `.only`/focused markers. | Configured framework syntax. | **Default next** in `testing@1`. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `testing/no-skipped-test` | Rejects skipped/ignored tests without accountability. | Known call/attribute + suppression policy. | **Default next** in `testing@1`. | [Clippy config](https://doc.rust-lang.org/clippy/lint_configuration.html) |
| `testing/assertion-required` | Flags test bodies without recognized assertions. | Test declaration + configured assertion calls. | **Suite**, framework-configured. | [Clean Code](https://openlibrary.org/books/OL26222911M/Clean_Code) |
| `testing/no-test-helper-in-production` | Keeps mocking/testing modules out of product source. | Import fact + configured paths. | **Suite**. | [Semgrep](https://semgrep.dev/docs/writing-rules/rule-ideas) |
| `testing/no-sleep-in-test` | Prevents nondeterministic real waits. | Direct sleep/timer call in test path. | **Configurable**. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `testing/no-network-in-unit-test` | Keeps unit tests local and deterministic. | Direct network import/call in configured unit paths. | **Configurable**. | [OWASP](https://cheatsheetseries.owasp.org/cheatsheets/Secure_Code_Review_Cheat_Sheet.html) |
| `testing/no-randomness-without-seed` | Makes tests reproducible. | Direct random API in test path. | **Configurable**. | [Clean Code](https://openlibrary.org/books/OL26222911M/Clean_Code) |
| `testing/no-empty-test` | Rejects a test with no executable statement. | Test declaration plus the existing empty-body machinery. | **Default next**; the cheapest test rule, since `maintainability/empty-function` already does the work. | [Empty Test smell](https://testsmells.org/pages/testsmells.html) |
| `testing/no-conditional-test-logic` | Rejects `if`/`for`/`while` in a test, which can pass by not executing. | Control flow inside a test path. | **Suite**; reuses the nesting fact. | [Conditional Test Logic smell](https://testsmells.org/pages/testsmells.html) |
| `testing/assertion-message-required` | Stops a failure that cannot say which assertion broke. | Several assertion calls, none carrying a message argument. | **Suite**, framework-configured. | [Assertion Roulette](https://testsmells.org/pages/testsmells.html) |
| `testing/no-duplicate-assertion` | Flags the same assertion twice, and `assertEqual(x, x)`. | Repeated or self-comparing assertion arguments. | **Suite**. | [Duplicate Assert, Redundant Assertion](https://testsmells.org/pages/testsmells.html) |
| `testing/no-production-fixture-import` | Prevents fixtures becoming production dependencies. | Import facts + fixture paths. | **Suite**. | [Godlint testing strategy](testing.md) |
| `architecture/no-cycle` | Rejects cycles and reports the edge chain. | Repository import graph. | **Default next**. | [Google JS](https://google.github.io/styleguide/jsguide.html), [Clean Architecture](https://www.informit.com/articles/article.aspx?p=2832399) |
| `architecture/dependency-boundary` | Enforces outer layers depending inward only. | Import graph + configured paths. | **Shipped**. | [Clean Architecture](https://www.informit.com/articles/article.aspx?p=2832399) |
| `architecture/restricted-import` | Prevents boundary bypass imports. | Direct import plus allowed paths. | **Shipped**. | [Airbnb](https://github.com/airbnb/javascript) |
| `security/forbidden-dependency` | Bans known unwanted dependencies. | Direct import facts; later manifests. | **Shipped**. | [cargo-deny](https://docs.rs/cargo-deny/latest/cargo_deny/) |
| `architecture/filename-case` | Enforces path naming convention by scope/language. | Filename and extension. | **Shipped**. | [Google style guides](https://google.github.io/styleguide/) |
| `architecture/module-independence` | Requires a named set of modules not to depend on **each other** — sibling isolation, which layering cannot express. | Import graph plus configured module sets; generalises the existing layer machinery rather than adding a subsystem. | **Default next**; the cheapest architecture win available. | [import-linter Independence contract](https://import-linter.readthedocs.io/en/stable/contract_types/) |
| `architecture/no-internal-import` | Requires consumers to enter through a package's public surface rather than a private submodule. | Import spelling; `_private` and `internal` are conventions, not resolution. | **Suite**. | [Clean Code G36 Avoid Transitive Navigation](https://github.com/janaipakos/Clean-Code-Smells-and-Heuristics), [Ruff SLF001](https://docs.astral.sh/ruff/rules/private-member-access/) |
| `architecture/no-undeclared-dependency` | Rejects importing a package absent from the manifest, or declared dev-only and used in production. | Needs a manifest fact. Genuinely cross-language: one rule, three manifests, which is exactly the shape no single-language tool has. | **Later semantic** until the manifest fact exists. | [dependency-cruiser `no-non-package-json`, `not-to-dev-dep`](https://github.com/sverweij/dependency-cruiser/blob/main/doc/rules-reference.md) |
| `architecture/no-cross-feature-import` | Stops feature internals leaking across features. | Path-aware import policy. | **Configurable**. | [Clean Architecture](https://www.informit.com/articles/article.aspx?p=2832399) |
| `architecture/public-api-only-import` | Requires consumers to use package public entrypoints. | Import path policy. | **Configurable**. | [Rust API guidelines](https://rust-lang.github.io/api-guidelines/) |
| `architecture/no-layer-skip` | Prevents UI/infrastructure directly reaching domain internals. | Layered import graph. | **Suite**. | [Clean Architecture](https://www.informit.com/articles/article.aspx?p=2832399) |
| `repository/no-generated-source-edit` | Protects generated paths from manual edits. | Path/header/config facts. | **Configurable**, SCM-aware later. | [Google style guides](https://google.github.io/styleguide/) |
| `repository/required-license-header` | Requires source copyright/license header. | Leading comment text. | **Configurable**, conflicts with no-comments. | [Google JS](https://google.github.io/styleguide/jsguide.html) |
| `repository/no-committed-secret-file` | Blocks configured `.env`, key, and credential paths. | Exact path/glob policy. | **Suite**; pair with secret scanning. | [OWASP](https://cheatsheetseries.owasp.org/cheatsheets/Secure_Code_Review_Cheat_Sheet.html) |
| `ci/pin-third-party-actions` | Requires SHA-pinned GitHub Actions. | Workflow YAML `uses:` value. | **Default next** in `security@1`; needs YAML reader. | [GitHub Actions hardening](https://docs.github.com/en/code-security/tutorials/secure-your-organization/protect-against-threats) |
| `ci/explicit-workflow-permissions` | Requires least-privilege workflow permissions. | Workflow-level/job-level YAML fields. | **Default next** in `security@1`. | [GitHub Actions hardening](https://docs.github.com/en/code-security/tutorials/secure-your-organization/protect-against-threats) |
| `ci/no-pull-request-target-checkout` | Flags privileged PR workflow checkout patterns. | Trigger + checkout/run relation. | **Later semantic** or dedicated action security tool. | [GitHub Actions hardening](https://docs.github.com/en/code-security/tutorials/secure-your-organization/protect-against-threats) |
| `ci/no-untrusted-interpolation` | Stops PR-controlled strings reaching privileged shells. | Expression/data-flow analysis. | **Delegate**. | [GitHub Actions hardening](https://docs.github.com/en/code-security/tutorials/secure-your-organization/protect-against-threats) |
| `ci/require-concurrency-cancel` | Avoids stale duplicate deployments/tests. | Workflow YAML policy. | **Configurable**; deployment model differs. | [GitHub Actions docs](https://docs.github.com/en/actions) |

## Practices to retain outside Godlint

| Practice | Correct owner | Why | Source |
| --- | --- | --- | --- |
| JS/TS layout, quote, semicolon, wrapping policy | Prettier / Biome / ESLint | Formatters fix these consistently and end style arguments. | [Prettier](https://prettier.io/docs/why-prettier.html), [Biome](https://biomejs.dev/linter/javascript/rules/) |
| Python layout, quotes, whitespace, import ordering | Ruff format + Ruff lint | Ruff intentionally avoids formatter-overlapping defaults. | [Ruff rules](https://docs.astral.sh/ruff/rules/), [Ruff formatter](https://docs.astral.sh/ruff/formatter/) |
| Rust formatting/import layout | rustfmt / Clippy | Native parser, edition, and compiler knowledge. | [Clippy](https://doc.rust-lang.org/clippy/index.html) |
| JS unused variables/unreachable code/global mistakes | ESLint/Biome | Mature local correctness and auto-fixes. | [ESLint](https://eslint.org/docs/latest/rules/) |
| TS promise/nullability/narrowing/assertion safety | `tsc` + typescript-eslint | Requires the real type graph. | [typescript-eslint](https://typescript-eslint.io/rules/) |
| Python idioms, unused names, modernization | Ruff/Pylint | Large maintained language catalogue. | [Ruff](https://docs.astral.sh/ruff/rules/) |
| Rust ownership, lifetimes, unsafe/performance | rustc + selected Clippy lints | Needs compiler facts. | [Clippy](https://doc.rust-lang.org/clippy/index.html) |
| Advisory/license/source checks | cargo-deny and ecosystem scanners | Needs registry/advisory/resolution data. | [cargo-deny](https://docs.rs/cargo-deny/latest/cargo_deny/) |
| Tainted SQL/shell/path/XSS/deserialization flows | CodeQL/Semgrep | Needs source-to-sink data flow. | [CodeQL](https://codeql.github.com/codeql-query-help/), [Semgrep](https://semgrep.dev/docs/writing-rules/rule-ideas) |
| Secrets across history/files/providers | Dedicated secret scanner | Needs entropy/history/provider validation. | [OWASP](https://cheatsheetseries.owasp.org/cheatsheets/Secure_Code_Review_Cheat_Sheet.html) |

## What the catalogues actually disagree about

The practices are close to settled: every catalogue surveyed names roughly the same defects. What they
differ on is the **exemption**, and that is where each of them got into trouble.

- Ruff shipped `S113`, requests without a timeout, and then had to special-case `httpx`, which has a
  five-second default — producing
  [a run of false-positive reports](https://github.com/astral-sh/ruff/issues/12240). A rule that names
  library behaviour inherits that library's defaults, so `reliability/network-timeout-required` must take
  its call catalogue from configuration rather than hardcoding one.
- Gitleaks needed [entropy plus per-rule allow-lists](https://github.com/gitleaks/gitleaks), because a
  pattern alone reports every example key in every piece of documentation.
- Clippy keeps `unwrap_used` in `restriction`, **off by default**, precisely because whether a panic is
  acceptable depends on the caller.

In each case the detection was the easy part and the escape hatch was the hard part. That is the
argument for Godlint's accountable suppression — owner, reason, expiry — being a load-bearing feature
rather than an accessory: it is the only exemption mechanism in this survey that expires.

A second pattern worth recording: **a rule can be right and its fact can be wrong.** `reliability/empty-error-handler`
shipped mutation-clean and still reported nothing for `except ValueError: pass`, because the adapter
beneath it read the exception where the body should be. [#88](https://github.com/tomerwave/godlint/issues/88)
tracks that gap. Any candidate in this document needs a fixture per syntactic form that produces its
fact, not one fixture per rule.

## Adoption order

Ordered by what each step costs rather than by how much it is wanted, so the early steps need no new
subsystem:

1. **`maintainability/cognitive-complexity`.** No new fact, no new subsystem — the same traversal as
   `decision-complexity` with a different accumulator. Godlint already measures branching and nesting
   separately and combines neither, which is the exact gap the metric was designed for.
2. **Finish the swallowed-error family**: `reliability/ignored-error` — which is also the only one of
   these that covers Rust, deliberately excluded from `empty-error-handler` — then `no-blind-catch` and
   `no-closure-over-loop-variable`.
3. **Add a test declaration fact** and ship the test family behind it. One fact unlocks
   focused, skipped, empty, assertion-free and conditional-test rules, and nothing else enforces them
   uniformly across four languages.
4. **`architecture/module-independence`.** Generalises the layer machinery instead of adding a graph.
5. **Known-client call catalogues**, then `reliability/network-timeout-required`, with the catalogue in
   configuration for the reason given above.
6. **Python-only high-confidence rules**: mutable defaults, `shell=True`, unsafe loaders.
7. **A YAML reader**, then GitHub Actions pinning and explicit permissions. New file type, new parser
   surface — hence its position, not its value.
8. **Import-graph support**, then `architecture/no-cycle`. Genuinely wanted and genuinely a new
   subsystem; [#57](https://github.com/tomerwave/godlint/issues/57) records the design and is on hold.
9. **Semantic integrations** rather than imitations of type or taint analysis.

Every implementation must have valid, invalid, configuration, exclusion, and
suppression fixtures; deterministic output; all rule lines reached by tests; mutation
coverage; documentation; and dogfooding. A candidate is not done merely because another
linter has a similarly named rule, and — per [#88](https://github.com/tomerwave/godlint/issues/88) — a
fixture per syntactic form that produces the fact, not one per rule.
