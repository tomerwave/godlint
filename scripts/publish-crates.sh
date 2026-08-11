#!/usr/bin/env bash
set -euo pipefail

version="$1"

for package in godlint-core godlint-cli; do
  if curl --fail --silent --show-error "https://crates.io/api/v1/crates/$package/$version" >/dev/null; then
    echo "$package@$version already exists; skipping"
  else
    cargo +1.97.1 publish --locked -p "$package"
  fi
done
