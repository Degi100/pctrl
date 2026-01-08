# Claude Code Instructions for pctrl

## Before Committing

**IMPORTANT:** Always run these commands before committing:

```bash
# Format all Rust code
cargo fmt

# Check for warnings/errors
cargo clippy

# Verify it compiles
cargo check
```

## Code Style Rules

### Rust Formatting
- Always run `cargo fmt` after making changes to Rust files
- Do not manually format code - let rustfmt handle it
- Lines should not exceed 100 characters (rustfmt default)
- Use 4 spaces for indentation

### File Locations
- CLI commands: `apps/cli/src/main.rs`
- Command handlers: `apps/cli/src/cli.rs`
- TUI interface: `apps/cli/src/tui.rs`
- Core types: `crates/core/src/lib.rs`
- Database CRUD: `crates/database/src/lib.rs`

## Project Architecture

This project follows a **project-centric architecture** (MASTERPLAN v6):

- **Projects** are the core organizing entity
- All resources (servers, containers, databases, domains, scripts) link to projects
- Use `project_resources` table for many-to-many relationships

## Entity Types

| Entity | Description |
|--------|-------------|
| Project | Central organizing unit with status (dev/staging/live/archived) |
| Server | VPS, dedicated, or local servers |
| Domain | Domain names with DNS info |
| DatabaseCredentials | Database connection credentials |
| Container | Docker containers |
| Script | Automation scripts |
| ProjectResource | Links projects to resources |

## Common Commands

```bash
# Run CLI
cargo run --package pctrl-cli

# Run TUI
cargo run --package pctrl-cli -- tui

# Run specific command
cargo run --package pctrl-cli -- project list

# Build all
cargo build

# Test all
cargo test
```

## CI/CD

The GitHub Actions workflow runs:
1. `cargo fmt -- --check` - Fails if code is not formatted
2. `cargo clippy` - Warns about code issues
3. `cargo test` - Runs all tests
4. `cargo build` - Builds the project

**Always run `cargo fmt` before committing to avoid CI failures.**
