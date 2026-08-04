# Rule reference

Fifty rules are implemented. Every one has an identifier of the form `family/name`, which is
what a configuration entry and a suppression directive both name. [The rule roadmap](rule-roadmap.md)
records the families still to come, and the reasoning behind each threshold `recommended@1` sets.
[Language support](#language-support) records which languages each rule covers.

## Language support

A rule sees a *dialect* rather than a language: TypeScript is read by the JavaScript analyzer, so no
rule covers one of them without the other. Three dialects are analysed, and `✓` means the rule is
enforced there.

| Rule | JS/TS | Python | Rust | Workflow |
| --- | --- | --- | --- | --- |
| `architecture/dependency-boundary` | ✓ | ✓ | ✓ | — |
| `architecture/filename-case` | ✓ | ✓ | ✓ | — |
| `architecture/module-independence` | ✓ | ✓ | ✓ | — |
| `architecture/no-internal-import` | ✓ | ✓ | — | — |
| `architecture/restricted-call` | ✓ | ✓ | ✓ | — |
| `architecture/restricted-import` | ✓ | ✓ | ✓ | — |
| `ci/bot-conditions` | — | — | — | ✓ |
| `ci/explicit-workflow-permissions` | — | — | — | ✓ |
| `ci/hardcoded-container-credentials` | — | — | — | ✓ |
| `ci/no-comments` | — | — | — | ✓ |
| `ci/no-inline-script` | — | — | — | ✓ |
| `ci/no-monolithic-job` | — | — | — | ✓ |
| `ci/no-silenced-failure` | — | — | — | ✓ |
| `ci/overprovisioned-secrets` | — | — | — | ✓ |
| `ci/pin-third-party-actions` | — | — | — | ✓ |
| `ci/secrets-inherit` | — | — | — | ✓ |
| `ci/stale-action-refs` | — | — | — | ✓ |
| `ci/template-injection` | — | — | — | ✓ |
| `ci/unredacted-secrets` | — | — | — | ✓ |
| `logging/no-production-log` | ✓ | ✓ | ✓ | — |
| `maintainability/cognitive-complexity` | ✓ | ✓ | ✓ | — |
| `maintainability/condition-complexity` | ✓ | ✓ | ✓ | — |
| `maintainability/decision-complexity` | ✓ | ✓ | ✓ | — |
| `maintainability/empty-function` | ✓ | ✓ | ✓ | — |
| `maintainability/file-size` | ✓ | ✓ | ✓ | — |
| `maintainability/function-nesting` | ✓ | ✓ | ✓ | — |
| `maintainability/function-size` | ✓ | ✓ | ✓ | — |
| `maintainability/function-statements` | ✓ | ✓ | ✓ | — |
| `maintainability/parameter-count` | ✓ | ✓ | ✓ | — |
| `maintainability/return-count` | ✓ | ✓ | ✓ | — |
| `policy/accountable-suppression` | ✓ | ✓ | ✓ | — |
| `policy/todo-requires-reference` | ✓ | ✓ | ✓ | — |
| `policy/unused-suppression` | ✓ | ✓ | ✓ | — |
| `reliability/empty-error-handler` | ✓ | ✓ | — | — |
| `reliability/explicit-timer-delay` | ✓ | — | — | — |
| `security/direct-environment-read` | ✓ | ✓ | ✓ | — |
| `security/forbidden-dependency` | ✓ | ✓ | ✓ | — |
| `security/no-dynamic-execution` | ✓ | ✓ | — | — |
| `security/no-insecure-random` | ✓ | ✓ | ✓ | — |
| `security/no-shell-command` | ✓ | ✓ | ✓ | — |
| `security/no-weak-hash` | ✓ | ✓ | ✓ | — |
| `style/no-comments` | ✓ | ✓ | ✓ | — |
| `testing/assertion-required` | ✓ | ✓ | ✓ | — |
| `testing/no-empty-test` | ✓ | ✓ | ✓ | — |
| `testing/no-focused-test` | ✓ | — | — | — |
| `testing/no-network-in-unit-test` | ✓ | ✓ | ✓ | — |
| `testing/no-randomness-without-seed` | ✓ | ✓ | ✓ | — |
| `testing/no-skipped-test` | ✓ | ✓ | ✓ | — |
| `testing/no-sleep-in-test` | ✓ | ✓ | ✓ | — |
| `testing/no-test-helper-in-production` | ✓ | ✓ | ✓ | — |

The `Workflow` column is GitHub Actions YAML rather than a programming language, and it is in the same
table because it answers the same question: *does this rule read that?* No rule reads both. A workflow
has no functions and no imports, and source has no jobs and no `uses:` references, so every source rule
is absent from the `Workflow` column and every `ci/` rule is absent from the other three.
`a_rule_reads_workflows_or_source_and_never_both` in `crates/godlint-core/tests/languages.rs` holds the
two apart.

`—` means the dialect has no such construct: Rust has no `catch` block for `empty-error-handler` to
find empty, no `.only` marker for `no-focused-test`, and no way to import another crate's internals for
`no-internal-import` to report. There is nothing to report and nothing to implement, and a rule that
claimed the dialect would only ever be silent.

`·` would mean the construct exists and Godlint has not taught the dialect this rule yet, which is a
gap to close rather than a fact about the language. No rule is in that state today, which is why the
distinction is in the table rather than in a sentence: the two look identical to a reader deciding
whether to file a bug.

Neither mark is a claim a reader has to take on trust. Each rule declares `Rule::LANGUAGES`, this
table is asserted against those declarations by `crates/godlint-core/tests/languages.rs`, and
`scripts/validate-pull-request.py` requires a fixture that reports the rule in every dialect it
claims. That check fails in both directions, as the coverage budget does: a claim no fixture backs
fails, and so does a fixture reporting a rule in a dialect the rule says it does not cover.

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

For Python, `parameter-count` recognizes a receiver by the first parameter's spelling (`self` or
`cls`), not by whether the function is declared directly in a class body. This supports bound-task
idioms while accepting one narrow false negative: a module-level function that happens to name its
first ordinary parameter `self` or `cls`. Across 2,170 functions measured in `requests` and `flask`,
the only module-level function with that shape was a Flask Celery task declared with `bind=True`,
where the first parameter is the bound task instance and excluding it is correct.

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
| `policy/unused-suppression` | A suppression that silences nothing |

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
| `security/no-shell-command` | A command run through a shell, which makes any interpolated value executable: Python `shell=True` on a `subprocess` launcher, `os.system`, `os.popen`; JavaScript `child_process.exec`/`execSync`, including a destructured or required `exec` where the module is imported; Rust `Command::new` given a shell as its program. `allow-in` exempts a release script |
| `security/forbidden-dependency` | An import of a package the project has ruled out |

