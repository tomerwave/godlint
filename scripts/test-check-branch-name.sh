#!/usr/bin/env bash

set -euo pipefail

for prefix in feat fix perf docs chore refactor style build revert test ci release; do
  BRANCH="$prefix/lower-case.description_1/continued" scripts/check-branch-name.sh >/dev/null
done

if BRANCH=codex/anything scripts/check-branch-name.sh >/dev/null 2>&1; then
  echo "codex/anything must be rejected" >&2
  exit 1
fi

if BRANCH=feat/Uppercase scripts/check-branch-name.sh >/dev/null 2>&1; then
  echo "an upper-case description must be rejected" >&2
  exit 1
fi

if BRANCH=feat/ scripts/check-branch-name.sh >/dev/null 2>&1; then
  echo "an empty description must be rejected" >&2
  exit 1
fi

echo "branch-name spellings are accepted and rejected as specified"
