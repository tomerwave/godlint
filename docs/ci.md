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
| `status` | The status `godlint check` exited with. |

A later step can read all three even when the action fails, which is the run that has something to
say — with one limit worth knowing: an output survives only if the step that set it ran. The action
stops at its first failing step, so a failed install leaves all three empty rather than zero. To act
on a count without failing the job at all, set `fail-on: off` in the configuration and decide
afterwards.

`status` is what a step reads to know **why** a run failed rather than only that it did:

| Status | Meaning |
| --- | --- |
| `0` | Nothing at or above `fail-on`. |
| `1` | Findings at or above `fail-on`. |
| `2` | Godlint could not check everything it was asked to — an unreadable configuration, a file it could not parse, a path it would not accept. Findings it did reach are still reported. |

The distinction matters because a workflow that treats every non-zero status as findings reports a
clean tree it could not read as a tree with problems.

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

## Repository workflows

The workflow files contain only executable configuration. The reasoning behind that configuration
lives here so it can be read and maintained as prose.

### Action (`action.yml`)

The pull request trigger includes `labeled` and `unlabeled` as well as the defaults, so applying a
drift label re-evaluates the check. Without them, the label would appear to do nothing until the next
push.

The repository job compares the released binary against the code in the pull request. It fails when
they disagree and says which of the two reasons it is, because red that needs a conversation to
interpret is worse than no check. It is not a required check, so it never blocks a merge. The action
under test is this checkout, so a change to it is exercised by its own pull request rather than only
after release. It runs against the repository itself, not a fixture: the action is held to the same
tree as the rest of CI, so a released binary that disagrees with this code fails here rather than
surprising whoever installs it.

The dirty-tree action step uses `continue-on-error` because the point is that it fails: the step must
report a failure and the annotations must still be emitted. The assertion reads the annotations file
rather than an output or the summary. A composite action withholds its outputs when it fails, and
`GITHUB_STEP_SUMMARY` is a separate file per step, so neither is readable from the assertion step.
Failing is the thing being asserted, so the assertion cannot depend on the action having succeeded.

### Coverage (`coverage.yml`)

The commit-pinned `Swatinem/rust-cache` action is version 2.

### Godlint (`godlint.yml`)

Godlint is built from this checkout rather than installed from a release, so the check is of the code
in the pull request. `--format github` puts each finding on its own line in Files changed, which is
the same output the published action produces.

### Labeler (`labeler.yml`)

The workflow uses `pull_request_target` because labelling a pull request needs write access and a pull
request from a fork has a read-only token. It is safe only while nothing here checks out or runs the
pull request's code, and nothing does: the labeller reads the changed paths from the API. It never
removes a label. A path rule has no business undoing a decision a person made, and the drift labels
are exactly the kind of decision that must survive a later commit.

### Mutation testing (`mutants.yml`)

The pull request paths are every path `.cargo/mutants.toml` examines, and nothing else. A path examined
by the configuration and missing here is the failure #88 records: the analysers were mutated by the
weekly sweep and by no pull request, so the layer that decides what is *seen* had no gate on the day
it changed. `check_mutation_scope` in `scripts/validate-pull-request.py` keeps the two lists from
drifting apart again.

The pull request run mutates only the lines the pull request touched, so the check stays proportionate
to it. The diff is not narrowed by path: `examine_globs` already decides which files carry mutants,
and narrowing here as well is what let the two lists disagree. A diff touching no examined file
reports that it changes no Rust source and passes. Both uses of the commit-pinned
`Swatinem/rust-cache` action are version 2.

### Pull request (`pull-request.yml`)

The change-scoped checks need the release line, so shallow history is not enough.

A branch name is the first thing a reader of the history sees, so `git/branch-naming` runs inside the
normal Godlint quality gate. It cannot refuse the push, but it does refuse the merge. `release` is not a
Conventional Commits type; it is allowed because a version bump is not honestly any of the others.

### Real-world corpus (`real-world.yml`)

Cloning nine repositories, two of them three hundred megabytes, takes long enough that running it on
every pull request would tax every change for the sake of the few that can break it. The binary is a
release build rather than a debug build because the corpus is a hundred thousand files, and a debug
binary spends longer scanning it than the runner should spend on a scheduled job. The commit-pinned
`Swatinem/rust-cache` action is version 2.

### Release (`release.yml`)

A tag is the only thing that starts a release, so it is the thing that must agree with the manifest
and the changelog. Publishing is irreversible: a version can be yanked but never replaced, so these
are checked before anything is built.

The x86-64 macOS binary is cross-compiled on an Arm runner, so the result cannot be executed there.
The musl targets are statically linked, which is what a container without glibc needs and what the
npm packages ship so that one binary per architecture runs against either libc. The grammars are C,
so a musl binary needs a musl C compiler and not only the Rust target. A musl build exists to run
where glibc is absent, so being static is the whole point and worth asserting. `file` reports
`static-pie linked`; `ldd`'s wording for such a binary varies.

Windows has neither `install` nor `shasum`, and a zip is what a Windows user expects. The PowerShell
packaging script uses `WriteAllText` rather than `Out-File`: `Out-File` ends the line with CRLF, and a
checksum file with a carriage return makes every Unix checker read the filename as ending in one, so
verification fails on a correct download. This explanation remains in the PowerShell scalar as well
because it is script content rather than a YAML comment. The wrappers repackage the executable itself
rather than an archive, and it is kept in a separate artifact so the release carries archives only.

The crates.io token is an environment secret rather than a repository secret, so no other workflow
can read it. `--workspace` publishes in dependency order and waits for each crate to appear in the
index, so `godlint-cli` never races the `godlint-core` it depends on.

npm trusted publishing exchanges the workflow identity for a credential that lives for one publish,
so no npm token is stored. The same identity lets npm attach a provenance attestation tying each
package to the commit and workflow that built it. The workflow installs a version of npm newer than
the one bundled with Node for provenance and trusted publishing.

Every platform package must exist before the package that names them, or npm resolves an optional
dependency that is not published yet and installs no binary at all. A trusted publisher cannot be
configured for a name that does not exist yet, so the first release authenticates with a token. Once
these packages exist, configure trusted publishing for each and delete both the environment block and
the secret. The assembler writes the order, dependencies first, so the pack check and publish step
walk the same list. Naming paths in the workflow twice is what published five packages and then failed
on a sixth that had been renamed. Paths carry `./` because npm reads a bare `owner/name` as a GitHub
shorthand and would fetch it from git.

PyPI uses trusted publishing, so no PyPI token exists to store or rotate. The wheels carry the same
binaries the release publishes everywhere else rather than a second build of them. The commit-pinned
`pypa/gh-action-pypi-publish` action is the `release/v1` line.

A GitHub release may already exist because listing an action on the Marketplace is done by editing
one. Creating it unconditionally failed in that case and the archives never attached, leaving the
release that the action resolves as latest with no binary at all. An archive that never attached is
invisible until someone tries to install, so the count is asserted rather than trusted.

`tomerwave/godlint@v1` is what a workflow writes, so the tag has to arrive at the commit of the release
that just published, and only after every registry and every archive succeeded. A `v1` sitting on a
half-published release hands everyone a broken action. `v1` is the action's interface version, not
the command line's. The inputs are what it promises, and they have not changed since the action
shipped while the binary is still `0.1.x`. A break in those inputs means a `v2` and a new value in the
workflow, not a bump of this one. Exact version tags are immutable by ruleset. `v1` is deliberately
outside it: a tag whose whole purpose is to move cannot also be protected against moving.

### Tests (`test.yml`)

The test command covers the whole workspace, so a new test target cannot be added without CI running
it.
