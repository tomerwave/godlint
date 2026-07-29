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

Unknown keys are rejected rather than ignored, so a misspelled rule name or threshold fails
validation instead of silently doing nothing.

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

A suite names a set of rules and their thresholds, so a repository adopts a standard in one line
rather than twenty-one:

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
