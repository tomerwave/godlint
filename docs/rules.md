# Rule reference

Twenty-five rules are implemented. Every one has an identifier of the form `family/name`, which is
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
| `maintainability/condition-complexity` | `&&`, `\|\|`, and ternary operators combined in a single condition |
| `maintainability/cognitive-complexity` | How hard a function is to follow, weighting nested control flow |
| `maintainability/return-count` | Exit paths from a function, explicit or implicit |
| `maintainability/function-statements` | Statements in a function |
| `maintainability/empty-function` | Function bodies that appear unintentionally empty |

`decision-complexity` counts a `match` or `switch` once rather than once per arm, and a guard on an
arm counts. `function-statements` counts through nested blocks but not into nested functions.

`cognitive-complexity` measures how hard a function is to *read* rather than how many paths run
through it, which is where it parts company with `decision-complexity`. Every branch costs one, plus the
nesting depth it sits at, so four flat guard clauses cost 4 while four nested branches cost 10. Three
structures are deliberately cheaper than a path count suggests: a `switch` or `match` costs one however
many arms it has, an `else if` costs one because the reader already paid for the `if`, and a run of one
logical operator costs one however long it is — `a && b && c && d` costs one, while `a && b || c && d`
costs three, because switching operator is what makes a condition hard to hold.

A closure's own complexity belongs to the closure, not to the function containing it, matching every
other function metric here. That deviates from Sonar's specification, which folds a lambda's body into
its enclosing method; Godlint reports per function, so a closure gets its own finding instead.

`decision-complexity` deliberately does not count short-circuit boolean operators, so a five-part
condition on one `if` scores the same as a one-part condition. `condition-complexity` is the
counterpart that does: it counts `&&`, `||`, and a nested ternary, per `if` and `while` condition,
flatly — three operators cost three, whichever operators they are. A standalone ternary not
attached to an `if` or `while` is out of scope, and a nested function inside a condition (a
callback passed to `.some()`, for instance) is not descended into: that closure's own logic is not
this condition's operator count.

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
| `security/no-dynamic-execution` | JavaScript `eval`, `Function`, `new Function`; Python `eval`, `exec`. A global-object spelling of the same built-in counts: `globalThis`, `window`, `self`, or `global` in JavaScript and TypeScript, and `builtins` in Python |
| `security/direct-environment-read` | Environment access outside a configuration boundary |
| `security/no-weak-hash` | A broken hash algorithm, named either by the callee — Python `hashlib.md5`/`hashlib.sha1`, Rust `md5::compute`, `Md5::new`, `Sha1::new` — or by a literal argument to a factory: `crypto.createHash("md5")`, `crypto.createHmac("sha1", …)`, `hashlib.new("md5")`. Spelling and case do not matter (`MD5`, `sha-1`). An algorithm it cannot read reports at warning rather than at the configured severity. `allow-in` exempts a cache key or an ETag, where collision resistance is not the point |
| `security/no-insecure-random` | A general-purpose random generator, which is predictable: JavaScript `Math.random` and `crypto.pseudoRandomBytes`, Python's `random` module, Rust `rand::random` and `rand::thread_rng`. `allow-in` exempts a path where unpredictability is not the point |
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

## Testing

| Rule | What it reports |
| --- | --- |
| `testing/no-focused-test` | A test or suite marked to run on its own: `it.only`, `describe.only` and the other runners' `.only` |
| `testing/no-empty-test` | A test whose body does nothing, so it cannot fail |
| `testing/no-skipped-test` | A test that does not run: `.skip` or `.todo` in JavaScript and TypeScript, `#[ignore]` beside `#[test]` in Rust, and a `pytest.mark.skip` or `unittest.skip` decorator in Python |
| `testing/no-sleep-in-test` | A test that waits on the clock: `time.sleep` or `asyncio.sleep` in Python, `thread::sleep` or `tokio::time::sleep` in Rust, and `page.waitForTimeout` or `browser.pause` in JavaScript and TypeScript |

`no-empty-test` reads the test's own body rather than any function inside it, so a test that registers
an empty callback is not empty itself. A test with no body to read at all, such as `it.todo('later')`,
is not reported here — that it does not run is `no-skipped-test`'s finding.

It also reports an empty *suite* — `describe('x', () => {})` — because a suite is a test declaration by
the same syntax, and an empty one is dead weight for the same reason. The message says "test" in both
cases, which reads oddly for a suite; naming them apart needs the fact to distinguish a suite from a
test, which it does not yet.

Two adjacent rules fire on the same empty test on purpose, and it is worth knowing before adopting
`recommended@1`: `maintainability/empty-function` reports the same body, so an empty test yields two
findings, at the same position in Rust. They are different policies — one says a function has no body,
the other says a test cannot fail — and neither suppresses the other.

`no-sleep-in-test` reports a sleep only where the call falls inside a test's range — at any depth, so a
loop body or a closure counts — which means a helper in the same file may still sleep, and so may a
`pytest.fixture` or a `beforeEach`. A sleeping fixture is exactly as flaky as a sleeping test; it is
outside every test's range, so this rule cannot see it, and closing that needs a fixture fact.

It matches the callee's written spelling per language, which leaves three gaps and one false positive.

