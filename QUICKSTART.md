# Quick Start Guide

Get up and running with pctrl in minutes!

## Prerequisites

- **Rust** 1.70 or higher ([install](https://rustup.rs/))
- **Node.js** 18 or higher (for desktop app)

## Installation

```bash
# Clone and install
git clone https://github.com/Degi100/pctrl.git
cd pctrl
cargo install --path apps/cli

# Verify installation
pctrl --version
```

## First Run

```bash
# Launch TUI (default)
pctrl

# Show all available commands
pctrl --help
```

## Three Interfaces

### TUI Mode (Default)

Beautiful terminal UI with keyboard navigation:

```bash
pctrl

# Navigation:
# ↑/↓ or j/k  - Move between panels
# a           - Add new item
# Tab         - Next form field
# Enter       - Save
# r           - Reload data
# q or Esc    - Quit
```

### CLI Mode

Fast, scriptable command-line interface:

```bash
pctrl project list
pctrl server list
pctrl --help
```

### Desktop App (GUI)

Native desktop application:

```bash
cd apps/desktop
npm install
npm run tauri dev
```

## Adding Resources

### Projects

Projects are the central organizing unit:

```bash
# Add a project
pctrl project add "My App" -d "My awesome app" -s "rust,react"

# With status
pctrl project add "API" --status live

# List projects
pctrl project list

# Show details
pctrl project show my-app
```

### Servers

```bash
# Add a VPS
pctrl server add "Production" 10.0.0.1 -t vps -p hetzner

# Add a local server
pctrl server add "Dev Machine" localhost -t local

# List servers
pctrl server list

# Show details
pctrl server show production
```

### Domains

```bash
# Add a domain
pctrl domain add app.example.com

# With type and SSL info
pctrl domain add staging.example.com -t staging --ssl

# List domains
pctrl domain list
```

### Database Credentials

```bash
# Add PostgreSQL credentials
pctrl db add "Production DB" -t postgres -H localhost -p 5432 -u myuser

# Add MongoDB
pctrl db add "Analytics" -t mongodb -H mongo.example.com

# List databases
pctrl db list

# Get specific field
pctrl db get production-db pass
```

### Scripts

```bash
# Add a local script
pctrl script add "Build" -c "cargo build --release"

# Add with description
pctrl script add "Deploy" -c "./deploy.sh" -d "Deploy to production"

# List scripts
pctrl script list

# Run a script
pctrl script run build
```

## Linking Resources to Projects

```bash
# Link a server to a project
pctrl project link my-app server production -r "production server"

# Link a domain
pctrl project link my-app domain app.example.com

# Link a database
pctrl project link my-app database production-db -r "main database"

# Show project with resources
pctrl project show my-app
```

## Database

Your data is stored in an SQLite database:

| Platform | Location |
|----------|----------|
| Linux/macOS | `~/.local/share/pctrl/pctrl.db` |
| Windows | `%LOCALAPPDATA%\pctrl\pctrl.db` |

### Custom Database Path

```bash
pctrl --db /custom/path/pctrl.db project list
```

### Reset Database

```bash
# Linux/macOS
rm ~/.local/share/pctrl/pctrl.db

# Windows
del %LOCALAPPDATA%\pctrl\pctrl.db
```

## Command Reference

### Project Commands

```bash
pctrl project list                  # List all projects
pctrl project add <name> [-d desc] [-s stack] [--status dev|staging|live|archived]
pctrl project show <name>           # Show project details
pctrl project remove <name>         # Remove a project
pctrl project link <project> <type> <id> [-r role]  # Link resource
pctrl project unlink <project> <link-id>            # Unlink resource
```

### Server Commands

```bash
pctrl server list                   # List all servers
pctrl server add <name> <host> [-t vps|dedicated|local|cloud] [-p provider]
pctrl server show <name>            # Show server details
pctrl server remove <name>          # Remove a server
```

### Domain Commands

```bash
pctrl domain list                   # List all domains
pctrl domain add <domain> [-t production|staging|dev] [--ssl]
pctrl domain show <domain>          # Show domain details
pctrl domain remove <domain>        # Remove a domain
```

### Database Commands

```bash
pctrl db list                       # List all database credentials
pctrl db add <name> -t <type> [-H host] [-p port] [-u user]
pctrl db show <name>                # Show credentials
pctrl db get <name> <field>         # Get specific field (user, pass, url)
pctrl db remove <name>              # Remove credentials
```

### Script Commands

```bash
pctrl script list                   # List all scripts
pctrl script add <name> -c <command> [-d desc] [-t local|ssh|docker]
pctrl script show <name>            # Show script details
pctrl script run <name> [--force]   # Run a script
pctrl script remove <name>          # Remove a script
```

## Troubleshooting

### "pctrl: command not found"

Add Cargo bin to your PATH:

```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$PATH:$HOME/.cargo/bin"
```

### TUI Not Working

The TUI works best in Windows Terminal or CMD. Git Bash may have issues with keyboard input.

### Database Locked

If you see "database locked" errors, ensure only one pctrl instance is running.

## Development

```bash
# Build debug version
cargo build -p pctrl-cli

# Run directly
cargo run -p pctrl-cli -- project list

# Run tests
cargo test
```

## Getting Help

- [GitHub Issues](https://github.com/Degi100/pctrl/issues)
- [README](README.md)
- [Architecture](ARCHITECTURE.md)
