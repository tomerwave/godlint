#!/usr/bin/env bash

set -euo pipefail

temporary="$(mktemp -d)"
trap 'rm -rf "$temporary"' EXIT

mkdir -p "$temporary/bin" "$temporary/runner"

# The stub answers the two questions the gate asks a release: which version it is, and whether it
# can read the configuration. CONFIGURATION_READS says which answer `config validate` gives.
#
# It matches whole command lines and rejects anything else with status 2, as the real binary does
# for an unknown command. A stub that matched only the first word would let the gate ask a question
# no release understands: the unknown-command status is 2, which the gate reads as an unreadable
# configuration, so the mistake would turn every exit 2 into a silent pass.
cat > "$temporary/bin/godlint" <<'EOF'
#!/usr/bin/env bash
case "$*" in
  "config validate")
    exit "$([ "${CONFIGURATION_READS:-true}" = "true" ] && echo 0 || echo 2)"
    ;;
  "--version")
    echo "godlint 0.3.0"
    ;;
  *)
    echo "Unknown command or arguments: $*" >&2
    exit 2
    ;;
esac
EOF
chmod +x "$temporary/bin/godlint"

run_explanation() {
  local status="$1"
  local fixes_false_positive="$2"
  local relaxes_a_rule="$3"
  local annotations="$4"
  local configuration_reads="${5:-true}"
  # The action fails when Godlint does, so the step's conclusion follows the status unless a case
  # is about the two disagreeing.
  local outcome="${6:-$([ "$status" = "0" ] && echo success || echo failure)}"

  printf '%s\n' "$annotations" > "$temporary/runner/godlint-annotations.txt"
  : > "$temporary/summary"

  PATH="$temporary/bin:$PATH" \
    RUNNER_TEMP="$temporary/runner" \
    GITHUB_STEP_SUMMARY="$temporary/summary" \
    ACCEPTED_DRIFT_FILE="$temporary/accepted-drift.md" \
    CONFIGURATION_READS="$configuration_reads" \
    OUTCOME="$outcome" \
    STATUS="$status" \
    FIXES_FALSE_POSITIVE="$fixes_false_positive" \
    RELAXES_A_RULE="$relaxes_a_rule" \
    scripts/explain-released-agreement.sh
}

run_explanation 0 false false "" > "$temporary/success.out"
grep -q 'reports nothing against this tree' "$temporary/success.out"

run_explanation 2 false false \
  'Configuration is invalid: godlint.yaml: maintainability/function-size: unknown field `max-parts`' \
  false > "$temporary/configuration.out"
grep -q 'cannot read this configuration' "$temporary/configuration.out"
grep -q 'goes green on its own after the next release' "$temporary/configuration.out"
if grep -q 'This means one of two things' "$temporary/configuration.out"; then
  echo "a configuration failure must not print drift-label guidance" >&2
  exit 1
fi

# The same status, and the release reads the configuration, so something else stopped it. This
# proves nothing about drift in either direction and must not be reported as if it did.
if run_explanation 2 false false 'godlint: unreadable input' true \
  > "$temporary/status-two.out" 2>&1; then
  echo "a failure the configuration does not explain must fail" >&2
  exit 1
fi
grep -q 'the configuration is not why' "$temporary/status-two.out"
if grep -q 'This means one of two things' "$temporary/status-two.out"; then
  echo "a failure that is not findings must not print drift-label guidance" >&2
  exit 1
fi

# A label declares drift, and this is not drift, so neither label may pass it.
if run_explanation 2 true true 'godlint: unreadable input' true \
  > "$temporary/status-two-labelled.out" 2>&1; then
  echo "a drift label must not excuse a check that never ran" >&2
  exit 1
fi

if run_explanation "" false false "" true \
  > "$temporary/no-status.out" 2>&1; then
  echo "an unreadable exit status must fail rather than be assumed" >&2
  exit 1
fi
grep -q '^::error::No exit status from the released Godlint' "$temporary/no-status.out"

# The same case with no binary to ask, which is what an action that failed to install leaves behind.
# The message written for it has to survive not being able to run `godlint --version` first.
if env PATH="/usr/bin:/bin" RUNNER_TEMP="$temporary/runner" \
  GITHUB_STEP_SUMMARY="$temporary/summary" ACCEPTED_DRIFT_FILE="$temporary/accepted-drift.md" \
  OUTCOME=failure STATUS="" FIXES_FALSE_POSITIVE=false RELAXES_A_RULE=false \
  scripts/explain-released-agreement.sh > "$temporary/no-binary.out" 2>&1; then
  echo "a missing released binary must fail" >&2
  exit 1
