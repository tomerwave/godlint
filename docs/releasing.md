# Releasing

Releases are tag-driven, and only a repository administrator can create the tag. Publishing cannot be
undone — a version may be yanked but never replaced — so everything checkable runs before anything is
built.

## Before tagging

Leave `.github/accepted-drift.md` in place, and delete it in the first pull request after the release
publishes. Its declarations describe temporary disagreement with the currently released binary, and
that binary is still the previous one while the release pull request is open — so a release pull
request that deleted the file would fail its own released-agreement check, naming the very drift the
release is about to resolve. The declaration expires when the new binary is published, not when the
tag is created.

Prefer this file to the `relaxes-a-rule` and `fixes-false-positive` labels: a label is a coarse,
temporary declaration that dies with its pull request, while the file names each rule precisely,
survives the merge, and keeps `main` honest.

Deleting it is enforced rather than remembered. Once the new binary is published it no longer
reports the rules the file names, and a declaration the released binary does not report **fails** the
released-agreement check — on every pull request, until the line is deleted. That is the intended
pressure and it is worth knowing before it arrives: the first pull request after a release either
deletes the file or goes red naming each line to remove.

The reason it fails rather than notices is what a declaration is for. It stands ready to accept
disagreement in one named rule, so a line left behind after the drift it described is resolved will
silently accept the *next* drift in that rule — the one case this check exists to catch. A stale
declaration is not untidiness, it is an exemption nobody is watching.

Three situations look like a stale declaration and are not. None of them is reported as stale, which
is not the same as none of them failing:

- **The release cannot read this repository's configuration.** It reported nothing about any rule, so
  its declarations are unexercised rather than spent, and they are reported as notices. The check
  passes — otherwise every pull request adding a configuration key would fail on declarations that
  are perfectly good.
- **The release stopped before it could finish.** The check fails, for its own reason and with its own
  message, and says nothing about declarations at all. A partial scan is not evidence about a rule it
  may never have reached.
- **The release reported findings this gate could not read as a list of rules** — a rule id it could
  not parse, or an annotations file with nothing in it. The unreadable finding may *be* the declared
  rule, so the declarations are reported as not examined. The findings themselves still have to be
  answered, so the check fails on them unless a drift label declares them — which is what a label has
  always done, and unreadable findings are no exception.

**A known limitation.** The released-agreement job runs on Linux, macOS and Windows, and each runner
judges the file against its own findings. So drift on one platform only cannot be declared: the
declaration is used on the runner that reports the rule and stale on the two that do not, which now
fails there instead of printing a notice. It has not come up: the only declaration this file has ever
carried was a raised `ci/no-monolithic-job` threshold, which every platform reports alike. The
alternative is a declaration any platform can leave unexamined, which is the accountability the file
exists to provide — so if it does come up, the answer is a declaration that names the platform, not a
quieter check.

The release notes are every category in the version's section except `Internal`, which is where a
change nothing a user can observe belongs — `check-release.py` drops it, so an entry recording that a
refactor changed no behaviour stays in the log without being announced to people who will never read
this repository.

Rename the changelog's `Unreleased` section to the version, make sure the workspace version already
says the same, and check that all three agree:

```bash
python3 scripts/check-release.py v0.2.0
```

It also prints the notes the release will carry, so the release body is derived from the changelog
rather than written twice.

## What the tag sets off

The workflow runs in stages, and the order is the point:

1. **agree** — re-checks that the tag, the workspace version and the changelog say the same thing.
2. **binaries** — builds seven targets: Linux and macOS on both architectures, Windows, and a
   statically linked musl build for containers without glibc.
3. **publish, npm, pypi** — the three registries, in parallel.
4. **announce** — creates or edits the GitHub release, attaches every archive, asserts the count
   attached matches the count built, and moves the floating `v1` tag.

The GitHub release is published **last, only after all three registries have succeeded.** A release
that exists while a registry failed is the worst of the failure modes: it is what the action resolves
as `latest`, so it hands every user a version that cannot be installed.

The archive count is asserted rather than trusted, because an archive that never attached is invisible
until somebody tries to install it.

## Credentials

Each registry is reached with the least durable credential it accepts:

- **crates.io** takes a token, held as an *environment* secret rather than a repository one, so no
  other workflow can read it.
- **npm** uses trusted publishing from its own environment, and each package carries a provenance
  attestation tying it to the commit and workflow that built it. npm cannot accept a trusted publisher
  for a name that does not exist, so a brand new package needs one token-authenticated release before
  it can be configured; every release after that is tokenless.
- **PyPI** uses trusted publishing too, and accepts it for a project that does not exist yet, so no
  PyPI token is ever needed.

## Tags

Exact version tags are immutable: a ruleset covering `refs/tags/v*.*.*` refuses creation, update and
deletion by anyone but an administrator. That is what lets a pinned version mean one thing forever.

`v1` is deliberately outside that rule, because a tag whose purpose is to move cannot also be protected
against moving. It names the *action's* interface rather than the binary's version, and the release
advances it in the `announce` stage — after publishing, never before. A breaking change to the action's
inputs means a new `v2`, not a redefinition of `v1`.

## Verifying a release by hand

Registries have a way of accepting a package that does not work. The two checks worth doing on a real
machine, because no dry run catches either:

```bash
pip install godlint==0.2.0 && godlint --version
npm install --ignore-scripts @godlint/cli@0.2.0 && npx godlint --version
```

Both have caught shipped bugs: a wheel whose binary installed without its executable bit, and a
checksum file with Windows line endings that made verification fail on every Windows install.
