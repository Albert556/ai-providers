# AI Providers (aip)

A Rust CLI tool for managing AI tool configuration profiles. Quickly switch between different configurations for various development contexts (work, personal, test, etc.).

Currently supports **Claude Code**. Architecture designed for multi-provider expansion (Codex, Cursor, etc.).

## Installation

### Quick Install (Recommended)

**Linux / macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/Albert556/ai-providers/main/install.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/Albert556/ai-providers/main/install.ps1 | iex
```

Options:

```bash
# Install specific version
VERSION=1.1.0 curl -fsSL .../install.sh | sh

# Custom install directory
INSTALL_DIR=/opt/bin curl -fsSL .../install.sh | sh

# Uninstall
UNINSTALL=1 curl -fsSL .../install.sh | sh

# Or run directly
./install.sh --version 1.1.0
./install.sh --uninstall
```

The installer downloads the prebuilt binary to `~/.local/bin` (Unix) or `%LOCALAPPDATA%\aip` (Windows) and automatically configures your PATH.

### Homebrew (macOS / Linux)

```bash
brew tap Albert556/tap
brew install ai-providers
```

### From Source

```bash
git clone https://github.com/Albert556/ai-providers.git
cd ai-providers
cargo build --release
cp target/release/aip ~/.local/bin/
```

This project checks in `Cargo.lock` so CLI builds and releases use a reproducible dependency set.

### Shell Completions

`aip` can generate completion scripts for `bash`, `zsh`, `fish`, `elvish`, and `powershell`.

```bash
# Preview in the terminal
aip completion zsh

# Bash
mkdir -p ~/.local/share/bash-completion/completions
aip completion bash > ~/.local/share/bash-completion/completions/aip

# Zsh
mkdir -p ~/.zsh/completions
aip completion zsh > ~/.zsh/completions/_aip

# Fish
mkdir -p ~/.config/fish/completions
aip completion fish > ~/.config/fish/completions/aip.fish
```

`aip completions <shell>` is also supported as an alias.

## Usage

### Interactive TUI

Launch the interactive terminal UI by running `aip` with no arguments:

```bash
aip
# or explicitly:
aip tui
```

The TUI provides a full-screen interface to browse, switch, and manage profiles:

```
┌─ aip · Claude Code ───────────────────────────┐
│                                                │
│  ▸ work            [current]                   │
│    personal                                    │
│    test                                        │
│    common          [base]                      │
│                                                │
├────────────────────────────────────────────────┤
│ Profile switched to 'work'                     │
│ q:Quit j/k:Navigate Enter:View u:Use a:Add... │
└────────────────────────────────────────────────┘
```

**Keybindings:**

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `j`/`↓`, `k`/`↑` | Navigate / Scroll |
| `Enter` | View profile detail |
| `u` | Use (switch to) selected profile |
| `a` | Add a new profile |
| `d` | Delete selected profile |
| `e` | Edit in $EDITOR (suspends TUI) |
| `c` | View active settings.json |
| `m` | Toggle merged view (in detail view) |

### CLI Commands

Provider commands are organized as `aip <provider> <command>`. Utility commands such as `aip tui` and `aip completion <shell>` live at the top level.

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

#### Generate shell completions

```bash
aip completion zsh
# or
aip completions bash
```

The completion script is written to stdout so you can redirect it into your shell's completion directory.

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

## Dependency Locking

- `Cargo.lock` is committed for this CLI project and should stay in version control.
- CI and release builds should use locked dependency resolution, for example `cargo fetch --locked`, `cargo build --locked`, and `cargo test --locked`.
- When intentionally refreshing dependency versions, update the lock file with `cargo update` and include the resulting `Cargo.lock` changes in the same commit.
- Profile names are validated to prevent path traversal attacks
- Atomic file operations using temporary files + rename

## Development Flow

This repository uses **GitHub Flow**:

- Create a feature branch from `main`
- Open a pull request for review
- Pull requests are gated by GitHub Actions running `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked`
- Merge the pull request back into `main`
- Avoid direct pushes to `main` during normal development

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

Dual-licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)


