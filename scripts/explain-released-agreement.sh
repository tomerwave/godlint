#!/usr/bin/env bash

set -euo pipefail

annotations="$RUNNER_TEMP/godlint-annotations.txt"
accepted_drift_file="${ACCEPTED_DRIFT_FILE:-.github/accepted-drift.md}"
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

reported_rules=()
reported_count=0

# A declaration says: the released binary reports this rule, and here is why that is accepted. Once
# a release ships that no longer reports it, the declaration is not merely untidy — it stands ready
# to accept the *next* drift in that rule, silently, which is the one thing this gate exists to
# catch. So a declaration the release did not use is an error wherever the release actually judged
# the tree, and the fix is to delete the line.
stale_declarations() {
  local stale=0
  local index reported used

  if [ "$declared_count" -eq 0 ]; then
    return 0
  fi

  for index in "${!declared_rules[@]}"; do
    used=false
    if [ "$reported_count" -gt 0 ]; then
      for reported in "${reported_rules[@]}"; do
        if [ "${declared_rules[$index]}" = "$reported" ]; then
          used=true
          break
        fi
      done
    fi
    if [ "$used" = "false" ]; then
      echo "::error::Stale declaration in $accepted_drift_file: ${declared_rules[$index]} is declared as a ${declared_kinds[$index]}, and Godlint $version does not report it. Delete the line — while it stands, the next real drift in that rule passes this check unremarked."
      stale=1
    fi
  done

  return "$stale"
}

# The other caller. Here the release never judged the tree, so a declaration is untested rather than
# stale, and saying otherwise would fail every pull request that adds a configuration key.
note_declarations_left_untested() {
  if [ "$declared_count" -gt 0 ]; then
    for rule in "${declared_rules[@]}"; do
      echo "::notice::Declaration not exercised in $accepted_drift_file: $rule, because the released binary reported nothing either way."
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

# Asked once the status is known to exist, so a release that never ran is reported by the guard
# above rather than by `command not found` on this line.
version="$(godlint --version | cut -d' ' -f2)"

# The step's own conclusion answers the one question the status cannot: whether the action failed
# for a reason that is not Godlint's verdict, such as a step after the check failing. Then the
# status is Godlint's and honest while the failure is the action's own. No step in the action can
# reach this today — the check exits 0 whatever it found, the summary is off here, and the step
# that fails does so only on a non-zero status — so this guards a future step, not a current one.
if [ "$OUTCOME" = "failure" ] && [ "$STATUS" = "0" ]; then
  echo "::error::The action failed while Godlint exited 0, so the failure is in the action rather than in this tree."
  exit 1
fi

if [ "$STATUS" = "0" ]; then
  echo "Godlint $version reports nothing against this tree."
  stale_declarations || exit 1
  exit 0
fi

if [ "$STATUS" != "1" ]; then
  # `config validate` reads ./godlint.yaml, so this asserts the assumption the next line depends on:
  # that this step's directory is the one the action checked. Without it a checkout with no
  # configuration at all takes the vacuous branch below — `config validate` cannot read a file that
  # is not there — and a repository with no policy would report as a release too old to read one.
  if [ ! -f godlint.yaml ]; then
    echo "::error::There is no godlint.yaml in $PWD, so nothing here states the policy the released binary was asked to apply."
    exit 1
  fi

  # Whether the configuration is why is asked of the release itself, because `config validate`
  # answers it with an exit status. Reading it out of an error message would tie this gate to the
  # wording a *past* release chose, which no test in this repository can hold still.
  if godlint config validate > /dev/null 2>&1; then
    {
      echo "The released Godlint $version could not check this tree, and the configuration is not why."
      echo
      echo "It exited $STATUS, which is neither agreement nor findings, so this run establishes"
      echo "nothing about drift in either direction: a file it could not parse or a path it would"
      echo "not accept leaves part of the tree unchecked, so whatever it did report is a partial"
      echo "answer. That is a failure to fix here, not drift to declare, and neither drift label"
      echo "nor an accepted-drift declaration can pass it — both answer findings, and this is not"
      echo "findings."
      echo
      echo "One case is neither a mistake nor drift: this pull request adds source the released"
      echo "grammar cannot parse yet. Nothing here can wave that through, so exclude the file"
      echo "until the next release ships a grammar that reads it."
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
  note_declarations_left_untested
  exit 0
fi

count="$(grep -c '^::' "$annotations" 2>/dev/null || true)"
count="${count:-0}"
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

stale=0
stale_declarations || stale=1

if [ "$unparseable_finding" = "true" ]; then
  undeclared_rules[$undeclared_count]="<unparseable rule id>"
  undeclared_count=$((undeclared_count + 1))
fi

# A stale declaration bars every exit 0 below, because it is a statement about the drift file rather
# than about this pull request, and nothing this pull request carries should excuse it. It does not
# short-circuit them: an undeclared finding is reported too, so a run with both says both.
if [ "$stale" = "0" ]; then
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
