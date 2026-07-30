# Using Godlint in CI

CI is the canonical, deterministic enforcement point shared by human and agent-authored
changes. Godlint is a single binary that exits non-zero when a finding is at or above
`fail-on`, so enforcement is one line in any CI system:

```yaml
- run: godlint check
```

Nothing below is required to enforce policy. It is what makes the failure readable.

## The GitHub Action

```yaml
permissions:
  contents: read

steps:
  - uses: actions/checkout@v4
  - uses: tomerwave/godlint@v1
    with:
      version: 0.1.9
```

The action downloads the release archive for the runner, verifies it against the `.sha256` published
beside it, and runs `godlint check --format github`. Every finding lands as an annotation on the exact
line it belongs to, visible in Files changed, and disappears when the finding is fixed — nothing has to
be resolved by hand.

It needs **no token and no permissions.** Annotations are written to the workflow's own output stream
rather than posted through the API, which is why the action also works on a pull request from a fork,
where the token is read-only and anything posted through the API fails.

### Inputs

| Input | Default | Meaning |
| --- | --- | --- |
| `version` | `latest` | The Godlint version to run, without a leading `v`. |
| `paths` | the working directory | Paths to check. |
| `working-directory` | `.` | Where to run, and so where `godlint.yaml` is read from. |
| `summary` | `true` | Write a per-rule count to the job summary. |

**Pin `version`.** The default resolves to the latest release, which means a release can change what
an open pull request is held to with no commit saying so. A pinned version makes that a reviewable
change.

`summary` exists because GitHub renders only so many annotations per run. When a first adoption
produces hundreds of findings, the annotations show a sample and the job summary carries the count for
every rule, so the shape of the work is still visible.

### Outputs

| Output | Meaning |
| --- | --- |
| `version` | The version that ran, resolved if the input was `latest`. |
| `findings` | How many findings were reported. |

A composite action withholds its outputs when it fails, so `findings` is readable only from a run that
passed. To act on a failing count, set `fail-on: off` in the configuration and decide in a later step.

### Versioning

`v1` is the action's interface — its inputs — and each release moves the tag, so `@v1` picks up fixes
without a workflow edit. The binary it installs is versioned separately, which is why the pre-`1.0`
command line and a `v1` action are not a contradiction. Pin the exact release instead when a workflow
must be reproducible:

```yaml
- uses: tomerwave/godlint@v0.1.9
```

## Output formats

`check --format` decides who is reading:

| Format | For |
| --- | --- |
| `terminal` | A person. The default. |
| `github` | Workflow annotations, so findings land on the line of a pull request diff. |
| `json` | Another tool. |
| `sarif` | Code-scanning dashboards and anything else that speaks SARIF. |

`json` and `sarif` are emitted even when there is nothing to report, because a consumer parses a
document rather than prose, and an empty run is a result rather than an absence of one.

## Two commands worth wiring in

```bash
godlint config validate
godlint suppressions
```

`config validate` rejects a configuration before it is trusted, which is worth its own step so a
broken configuration fails as a configuration error rather than as a confusing absence of findings.
`suppressions` lists every exemption with its owner and expiry — see
[inline suppression](suppressions.md).

## How Godlint tests its own action

The repository runs the released action against itself on Linux, macOS and Windows, and separately
runs the in-tree action against a small tree that is meant to have findings, to prove that a failure
really fails and really annotates. The first of those can go legitimately red, and
[CONTRIBUTING.md](../CONTRIBUTING.md) explains the two labels that say why.
