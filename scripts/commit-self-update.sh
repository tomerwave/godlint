#!/usr/bin/env bash
set -euo pipefail

if [ -z "$(git status --porcelain)" ]; then
  echo "already current"
  exit 0
fi

git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git add -A
git commit -m "chore: sync godharness install to latest release"
git push origin HEAD:main
