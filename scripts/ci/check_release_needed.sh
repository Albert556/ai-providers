#!/bin/sh

set -eu

if [ "$#" -ne 2 ]; then
    printf 'Usage: %s <before_commit> <after_commit>\n' "$0" >&2
    exit 2
fi

before_commit="$1"
after_commit="$2"
script_dir="$(CDPATH= cd -- "$(dirname "$0")" && pwd)"
repo_root="$(CDPATH= cd -- "$script_dir/../.." && pwd)"
compare_versions_script="$repo_root/.gitea/scripts/compare_versions.py"
temp_dir="$(mktemp -d)"

cleanup() {
    rm -rf "$temp_dir"
}

trap cleanup EXIT INT TERM HUP

emit_output() {
    printf '%s=%s\n' "$1" "$2"
    if [ -n "${GITHUB_OUTPUT:-}" ]; then
        printf '%s=%s\n' "$1" "$2" >> "$GITHUB_OUTPUT"
    fi
}

read_manifest_version() {
    manifest_path="$1"
    bash "$script_dir/read_cargo_version.sh" "$manifest_path"
}

export_manifest_from_commit() {
    commit_ref="$1"
    destination="$2"

    if ! git rev-parse --verify "${commit_ref}^{commit}" >/dev/null 2>&1; then
        printf 'Commit %s is not reachable\n' "$commit_ref" >&2
        exit 1
    fi

    git show "${commit_ref}:Cargo.toml" > "$destination"
}

after_manifest="$temp_dir/after-Cargo.toml"
export_manifest_from_commit "$after_commit" "$after_manifest"
new_version="$(read_manifest_version "$after_manifest")"

if printf '%s' "$before_commit" | grep -qE '^0+$'; then
    old_version=""
    version_changed="true"
    should_release="true"
else
    before_manifest="$temp_dir/before-Cargo.toml"
    export_manifest_from_commit "$before_commit" "$before_manifest"
    old_version="$(read_manifest_version "$before_manifest")"

    if [ "$old_version" = "$new_version" ]; then
        version_changed="false"
    else
        version_changed="true"
    fi

    compare_result="$(python3 "$compare_versions_script" "$old_version" "$new_version")"
    if [ "$compare_result" = "true" ]; then
        should_release="true"
    else
        should_release="false"
    fi
fi

tag="v${new_version}"
if printf '%s' "$new_version" | grep -qE '[-+]'; then
    is_prerelease="true"
else
    is_prerelease="false"
fi

emit_output "old_version" "$old_version"
emit_output "new_version" "$new_version"
emit_output "version" "$new_version"
emit_output "version_changed" "$version_changed"
emit_output "should_release" "$should_release"
emit_output "release_needed" "$should_release"
emit_output "tag" "$tag"
emit_output "is_prerelease" "$is_prerelease"
