# AI Providers (aip)

A Rust CLI tool for managing AI tool configuration profiles. Quickly switch between different configurations for various development contexts (work, personal, test, etc.).

Currently supports **Claude Code**. Architecture designed for multi-provider expansion (Codex, Cursor, etc.).

## Installation

### From Source

```bash
git clone <repository-url>
cd ai-providers
cargo build --release
```

The binary will be available at `target/release/aip`.

### Add to PATH

```bash
sudo cp target/release/aip /usr/local/bin/
# or
sudo ln -s $(pwd)/target/release/aip /usr/local/bin/aip
```

## Usage

Commands are organized by provider: `aip <provider> <command>`.

### Claude Code

#### List all profiles

```bash
aip claude list
# or
aip claude ls
```

Output:
```
Claude Code profiles:
  * work      (current)
    personal
    test
```

#### Show current active profile

```bash
aip claude current
```

#### Show profile details

```bash
aip claude show <profile>
```

#### Show current Claude Code configuration

```bash
aip claude config
```

Shows the actual content of `~/.claude/settings.json`.

#### Add a new profile

```bash
# Create from current Claude Code config (default)
aip claude add work

# Create empty profile
aip claude add work --empty

# Copy from existing profile
aip claude add work --from personal
```

#### Delete a profile

```bash
# With confirmation
aip claude delete work

# Force delete without confirmation
aip claude delete work -f
```

#### Edit a profile

```bash
aip claude edit work
```

Uses `$EDITOR` or `$VISUAL` environment variable (falls back to vim, vi, nano).

**Note**: Editing a profile does not automatically apply changes. Use `aip claude use <profile>` to apply.

#### Switch to a profile

```bash
aip claude use work
```

**Note**: This overwrites `~/.claude/settings.json` with the profile content. Current settings are not auto-saved. Use `aip claude add` to save your current configuration first if needed.

## Configuration

### Profile Storage

Profiles are organized by provider:

```
~/.ai-providers/
├── state.json          # Tracks current profile per provider
├── claude/
│   ├── work.json
│   ├── personal.json
│   └── test.json
└── codex/              # Future
    └── ...
```

### State File

Each provider independently tracks its current active profile:

```json
{
  "claude": {
    "current_profile": "work"
  }
}
```

### Profile Format

Each profile is a JSON file containing the provider's configuration directly (no metadata wrapper):

```json
{
  "model": "claude-opus-4-6",
  "permissions": {
    "allow": ["Read", "Grep", "Glob"],
    "ask": ["Edit", "Write"],
    "deny": ["Bash"]
  }
}
```

## Examples

### Basic Workflow

```bash
# Save current Claude Code config as a profile
aip claude add work

# Create another profile
aip claude add personal --empty
aip claude edit personal

# Switch between profiles
aip claude use work
aip claude use personal

# Verify
aip claude current
aip claude config
```

### Managing Multiple Environments

```bash
# Create profiles for different contexts
aip claude add development
aip claude add production --empty
aip claude add testing --empty

# Configure each
aip claude edit production
aip claude edit testing

# Switch as needed
aip claude use development
aip claude use production
```

## Security

- Profile files are created with `0600` permissions (owner read/write only) on Unix
- Profile names are validated to prevent path traversal attacks
- Atomic file operations using temporary files + rename

## Development Flow

This repository uses **GitHub Flow**:

- Create a feature branch from `main`
- Open a pull request for review
- Merge the pull request back into `main`
- Avoid direct pushes to `main` during normal development

`main` is the only release branch. Regular work should land through PR merges.

## Release Flow

Releases are automated through Gitea Actions and are driven by `Cargo.toml`.

- The release workflow runs only for pushes to `main`
- The workflow compares `Cargo.toml` between the push `before` and `after` commits
- If `package.version` does not change, the workflow skips build and release
- If `package.version` changes, the workflow creates `vX.Y.Z`, builds binaries, and publishes a Gitea Release
- Existing tags or releases are treated as already published and are skipped safely

### Maintainer Release Steps

1. Create a feature branch from `main`
2. Make the code and documentation changes
3. Bump `package.version` in `Cargo.toml` only when that PR should produce a release
4. Merge the PR into `main`
5. Wait for the Gitea Action to create the matching `vX.Y.Z` Release

Current release artifacts target:

- Linux `x86_64-unknown-linux-gnu`
- macOS native runner architecture
- Windows `x86_64-pc-windows-gnu` via Linux cross-compilation

## Architecture

See [docs/architecture.md](docs/architecture.md) for detailed architecture documentation including:
- Provider trait design
- Design decision records
- Implementation details and extension guide

## Development

```bash
cargo build          # Build
cargo test           # Test
cargo clippy         # Lint
cargo fmt            # Format
cargo run -- <args>  # Run
```

## License

[Add your license here]