`no-shell-command` reads three different signals, because the languages put the defect in three
different places. In Python the callee is innocent and the *argument* is the finding, so the check is
callee-blind: **any** call passing a truthy `shell=` reports, not only a `subprocess` launcher. That is
what lets it see `sp.run(...)` after `import subprocess as sp`, and `run(...)` after `from subprocess
import run`, without listing either. The price is that a domain function of your own taking a `shell`
keyword is reported too. `shell=False` and `shell=0` are read rather than merely looked for. In
JavaScript the callee is the finding — `exec` shells out and `execFile` does not — but the common
spelling destructures it, so a bare `exec` counts only in a file that imports `child_process`, by
either `import` or `require`. A member call is read the same way with one difference: a receiver spelled
`child_process` or `childProcess` names the module itself and needs no corroboration, while a short alias
(`cp`) is only the module in a file that imports it. Both halves are load-bearing — without the import a
bare `exec` is a regular expression's, and accepting *any* receiver in a file that imports the module
reports every regular expression in it. The cost is that an unusual alias is missed.

Python's bare names are read the same way, which closes the `from`-import forms: `from os import system`
then `system(cmd)` is reported, and so are `popen`, `getoutput` and `getstatusoutput`, each gated on the
file importing `os`, `commands` or `subprocess`. That gate matters more in Python than in JavaScript
because `import os` is everywhere, so one more condition applies in both languages: a name the **file
declares itself** is never the module's. A file with its own `def system(x)` or `function exec(p)` is
silent, which is where a reported false positive came from. In Rust the program is the finding, so
`Command::new("sh")` is reported and `Command::new("git")` is not, by basename, so `/bin/sh` counts; a
program Godlint cannot read is left alone.

