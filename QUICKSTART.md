# Quick Start Guide

Get up and running with pctrl in minutes!

## Prerequisites

- **Rust** 1.70 or higher ([install](https://rustup.rs/))
- **Node.js** 18 or higher (for desktop/mobile apps)

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
# Show the status dashboard
pctrl

# Show all available commands
pctrl --help
```

You'll see the styled pctrl banner with your current configuration status.

## Three Modes

### CLI Mode (Default)

Fast, scriptable command-line interface:

```bash
pctrl ssh list
pctrl docker hosts
pctrl --help
```

### TUI Mode (Terminal Dashboard)

Beautiful terminal UI with keyboard navigation:

```bash
pctrl -m tui

# Navigation:
# ↑/↓ or j/k  - Move between items
# Enter       - Select (coming soon)
# q or Esc    - Quit
```

### GUI Mode (Desktop App)

Native desktop application:

```bash
cd apps/desktop
npm install
npm run tauri dev
```

## Adding Your First Resources

### SSH Connection

```bash
# Add a server
pctrl ssh add "Production" 10.0.0.1 -u root -p 22

# With custom key
pctrl ssh add "Staging" 10.0.0.2 -u deploy -k ~/.ssh/staging_key

# List connections
pctrl ssh list

# Test connection
pctrl ssh exec production "hostname"
```

### Docker Host

```bash
# Add local Docker
pctrl docker add "Local"

# Add remote Docker host
pctrl docker add "Remote Server" -u tcp://10.0.0.1:2375

# List hosts
pctrl docker hosts

# List containers
pctrl docker list local
```

### Coolify Instance

```bash
# Add Coolify instance
pctrl coolify add "Production" -u https://coolify.example.com -t your-api-token

# List instances
pctrl coolify instances

# List deployments
pctrl coolify list production
```

### Git Repository

```bash
# Add a repository
pctrl git add "My Project" -p /path/to/repo

# List repos
pctrl git repos

# List releases/tags
pctrl git list my-project

# Create a release
pctrl git create my-project v1.0.0 "First release"
```

## Database

Your data is stored in an encrypted SQLite database:

| Platform | Location |
|----------|----------|
| Linux/macOS | `~/.local/share/pctrl/pctrl.db` |
| Windows | `%LOCALAPPDATA%\pctrl\pctrl.db` |

### Custom Database Path

```bash
pctrl --db /custom/path/pctrl.db ssh list
```

### Reset Database

```bash
# Linux/macOS
rm ~/.local/share/pctrl/pctrl.db

# Windows
del %LOCALAPPDATA%\pctrl\pctrl.db
```

## Command Reference

### SSH Commands

```bash
pctrl ssh list                    # List all connections
pctrl ssh add <name> <host> -u <user> [-p <port>] [-k <key>]
pctrl ssh remove <id>             # Remove a connection
pctrl ssh connect <id>            # Connect to host
pctrl ssh exec <id> "<command>"   # Execute command
```

### Docker Commands

```bash
pctrl docker hosts                # List configured hosts
pctrl docker add <name> [-u <url>]
pctrl docker remove <id>          # Remove a host
pctrl docker list <host-id>       # List containers
pctrl docker start <host-id> <container-id>
pctrl docker stop <host-id> <container-id>
```

### Coolify Commands

```bash
pctrl coolify instances           # List instances
pctrl coolify add <name> -u <url> -t <token>
pctrl coolify remove <id>         # Remove instance
pctrl coolify list <instance-id>  # List deployments
pctrl coolify deploy <instance-id> <project-id>
```

### Git Commands

```bash
pctrl git repos                   # List repositories
pctrl git add <name> -p <path>
pctrl git remove <id>             # Remove repository
pctrl git list <repo-id>          # List releases/tags
pctrl git create <repo-id> <tag> <message>
pctrl git push <repo-id>          # Push tags to remote
```

## Troubleshooting

### "pctrl: command not found"

Add Cargo bin to your PATH:

```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$PATH:$HOME/.cargo/bin"
```

### SSH Connection Errors

1. Check your SSH key is loaded:
   ```bash
   ssh-add -l
   ssh-add ~/.ssh/id_rsa
   ```

2. Verify the connection works with standard SSH:
   ```bash
   ssh user@host
   ```

### Docker Connection Errors

1. Check Docker is running
2. For remote hosts, ensure the Docker API is exposed
3. Verify the URL format: `unix:///var/run/docker.sock` or `tcp://host:2375`

### Database Locked

If you see "database locked" errors, ensure only one pctrl instance is running.

## Development

```bash
# Build debug version
cargo build -p pctrl-cli

# Run directly
cargo run -p pctrl-cli -- ssh list

# Run tests
cargo test
```

## Getting Help

- [GitHub Issues](https://github.com/Degi100/pctrl/issues)
- [README](README.md)
- [Architecture](ARCHITECTURE.md)