fi
grep -q '^::error::No exit status from the released Godlint' "$temporary/no-binary.out"
if grep -q 'command not found' "$temporary/no-binary.out"; then
  echo "the gate must report the missing status, not die probing the version" >&2
  exit 1
fi

# A checkout with no godlint.yaml. The release could not read a configuration that is not there, so
# without a guard this reads as a release too old to understand one, and passes.
gate="$PWD/scripts/explain-released-agreement.sh"
printf '%s\n' 2 > "$temporary/runner/godlint-status.txt"
if (
  cd "$temporary" &&
  env PATH="$temporary/bin:/usr/bin:/bin" RUNNER_TEMP="$temporary/runner" \
    GITHUB_STEP_SUMMARY="$temporary/summary" \
    ACCEPTED_DRIFT_FILE="$temporary/accepted-drift.md" \
    CONFIGURATION_READS=false OUTCOME=failure STATUS=2 \
    FIXES_FALSE_POSITIVE=false RELAXES_A_RULE=false "$gate"
) > "$temporary/no-config.out" 2>&1; then
  echo "a tree with no godlint.yaml must fail rather than read as an old release" >&2
  exit 1
fi
grep -q '^::error::There is no godlint.yaml' "$temporary/no-config.out"

# Godlint agreed and the action failed anyway, so the failure is the action's own.
if run_explanation 0 false false "" true failure \
  > "$temporary/failed-action.out" 2>&1; then
  echo "an action that failed while Godlint exited 0 must fail" >&2
  exit 1
fi
grep -q '^::error::The action failed while Godlint exited 0' "$temporary/failed-action.out"

finding='::error file=src/main.rs,line=1,title=maintainability/function-size::too large'
if run_explanation 1 false false "$finding" \
  > "$temporary/unlabelled.out" 2>&1; then
  echo "findings without a drift label must fail" >&2
  exit 1
fi

printf '%s\n' \
  '# Accepted released drift' \
  '' \
  '- `ci/no-monolithic-job` — relaxed rule: the corpus said the old threshold was wrong.' \
  > "$temporary/accepted-drift.md"
declared_finding='::error file=.github/workflows/release.yml,line=1,title=ci/no-monolithic-job::too many steps'
run_explanation 1 false false "$declared_finding" > "$temporary/file-declaration.out"
grep -q 'ci/no-monolithic-job is a relaxed rule' "$temporary/file-declaration.out"
grep -q 'Reason: "the corpus said the old threshold was wrong."' "$temporary/file-declaration.out"

if run_explanation 1 false false "$declared_finding
$finding" > "$temporary/partly-declared.out" 2>&1; then
  echo "a declaration for one finding must not accept another finding" >&2
  exit 1
fi
grep -q '^::error::Undeclared rules reported by released Godlint: maintainability/function-size$' "$temporary/partly-declared.out"

if run_explanation 1 false false "$finding" > "$temporary/wrong-declaration.out" 2>&1; then
  echo "an unrelated declaration must not accept an undeclared finding" >&2
  exit 1
fi
grep -q '^::error::Undeclared rules reported by released Godlint: maintainability/function-size$' "$temporary/wrong-declaration.out"
grep -q '^::error::Stale declaration.*ci/no-monolithic-job is declared as a relaxed rule' "$temporary/wrong-declaration.out"

# The release ran and reported nothing, so the declaration is not merely unexercised: the drift it
# accepts no longer happens, and while the line stands the next drift in that rule passes unremarked.
if run_explanation 0 false false "" > "$temporary/stale-on-clean.out" 2>&1; then
  echo "a declaration the release does not report must fail once the release reports nothing" >&2
  exit 1
fi
grep -q '^::error::Stale declaration.*ci/no-monolithic-job' "$temporary/stale-on-clean.out"

# A stale declaration is about the drift file, not about this pull request, so a drift label must not
# excuse it — and the label path is the one that would otherwise exit 0 first.
if run_explanation 1 true false "$finding" > "$temporary/stale-with-label.out" 2>&1; then
  echo "a drift label must not excuse a stale declaration" >&2
  exit 1