A literal command with nothing interpolated is reported too. It is not injectable today, but the
argument-array form is no harder to write, and a rule that reports only interpolated strings would
have to decide what interpolation looks like inside an f-string.

## Continuous integration

| Rule | What it reports |
| --- | --- |
| `ci/bot-conditions` | A step or job condition that compares an attacker-influenced actor field with a configured bot identity |
| `ci/pin-third-party-actions` | A workflow step using a third-party action at a ref that can move |
| `ci/stale-action-refs` | A commit-pinned action without an inline version label, or contradictory labels and pins within the repository |
| `ci/explicit-workflow-permissions` | A job that runs with whatever the repository grants by default |
| `ci/overprovisioned-secrets` | A step input or environment variable receiving the whole `secrets` context |
| `ci/hardcoded-container-credentials` | A literal username or password in a job container or service |
| `ci/no-comments` | A comment in a workflow |
| `ci/template-injection` | An attacker-influenced expression interpolated directly into a `run:` script |
| `ci/no-inline-script` | A `run:` script exceeding its effective-line limit |
| `ci/no-monolithic-job` | A job exceeding its step limit |
| `ci/secrets-inherit` | A reusable-workflow call passing every secret available to its job |
| `ci/unredacted-secrets` | A script directly writing a secret expression to `GITHUB_ENV` or `GITHUB_OUTPUT` |
| `ci/no-silenced-failure` | A step or job configured to stay green after failure, or a script that discards a failing exit status |


`ci/secrets-inherit` reports the `inherit` value in a job-level `secrets: inherit` declaration.
The called workflow receives every secret available to the caller whether it needs them or not;
name each secret explicitly instead. A named `secrets:` mapping and an absent `secrets:` declaration
are silent. `allow-in` accepts path globs for trusted callers where inheritance is deliberate.

`ci/overprovisioned-secrets` reports a step input or environment variable whose value is exactly
`${{ secrets }}` or `${{ toJSON(secrets) }}`. The finding names the setting that receives the whole
context. A reference to one member, including `${{ secrets.NPM_TOKEN }}` and
`${{ toJSON(secrets.NPM_TOKEN) }}`, is silent.

`ci/unredacted-secrets` reports a `run:` script only when that same script contains both a direct
`secrets.*` expression and a reference to `$GITHUB_ENV` or `$GITHUB_OUTPUT`. Merely using a secret is
silent, as is writing a non-secret value to either file. The rule cannot see a secret laundered
through a variable in an earlier step; that would require data flow the workflow facts do not have.

`ci/no-silenced-failure` reports a literal `continue-on-error: true` on a step or job and a `run:`
script ending `|| true`, `; exit 0`, or `|| exit 0`. A step is silent when it has an `id` and an
expression anywhere in the same job reads `steps.<id>.outcome` or `steps.<id>.conclusion`; braced
`${{ }}` values and the implicit expression in an unbraced `if:` both count. That is observable soft
failure rather than discarded failure. The search does not cross the job body, because a step
outcome is not available from another job. Expression-valued and false `continue-on-error` settings
remain literal facts and are silent.

The value is read as YAML reads it, so `true`, `True` and `TRUE` are all reported — those are the
three spellings YAML's core schema calls true, and GitHub honours each of them. Matching only the
lowercase form let a capital letter silence a step with nothing said about it. Anything outside that
set stays silent, including `yes`, `on` and a quoted `"true"`: they are not booleans in YAML's core
schema, and reporting a value on the guess that GitHub coerces it anyway would be a false positive
resting on an assumption this rule cannot check without a network.