A sleep reached through an alias — `from time import sleep` and then a bare `sleep(2)` — is not reported,
because that takes import resolution.

JavaScript's most common test sleep is a *shape* rather than a name — `await new Promise((r) =>
setTimeout(r, 500))` — so it is matched as one: a `setTimeout` or `setInterval` inside a `Promise` whose
only call it is. That last condition is what separates a sleep from a timeout guard, because
`new Promise((resolve, reject) => { server.on('ready', resolve); setTimeout(() => reject(e), 5000) })`
is waiting on the condition, which is the fix this rule asks for. A bare timer under fake timers is not
reported either, for the same reason: the promise wrapper is what makes it a wait.

No other linter appears to catch this. Cypress's `no-unnecessary-waiting`, Playwright's
`no-wait-for-timeout` and `eslint-plugin-ui-testing`'s `no-hard-wait` all match a framework's own wait
API by name; the promise idiom needs the shape.

Also covered by name: `page.waitForTimeout` and `browser.pause`. `cy.wait(500)` is not, because Godlint
cannot yet tell a number argument from a string one and `cy.wait('@alias')` is the fix rather than the
defect.

The false positive: a mocked sleep. `with patch("time.sleep"): time.sleep(999)` is instant and is still
reported, because seeing the patch takes the same resolution the alias gap needs. Suppress it where it
matters.

What counts as a test is decided by syntax alone — a runner call, a `#[test]` attribute, a `test_`
prefix or a `pytest.mark` decorator. Neither rule knows about test directories, because an analyzer
sees no configuration; a repository that keeps tests somewhere unusual is not covered by that alone.

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
| `architecture/module-independence` | A dependency between modules declared independent of each other |
| `architecture/filename-case` | A file name that does not follow the convention for its extension or scope |

`module-independence` is the counterpart to `dependency-boundary`, for the constraint layering cannot
express. A layer order says a dependency is wrong in one direction; independence says it is wrong in
*both*, which is what keeps two feature modules from quietly growing into one. Each member declares the
same two halves a layer does — the `paths` it contains and the `modules` that name it — and a set is
violated when one member reaches another:

```yaml
architecture/module-independence:
  severity: error
  sets:
    - name: features
      members:
        - name: billing
          paths: [src/billing/**]
          modules: [crate::billing]
        - name: notifications
          paths: [src/notifications/**]
          modules: [crate::notifications]
```

Three things are deliberately not violations: a member importing its own internals, a file outside the
set importing a member, and a member importing anything the set does not name. The set constrains its
members' dependencies on each other, not everyone else's.

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

They read the callee and not the arguments, so a policy about a value passed *to* a call cannot be
expressed. `crypto.createHash("md5")` and `crypto.createHash("sha256")` are the same callee, and
Python's `hashlib.new("md5")` is the same as any other `hashlib.new`. That is why
`security/no-weak-hash` covers Python and Rust, where the algorithm is part of the name, and covers
JavaScript and TypeScript not at all: matching `crypto.createHash` would report every SHA-256 call as
a weak hash, and a security rule that is wrong on the safe case is worse than one that stays quiet.
A call-argument fact is what closes that.

A literal argument is readable, and `security/no-weak-hash` uses that: `crypto.createHash("md5")` is
reported and `crypto.createHash("sha256")` is not. A value that is not a literal is a different case,
and it reports at warning rather than at the rule's configured severity: something worth a look, not
something worth failing a build over.

The comparison is worth recording. SonarJS `S4790` reports the non-literal case as an ordinary finding,
and its documented failure mode is a false positive whenever the value cannot be inferred — which is
why it ships as a review-required hotspot rather than as an error. `eslint-plugin-security` has no
weak-hash rule at all, and its one randomness rule matches a callee name and nothing else. Godlint
splits the difference by severity instead of by rule: what it can read is an error, what it cannot is a
warning that says so, and the reader can tell which is which from the message.

How the two tiers divide real code was measured rather than assumed. Across 22,562 Python files in a
stdlib and site-packages tree there are 90 direct `hashlib.md5`/`hashlib.sha1` calls and 40
`hashlib.new` calls, of which 13 name their algorithm with a literal. So 103 sites land in the certain
tier and 27 in the uncertain one, and those 27 cluster in hashing library code that parameterises the
algorithm on purpose — which is exactly the code where an error would be wrong and a warning is the
honest answer.

Inside a Rust macro invocation they see nothing. A grammar keeps a macro's arguments as an unparsed
token tree, so `md5::compute(payload)` reports on its own and the same call inside
`format!("{:x}", md5::compute(payload))` does not. This holds for every call rule, and the failure is
silence rather than a diagnostic.

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

One finding can report below the severity its rule is configured at, when the rule is certain
something is wrong but not certain enough to block. `security/no-weak-hash` is the case that exists:
`crypto.createHash("md5")` names a broken algorithm and reports at the configured severity, while
`crypto.createHash(algorithm)` reports at warning, because the algorithm might be SHA-256 and Godlint
cannot tell. The message says which of the two it is, so a reader is never left guessing why one line
is an error and the next is not.

A rule can only lower a finding this way, never raise it: a repository that configured the rule at
`info` still gets `info`. That direction is deliberate — the configured severity is a ceiling the
repository sets, and no rule may argue with it.
