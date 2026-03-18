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
See PLAN.md for detailed architecture design, design decisions, and technical specifications.

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

# Switch to a profile
cargo run -- claude use <profile_name>
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

## Project Structure

```
src/
├── main.rs              # Entry point, CLI definition (nested subcommands)
├── provider/            # Provider trait abstraction
│   ├── mod.rs           # Provider trait definition
│   └── claude.rs        # ClaudeProvider implementation
├── profile/             # Profile management core
│   ├── mod.rs
│   ├── manager.rs       # ProfileManager (generic, takes a Provider)
│   └── storage.rs       # File I/O (atomic writes, state management)
└── commands/            # Subcommand implementations
    ├── mod.rs
    ├── list.rs
    ├── current.rs
    ├── show.rs
    ├── config.rs
    ├── add.rs
    ├── delete.rs
    ├── edit.rs
    └── use_cmd.rs
```

## Key Design Decisions

- CLI structure: `aip <provider> <command>` (e.g., `aip claude list`)
- Storage: `~/.ai-providers/<provider>/<profile>.json` per-provider directories
- State: each provider tracks its current profile independently in `~/.ai-providers/state.json`
- Profile format: pure config (profile.json content = settings.json content, no metadata)
- Switching: `use` overwrites settings.json without auto-saving current config
- Editing: `edit` only modifies the profile file, does not sync to settings.json
- Deleting current profile: allowed with warning, clears state after deletion
- Architecture: `Provider` trait with `ClaudeProvider` impl; add new providers by implementing trait
