#!/bin/sh

set -eu

if [ "$#" -ne 3 ]; then
    printf 'Usage: %s <source_binary> <version> <target>\n' "$0" >&2
    exit 2
fi

source_binary="$1"
version="$2"
target="$3"

if [ ! -f "$source_binary" ]; then
    printf 'Binary not found: %s\n' "$source_binary" >&2
    exit 1
fi

source_dir="$(CDPATH= cd -- "$(dirname "$source_binary")" && pwd)"
source_name="$(basename "$source_binary")"
base_name="${source_name%.exe}"
extension=""

if [ "$target" = "x86_64-pc-windows-gnu" ]; then
    extension=".exe"
fi

packaged_path="${source_dir}/${base_name}-v${version}-${target}${extension}"
cp "$source_binary" "$packaged_path"

printf '%s\n' "$packaged_path"
