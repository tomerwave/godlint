#!/usr/bin/env bash

# The action's check step is shipped surface and it is inline bash, so nothing type-checks it and
# nothing ran it before CI did. It has broken twice in ways a second of local execution would have
# caught: once because GitHub invokes the step with -e that the script's own `set -o` cannot undo,
# and once because PIPESTATUS was read twice and an assignment resets it. Both were silent — the
# step failed, GitHub skipped the rest of the action, and the job still failed the way findings
# fail, which is what the `dirty` workflow job used to assert.
#
# This runs the step's own body, extracted from action.yml so the test cannot drift from the file,
# under the shell GitHub actually uses: bash --noprofile --norc -e -o pipefail.

set -euo pipefail

temporary="$(mktemp -d)"
trap 'chmod -R u+w "$temporary" 2>/dev/null || true; rm -rf "$temporary"' EXIT

action="${ACTION_FILE:-action.yml}"
body="$temporary/step.sh"

# Extracted rather than copied: a test holding its own copy of the step would keep passing while
# the shipped step broke. Stdlib only, matching the other gate scripts.
python3 - "$action" > "$body" <<'PY'
import sys

lines = open(sys.argv[1], encoding="utf-8").read().splitlines()

try:
    step = next(i for i, line in enumerate(lines) if line.strip() == "- name: Run Godlint")
except StopIteration:
    sys.exit("action.yml has no step named 'Run Godlint'")

run = next(i for i in range(step, len(lines)) if lines[i].strip() == "run: |")
indent = len(lines[run]) - len(lines[run].lstrip()) + 2

collected = []
for line in lines[run + 1 :]:
    if line.strip() and len(line) - len(line.lstrip()) < indent:
        break
    collected.append(line[indent:] if line.strip() else "")

if not collected:
    sys.exit("the 'Run Godlint' step has an empty body, so this test would prove nothing")

print("\n".join(collected))
PY

grep -q 'godlint check --format github' "$body" ||
  { echo "extracted the wrong block: it does not run godlint check" >&2; exit 1; }

mkdir -p "$temporary/bin"

# Answers with whatever status the case under test needs, and prints one annotation per finding so
# the step's own count is exercised rather than assumed.
cat > "$temporary/bin/godlint" <<'EOF'
#!/usr/bin/env bash
index=0
while [ "$index" -lt "${ANNOTATIONS:-0}" ]; do
  echo "::error file=a.js,line=1,title=logging/no-production-log::logs from production code"
  index=$((index + 1))
done
if [ "${GODLINT_STATUS:-0}" != "0" ] && [ "${ANNOTATIONS:-0}" = "0" ]; then
  echo "Configuration is invalid: godlint.yaml: unknown field" >&2
fi
exit "${GODLINT_STATUS:-0}"
EOF
chmod +x "$temporary/bin/godlint"

run_step() {
  local status="$1"
  local annotations="$2"
  local runner="$3"

  : > "$temporary/output.txt"

  env -i PATH="$temporary/bin:/usr/bin:/bin" \
    RUNNER_TEMP="$runner" \
    GITHUB_OUTPUT="$temporary/output.txt" \
    PATHS="" \
    GODLINT_STATUS="$status" \
    ANNOTATIONS="$annotations" \
    /usr/bin/env bash --noprofile --norc -e -o pipefail "$body"
}

output() {
  tr '\n' ' ' < "$temporary/output.txt"
}

mkdir -p "$temporary/runner"

# Findings. The step has to survive its own command failing and report what it found.
run_step 1 2 "$temporary/runner" > /dev/null
[ "$(output)" = "findings=2 status=1 " ] ||
  { echo "findings: expected 'findings=2 status=1', got '$(output)'" >&2; exit 1; }
grep -q '^::error .*logging/no-production-log' "$temporary/runner/godlint-annotations.txt" ||
  { echo "findings: the annotations were not written" >&2; exit 1; }

# A clean tree.
run_step 0 0 "$temporary/runner" > /dev/null
[ "$(output)" = "findings=0 status=0 " ] ||
  { echo "clean: expected 'findings=0 status=0', got '$(output)'" >&2; exit 1; }

# A status that is neither: the step reports it rather than flattening it to failure, which is what
# the released-agreement gate reads to tell an unreadable configuration from drift.
run_step 2 0 "$temporary/runner" > /dev/null
[ "$(output)" = "findings=0 status=2 " ] ||
  { echo "unreadable configuration: expected 'findings=0 status=2', got '$(output)'" >&2; exit 1; }

# The annotations could not be written, so every count and every later step would describe a
# shorter run than the one that happened. That must fail rather than report a truncated answer.
mkdir -p "$temporary/readonly"
chmod a-w "$temporary/readonly"
if run_step 1 2 "$temporary/readonly" > /dev/null 2>&1; then
  echo "unwritable annotations: the step must fail rather than count a truncated file" >&2
  exit 1
fi
chmod u+w "$temporary/readonly"

echo "the action's check step reports findings, a clean tree and a status that is neither, and fails when it cannot write the annotations"
