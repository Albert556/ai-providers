# AI Providers (aip)

A Rust CLI tool for managing Claude Code configuration profiles. Quickly switch between different Claude Code configurations for various development contexts (work, personal, test, etc.).

## Features

- 📦 Manage multiple Claude Code configuration profiles
- 🔄 Quick profile switching
- 📝 Edit profiles with your favorite editor
- 🎨 Colorful terminal output
- 🔒 Secure file permissions (0600 on Unix)
- ✅ Profile validation and error handling

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
# Copy to a directory in your PATH
sudo cp target/release/aip /usr/local/bin/

# Or create a symlink
sudo ln -s $(pwd)/target/release/aip /usr/local/bin/aip
```

## Usage

### List all profiles

```bash
aip list
# or
aip ls
```

Output:
```
Available profiles:
  * work      (current)
    personal
    test
```

### Show current active profile

```bash
aip current
```

### Show profile details

```bash
aip show <profile>
```

### Show current Claude Code configuration

```bash
aip config
```

### Add a new profile

```bash
# Create from current Claude Code config (default)
aip add work

# Create empty profile
aip add work --empty

# Copy from existing profile
aip add work --from personal
```

### Delete a profile

```bash
# With confirmation
aip delete work

# Force delete without confirmation
aip delete work -f
```

### Edit a profile

```bash
aip edit work
```

Uses `$EDITOR` environment variable (falls back to vim → vi → nano).

### Switch to a profile

```bash
aip use work
```

## Configuration

### Profile Storage

Profiles are stored in `~/.ai-providers/`:

```
~/.ai-providers/
├── state.json          # Current active profile
├── work.json           # work profile
├── personal.json       # personal profile
└── test.json           # test profile
```

### Claude Code Configuration

The tool manages `~/.claude/settings.json`.

### Profile Format

Each profile is a JSON file containing Claude Code settings:

```json
{
  "$schema": "https://json.schemastore.org/claude-code-settings.json",
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
# Create a work profile from current config
aip add work

# Create a personal profile
aip add personal --empty

# Edit the personal profile
aip edit personal

# Switch to work profile
aip use work

# Verify current profile
aip current

# List all profiles
aip list

# Switch to personal profile
aip use personal
```

### Managing Multiple Environments

```bash
# Create profiles for different contexts
aip add development --empty
aip add production --empty
aip add testing --empty

# Configure each profile
aip edit development
aip edit production
aip edit testing

# Switch between them as needed
aip use development
aip use production
```

## Security

- Profile files are created with `0600` permissions (owner read/write only) on Unix systems
- Profile names are validated to prevent path traversal attacks
- Atomic file operations using temporary files + rename

## Error Handling

The tool provides clear, actionable error messages:

```bash
$ aip show nonexistent
Error: Profile 'nonexistent' not found

$ aip add work
Error: Profile 'work' already exists

$ aip add "invalid/name"
Error: Profile name cannot contain path separators
```

## Development

### Build

```bash
cargo build          # Debug build
cargo build --release # Release build
```

### Run

```bash
cargo run -- <args>
```

### Test

```bash
cargo test
```

### Lint and Format

```bash
cargo clippy
cargo fmt
```

## Architecture

See [PLAN.md](PLAN.md) for detailed architecture documentation.

## License

[Add your license here]

## Contributing

[Add contributing guidelines here]
