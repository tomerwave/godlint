# Rule reference

Twenty-two rules are implemented. Every one has an identifier of the form `family/name`, which is
what a configuration entry and a suppression directive both name. [The rule roadmap](rule-roadmap.md)
records the families still to come, and the reasoning behind each threshold `recommended@1` sets.

## Maintainability

| Rule | What it measures |
| --- | --- |
| `maintainability/file-size` | Effective lines in a file |
| `maintainability/function-size` | Effective lines in a function |
| `maintainability/function-nesting` | How deeply control-flow blocks nest inside a function |
| `maintainability/parameter-count` | Declared parameters, excluding a method receiver |
| `maintainability/decision-complexity` | Branch points in a function |
| `maintainability/return-count` | Exit paths from a function, explicit or implicit |
| `maintainability/function-statements` | Statements in a function |
| `maintainability/empty-function` | Function bodies that appear unintentionally empty |

`decision-complexity` counts a `match` or `switch` once rather than once per arm, and a guard on an
arm counts. `function-statements` counts through nested blocks but not into nested functions.

## Policy

| Rule | What it reports |
| --- | --- |
| `policy/todo-requires-reference` | A TODO-style marker with no issue reference |
| `policy/accountable-suppression` | A suppression that cannot account for itself |
| `policy/unused-suppression` | A suppression that no longer silences an enabled finding |

Neither policy rule about suppressions can itself be suppressed. See
[inline suppression](suppressions.md).

## Style

| Rule | What it reports |
| --- | --- |
| `style/no-comments` | Commentary where the code should speak for itself |

## Security

| Rule | What it reports |
| --- | --- |
| `security/no-dynamic-execution` | JavaScript `eval`, `Function`, `new Function`; Python `eval`, `exec` |
| `security/direct-environment-read` | Environment access outside a configuration boundary |
| `security/forbidden-dependency` | An import of a package the project has ruled out |

## Reliability

| Rule | What it reports |
| --- | --- |
| `reliability/explicit-timer-delay` | A JavaScript or TypeScript timer that omits its millisecond delay |
| `reliability/empty-error-handler` | An error handler whose body discards the error |

### What counts as an empty handler

`reliability/empty-error-handler` reports a `catch` or `except` body that holds nothing but a
placeholder. `pass`, `...` and a lone `;` are placeholders — and so is a comment. A comment neither
handles the error nor re-raises it, and Godlint already has a way to say a swallow is deliberate:
an [inline suppression](suppressions.md), which carries an owner and an expiry that a comment cannot
be held to.

A body that evaluates anything else is left alone, including a lone string literal.

Rust is out of scope. It has no `catch`, and a discarded `Result` is `reliability/ignored-error` on
[the roadmap](rule-roadmap.md) rather than this rule.

## Logging

| Rule | What it reports |
| --- | --- |
| `logging/no-production-log` | Debug logging outside the paths a repository approves |

## Architecture

| Rule | What it reports |
| --- | --- |
| `architecture/restricted-call` | An abrupt process exit, plus configured callees outside their approved paths |
| `architecture/restricted-import` | An import of a module a repository puts behind a boundary |
| `architecture/dependency-boundary` | A dependency that runs against the declared layer order |
| `architecture/filename-case` | A file name that does not follow the convention for its extension or scope |

`filename-case` expects `PascalCase` for `.tsx` and `.jsx`, `kebab-case` for other JavaScript and
TypeScript, and `snake_case` for Rust and Python.

## What a function means

A function means the same thing in every language: Rust `fn` items and closures, Python `def`
functions and lambdas, and JavaScript and TypeScript function declarations, function expressions,
methods, and arrow functions. A rule about functions therefore measures the same shape whichever
language it lands in, which is the whole point of one threshold across a repository.

## What the call and import rules cannot see yet

The call rules read the callee exactly as it is spelled, and the import rules read the module the same
way. `std::env::var` is matched; the aliased `env::var` after `use std::env` is not, because knowing
they name the same function needs resolution Godlint does not have yet —
[the rule roadmap](rule-roadmap.md) records what that defers.

They also have no scope analysis, so a local binding that shadows a restricted name is reported: a
Python parameter called `exec`, or a `const process = …` in TypeScript. Enable them deliberately; each
is off until a repository configures it.

## Which language a restriction belongs to

One consequence of built-in restrictions being language-bound is worth knowing before writing a
policy: **a name a built-in already claims belongs to that built-in's language.** Giving `sys.exit` an
`allow-in` boundary scopes Python's, and a call spelled `sys.exit` in TypeScript is left alone. There
is no language key to say which you meant, so the policy is silent rather than wrong.

**A name no built-in claims belongs to no language** and applies wherever it is called, which is what
a policy about `loadConfig` means. `print`, `console.log`, `console.debug` and `dbg!` are in that
second group rather than the first: `logging/no-production-log` owns them as dialect-bound defaults,
so naming one under `architecture/restricted-call` restricts it in *every* language. Restrict debug
logging through the logging rule, which keeps the binding.

## Severity

A finding below the configured `fail-on` severity is reported without failing the command, which is
how a rule can be adopted as a warning before it is adopted as a gate. See
[configuration](configuration.md).
\n## ci/anonymous-definition\nWorkflows must have a top-level name.
