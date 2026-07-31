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
  local status="$1"
  local outcome="$2"
  local fixes_false_positive="$3"
  local relaxes_a_rule="$4"
  local annotations="$5"

  printf '%s\n' "$annotations" > "$temporary/runner/godlint-annotations.txt"
  : > "$temporary/summary"

  PATH="$temporary/bin:$PATH" \
    RUNNER_TEMP="$temporary/runner" \
    GITHUB_STEP_SUMMARY="$temporary/summary" \
    STATUS="$status" \
    OUTCOME="$outcome" \
    FIXES_FALSE_POSITIVE="$fixes_false_positive" \
    RELAXES_A_RULE="$relaxes_a_rule" \
    scripts/explain-released-agreement.sh
}

run_explanation 0 success false false "" > "$temporary/success.out"
grep -q 'reports nothing against this tree' "$temporary/success.out"

run_explanation 2 failure false false \
  'Configuration is invalid: godlint.yaml: rules: unknown field `ci/no-monolithic-job`' \
  > "$temporary/configuration.out"
grep -q 'cannot read this configuration' "$temporary/configuration.out"
grep -q 'goes green on its own after the next release' "$temporary/configuration.out"
if grep -q 'This means one of two things' "$temporary/configuration.out"; then
  echo "a configuration failure must not print drift-label guidance" >&2
  exit 1
fi

if run_explanation 2 failure false false 'godlint: unreadable input' \
  > "$temporary/status-two.out" 2>&1; then
  echo "status 2 without a configuration error must fail" >&2
  exit 1
fi

finding='::error file=src/main.rs,line=1,title=maintainability/function-size::too large'
if run_explanation 1 failure false false "$finding" \
  > "$temporary/unlabelled.out" 2>&1; then
  echo "findings without a drift label must fail" >&2
  exit 1
fi

run_explanation 1 failure true false "$finding" > "$temporary/false-positive.out"
grep -q 'fixes a false positive' "$temporary/false-positive.out"

run_explanation 1 failure false true "$finding" > "$temporary/relaxed-rule.out"
grep -q 'relaxes a rule' "$temporary/relaxed-rule.out"

echo "released agreement accepts success and known configuration incompatibility, rejects other status 2 failures, and requires either drift label for findings"
