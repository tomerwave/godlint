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
    severity: off          # declined
```

Every rule takes `severity`. Beyond that each takes the settings its measurement needs, named after
what they mean:

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
