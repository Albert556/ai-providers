#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <path-to-cargo-toml>" >&2
  exit 1
fi

manifest_path="$1"

if [[ ! -f "$manifest_path" ]]; then
  echo "Cargo.toml not found: $manifest_path" >&2
  exit 1
fi

version_line="$(sed -nE 's/^version[[:space:]]*=[[:space:]]*"([^"]+)"[[:space:]]*$/\1/p' "$manifest_path" | head -n 1)"

if [[ -z "$version_line" ]]; then
  echo "package.version not found in $manifest_path" >&2
  exit 1
fi

printf '%s\n' "$version_line"
