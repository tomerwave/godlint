#!/usr/bin/env bash

set -euo pipefail

annotations="$RUNNER_TEMP/godlint-annotations.txt"
accepted_drift_file="${ACCEPTED_DRIFT_FILE:-.github/accepted-drift.md}"
version="$(godlint --version | cut -d' ' -f2)"
drift_pattern='^- `([a-z0-9-]+/[a-z0-9-]+)` — (relaxed rule|fixed false positive): ([^[:space:]].*)$'
declared_rules=()
declared_kinds=()
declared_reasons=()
declared_count=0

if [ -f "$accepted_drift_file" ]; then
  while IFS= read -r line; do
    if [[ "$line" =~ $drift_pattern ]]; then
      declared_rules[$declared_count]="${BASH_REMATCH[1]}"
      declared_kinds[$declared_count]="${BASH_REMATCH[2]}"
      declared_reasons[$declared_count]="${BASH_REMATCH[3]}"
      declared_count=$((declared_count + 1))
    fi
  done < "$accepted_drift_file"
fi

report_all_declarations_unused() {
  if [ "$declared_count" -gt 0 ]; then
    for rule in "${declared_rules[@]}"; do
      echo "::notice::Unused declaration in $accepted_drift_file: $rule was not reported by the released binary."
    done
  fi
}

# The status the released binary exited with, not the conclusion of the step that ran it: a step
# says only whether it failed, while the status says what happened. 0 agrees, 1 reports findings,
# anything else means the release stopped before it could judge the tree. It is empty when the
# action failed before the check ran at all, which is not a statement about this tree either.
if ! [[ "$STATUS" =~ ^[0-9]+$ ]]; then
  echo "::error::No exit status from the released Godlint, so this check establishes nothing about drift."
  exit 1
fi

status="$STATUS"

# The step's own conclusion answers the one question the status cannot: whether the action failed
# for a reason that is not Godlint's verdict, such as a step after the check failing. Then the
# status is Godlint's and honest while the failure is the action's own.
if [ "$OUTCOME" = "failure" ] && [ "$status" = "0" ]; then
  echo "::error::The action failed while Godlint exited 0, so the failure is in the action rather than in this tree."
  exit 1
fi

if [ "$status" = "0" ]; then
  echo "Godlint $version reports nothing against this tree."
  report_all_declarations_unused
  exit 0
fi

if [ "$status" != "1" ]; then
  # Whether the configuration is why is asked of the release itself, because `config validate`
  # answers it with an exit status. Reading it out of an error message would tie this gate to the
  # wording a *past* release chose, which no test in this repository can hold still.
  if godlint config validate > /dev/null 2>&1; then
    {
      echo "The released Godlint $version could not check this tree, and the configuration is not why."
      echo
      echo "It exited $status, which is neither agreement nor findings, so this run establishes"
      echo "nothing about drift in either direction: a file it could not parse or a path it would"
      echo "not accept leaves part of the tree unchecked, so whatever it did report is a partial"
      echo "answer. That is a failure to fix here, not drift to declare."
    } | tee -a "$GITHUB_STEP_SUMMARY"
    exit 1
  fi

  {
    echo "The released Godlint $version cannot read this configuration."
    echo
    echo "This is expected when a pull request adds a configuration key, a suite or a"
    echo "configuration version that the release does not have. It is not drift, needs"
    echo "neither drift label, and goes green on its own after the next release."
    echo
    echo "Adding a *rule* does not land here: a release ignores a rule key it does not"
    echo "know, with a notice, so the configuration still reads."
  } | tee -a "$GITHUB_STEP_SUMMARY"
  report_all_declarations_unused
  exit 0
fi

count="$(grep -c '^::' "$annotations" 2>/dev/null || true)"
count="${count:-0}"
reported_rules=()
reported_count=0
unparseable_finding=false

while IFS= read -r line; do
  [[ "$line" == ::* ]] || continue
  rule=""
  if [[ "$line" =~ title=([^,:]+) ]]; then
    rule="${BASH_REMATCH[1]}"
  fi
  if [[ "$rule" =~ ^[a-z0-9-]+/[a-z0-9-]+$ ]]; then
    already_reported=false
    if [ "$reported_count" -gt 0 ]; then
      for reported_rule in "${reported_rules[@]}"; do
        if [ "$reported_rule" = "$rule" ]; then
          already_reported=true
          break
        fi
      done
    fi
    if [ "$already_reported" = "false" ]; then
      reported_rules[$reported_count]="$rule"
      reported_count=$((reported_count + 1))
    fi
  else
    unparseable_finding=true
  fi
done < "$annotations"

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
  if [ "$reported_count" -gt 0 ]; then
    for rule in "${reported_rules[@]}"; do
      echo "  $rule"
    done
  fi
  if [ "$unparseable_finding" = "true" ]; then
    echo "  <unparseable rule id>"
  fi
} | tee -a "$GITHUB_STEP_SUMMARY"

undeclared_rules=()
undeclared_count=0

if [ "$reported_count" -gt 0 ]; then
  for reported_rule in "${reported_rules[@]}"; do
    declaration_index=-1
    if [ "$declared_count" -gt 0 ]; then
      for index in "${!declared_rules[@]}"; do
        if [ "${declared_rules[$index]}" = "$reported_rule" ]; then
          declaration_index="$index"
          break
        fi
      done
    fi
    if [ "$declaration_index" -ge 0 ]; then
      echo "::notice::Declared in $accepted_drift_file: $reported_rule is a ${declared_kinds[$declaration_index]}. Reason: \"${declared_reasons[$declaration_index]}\""
    else
      undeclared_rules[$undeclared_count]="$reported_rule"
      undeclared_count=$((undeclared_count + 1))
    fi
  done
fi

if [ "$declared_count" -gt 0 ]; then
  for index in "${!declared_rules[@]}"; do
    declaration_used=false
    if [ "$reported_count" -gt 0 ]; then
      for reported_rule in "${reported_rules[@]}"; do
        if [ "${declared_rules[$index]}" = "$reported_rule" ]; then
          declaration_used=true
          break
        fi
      done
    fi
    if [ "$declaration_used" = "false" ]; then
      echo "::notice::Unused declaration in $accepted_drift_file: ${declared_rules[$index]} was not reported by the released binary."
    fi
  done
fi

if [ "$unparseable_finding" = "true" ]; then
  undeclared_rules[$undeclared_count]="<unparseable rule id>"
  undeclared_count=$((undeclared_count + 1))
fi

if [ "$undeclared_count" -eq 0 ] && [ "$reported_count" -gt 0 ]; then
  exit 0
fi

if [ "$FIXES_FALSE_POSITIVE" = "true" ]; then
  echo "::notice::Declared: this pull request fixes a false positive, so the released binary still reports what the fix removed."
  exit 0
fi

if [ "$RELAXES_A_RULE" = "true" ]; then
  echo "::notice::Declared: this pull request relaxes a rule, so the released binary still enforces the stricter form."
  exit 0
fi

if [ "$undeclared_count" -gt 0 ]; then
  undeclared_list="${undeclared_rules[0]}"
  index=1
  while [ "$index" -lt "$undeclared_count" ]; do
    undeclared_list="$undeclared_list, ${undeclared_rules[$index]}"
    index=$((index + 1))
  done
  echo "::error::Undeclared rules reported by released Godlint: $undeclared_list"
fi

exit 1
