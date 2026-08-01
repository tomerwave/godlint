#!/usr/bin/env bash

set -euo pipefail

temporary="$(mktemp -d)"
trap 'rm -rf "$temporary"' EXIT

mkdir -p "$temporary/bin" "$temporary/runner"

cat > "$temporary/bin/godlint" <<'EOF'
#!/usr/bin/env bash
echo "godlint 0.3.0"
EOF
chmod +x "$temporary/bin/godlint"

run_explanation() {
  local outcome="$1"
  local fixes_false_positive="$2"
  local relaxes_a_rule="$3"
  local annotations="$4"

  printf '%s\n' "$annotations" > "$temporary/runner/godlint-annotations.txt"
  : > "$temporary/summary"

  PATH="$temporary/bin:$PATH" \
    RUNNER_TEMP="$temporary/runner" \
    GITHUB_STEP_SUMMARY="$temporary/summary" \
    ACCEPTED_DRIFT_FILE="$temporary/accepted-drift.md" \
    OUTCOME="$outcome" \
    FIXES_FALSE_POSITIVE="$fixes_false_positive" \
    RELAXES_A_RULE="$relaxes_a_rule" \
    scripts/explain-released-agreement.sh
}

run_explanation success false false "" > "$temporary/success.out"
grep -q 'reports nothing against this tree' "$temporary/success.out"

run_explanation failure false false \
  'Configuration is invalid: godlint.yaml: rules: unknown field `ci/no-monolithic-job`' \
  > "$temporary/configuration.out"
grep -q 'cannot read this configuration' "$temporary/configuration.out"
grep -q 'goes green on its own after the next release' "$temporary/configuration.out"
if grep -q 'This means one of two things' "$temporary/configuration.out"; then
  echo "a configuration failure must not print drift-label guidance" >&2
  exit 1
fi

if run_explanation failure false false 'godlint: unreadable input' \
  > "$temporary/status-two.out" 2>&1; then
  echo "a failure without a configuration error must be read as drift and fail" >&2
  exit 1
fi

finding='::error file=src/main.rs,line=1,title=maintainability/function-size::too large'
if run_explanation failure false false "$finding" \
  > "$temporary/unlabelled.out" 2>&1; then
  echo "findings without a drift label must fail" >&2
  exit 1
fi

cp .github/accepted-drift.md "$temporary/accepted-drift.md"
declared_finding='::error file=.github/workflows/release.yml,line=1,title=ci/no-monolithic-job::too many steps'
run_explanation failure false false "$declared_finding" > "$temporary/file-declaration.out"
grep -q 'ci/no-monolithic-job is a relaxed rule' "$temporary/file-declaration.out"
grep -q 'Reason: "Across 231 real jobs, the median was 6 steps and 36% exceeded the old threshold of 7' "$temporary/file-declaration.out"

if run_explanation failure false false "$declared_finding
$finding" > "$temporary/partly-declared.out" 2>&1; then
  echo "a declaration for one finding must not accept another finding" >&2
  exit 1
fi
grep -q '^::error::Undeclared rules reported by released Godlint: maintainability/function-size$' "$temporary/partly-declared.out"

if run_explanation failure false false "$finding" > "$temporary/wrong-declaration.out" 2>&1; then
  echo "an unrelated declaration must not accept an undeclared finding" >&2
  exit 1
fi
grep -q '^::error::Undeclared rules reported by released Godlint: maintainability/function-size$' "$temporary/wrong-declaration.out"
grep -q 'Unused declaration.*ci/no-monolithic-job was not reported' "$temporary/wrong-declaration.out"

run_explanation success false false "" > "$temporary/unused-declaration.out"
grep -q 'Unused declaration.*ci/no-monolithic-job was not reported' "$temporary/unused-declaration.out"

unparseable_finding='::error file=src/main.rs,line=1::too large'
if run_explanation failure false false "$unparseable_finding" > "$temporary/unparseable.out" 2>&1; then
  echo "a finding whose rule id cannot be parsed must fail" >&2
  exit 1
fi
grep -q '^::error::Undeclared rules reported by released Godlint: <unparseable rule id>$' "$temporary/unparseable.out"
rm "$temporary/accepted-drift.md"

run_explanation failure true false "$finding" > "$temporary/false-positive.out"
grep -q 'fixes a false positive' "$temporary/false-positive.out"

run_explanation failure false true "$finding" > "$temporary/relaxed-rule.out"
grep -q 'relaxes a rule' "$temporary/relaxed-rule.out"

printf '%s\n' '# Accepted released drift' > "$temporary/accepted-drift.md"
if run_explanation failure false false "$finding" \
  > "$temporary/empty-declaration.out" 2>&1; then
  echo "an accepted-drift file without a recognisable declaration must fail" >&2
  exit 1
fi

echo "released agreement accepts success, unreadable configuration, labels, and recognisable tree declarations, and rejects undeclared drift"
