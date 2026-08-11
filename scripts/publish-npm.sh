#!/usr/bin/env bash
set -euo pipefail

while read -r package; do
  name="$(node -p "require('$package/package.json').name")"
  version="$(node -p "require('$package/package.json').version")"
  if npm view "$name@$version" version --registry https://registry.npmjs.org >/dev/null 2>&1; then
    echo "$name@$version already exists; skipping"
  else
    npm publish "$package" --access public --provenance
  fi
done < "$1"
