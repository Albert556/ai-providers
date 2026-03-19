# Gitea Release Action Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Automate releases on version changes when pushing to `main`, building for Linux x86_64, macOS aarch64, and Windows x86_64.

**Architecture:** Single workflow file with 3 sequential jobs: version detection → matrix build (3 targets) → Gitea Release creation. No external scripts.

**Tech Stack:** Gitea Actions (GitHub Actions compatible YAML), Rust/Cargo, mingw cross-compiler

**Spec:** `docs/superpowers/specs/2026-03-19-gitea-release-action-design.md`

---

## Chunk 1: Workflow File

### Task 1: Create the workflow directory and file

**Files:**
- Create: `.gitea/workflows/release.yml`

- [ ] **Step 1: Create the workflow file with all three jobs**

Create `.gitea/workflows/release.yml` with the complete workflow:

```yaml
name: Release

on:
  push:
    branches: [main]

jobs:
  check-version:
    runs-on: ubuntu-24.04
    outputs:
      release_needed: ${{ steps.check.outputs.release_needed }}
      version: ${{ steps.check.outputs.version }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 20

      - name: Check version change
        id: check
        run: |
          BEFORE="${{ github.event.before }}"
          # Extract current version
          NEW_VERSION=$(grep '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
          echo "Current version: $NEW_VERSION"

          # Handle initial push (all-zeros SHA)
          if echo "$BEFORE" | grep -qE '^0+$'; then
            echo "Initial push detected, triggering release"
            echo "release_needed=true" >> "$GITHUB_OUTPUT"
            echo "version=$NEW_VERSION" >> "$GITHUB_OUTPUT"
            exit 0
          fi

          # Validate before commit is reachable
          if ! git rev-parse --verify "${BEFORE}^{commit}" >/dev/null 2>&1; then
            echo "::error::Before commit $BEFORE is not reachable within fetch-depth=20"
            exit 1
          fi

          # Extract old version
          OLD_VERSION=$(git show "${BEFORE}":Cargo.toml | grep '^version' | sed 's/.*"\(.*\)".*/\1/')
          echo "Previous version: $OLD_VERSION"

          if [ "$OLD_VERSION" != "$NEW_VERSION" ]; then
            echo "Version changed: $OLD_VERSION -> $NEW_VERSION"
            echo "release_needed=true" >> "$GITHUB_OUTPUT"
          else
            echo "Version unchanged"
            echo "release_needed=false" >> "$GITHUB_OUTPUT"
          fi
          echo "version=$NEW_VERSION" >> "$GITHUB_OUTPUT"

  build:
    needs: check-version
    if: needs.check-version.outputs.release_needed == 'true'
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            runner: ubuntu-24.04
            cross_deps: ""
          - target: aarch64-apple-darwin
            runner: macos
            cross_deps: ""
          - target: x86_64-pc-windows-gnu
            runner: ubuntu-24.04
            cross_deps: "gcc-mingw-w64-x86-64"
    runs-on: ${{ matrix.runner }}
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
          echo "$HOME/.cargo/bin" >> "$GITHUB_PATH"

      - name: Add target
        run: rustup target add ${{ matrix.target }}

      - name: Install cross-compile dependencies
        if: matrix.cross_deps != ''
        run: |
          sudo apt-get update
          sudo apt-get install -y ${{ matrix.cross_deps }}

      - name: Configure Windows cross-linker
        if: matrix.target == 'x86_64-pc-windows-gnu'
        run: |
          mkdir -p .cargo
          cat >> .cargo/config.toml <<EOF
          [target.x86_64-pc-windows-gnu]
          linker = "x86_64-w64-mingw32-gcc"
          EOF

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Rename binary
        run: |
          VERSION="${{ needs.check-version.outputs.version }}"
          TARGET="${{ matrix.target }}"
          SRC="target/${TARGET}/release/aip"
          if [ "$TARGET" = "x86_64-pc-windows-gnu" ]; then
            SRC="${SRC}.exe"
            DEST="aip-v${VERSION}-${TARGET}.exe"
          else
            DEST="aip-v${VERSION}-${TARGET}"
          fi
          cp "$SRC" "$DEST"
          echo "Built: $DEST"

      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: aip-${{ matrix.target }}
          path: aip-v*

  release:
    needs: [check-version, build]
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v3
        with:
          path: artifacts

      - name: Flatten artifacts
        run: |
          mkdir -p release-files
          find artifacts -type f -name 'aip-v*' -exec mv {} release-files/ \;
          echo "Release files:"
          ls -la release-files/

      - name: Check if tag exists
        id: tag-check
        run: |
          VERSION="${{ needs.check-version.outputs.version }}"
          if git ls-remote --tags origin | grep -q "refs/tags/v${VERSION}$"; then
            echo "Tag v${VERSION} already exists, skipping release"
            echo "tag_exists=true" >> "$GITHUB_OUTPUT"
          else
            echo "tag_exists=false" >> "$GITHUB_OUTPUT"
          fi

      - name: Create release
        if: steps.tag-check.outputs.tag_exists != 'true'
        uses: https://gitea.lan.wiqun.com/actions/gitea-release-action@v1
        with:
          tag_name: v${{ needs.check-version.outputs.version }}
          name: v${{ needs.check-version.outputs.version }}
          body: "Release v${{ needs.check-version.outputs.version }}"
          files: |-
            release-files/**
```

- [ ] **Step 2: Validate YAML syntax**

Run: `python3 -c "import yaml; yaml.safe_load(open('.gitea/workflows/release.yml'))"`
Expected: No output (valid YAML)

If `pyyaml` not available, use: `ruby -e "require 'yaml'; YAML.load_file('.gitea/workflows/release.yml')"`

- [ ] **Step 3: Commit**

```bash
git add .gitea/workflows/release.yml
git commit -m "feat: add Gitea release workflow

Triggers on push to main, detects Cargo.toml version changes,
builds for Linux x86_64, macOS aarch64, Windows x86_64,
and creates a Gitea Release with binary artifacts."
```

### Task 2: Update project documentation

**Files:**
- Modify: `CLAUDE.md` (add workflow file to File Index)

- [ ] **Step 1: Add `.gitea` section to the File Index in CLAUDE.md**

After the `### root` section, add:

```markdown
### .gitea

- `.gitea/workflows/release.yml` :: release automation; version-detect+matrix-build+gitea-release
```

- [ ] **Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add release workflow to CLAUDE.md file index"
```
