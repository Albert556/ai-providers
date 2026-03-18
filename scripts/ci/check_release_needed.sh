#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <before-sha> <after-sha>" >&2
  exit 1
fi

before_sha="$1"
after_sha="$2"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
read_version_script="$script_dir/read_cargo_version.sh"

if [[ ! -x "$read_version_script" ]]; then
  echo "missing helper script: $read_version_script" >&2
  exit 1
fi

zero_sha='0000000000000000000000000000000000000000'
if [[ -z "$before_sha" || "$before_sha" == "$zero_sha" ]]; then
  echo "missing usable before SHA" >&2
  exit 1
fi

if [[ -z "$after_sha" || "$after_sha" == "$zero_sha" ]]; then
  echo "missing usable after SHA" >&2
  exit 1
fi

before_manifest="$(mktemp)"
after_manifest="$(mktemp)"
trap 'rm -f "$before_manifest" "$after_manifest"' EXIT

git show "${before_sha}:Cargo.toml" > "$before_manifest"
git show "${after_sha}:Cargo.toml" > "$after_manifest"

old_version="$("$read_version_script" "$before_manifest")"
new_version="$("$read_version_script" "$after_manifest")"
tag="v${new_version}"

if [[ "$old_version" != "$new_version" ]]; then
  version_changed=true
else
  version_changed=false
fi

if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null 2>&1; then
  tag_exists=true
else
  tag_exists=false
fi

release_exists=false
api_base="${GITEA_SERVER_URL:-}"
repo_name="${GITHUB_REPOSITORY:-}"
token="${GITEA_RELEASE_TOKEN:-}"

if [[ -n "$api_base" && -n "$repo_name" && -n "$token" ]]; then
  release_url="${api_base%/}/api/v1/repos/${repo_name}/releases/tags/${tag}"
  http_code="$(
    curl -sS -o /dev/null -w '%{http_code}' \
      -H "Authorization: token ${token}" \
      "$release_url"
  )"

  if [[ "$http_code" == "200" ]]; then
    release_exists=true
  elif [[ "$http_code" != "404" ]]; then
    echo "unexpected response when checking release ${tag}: ${http_code}" >&2
    exit 1
  fi
fi

if [[ "$version_changed" == true && "$tag_exists" == false && "$release_exists" == false ]]; then
  should_release=true
else
  should_release=false
fi

emit_output() {
  local line="$1"
  if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    printf '%s\n' "$line" >> "$GITHUB_OUTPUT"
  fi
  printf '%s\n' "$line"
}

emit_output "old_version=${old_version}"
emit_output "new_version=${new_version}"
emit_output "version_changed=${version_changed}"
emit_output "tag=${tag}"
emit_output "tag_exists=${tag_exists}"
emit_output "release_exists=${release_exists}"
emit_output "should_release=${should_release}"
