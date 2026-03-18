# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Project Overview

This is a Rust CLI tool named `aip` (AI Providers) for managing Codex configuration profiles. It enables quick switching between different Codex configurations by providing commands to view, add, delete, edit, and switch profiles.

### Purpose
- Manage multiple Codex configuration profiles (~/.Codex/settings.json)
- Quick profile switching for different development contexts (work, personal, test, etc.)
- Centralized configuration management stored in ~/.ai-providers/

### Architecture
See PLAN.md for detailed architecture design, implementation steps, and technical specifications.

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
# View current configuration
cargo run -- list

# Add a new profile
cargo run -- add <profile_name>

# Delete a profile
cargo run -- delete <profile_name>

# Switch to a profile
cargo run -- set <profile_name>
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

- `src/main.rs` - Application entry point and CLI argument parsing
- `Cargo.toml` - Project manifest and dependencies
- `target/` - Build artifacts (gitignored)

## Architecture Notes

### Configuration Management
- The tool manages configuration files for Codex and Codex
- Profiles allow switching between different API keys, endpoints, or settings
- Configuration files are typically stored in user home directory or XDG config paths

### Expected CLI Commands
- `list` / `ls` - Display all available profiles
- `add <profile>` - Create a new profile configuration
- `delete <profile>` / `rm <profile>` - Remove a profile
- `set <profile>` / `use <profile>` - Activate a specific profile
- `show <profile>` - Display profile details
