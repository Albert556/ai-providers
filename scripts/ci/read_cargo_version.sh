#!/bin/sh

set -eu

manifest_path="${1:-Cargo.toml}"

if [ ! -f "$manifest_path" ]; then
    printf 'Manifest not found: %s\n' "$manifest_path" >&2
    exit 1
fi

version="$(
    awk '
        BEGIN {
            in_package = 0
            found = 0
        }

        /^\[package\][[:space:]]*$/ {
            in_package = 1
            next
        }

        /^\[/ {
            in_package = 0
        }

        in_package && /^[[:space:]]*version[[:space:]]*=/ {
            line = $0
            sub(/^[^"]*"/, "", line)
            sub(/".*$/, "", line)
            print line
            found = 1
            exit
        }

        END {
            if (!found) {
                exit 1
            }
        }
    ' "$manifest_path"
)" || {
    printf 'Could not read package version from %s\n' "$manifest_path" >&2
    exit 1
}

printf '%s\n' "$version"
