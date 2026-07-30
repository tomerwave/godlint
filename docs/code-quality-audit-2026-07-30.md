# Code-quality audit — 2026-07-30

Audited revision: `31cabad` (`origin/main` at the time of the final verification).

## Verdict

The rule engine and analyzer core are disciplined and well tested, but the repository is
not yet release-grade under hostile input or concurrent test execution. No critical
defect or secret was found. Four defects are confirmed by reproduction, two robustness
gaps follow directly from the scan control flow, and two verification gaps remain.

| Priority | Gap | Evidence | Decision |
| --- | --- | --- | --- |
| High | Test temporary paths can collide across processes | Concurrent workspace tests failed at `crates/godlint-core/tests/discovery.rs:29` with `File exists (os error 17)` during the audit; the unchanged timestamp-plus-process-local-counter pattern also exists in `crates/godlint-cli/tests/formats.rs:13-19`. An isolated rerun and a later two-process probe passed, confirming intermittency rather than safety | Replace the hand-rolled names with one atomically allocated temporary workspace helper |
| Medium | Filenames can inject newlines and ANSI control sequences into terminal and CI output | `crates/godlint-cli/src/report.rs:48-57`, `crates/godlint-cli/src/commands/check.rs:119-122`, `crates/godlint-cli/src/commands/config.rs:24-28`, `crates/godlint-cli/src/commands/suppressions.rs:39-71`, and `crates/godlint-cli/src/main.rs:29-34` render unescaped paths or arguments; a filename containing `\n` and `ESC[31m` reached stdout byte-for-byte on the audited revision | Escape untrusted terminal text inside the reporter and route every remaining diagnostic through it |
| Medium | Optional chaining bypasses direct environment-read policy | `crates/godlint-core/src/analyzers/mod.rs:342-354` rejects `?.` in a direct path; `process?.env.PORT` produced no finding while `process.env?.PORT` did on the audited revision | Normalize optional-member access in the JavaScript/TypeScript analyzer and add equivalent JS, JSX, TS, and TSX fixtures |
| Medium | A global-object spelling bypasses dynamic-execution policy | `crates/godlint-core/src/rules/no_dynamic_execution.rs:35-38` matches `eval` and `Function` only; `globalThis.eval(code)` produced no finding while `eval(code)` did | Decide and document the supported global spellings, then extend the existing rule and regression fixtures |
| Medium | Repository source files are read without a byte ceiling | `crates/godlint-core/src/scan.rs:69-74` calls `read_to_string` before enforcing any resource bound | Define a documented source-byte ceiling and report an oversized file as a scan issue before parsing |
| Medium | A per-entry discovery failure aborts the whole scan | `crates/godlint-core/src/discovery.rs:43-55,115-126` propagates metadata and directory-entry failures through `crates/godlint-core/src/scan.rs:33`; `docs/architecture.md:238-242` requires file-specific failures not to discard other findings | Preserve requested-root failures as fatal; convert failures reached during traversal into sorted `ScanIssue` values |
| Medium | Path-safety and mixed CLI outcomes lack black-box coverage | Format coverage is extensive, but outside-root and symlink rejection at `crates/godlint-cli/src/workspace.rs:99-126` and mixed finding-plus-scan-issue precedence at `crates/godlint-cli/src/commands/check.rs:89-106` have no direct process-level cases | Add symlink, outside-root, and mixed outcome tests that pin stderr and exit codes |
| Medium | Dependency advisories are not checked | No dependency-audit workflow exists and `cargo-audit` is not installed locally | Add a locked advisory audit to CI; keep it outside Godlint because advisory data is external and time-varying |

The optional-call form `eval?.()` was tested and is detected; the missed
dynamic-execution case is the separately spelled direct callee `globalThis.eval()`.

## Rule proposals

The defects do not all justify new Godlint rules. Prefer the smallest deterministic
enforcement surface.

### `security/terminal-output-boundary`

Policy: untrusted repository text reaches a terminal only through an escaping reporter.

- Confidence: medium until source-to-sink tracking exists.
- Required facts: output sink, argument origin, and configured safe reporter paths.
- Invalid: printing a repository path, source-derived value, or CLI argument directly.
- Valid: passing the value to the configured terminal escaping boundary.
- Near-term enforcement: configure `architecture/restricted-call` for `println!`,
  `eprintln!`, and equivalent language APIs, allowing them only in reporter modules.

### `reliability/bounded-file-read`

Policy: repository-controlled files are not loaded wholly without an explicit byte bound.

- Confidence: medium; configuration and manifest reads need scoped allowances.
- Required facts: file-read call, path origin, and a dominating size check or bounded
  reader.
- Invalid: `read_to_string` on a discovered repository file.
- Valid: a bounded reader that rejects or truncates before allocation.
- Near-term enforcement: restrict direct whole-file APIs to one reviewed I/O boundary
  with `architecture/restricted-call`.

### `testing/isolated-temporary-storage`

Policy: tests allocate private temporary files and directories atomically rather than
constructing names under a shared temp root.

- Confidence: high for configured test paths.
- Required facts: test scope, temporary-root calls, and atomic temp allocation calls.
- Invalid: timestamp, random suffix, or process-local counter joined to a shared temp
  directory and then created separately.
- Valid: an atomic temporary file/directory primitive owned by an RAII/context manager.
- Near-term enforcement: restrict `std::env::temp_dir`, `tempfile.gettempdir`, and
  `os.tmpdir` to a shared test helper with `architecture/restricted-call`.

### Existing-rule hardening

`security/direct-environment-read` should own the optional-member-access fix. Creating a
second rule would split one policy by syntax. `security/no-dynamic-execution` should
explicitly decide whether global-object spellings such as `globalThis.eval` are in scope.

Discovery degradation, CLI branch coverage, and dependency advisory checks are product or
verification invariants, not source-policy rules. Enforce them with integration tests,
coverage gates, and CI jobs.

## Verification evidence

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Passed |
| `cargo test --workspace --all-targets` in isolation | Passed |
| Concurrent workspace test execution | Failed from temporary-path collision |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | Passed |
| `cargo run -q -p godlint-cli -- check .` | Passed with no findings |
| `python3 scripts/validate-pull-request.py` | Passed |
| Rule-line coverage gate | Passed |
| Advisory database audit | Not run; tooling is absent |

Mutation testing was not rerun because no rule implementation changed. The existing CI
workflow covers changed rule lines on pull requests and the full rule layer on schedule.
