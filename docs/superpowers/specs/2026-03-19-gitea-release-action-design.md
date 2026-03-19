# Gitea Release Action Design

## Overview

Automated release workflow for the `aip` CLI tool. On push to `main`, detects version changes in `Cargo.toml` and builds + publishes releases for 3 targets.

## Trigger

```yaml
on:
  push:
    branches: [main]
```

## Architecture

Single workflow file: `.gitea/workflows/release.yml`

Three sequential jobs:

```
check-version → build (matrix, 3 targets) → release
```

## Runner Labels

| Purpose | Label |
|---------|-------|
| Linux   | `ubuntu-24.04` |
| macOS   | `macos` |

## Job 1: `check-version` (runner: `ubuntu-24.04`)

**Purpose:** Compare `Cargo.toml` version between `github.event.before` commit and current commit.

**Gitea compatibility note:** Gitea Actions supports `github.event.before` and `github.event.after` on push events, matching GitHub Actions behavior.

**Steps:**

1. `actions/checkout@v4` with `fetch-depth: 20`
2. Check if `github.event.before` is all-zeros (`0000000...`) — if so, skip to step 5 with `release_needed=true`
3. Validate `before` commit is reachable: `git rev-parse --verify ${{ github.event.before }}^{commit}` — if fails, **error and exit**
4. Extract old version: `git show ${{ github.event.before }}:Cargo.toml | grep '^version' | sed 's/.*"\(.*\)".*/\1/'`
5. Extract new version: `grep '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/'`
6. Compare: if different → `release_needed=true`, otherwise `false`

**Outputs:**
- `release_needed`: `'true'` or `'false'`
- `version`: the new version string (e.g., `0.2.0`)

## Job 2: `build` (Matrix)

**Condition:** `if: needs.check-version.outputs.release_needed == 'true'`

**Matrix:**

| target                       | runner          | cross-compile deps        |
|------------------------------|-----------------|---------------------------|
| `x86_64-unknown-linux-gnu`   | `ubuntu-24.04`  | none                      |
| `aarch64-apple-darwin`       | `macos`         | none (native)             |
| `x86_64-pc-windows-gnu`      | `ubuntu-24.04`  | `gcc-mingw-w64-x86-64`   |

**Rust toolchain:** Install via `rustup` (stable channel). Each step:
1. `actions/checkout@v4` (default shallow clone)
2. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable`
3. Add target: `rustup target add ${{ matrix.target }}`
4. Install cross-compile dependencies (Windows target only): `sudo apt-get update && sudo apt-get install -y gcc-mingw-w64-x86-64`
5. `cargo build --release --target ${{ matrix.target }}`
6. Rename binary from `target/${{ matrix.target }}/release/aip` (or `aip.exe` for Windows) to:
   - Linux/macOS: `aip-v{version}-{target}`
   - Windows: `aip-v{version}-{target}.exe`
7. Upload artifact via `actions/upload-artifact`

## Job 3: `release` (runner: `ubuntu-24.04`)

**Condition:** depends on `build` job completion.

**Release action:** `https://gitea.lan.wiqun.com/actions/gitea-release-action` (mirror of `akkuman/gitea-release-action`)

**Steps:**

1. Download all build artifacts via `actions/download-artifact`
2. Check if tag `v{version}` already exists — if so, **skip release** (idempotency)
3. Create Gitea Release using the release action:

```yaml
- uses: https://gitea.lan.wiqun.com/actions/gitea-release-action@v1
  with:
    tag_name: v${{ needs.check-version.outputs.version }}
    name: v${{ needs.check-version.outputs.version }}
    body: "Release v${{ needs.check-version.outputs.version }}"
    files: |-
      artifacts/**
```

**Authentication:** Uses default `${{ github.token }}` (no extra secrets needed).

## Artifact Naming

| Platform | Filename                                    |
|----------|---------------------------------------------|
| Linux    | `aip-v0.1.0-x86_64-unknown-linux-gnu`       |
| macOS    | `aip-v0.1.0-aarch64-apple-darwin`           |
| Windows  | `aip-v0.1.0-x86_64-pc-windows-gnu.exe`      |

## Error Handling

- `before` is all-zeros → treat as first release, proceed
- `before` commit not in 20-depth history → **fail with error** (no fallback)
- Any build target fails → whole release is blocked (matrix job fails)
- Tag already exists → skip release creation (idempotent)

## Version Source

Version is read from `Cargo.toml` field using `grep` + `sed`:

```toml
[package]
version = "0.1.0"
```

Parsing: `grep '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/'`

## Decisions Log

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Runner labels | `ubuntu-24.04` / `macos` | User's Gitea instance labels |
| Runner setup | Linux + macOS | Available runners; Windows via cross-compile |
| macOS arch | aarch64 only | User requirement |
| Linux/Windows arch | x86_64 only | User requirement |
| Package format | Bare binary | Simple, no archive overhead |
| Version detection | `github.event.before` vs current Cargo.toml | Push event provides before/after SHAs |
| Fetch depth | 20 | Covers most push scenarios without full clone |
| Fallback on deep push | Error and exit | Keep it simple |
| Workflow structure | Single YAML, no external scripts | Cohesive, easy to maintain |
| Release action | `gitea.lan.wiqun.com/actions/gitea-release-action` | User-specified, self-hosted mirror |
| Rust installation | `rustup` stable channel | Runners don't have Rust pre-installed |
| Tag collision | Skip release | Idempotent behavior |
| Version parsing | `grep` + `sed` | Simple, no extra dependencies |
