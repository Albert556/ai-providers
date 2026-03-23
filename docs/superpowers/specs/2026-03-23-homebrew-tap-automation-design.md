# Homebrew Tap Automation Design

## Goal

Automatically update the `Albert556/homebrew-tap` repository with a new formula whenever a new version of `ai-providers` is released via the existing GitHub Actions release workflow.

## Approach

Add an `update-homebrew` job to `.github/workflows/release.yml` that runs after the `release` job. This job generates an `ai-providers.rb` formula from a template and pushes it to the tap repository.

## Components

### 1. Formula Template

**File:** `packaging/homebrew/ai-providers.rb.template`

A Ruby formula template with placeholders:
- `{{version}}` — the release version (e.g., `1.1.0`)
- `{{macos_arm64_sha256}}` — sha256 of the `aarch64-apple-darwin` binary
- `{{linux_amd64_sha256}}` — sha256 of the `x86_64-unknown-linux-gnu` binary

The formula:
- Class name: `AiProviders`
- `desc "Manage AI tool configuration profiles from the command line"`
- `homepage "https://github.com/Albert556/ai-providers"`
- `license any_of: ["MIT", "Apache-2.0"]`
- Downloads precompiled binaries from GitHub Releases
- URL pattern: `https://github.com/Albert556/ai-providers/releases/download/v{{version}}/aip-v{{version}}-<target>`
- Uses `on_macos` / `on_linux` blocks for platform-specific URLs and sha256
- macOS block targets `aarch64-apple-darwin` only (Apple Silicon); Intel Macs are not supported since the build matrix does not produce an `x86_64-apple-darwin` binary
- Installs the binary as `aip` (the CLI binary name)
- Generates shell completions using `generate_completions_from_executable(bin/"aip", "completion")`
- Test block verifies `aip --version` contains the version string
- Windows binaries are excluded because Homebrew does not support Windows

### 2. Workflow Job: `update-homebrew`

**File:** `.github/workflows/release.yml` (modified)

New job added after `release`, with `needs: [check-version, build, release]`.

**Conditions (job-level `if`):**
- `needs.check-version.outputs.is_prerelease != 'true'` — prerelease versions must NOT update the tap
- The `release` job must expose a `released` output (set to `'true'` when the release step actually runs) so the update-homebrew job can confirm the release was created

Steps:
1. Checkout the `ai-providers` repository (to access the formula template)
2. Download build artifacts by name: `aip-aarch64-apple-darwin` and `aip-x86_64-unknown-linux-gnu`
3. Compute sha256 for each binary
4. Substitute placeholders in the template via `sed` to produce `ai-providers.rb`
5. Clone `Albert556/homebrew-tap` using the PAT, `mkdir -p Formula/`, copy formula in, commit and push
   - Git user: `github-actions[bot]`
   - Commit message: `ai-providers {{version}}`
   - Direct push to default branch (appropriate for a personal tap)

**Authentication:** Uses a `HOMEBREW_TAP_TOKEN` secret (a GitHub PAT with `repo` scope for `Albert556/homebrew-tap`).

**Error handling:** A tap push failure marks the workflow run as failed so the owner is notified. The push is idempotent — re-running for the same version overwrites with identical content.

### 3. Tap Repository Structure

The automation expects this structure in `Albert556/homebrew-tap`:

```
homebrew-tap/
└── Formula/
    └── ai-providers.rb
```

The job creates the `Formula/` directory if it doesn't exist via `mkdir -p`.

## User Installation

```bash
brew tap Albert556/tap
brew install ai-providers
```

## Required Setup

The repository owner must create a GitHub secret named `HOMEBREW_TAP_TOKEN` containing a Personal Access Token with write access to `Albert556/homebrew-tap`.

## Files Changed

| File | Change |
|------|--------|
| `packaging/homebrew/ai-providers.rb.template` | New — formula template |
| `.github/workflows/release.yml` | Add `update-homebrew` job; add `released` output to `release` job |
| `CLAUDE.md` | Update File Index |
| `README.md` | Add Homebrew install instructions |
| `docs/architecture.md` | Add Homebrew distribution section |
