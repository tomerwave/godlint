# Contribution conventions

Read [CONTRIBUTING.md](../CONTRIBUTING.md) for the public contribution process.

For repository changes:

- Keep changes focused, reviewable, and reversible.
- Document and justify new dependencies.
- Keep source analysis local; use explicit argument arrays for any external command.
- Maintain deterministic diagnostic ordering and stable finding fingerprints.
- Update documentation when public behavior, configuration, suite defaults, or rule
  semantics change.
- Keep test code outside `src/`: crate contracts live in `crates/<crate>/tests/`, and
  rule behavior lives in fixtures under
  `crates/godlint-cli/tests/fixtures/rules/<rule-id>/`.
- Do not add a public API or crate boundary before a real implementation need exists.
- Never commit `.omx/` planning/runtime files.
- Branch from `main` and name the branch with a Conventional Commits type followed by a slash
  and a lower-case description, as in `feat/import-fact`: `feat`, `fix`, `perf`, `docs`,
  `style`, `refactor`, `test`, `build`, `ci`, `chore` or `revert`. Further slashes are allowed.
  A required check enforces this. It cannot refuse the push, only the merge, because GitHub's
  own branch-name rule is not available on this repository's plan.
- `main` is protected: it takes no direct push, no force-push and no deletion, and a pull
  request must have every required check green before it merges. Merges are merge commits, and
  the branch is deleted afterwards. A repository administrator can bypass, which exists to
  unstick a broken check rather than to skip review.
- Pick the pull request template that matches the change: `new-rule` when adding or
  changing a rule, `infrastructure` for build, CI, tooling, or documentation work. Append
  `?template=new-rule.md` or `?template=infrastructure.md` to the pull request URL.
- Releases are tag-driven and only a repository administrator can create the tag. Before tagging,
  rename the changelog's `Unreleased` section to the version and make sure the workspace version
  already says the same: `python3 scripts/check-release.py v<version>` checks all three agree and
  prints the notes the release will carry. Publishing cannot be undone — a version may be yanked
  but never replaced — so the check runs before anything is built. crates.io is reached with a
  token held as an environment secret, so no other workflow can read it. npm is reached by trusted
  publishing from its own environment, and each package carries a provenance attestation tying it to
  the commit and workflow that built it. npm cannot accept a trusted publisher for a name that does
  not exist, so the first release of a new package authenticates with a token; once the name exists,
  configure trusted publishing for it and delete the token. `packaging/build-npm.py` assembles the npm packages from the
  binaries the release built; it takes `--only` so a single platform can be built and installed on
  one machine to check the shim end to end.
- `python3 scripts/validate-pull-request.py` enforces the parts of those templates that
  can be checked. Run it locally; CI runs it too. Its change-scoped checks measure the
  branch against `origin/main` rather than the pull request's target, so a stack of pull
  requests needs one changelog entry for the change and not one per pull request.
