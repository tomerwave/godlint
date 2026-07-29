# Releasing

Releases are tag-driven, and only a repository administrator can create the tag. Publishing cannot be
undone — a version may be yanked but never replaced — so everything checkable runs before anything is
built.

## Before tagging

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
