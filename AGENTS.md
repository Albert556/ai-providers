# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Project Overview

This is a Rust CLI tool named `aip` (AI Providers) for managing AI tool configuration profiles. It supports multiple providers (Claude Code, Codex, etc.) via a `Provider` trait abstraction. Commands are organized by provider: `aip claude <command>`.

**Current version (v1) implements Claude Code support only. Architecture supports multi-provider expansion.**

### Purpose
- Manage multiple configuration profiles for AI coding tools
- Quick profile switching for different development contexts (work, personal, test, etc.)
- Centralized profile storage in `~/.ai-providers/<provider>/`

### Architecture
See [docs/architecture.md](docs/architecture.md) for detailed architecture, implementation details, and extension guide.

## Development Commands

### Building
```bash
cargo build          # Debug build
cargo build --release # Release build
```

### Running
```bash
cargo run            # Run the application
cargo run --release  # Run optimized build
cargo run -- <args>  # Pass arguments to the CLI
```

### CLI Usage Examples
```bash
# List all Claude Code profiles
cargo run -- claude list

# Show current active profile
cargo run -- claude current

# Show current Claude Code settings.json content
cargo run -- claude config

# Show a specific profile
cargo run -- claude show <profile_name>

# Add a new profile (from current config)
cargo run -- claude add <profile_name>

# Add empty profile
cargo run -- claude add <profile_name> --empty

# Delete a profile
cargo run -- claude delete <profile_name>

# Edit a profile with $EDITOR
cargo run -- claude edit <profile_name>

# Switch to a profile (merges with common config if it exists)
cargo run -- claude use <profile_name>

# Show profile merged with common config
cargo run -- claude show <profile_name> --merged

# Create a common (shared base) config
cargo run -- claude add common --empty
cargo run -- claude edit common
```

### Testing
```bash
cargo test           # Run all tests
cargo test <test_name> # Run specific test
cargo test -- --nocapture # Run tests with stdout visible
```

### Linting and Formatting
```bash
cargo clippy         # Run linter
cargo fmt            # Format code
cargo fmt -- --check # Check formatting without modifying
```

### Other Useful Commands
```bash
cargo check          # Fast compile check without producing binary
cargo clean          # Remove build artifacts
cargo doc --open     # Generate and open documentation
```

## File Index

> **IMPORTANT**: Any file addition, removal, or rename MUST update this index.

### Root Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Project manifest: name=ai-providers, bin=aip, edition=2021, dependencies (clap, serde, serde_json, anyhow, colored) |
| `Cargo.lock` | Dependency lock file |
| `README.md` | User-facing documentation: installation, usage examples, configuration |
| `CLAUDE.md` | This file. Claude Code guidance, file index, development commands |
| `AGENTS.md` | Development guidance (symlink target for CLAUDE.md context) |
| `install.sh` | Build + installation helper script (cargo build --release, 3 install options) |
| `.gitignore` | Git ignore rules |

### Automation (`.gitea/`)

| File | Purpose |
|------|---------|
| `.gitea/workflows/release.yml` | Gitea Actions workflow: detect version changes on `main`, build release binaries, create/publish releases idempotently |

### Documentation (`docs/`)

| File | Purpose |
|------|---------|
| `docs/architecture.md` | Architecture and implementation details (Chinese): layered architecture, Provider trait, ProfileManager API, storage internals, security, extension guide |

### Automation Scripts (`scripts/ci/`)

| File | Purpose |
|------|---------|
| `scripts/ci/read_cargo_version.sh` | Extract `package.version` from a `Cargo.toml` file for CI logic |
| `scripts/ci/check_release_needed.sh` | Compare push `before`/`after` versions, detect release necessity, and emit workflow outputs |
| `scripts/ci/package_binary.sh` | Copy release binaries into a stable `aip-vX.Y.Z-<target>` naming scheme |

### Source: Entry Point (`src/`)

| File | Purpose |
|------|---------|
| `src/main.rs` | CLI definition (clap derive): `Cli` → `ProviderCommand` → `ProfileCommands` enums, command dispatch via `handle_profile_command()`, top-level error handling |

### Source: Provider Layer (`src/provider/`)

| File | Purpose |
|------|---------|
| `src/provider/mod.rs` | `Provider` trait: `name()`, `config_path()`, `validate_config()` (default impl accepts any JSON) |
| `src/provider/claude.rs` | `ClaudeProvider` struct: name="claude", config_path=`~/.claude/settings.json` |

### Source: Profile Layer (`src/profile/`)

| File | Purpose |
|------|---------|
| `src/profile/mod.rs` | Module exports: `manager`, `storage` |
| `src/profile/manager.rs` | `ProfileManager<'a>` (holds `&dyn Provider`): list/get/add/delete/use profiles, name validation, `ProfileSource` enum (Empty, FromCurrent, FromProfile) |
| `src/profile/storage.rs` | File I/O functions: `read_json`, `write_json` (atomic: temp+rename, Unix 0600 perms), `remove_file`, `read_current_profile`, `update_current_profile` |

### Source: Commands (`src/commands/`)

| File | Command | Purpose |
|------|---------|---------|
| `src/commands/mod.rs` | — | Module exports for all commands |
| `src/commands/list.rs` | `list` / `ls` | List all profiles, highlight current (green + `*`) |
| `src/commands/current.rs` | `current` | Show current active profile name |
| `src/commands/show.rs` | `show <profile>` | Display profile JSON content |
| `src/commands/config.rs` | `config` | Display active `settings.json` content |
| `src/commands/add.rs` | `add <profile> [--from] [--empty]` | Create new profile from current config / empty / existing profile |
| `src/commands/delete.rs` | `delete <profile> [-f]` | Delete profile with confirmation, warn if current |
| `src/commands/edit.rs` | `edit <profile>` | Open in `$EDITOR`/`$VISUAL`/vim/vi/nano, JSON validation loop |
| `src/commands/use_cmd.rs` | `use <profile>` | Overwrite `settings.json` with profile, update state |

### Tests (`tests/`)

| File | Purpose |
|------|---------|
| `tests/ci_scripts.rs` | Integration tests for CI helper scripts: version extraction, release gating, artifact naming |

## Key Design Decisions

- CLI structure: `aip <provider> <command>` (e.g., `aip claude list`)
- Storage: `~/.ai-providers/<provider>/<profile>.json` per-provider directories
- State: each provider tracks its current profile independently in `~/.ai-providers/state.json`
- Profile format: pure config (profile.json content = settings.json content, no metadata)
- Switching: `use` overwrites settings.json without auto-saving current config
- Editing: `edit` only modifies the profile file, does not sync to settings.json
- Deleting current profile: allowed with warning, clears state after deletion
- Architecture: `Provider` trait with `ClaudeProvider` impl; add new providers by implementing trait
- Release branch: `main` is the only release branch and should receive changes via PR merge
- Release version source: `Cargo.toml` `package.version` is the single source of truth for automated releases
- Release trigger: Gitea Actions publish only when a `main` push changes the version between `before` and `after`

## Documentation

- **README.md**: User-facing documentation (installation, usage, examples)
- **docs/architecture.md**: Architecture and implementation details (Chinese, code-level documentation)

### Documentation Maintenance

Any code changes must be reflected in the relevant documentation:
- **File added/removed/renamed → update the File Index above**
- New commands or CLI changes → update README.md and docs/architecture.md
- Architecture or module changes → update docs/architecture.md
- New providers → update all three docs and File Index