fi
grep -q '^::error::Stale declaration.*ci/no-monolithic-job' "$temporary/stale-with-label.out"
# Both facts, not the first one only.
grep -q '^::error::Undeclared rules reported by released Godlint: maintainability/function-size$' "$temporary/stale-with-label.out"

# Two declarations of different kinds, both stale, because every case above has exactly one: with one
# line nothing distinguishes reporting each from reporting the first, or naming each kind from naming
# the first kind. An author with three stale lines must learn all three at once.
printf '%s\n' \
  '# Accepted released drift' \
  '' \
  '- `ci/no-monolithic-job` — relaxed rule: the corpus said the old threshold was wrong.' \
  '- `security/no-weak-hash` — fixed false positive: it read a digest name it should not have.' \
  > "$temporary/accepted-drift.md"
if run_explanation 1 false false "$finding" > "$temporary/two-stale.out" 2>&1; then
  echo "two stale declarations must fail" >&2
  exit 1
fi
grep -q '^::error::Stale declaration.*ci/no-monolithic-job is declared as a relaxed rule' "$temporary/two-stale.out"
grep -q '^::error::Stale declaration.*security/no-weak-hash is declared as a fixed false positive' "$temporary/two-stale.out"

# A rule id the gate could not parse might be the declared rule itself, so this run is no evidence
# that the declaration is spent. Telling the author to delete a live exemption is worse than silence.
unreadable_finding='::error file=src/main.rs,line=1::too large'
if run_explanation 1 false false "$unreadable_finding" > "$temporary/unreadable-record.out" 2>&1; then
  echo "an unparseable finding must still fail" >&2
  exit 1
fi
if grep -q 'Stale declaration' "$temporary/unreadable-record.out"; then
  echo "a finding whose rule id could not be read cannot make a declaration stale" >&2
  exit 1
fi
grep -q '^::notice::Declarations not examined' "$temporary/unreadable-record.out"

# The release said findings and the annotations hold none. That is not a record of nothing, it is no
# record, so it cannot retire a declaration either — and it must still fail, because the two disagree.
if run_explanation 1 false false "" > "$temporary/empty-record.out" 2>&1; then
  echo "a status of 1 with no readable findings must fail" >&2
  exit 1
fi
if grep -q 'Stale declaration' "$temporary/empty-record.out"; then
  echo "an empty findings record cannot make a declaration stale" >&2
  exit 1
fi

printf '%s\n' \
  '# Accepted released drift' \
  '' \
  '- `ci/no-monolithic-job` — relaxed rule: the corpus said the old threshold was wrong.' \
  > "$temporary/accepted-drift.md"

# The trap this change had to avoid. The release could not read the configuration, so it reported
# nothing about any rule — every declaration looks unused, and failing here would fail every pull
# request that adds a configuration key.
run_explanation 2 false false 'Configuration is invalid: godlint.yaml: unknown field' false \
  > "$temporary/untested-declaration.out"
grep -q '^::notice::Declaration not exercised.*ci/no-monolithic-job' "$temporary/untested-declaration.out"
if grep -q 'Stale declaration' "$temporary/untested-declaration.out"; then
  echo "a release that judged nothing cannot make a declaration stale" >&2
  exit 1
fi

unparseable_finding='::error file=src/main.rs,line=1::too large'
if run_explanation 1 false false "$unparseable_finding" > "$temporary/unparseable.out" 2>&1; then
  echo "a finding whose rule id cannot be parsed must fail" >&2
  exit 1
fi
grep -q '^::error::Undeclared rules reported by released Godlint: <unparseable rule id>$' "$temporary/unparseable.out"
rm "$temporary/accepted-drift.md"

run_explanation 1 true false "$finding" > "$temporary/false-positive.out"
grep -q 'fixes a false positive' "$temporary/false-positive.out"

run_explanation 1 false true "$finding" > "$temporary/relaxed-rule.out"
grep -q 'relaxes a rule' "$temporary/relaxed-rule.out"

printf '%s\n' '# Accepted released drift' > "$temporary/accepted-drift.md"
if run_explanation 1 false false "$finding" \
  > "$temporary/empty-declaration.out" 2>&1; then
  echo "an accepted-drift file without a recognisable declaration must fail" >&2
  exit 1
fi

echo "released agreement reads the exit status, accepts an unreadable configuration, labels and recognisable tree declarations, and rejects undeclared drift, a check that could not finish, and a status it cannot read"
