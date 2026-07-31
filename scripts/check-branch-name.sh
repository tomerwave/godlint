#!/usr/bin/env bash

set -euo pipefail

if printf '%s' "$BRANCH" | grep -Eqx '(feat|fix|perf|docs|chore|refactor|style|build|revert|test|ci|release)/[a-z0-9][a-z0-9._/-]*'; then
  echo "$BRANCH"
  exit 0
fi

echo "Branch name '$BRANCH' does not carry a conventional prefix." >&2
echo "Use one of feat, fix, perf, docs, style, refactor, test, build, ci, chore," >&2
echo "revert or release, then a slash and a lower-case description." >&2
echo "For example feat/import-fact. Further slashes are allowed." >&2
exit 1
