#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <source-binary> <version> <target>" >&2
  exit 1
fi

source_binary="$1"
version="$2"
target="$3"

if [[ ! -f "$source_binary" ]]; then
  echo "source binary not found: $source_binary" >&2
  exit 1
fi

output_dir="${DIST_DIR:-$(pwd)/dist}"
mkdir -p "$output_dir"

base_name="aip-v${version}-${target}"
if [[ "$source_binary" == *.exe ]]; then
  base_name="${base_name}.exe"
fi

packaged_path="${output_dir}/${base_name}"
cp "$source_binary" "$packaged_path"

printf '%s\n' "$packaged_path"
