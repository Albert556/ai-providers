# Homebrew Tap Automation Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Automatically update the `Albert556/homebrew-tap` Homebrew formula when a new stable release is published.

**Architecture:** Add an `update-homebrew` job to the existing `release.yml` workflow. The job downloads build artifacts, computes sha256 checksums, renders a formula from a template, and pushes it to the tap repository. A prerelease guard prevents unstable versions from reaching Homebrew users.

**Tech Stack:** GitHub Actions, Homebrew Ruby formula, shell scripting (sed/sha256sum)

---

## Chunk 1: Formula Template and Workflow

### Task 1: Create the Homebrew formula template

**Files:**
- Create: `packaging/homebrew/ai-providers.rb.template`

- [ ] **Step 1: Create the formula template file**

```ruby
class AiProviders < Formula
  desc "Manage AI tool configuration profiles from the command line"
  homepage "https://github.com/Albert556/ai-providers"
  version "{{version}}"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    on_arm do
      url "https://github.com/Albert556/ai-providers/releases/download/v{{version}}/aip-v{{version}}-aarch64-apple-darwin"
      sha256 "{{macos_arm64_sha256}}"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Albert556/ai-providers/releases/download/v{{version}}/aip-v{{version}}-x86_64-unknown-linux-gnu"
      sha256 "{{linux_amd64_sha256}}"
    end
  end

  def install
    if OS.mac?
      bin.install "aip-v#{version}-aarch64-apple-darwin" => "aip"
    elsif OS.linux?
      bin.install "aip-v#{version}-x86_64-unknown-linux-gnu" => "aip"
    end

    generate_completions_from_executable(bin/"aip", "completion")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/aip --version")
  end
end
```

- [ ] **Step 2: Verify template placeholders**

Run: `grep -o '{{' packaging/homebrew/ai-providers.rb.template | wc -l`
Expected: `7` (version x5, macos_sha256 x1, linux_sha256 x1)

- [ ] **Step 3: Commit**

```bash
git add packaging/homebrew/ai-providers.rb.template
git commit -m "Add Homebrew formula template for ai-providers"
```

### Task 2: Add `released` output to the release job

**Files:**
- Modify: `.github/workflows/release.yml` (the `release` job)

The existing `release` job has a step-level `tag_exists` output but does not expose it as a job output. The `update-homebrew` job needs to know whether a release was actually created. Add a job-level `released` output.

- [ ] **Step 1: Add `outputs` to the release job**

Add an `outputs` block to the `release` job that maps `released` to the negation of `tag_exists`:

```yaml
  release:
    needs: [check-version, build]
    runs-on: ubuntu-slim
    outputs:
      released: ${{ steps.tag-check.outputs.tag_exists != 'true' }}
    steps:
      # ... existing steps unchanged ...
```

This uses the existing `steps.tag-check.outputs.tag_exists` step output and exposes it as a job-level boolean.

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "Expose released output from release job"
```

### Task 3: Add the `update-homebrew` job

**Files:**
- Modify: `.github/workflows/release.yml` (append new job)

- [ ] **Step 1: Add the `update-homebrew` job at the end of release.yml**

Append this job after the `release` job:

```yaml
  update-homebrew:
    needs: [check-version, build, release]
    if: >-
      needs.release.outputs.released == 'true' &&
      needs.check-version.outputs.is_prerelease != 'true'
    runs-on: ubuntu-slim
    steps:
      - uses: actions/checkout@v5

      - name: Download macOS ARM64 artifact
        uses: actions/download-artifact@v7
        with:
          name: aip-aarch64-apple-darwin
          path: artifacts/macos

      - name: Download Linux AMD64 artifact
        uses: actions/download-artifact@v7
        with:
          name: aip-x86_64-unknown-linux-gnu
          path: artifacts/linux

      - name: Generate Homebrew formula
        env:
          VERSION: ${{ needs.check-version.outputs.version }}
        run: |
          MACOS_SHA=$(sha256sum artifacts/macos/aip-v${VERSION}-aarch64-apple-darwin | cut -d' ' -f1)
          LINUX_SHA=$(sha256sum artifacts/linux/aip-v${VERSION}-x86_64-unknown-linux-gnu | cut -d' ' -f1)

          sed \
            -e "s/{{version}}/${VERSION}/g" \
            -e "s/{{macos_arm64_sha256}}/${MACOS_SHA}/g" \
            -e "s/{{linux_amd64_sha256}}/${LINUX_SHA}/g" \
            packaging/homebrew/ai-providers.rb.template > ai-providers.rb

          echo "Generated formula:"
          cat ai-providers.rb

          if grep -q '{{' ai-providers.rb; then
            echo "ERROR: Unreplaced placeholders found in formula"
            grep '{{' ai-providers.rb
            exit 1
          fi

      - name: Update Homebrew tap
        env:
          HOMEBREW_TAP_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
          VERSION: ${{ needs.check-version.outputs.version }}
        run: |
          git clone "https://x-access-token:${HOMEBREW_TAP_TOKEN}@github.com/Albert556/homebrew-tap.git" homebrew-tap
          cd homebrew-tap
          mkdir -p Formula
          cp ../ai-providers.rb Formula/ai-providers.rb

          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add Formula/ai-providers.rb
          git diff --cached --quiet && echo "No changes to commit" && exit 0
          git commit -m "ai-providers ${VERSION}"
          git push
```

- [ ] **Step 2: Verify YAML syntax**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"`
Expected: no error

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "Add update-homebrew job to release workflow"
```

## Chunk 2: Documentation Updates

### Task 4: Update CLAUDE.md File Index

**Files:**
- Modify: `CLAUDE.md`

- [ ] **Step 1: Add packaging entry to File Index**

In the File Index section, add a `### packaging` subsection (after the `.gitea` section):

```markdown
### packaging

- `packaging/homebrew/ai-providers.rb.template` :: Homebrew formula template; placeholders for version+sha256; used by release workflow
```

- [ ] **Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "Add packaging to File Index in CLAUDE.md"
```

### Task 5: Update README.md with Homebrew install instructions

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add Homebrew section to Installation**

Add a `### Homebrew (macOS / Linux)` section after the existing `### Quick Install (Recommended)` section:

```markdown
### Homebrew (macOS / Linux)

```bash
brew tap Albert556/tap
brew install ai-providers
```
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "Add Homebrew install instructions to README"
```

### Task 6: Update docs/architecture.md

**Files:**
- Modify: `docs/architecture.md`

- [ ] **Step 1: Add distribution section**

Add a section at the end (before the design decisions table) describing the Homebrew distribution pipeline:

```markdown
## 分发：Homebrew

release workflow 成功发布新版本后，`update-homebrew` job 自动更新 Homebrew tap（`Albert556/homebrew-tap`）：

1. 下载 macOS ARM64 和 Linux AMD64 构建产物
2. 计算 sha256 校验和
3. 用模板 `packaging/homebrew/ai-providers.rb.template` 生成 formula
4. 推送到 tap 仓库的 `Formula/ai-providers.rb`

预发布版本不会更新 tap。

用户安装：
\`\`\`bash
brew tap Albert556/tap
brew install ai-providers
\`\`\`
```

- [ ] **Step 2: Commit**

```bash
git add docs/architecture.md
git commit -m "Add Homebrew distribution section to architecture docs"
```
