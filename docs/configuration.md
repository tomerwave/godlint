# Configuration

Godlint reads `godlint.yaml` from the repository root, and enforces nothing until that file asks it
to. The smallest configuration that does anything adopts a suite:

```yaml
version: 1
suites: [recommended@1]
```

Validate a file before trusting it:

```bash
godlint config validate
godlint config validate --config path/to/godlint.yaml
```

Unknown keys are rejected rather than ignored, so a misspelled threshold fails validation instead of
silently doing nothing. One key is treated differently, and [what a version may
read](#what-a-version-may-read) says which and why.

## What a version may read

One configuration usually outlives one version of Godlint, and often has to be read by two at once: a
pinned version in CI and a newer one on a developer's machine, or the reverse. So the strictness has a
seam in exactly one place.

**An unrecognised key under `rules:` is reported and ignored.** A rule name this version does not know
can only mean a rule it cannot run, so refusing the file would stop the run over a rule that would have
reported nothing anyway. `check` and `config validate` both name it on standard error:

```text
godlint.yaml: rules: testing/no-network-in-unit-test is not a rule godlint 0.3.0 knows, so it is ignored.
```

The cost is that a *misspelling* also becomes a rule that quietly does nothing, which is why the notice
offers the nearest name it knows when there is one:

```text
godlint.yaml: rules: maintainability/function-siz is not a rule godlint 0.3.0 knows, so it is ignored. Did you mean maintainability/function-size?
```

**Everything else is still refused.** The line is what an unrecognised name can cost:

| Unrecognised | Outcome | Why |
| --- | --- | --- |
| A key under `rules:` | Reported and ignored | It can only subtract the one rule it names |
| An option inside a rule | Refused | The rule *does* exist, and its options decide how it behaves, so ignoring one would enforce a policy the file does not describe |
| A top-level key | Refused | `exclude` and `fail-on` decide what is scanned and what fails; ignoring one could silently pass a run that should not have |
| A suite name | Refused | A configuration that adopts only `recommended@2` would enforce **nothing** if the name were ignored, and the run would go green |
| A `version:` value | Refused | It states which schema the file is written against, so an unknown value means nothing else in the file can be trusted |

So a newer configuration degrades on an older Godlint rule by rule, and stops outright when the
difference is one that could make the run mean something other than it says.

## Top-level keys

| Key | Default | Meaning |
| --- | --- | --- |
| `version` | required | Schema version. `1` is the only accepted value. |
| `suites` | none | Suites to adopt, by name and version. |
| `rules` | none | Per-rule settings, which override any suite. |
| `fail-on` | `error` | The lowest severity that makes `check` exit non-zero. |
| `exclude` | see below | Glob patterns to skip, added to the built-in list. |

### `fail-on`

Severities are `off`, `info`, `warning` and `error`. A finding below `fail-on` is still reported, just
without failing the command — which is how a rule is adopted as a warning before it is adopted as a
gate:

```yaml
fail-on: warning
```

### `exclude`

Godlint already skips the directories nobody wants linted: `.git`, `.mypy_cache`, `.next`, `.tox`,
`.venv`, `__pycache__`, `build`, `coverage`, `dist`, `node_modules`, `target` and `vendor`. Anything
listed in `exclude` is skipped as well:

```yaml
exclude:
  - generated/**
  - scripts/**
```

`exclude` is the file-wide escape hatch, and deliberately the only one — there is no file-wide
suppression comment. When a single site rather than a whole file needs an exemption, use
[inline suppression](suppressions.md), which records who owns it and when it lapses.

## Suites

A suite names a set of rules and their thresholds, so a repository adopts a standard
without naming every rule individually:

```yaml
version: 1
suites:
  - recommended@1
```

`recommended@1` enables every rule at `error`. Its thresholds are measured rather than borrowed — see
[the rule roadmap](rule-roadmap.md) for each number and the reasoning behind it.

Suites are opt-in: a configuration naming none enforces nothing. The name carries its version, so a
suite's contents can change without changing what an existing repository is held to.

## Overriding a suite

A `rules:` entry wins over the suite for that rule, in either direction. A repository can loosen one
threshold, tighten it, or decline a rule entirely without abandoning the rest:

```yaml
version: 1
suites: [recommended@1]

rules:
  maintainability/function-size:
    severity: error
    max-lines: 80          # looser than the suite
  maintainability/file-size:
    severity: warning      # reported, does not fail
  style/no-comments:
    severity: off          # declined (every rule but policy/unused-suppression)
```

Rules take three settings: `severity`, and the two that say which files it applies to.

| Setting | Default | Meaning |
| --- | --- | --- |
| `severity` | required | `off`, `info`, `warning` or `error`. |
| `only-in` | every file | Glob patterns the rule is *about*. Empty means every file. |
| `allow-in` | nothing | Glob patterns exempted, including inside `only-in`. |

**One rule takes only `severity`, and not `off`.** `policy/unused-suppression` reports the
suppression comments that silence nothing, so a configuration able to retire it could retire every
exemption it audits. `severity: off`, `only-in` and `allow-in` are rejected on it as invalid
configuration; `warning` is accepted and is how to keep it reporting without failing the build. A
top-level `exclude` still applies to it, as it does to every rule. See
[inline suppression](suppressions.md).

`only-in` exists because many rules are inherently scoped. "No logging in production code" has
nothing to say about a build script, and "no sleeps in tests" has nothing to say outside them.
Without it the only way to say so is `exclude`, which drops the path for **every** rule at once — so
one rule that does not belong somewhere costs you every other rule there too.

The narrower of the two decides, so `allow-in` carves exceptions out of `only-in`:

```yaml
rules:
  logging/no-production-log:
    severity: error
    only-in: [src/**]
    allow-in: [src/generated/**]
```

That reports `src/server.js`, says nothing about `src/generated/client.js`, and says nothing about
`scripts/build.js` — while every other rule still applies to all three.

**A pattern that matches nothing silences the rule.** `only-in` narrows, so an `only-in` naming a
path that does not exist leaves the rule with nowhere to apply and it reports nothing, anywhere,
without saying so. This is the opposite of `exclude` and `allow-in`, where a pattern matching nothing
changes nothing — so a typo in those is harmless and a typo here is not. A misspelled *key* is caught
(`onlyin` fails validation naming the fields it expected); a misspelled *path* is not.

A suppression comment for a rule that `only-in` or `allow-in` has removed from that path is reported
by `policy/unused-suppression`, because from where it stands the rule silences nothing there. Scoping
a rule into `src/**` therefore reports the suppressions left behind in the rest of the tree.

Two rules read paths for a second purpose beyond scope. `testing/no-network-in-unit-test` takes
`unit-paths`, which is what the rule is *for* rather than an exemption, and
`testing/no-test-helper-in-production` takes `test-paths` to decide which tree a helper lives in.
Both still take `only-in` and `allow-in` like every other rule.

Beyond those three, each rule takes the settings its measurement needs, named after what they mean:

```yaml
rules:
  maintainability/function-size:
    severity: error
    max-lines: 50
    skip-blank-lines: true
    skip-comments: true

  logging/no-production-log:
    severity: error
    allow-in:
      - scripts/**

  architecture/dependency-boundary:
    severity: error
    layers:
      - name: interface
        paths: [src/interface/**]
        modules: [crate::interface]
      - name: application
        paths: [src/application/**]
        modules: [crate::application]
      - name: domain
        paths: [src/domain/**]
        modules: [crate::domain]
```

A layer declares both halves of its identity, and must: `paths` are the files that belong to it, and
`modules` are the names an import uses to reach it. One without the other fails validation, because
a layer that cannot be named cannot be depended upon.

**Order runs outermost first.** A layer may depend on layers declared after it, never on one declared
before it — so with the layers above, `interface` may reach `domain` and `domain` reaching `interface`
is the violation. The most specific matching path decides which layer a file belongs to, so a nested
layer inside a broader one is read as the nested one.

Rules that name callees or modules — `architecture/restricted-call`,
`architecture/restricted-import`, `security/forbidden-dependency` — are off until configured, and
match the name exactly as it is spelled in the source. Read
[what the call and import rules cannot see yet](rules.md#what-the-call-and-import-rules-cannot-see-yet)
before relying on one; an alias escapes them, and a shadowing local binding is reported.

`testing/no-network-in-unit-test` is off until configured for a different reason: it takes paths rather
than names. `unit-paths` declares which directories hold unit tests, and the rule reports nothing until
it is set, because whether a test is a unit test is a property of the repository. `allow-in` then carves
exemptions back out of those paths.

`ci/stale-action-refs` scopes what it *reads*, not only what it reports, and it is the one rule where
that distinction is visible. It reports one commit labelled two ways across different workflow files,
so a file outside its scope must not supply half of a contradiction — otherwise excluding a workflow
would still produce a finding elsewhere, caused by the file you excluded.