Job outcomes have no same-workflow expression equivalent, so a literal job-level setting cannot
prove its own intent. It is reported at warning, and excluding that workflow is the escape hatch for
a deliberate always-soft job. Step-level `continue-on-error` is capped at warning too: 49 instances
appeared in the 94-workflow corpus, and after the outcome/conclusion exemption 42 remained, many on
deliberate diagnostic, cleanup, or artifact-upload steps. The suite still configures the rule at
error, so the cap preserves that measured uncertainty rather than weakening definite findings.

The script half is a substring judgement at the trimmed end of `run()`, at the same confidence boundary as
`ci/unredacted-secrets`' `$GITHUB_ENV` match; it is not a shell parser. An indirect status such as
`x=true` followed by `exit $x` is not seen, while `|| true` inside a quoted string is reported. All
nine measured `|| true` matches were deliberate cleanup or diagnostics, so that spelling is capped
at warning. The more explicit `; exit 0` and `|| exit 0` spellings stay at the configured severity;
neither appeared in the corpus.

`ci/no-comments` reports workflow comments except a comment trailing a `uses:` value on the same line.
That label makes a pinned SHA reviewable and lets `ci/stale-action-refs` compare the claim with every
other workflow in the repository, reconciling that rule with `ci/pin-third-party-actions`. The exemption is
unconditional because repositories that pin by SHA need the label, while an unpinned reference has no
pin label to preserve. A comment above `uses:` or trailing any other key remains commentary and is
reported. YAML has no doc-comment construct, so `allow-doc-comments` has no workflow equivalent and
this rule has no option beyond `severity`. Workflow suppressions are not available: excluding a path
is the only way to remove a workflow from the rule's scope.

`ci/stale-action-refs` reports three signals available from repository contents alone. A forty-character
commit pin with no comment trailing its `uses:` value reports because a reviewer cannot read the SHA.
That signal is capped at warning: in the 94-workflow corpus, 396 of 765 commit pins lack an inline label,
including all 385 pins in Deno, which keeps version information in a trailing block instead. The rule also
reports every occurrence when one action and SHA carry different labels, or one action and label name
different SHAs. Those contradictions stay at the configured severity because the repository itself proves
that the claims cannot all agree. Action names, hexadecimal SHA spelling, and labels are compared without
ASCII case distinctions. One leading `v` in a label is ignored, so `v4.6.2` and `4.6.2` agree; no other
part of a label is normalized. `allow-in` path globs remove a workflow from both findings and
cross-workflow comparison.

