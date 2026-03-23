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

## Build Cleanliness Policy

- Any task that changes Rust source, build scripts, workflow build logic, or compiler-facing configuration must end with a relevant build verification.
- Default expectation: build output must be clean. Do not leave compile errors or warnings behind.
- Informational diagnostics should also be removed whenever practical. Treat them as something to clean up, not background noise.
- When fixing compile errors, warnings, or infos, always prefer resolving the root cause.
- Do not silence diagnostics with `#[allow(...)]`, broad lint suppression, weakened build commands, or similar "ignore it" approaches unless the user explicitly instructs otherwise.
- If the root cause cannot be fixed safely, or fixing it would introduce other meaningful tradeoffs or risks, report the situation to the user first before proceeding.
- If the user explicitly approves leaving a build diagnostic unresolved, record it in `docs/build-exceptions.md` and treat it as a known exception in future work.
- When a diagnostic is already listed in `docs/build-exceptions.md`, do not re-ask about it on every task unless the diagnostic changed materially or the existing exception is no longer accurate.

## Dependency Locking Policy

- This repository is a CLI application, so `Cargo.lock` must be tracked in git.
- Do not add `Cargo.lock` back to `.gitignore`.
- When build or release workflows resolve dependencies, prefer locked mode such as `cargo fetch --locked`, `cargo build --locked`, and `cargo test --locked`.
- If a dependency update is intentional, regenerate the lock file with Cargo and commit the resulting `Cargo.lock` change together with the manifest or workflow change that requires it.

## File Index

> Rule: add/remove/rename files => update this index.

### root

- `Cargo.toml` :: manifest; bin=`aip`; deps=`clap,serde,serde_json,anyhow,colored,ratatui,crossterm`
- `Cargo.lock` :: lockfile
- `README.md` :: user-docs; install+usage+config
- `CLAUDE.md` :: dev-guide; alias-context
- `AGENTS.md` :: dev-guide; source-of-truth
- `install.sh` :: unix-installer; curl|sh; downloads binary from Gitea Releases to ~/.local/bin
- `install.ps1` :: windows-installer; irm|iex; downloads binary from Gitea Releases to %LOCALAPPDATA%\aip
- `.gitignore` :: git ignore rules

### .github

- `.github/workflows/release.yml` :: GitHub release automation; version-detect+matrix-build+github-release

### .gitea

- `.gitea/workflows/release.yml` :: release automation; version-detect+matrix-build+gitea-release
- `.gitea/scripts/compare_versions.py` :: semver comparison; used by release workflow
- `.gitea/scripts/latest_semver_tag.py` :: release helper; fetch latest semver tag from Git refs API

### docs

- `docs/architecture.md` :: arch-doc(zh); layers+Provider+ProfileManager+storage+security+ext
- `docs/build-exceptions.md` :: approved unresolved build diagnostics registry; agents consult this before re-asking

### src

- `src/main.rs` :: cli-entry; clap types+dispatch+top-level error handling; no-args → TUI
- `src/util.rs` :: shared utilities; `resolve_editor()` for editor detection

### src/tui

- `src/tui/mod.rs` :: TUI entry point; `run_tui()`; terminal setup/restore; main event loop; editor suspension
- `src/tui/app.rs` :: App state struct; Mode enum (ProfileList/ProfileDetail/ActiveConfig/AddProfile/ConfirmDelete); state transitions
- `src/tui/ui.rs` :: all rendering logic; profile list, detail view, dialogs, status bar
- `src/tui/event.rs` :: crossterm event polling wrapper
- `src/tui/handler.rs` :: keybinding dispatch per mode; returns Action enum (None/Quit/SuspendForEditor)

### src/provider

- `src/provider/mod.rs` :: trait `Provider`; name+config_path+validate_config
- `src/provider/claude.rs` :: impl `ClaudeProvider`; config=`~/.claude/settings.json`

### src/profile

- `src/profile/mod.rs` :: exports `manager,storage`
- `src/profile/manager.rs` :: `ProfileManager`; list/get/add/delete/use; name validation; `ProfileSource`
- `src/profile/storage.rs` :: json io; atomic write; 0600; state read/write

### src/commands

- `src/commands/mod.rs` :: module exports
- `src/commands/list.rs` :: cmd=`list|ls`; list profiles; mark current
- `src/commands/current.rs` :: cmd=`current`; show current profile
- `src/commands/show.rs` :: cmd=`show`; print profile json
- `src/commands/config.rs` :: cmd=`config`; print active settings.json
- `src/commands/add.rs` :: cmd=`add`; create from current/empty/profile
- `src/commands/delete.rs` :: cmd=`delete`; confirm delete; warn if current
- `src/commands/edit.rs` :: cmd=`edit`; open editor; validate json
- `src/commands/use_cmd.rs` :: cmd=`use`; apply profile; update state

## Key Design Decisions

- CLI structure: `aip <provider> <command>` (e.g., `aip claude list`)
- Storage: `~/.ai-providers/<provider>/<profile>.json` per-provider directories
- State: each provider tracks its current profile independently in `~/.ai-providers/state.json`
- Profile format: pure config (profile.json content = settings.json content, no metadata)
- Switching: `use` overwrites settings.json without auto-saving current config
- Editing: `edit` only modifies the profile file, does not sync to settings.json
- Deleting current profile: allowed with warning, clears state after deletion
- Architecture: `Provider` trait with `ClaudeProvider` impl; add new providers by implementing trait

## Documentation

- **README.md**: User-facing documentation (installation, usage, examples)
- **docs/architecture.md**: Architecture and implementation details (Chinese, code-level documentation)

### Documentation Maintenance

Any code changes must be reflected in the relevant documentation:
- **File added/removed/renamed → update the File Index above**
- New commands or CLI changes → update README.md and docs/architecture.md
- Architecture or module changes → update docs/architecture.md
- New providers → update all three docs and File Index
- User-approved unresolved build diagnostics → update `docs/build-exceptions.md`
