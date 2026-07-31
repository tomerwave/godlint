#!/usr/bin/env bash

set -euo pipefail

annotations="$RUNNER_TEMP/godlint-annotations.txt"
version="$(godlint --version | cut -d' ' -f2)"

if grep -Fq 'Configuration is invalid' "$annotations" 2>/dev/null; then
  {
    echo "The released Godlint $version cannot read this configuration."
    echo
    echo "This is expected when a pull request adds a rule or configuration key that"
    echo "the release does not have. It is not drift, needs neither drift label, and"
    echo "goes green on its own after the next release."
  } | tee -a "$GITHUB_STEP_SUMMARY"
  exit 0
fi

if [ "$OUTCOME" = "success" ]; then
  echo "Godlint $version reports nothing against this tree."
  exit 0
fi

count="$(grep -c '^::' "$annotations" 2>/dev/null || echo 0)"

{
  echo "The released Godlint $version reports $count findings against this tree."
  echo
  echo "This means one of two things."
  echo
  echo "1. This pull request fixed a false positive, so the released binary still"
  echo "   reports what the fix removed. Label it fixes-false-positive."
  echo
  echo "2. This pull request relaxed a rule, so the released binary still enforces the"
  echo "   stricter form. Label it relaxes-a-rule."
  echo
  echo "Either label passes this check and records which one it was. It goes green on its"
  echo "own after the next release."
  echo
  echo "3. Neither of those happened, and this repository has drifted from the standard"
  echo "   it publishes. Then the findings below are real and belong fixed here, and no"
  echo "   label is the right answer."
  echo
  echo "Adding a rule or tightening a threshold does not land here: the released binary is"
  echo "always the more permissive one, so it stays quiet."
  echo
  echo "Rules reported:"
  sed -n 's/^::[a-z]* .*title=\([^:]*\)::.*/  \1/p' "$annotations" | sort -u
} | tee -a "$GITHUB_STEP_SUMMARY"

if [ "$FIXES_FALSE_POSITIVE" = "true" ]; then
  echo "::notice::Declared: this pull request fixes a false positive, so the released binary still reports what the fix removed."
  exit 0
fi

if [ "$RELAXES_A_RULE" = "true" ]; then
  echo "::notice::Declared: this pull request relaxes a rule, so the released binary still enforces the stricter form."
  exit 0
fi

exit 1