The rule does not determine whether a label such as `# v4` genuinely names the pinned commit. That requires
looking up the action repository, and Godlint does not use network access. It has no online mode or feature
flag. Use [zizmor's `stale-action-refs` and `ref-version-mismatch` audits](https://docs.zizmor.sh/audits/)
for that external verification.

Godlint reads workflows, not composite actions: `Workflow::names` recognizes YAML files directly in
`.github/workflows/`, so nothing in the `ci/` family sees a root `action.yml`.

`ci/hardcoded-container-credentials` reports each literal `username` and `password` under a job's
`container.credentials` or any `services.*.credentials` block. A value containing a GitHub expression
is treated as interpolated and stays silent. The workflow reader deliberately exposes only those two
credential keys; arbitrary container settings are not credentials, and this rule does not guess at
secret-looking values elsewhere.

`ci/template-injection` reports expressions in a step's `run:` script when the expression reads a
context GitHub documents as attacker-influenced. Event-driven values such as issue and pull request
text, comments, reviews, discussions, commit data, page names, head refs and workflow-run data report
at the configured severity. The runner expands the expression before the shell runs, so shell quoting
around the expression does not turn it back into data. Bind the expression to an `env:` variable and
reference that variable quoted in the script instead.

`inputs.*` and `github.event.inputs.*` report at no higher than warning. A manually dispatched
workflow can only receive those values from someone with write access, who can already edit and run
the workflow; a reusable workflow can, however, receive a value its calling workflow does not
control. In a measurement of 94 workflows from the nine pinned corpus repositories, the rule reported
16 findings: every one was a dispatch input and none was an event-driven context. Capping this tier
keeps that uncertainty visible without making the measured false-positive class block adoption.

Expressions in `env:` and `with:` values are deliberately silent. In particular, passing an
attacker-influenced value through `env:` is GitHub's documented remedy; reporting it would tell the
author to undo the fix. `allow-in` accepts path globs for workflows whose scripts are reviewed under a
different policy.

`ci/bot-conditions` reports braced and unbraced step-level and job-level `if:` expressions that
compare `github.actor` or `github.triggering_actor` with an identity in `bots`, or pass one of those
actor fields to `contains` with a configured identity or a non-empty substring of one. Those actor
fields are attacker-influenced on several triggers, so matching a bot-looking name does not prove
that the bot opened the pull request.
The list defaults to `dependabot[bot]`, `github-actions[bot]`, and `renovate[bot]`; repositories can
add their own identities. Compare `github.event.pull_request.user.login` instead, or verify the app
that produced the change rather than trusting the actor name.

A `uses:` reference names either a commit or something mutable. A tag, a branch and a version string can
all be repointed by whoever owns the action, and whatever they point at next runs inside your workflow
with your token — so this is the one supply-chain hole in CI with a one-line fix. Only a full
forty-character commit SHA counts as pinned; a short SHA does not, because it is neither what GitHub
resolves nor collision-resistant.

Three references are nobody else's code and are silent: a local `./path` action, a `docker://` image,
and anything whose owner is listed in `trusted-owners`. That list defaults to `actions` and `github`,
the two accounts GitHub itself publishes from, because a repository that pins those as well is making a
policy decision rather than closing a hole — set `trusted-owners: []` to require every action to be
pinned, including GitHub's own.

A reference with no version at all reports a different message, because it is a different mistake: it
runs whatever the action's default branch holds today.

`no-inline-script` reports the script range of a `run:` step with more than `max-lines` effective
source lines. Blank lines and lines whose first non-whitespace character is `#` are skipped by
default, using the same configurable effective-line treatment as the maintainability size rules.
The rule measures the YAML source rather than interpreting a shell, so a single line that chains
commands or pipelines stays silent; detecting that requires a separate command-chain or control-flow
signal rather than pretending a line count can see shell structure.

`no-monolithic-job` reports a job with more than `max-steps` steps. It counts declared steps, and a
matrix job declares the steps of every platform it serves, even when platform conditions mean no
target runs all of them. `allow-in` accepts path globs for that measurement artefact. The rule counts
workflow units that can be reviewed and retried independently, not the commands hidden inside them.
The two rules do not compensate for each other: splitting six commands across six `run:` lines in one
step belongs to `no-inline-script`, while spelling those commands as six separate steps belongs to
`no-monolithic-job`. Keeping the step count low never increases the inline-script budget.

### What `explicit-workflow-permissions` reports, and where

A workflow with no `permissions` block inherits the repository default, which is usually broader than
any job needs, and the failure is invisible until a token is abused. What the rule reports depends on
what is missing, because the fix does:

| The workflow | Reported |
| --- | --- |
| declares no `permissions`, and no job does either | once, at the file — one line at the top fixes it |
| declares none, and *some* jobs declare their own | once per job that does not, at that job's line |
| declares `permissions` | nothing; every job is covered by it |
| declares `permissions`, under `require-per-job: true` | once per job that does not narrow it further |

Reporting per job in the mixed case rather than once at the top is deliberate: a workflow whose other
jobs are already narrowed does not need a blanket block, and the finding should name the job that is
still open. Reporting once when nothing is declared anywhere is the same principle from the other side —
six findings for one missing line would be noise.

`require-per-job` is off in `recommended@1`. A job inheriting a workflow-level block is a deliberate
choice a repository may have made, and a rule that argued with it would be turned off rather than
tightened.

Godlint reads `.github/workflows/*.yml` with a YAML grammar rather than by matching text, so a `uses:`
inside a comment, inside a string, or in a step *named* `uses:` is not a use. A workflow removed by an
`exclude` glob is not read at all, which is the only way to silence this rule today: an inline
suppression cannot reach it, because comment facts come from source files and not from YAML.

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
| `testing/no-randomness-without-seed` | A test drawing from a general-purpose generator in a file that never seeds one, so a failure cannot be reproduced |
| `testing/no-network-in-unit-test` | A test in a declared unit path calling an HTTP or socket client, so it is slow, dependent on a service being up, and unable to run offline |
| `testing/assertion-required` | A test that asserts nothing, so it passes unless the code raises. Reported at warning, whatever severity is configured |
| `testing/no-test-helper-in-production` | A production file importing its own test tree: a local import naming `tests`, `test`, `__tests__`, `__mocks__`, `fixtures`, `mocks` or `conftest`. `test-paths` says which files count as tests, `helpers` which segments name scaffolding |

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

`no-randomness-without-seed` shares its generator catalogue with `security/no-insecure-random`: the
same call is unpredictable to an attacker and unreproducible to whoever has to read the failure. The
two rules differ in what excuses it. Seeding is a property of the file rather than of the call, because
`random.seed(1)` and `random.sample(...)` are separate calls and the second is what a rule sees, so a
file containing any seeding call is exempt in full. That under-reports — a seeded test beside an
unseeded one silences both — which is the safe direction for a rule enabled at error. A seed spelled
for another language does not count.

Property-based suites are the false positive to configure around. Their own generators are not in the
catalogue, so most are already silent; `allow-in` covers a suite that draws from the standard library
on purpose and reports its own seed.

The catalogue matches the written spelling, so `from random import sample` then `sample(pool, 3)` is not
reported — the same alias limit the call rules document.

Rust gets its own remedy, because `rand::random` and `rand::thread_rng` cannot be seeded at all: there the
message asks you to replace the generator with a seeded `StdRng` rather than to seed the one you have. A
file that does exactly that is exempt, so `StdRng::seed_from_u64`, `SmallRng::seed_from_u64` and
`SeedableRng::from_seed` all count as seeding. `rand::rng` is covered too, which is what `thread_rng` was
renamed to in rand 0.9.

numpy is covered on both sides. It used to know `np.random.seed` without knowing `np.random.rand`, which
meant a numpy seed could exempt a file for a generator the rule never reported on.

`no-network-in-unit-test` is silent until configured, which puts it in an established category rather
than a new one: `architecture/restricted-call`, `restricted-import`, `dependency-boundary`,
`module-independence`, `security/forbidden-dependency` and `filename-case` all ship in `recommended@1`
at error with an empty list and say nothing until a repository fills it in. What is new here is only
*which* fact is missing — whether a given test is a unit test is a property of the repository rather
than of the file, so the rule reports nothing until `unit-paths` names the directories that hold them:

```yaml
rules:
  testing/no-network-in-unit-test:
    severity: error
    unit-paths:
      - tests/unit/**
```

`allow-in` carves an exemption out of the declared paths, which is what a mocked client needs: a test
that assigns `global.fetch` and then calls it is following this rule's own advice, and a callee match
cannot tell that from a real request.

`recommended@1` enables it at error like every other rule, and with no `unit-paths` it stays silent
rather than guessing. Guessing was the alternative and it is worse in both directions: a repository
following Rust's convention keeps its integration tests in `tests/`, where reaching the real service is
the entire point, and a repository with no such split would have every test reported at error.

This repository does not name the rule in its own `godlint.yaml`, and cannot yet: the released-agreement
check runs the *published* binary against this tree, and the configuration schema rejects an unknown
rule key outright, so a `godlint.yaml` naming a rule that does not exist in the last release fails to
parse. The rule is dogfooded through the adopted suite, and the fixture under
`crates/godlint-cli/tests/fixtures/rules/no-network-in-unit-test/` is the worked example instead. That
interaction is filed separately; it will hold for every future rule that needs configuration to say
anything.

`no-network-in-unit-test` matches the client's written spelling, so it inherits the same two blind spots
the call rules document above, and one more of its own. An alias escapes it: `from requests import get`
then `get(url)` is not reported. A seam escapes it: `requests.Session().get(...)`,
`httpx.Client().get(...)` and `reqwest::Client::new().get(...)` are indirect calls, so no callee fact
names them. And a client reached from a fixture rather than from the test — `beforeEach(async () => {
await fetch(url) })`, or a `pytest.fixture` — falls outside every test's range and is silent, which is a
common shape of exactly this smell. All three want import resolution or a fixture fact; none is a
configuration matter.

`assertion-required` reports at warning whatever severity it is configured at. Whether a test asserts
through a helper is not decidable without resolution, and with `fail-on` at its default of `error` that
means the rule informs rather than fails a build.

The cap reuses `Violation::cap()`, but it is worth being precise about how this use differs from
`security/no-weak-hash`'s, because the two are not the same shape. `no-weak-hash` emits two violations
and caps only one: an algorithm it cannot read is capped, and an algorithm named outright keeps its
configured severity, so the rule stays sharp on what it can prove. `assertion-required` has one
violation and caps it, so the cap is rule-wide. That is the blunter instrument, and it is the honest one
here — there is no subcase where the rule *can* prove a test asserts nothing, because the assertion may
always be inside a helper it cannot follow.

A hard gate is still reachable, and the documentation would be misleading not to say so: `fail-on:
warning` makes any warning fail the run. The cost is that it is not rule-scoped — it promotes every
warning in the repository, including `no-weak-hash`'s unreadable algorithm. There is no per-rule route
to a gate.

Three things that look assertion-free are not, and are silent:

| Shape | Why |
| --- | --- |
| `pytest.raises`, `#[should_panic]` | Asserting that something raises is asserting |
| A `describe` or other suite | It asserts through the tests inside it, so reporting it would double every finding |
| An empty test | That is `no-empty-test`'s finding, and two findings for one defect is noise |

What remains is the helper case, and `extra-assertions` is the answer to it — a repository that asserts
through `verify_refund` names it rather than turning the rule off. Names match the spelling as written,
so `helpers.verifyOrder` and a bare `verifyOrder` are different entries:

```yaml
rules:
  testing/assertion-required:
    severity: error
    extra-assertions:
      - verify_refund
```

`no-test-helper-in-production` reads the import path and the importing file's path, and one restriction
does most of the work: only a **local** import counts — `./`, `../` or a bare `.` prefix, and Rust's
`crate::` or `super::`. That is what keeps `some-lib/tests/util` and `from testing.helpers import fake`
silent, because a third-party package's own test tree is its own business and cannot be shipped by you.
Segments match whole and case-insensitively, so `Tests/` counts and `testing-utils/` does not.

A file that is itself a test is exempt, since a test using its own helpers is the arrangement this rule
is protecting. `test-paths` decides that, and its defaults are the conventions of all four languages —
`**/tests/**`, `**/__tests__/**`, `**/*.test.*`, `**/*.spec.*`, `**/test_*.py`, `**/conftest.py` and the
rest. Setting either list *replaces* the default rather than adding to it.

What counts as a test is decided by syntax alone — a runner call, a `#[test]` attribute, a `test_`
prefix or a `pytest.mark` decorator. None of the test rules knows about test directories on its own,
because an analyzer sees no configuration; a repository that keeps tests somewhere unusual is covered
only where a rule takes paths from configuration, as `no-network-in-unit-test` does.

## Logging

| Rule | What it reports |
| --- | --- |
| `logging/no-production-log` | Debug logging outside the paths a repository approves |

## Architecture

| Rule | What it reports |
| --- | --- |
| `architecture/restricted-call` | An abrupt process exit, plus configured callees outside their approved paths |
| `architecture/no-internal-import` | An import that reaches past a package's public surface: a path segment named `internal`, `private`, `impl` or `_internal`, a Python segment beginning with `_`, or a build-output segment `dist`, `src` or `build`, which reports at warning. `allow` exempts a module the project must reach into |
| `architecture/restricted-import` | An import of a module a repository puts behind a boundary |
| `architecture/dependency-boundary` | A dependency that runs against the declared layer order |
| `architecture/module-independence` | A dependency between modules declared independent of each other |
| `architecture/filename-case` | A file name that does not follow the convention for its extension or scope |

`no-internal-import` reads the import path and nothing else, which is honest but has consequences. A
marker only counts *after* the first segment, so `src/utils` — a path alias to this project's own code —
is silent while `some-lib/src/deep` is not, and `from __future__ import annotations` is silent while
`package._private.helpers` is not. A relative import is always silent: your own internals are yours to
reach into. An alias in a bundler or `tsconfig` escapes the rule entirely, the same limitation
`architecture/restricted-import` documents.

Rust is silent entirely, and that is a fact about the language rather than a gap. A `use` path either
names your own crate — `crate::`, `self::`, `super::` — or names another crate's *public* surface,
because `rustc` refuses the rest. There is no reaching past a boundary for the rule to report, which is
why [the support matrix](#language-support) marks Rust as having no such construct.

Two segment shapes are exempt for reasons that are not conventions at all. A scoped package's name spans
*two* segments, so `@scope/internal` may be the whole published package and is silent, while
`@scope/pkg/src/deep` is not. And a Python `__dunder__` is a language protocol rather than an author's
decision: `import package.__main__` is how you run a module, so it is silent, while
`package._private.helpers` is not.

Two tiers, because two of these conventions are not the same claim. `internal`, `private`, `impl` and a
Python `_` prefix say *the author did not mean this for you*, and report at error. `dist`, `src` and
`build` merely name build output, which some packages publish as their documented entry, so they report
at warning. A path naming both is certain, and the message names the marker that decided it.

One false positive survives and is worth knowing before adopting the rule at error. A project's own
tests importing its own package **absolutely** are reported, because only relative imports are exempt:
requests' `tests/test_utils.py` does `from requests._internal_utils import ...`, and that reads exactly
like reaching into a third party. Measured against requests, flask, express and zod, it was the only
finding of any kind. `allow` is the remedy today — `allow: ["requests/**"]` — and the real fix is for the
rule to recognise a first segment that names a package in the scanned tree, which needs repository
layout it does not currently see.

Rust is out of scope. Module privacy there is enforced by the compiler, so a module you are able to
import is one its author made public, and there is no reaching past anything to report.

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
`security/no-weak-hash` names a callee only in Python and Rust, where the algorithm is part of the
name, and never in JavaScript or TypeScript: matching `crypto.createHash` would report every SHA-256
call as a weak hash, and a security rule that is wrong on the safe case is worse than one that stays
quiet. Reading the literal argument is what closes that, and it is what gives the rule its JS/TS
coverage.

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
something is wrong but not certain enough to block. `security/no-weak-hash` uses this split:
`crypto.createHash("md5")` names a broken algorithm and reports at the configured severity, while
`crypto.createHash(algorithm)` reports at warning, because the algorithm might be SHA-256 and Godlint
cannot tell. `ci/template-injection` likewise keeps event-driven contexts at the configured severity
and caps workflow inputs at warning. Each tier has its own message, so a reader is never left guessing
why one line is an error and the next is not.

A rule can only lower a finding this way, never raise it: a repository that configured the rule at
`info` still gets `info`. That direction is deliberate — the configured severity is a ceiling the
repository sets, and no rule may argue with it.
