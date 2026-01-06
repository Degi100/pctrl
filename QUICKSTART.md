# Quick Start Guide

Get up and running with pctrl in minutes!

## Prerequisites

- **Rust** 1.70 or higher ([install](https://rustup.rs/))
- **Node.js** 18 or higher ([install](https://nodejs.org/))
- **Git** for version control

## Installation

### Option 1: From Source

```bash
# Clone the repository
git clone https://github.com/Degi100/pctrl.git
cd pctrl

# Build the CLI
cargo build -p pctrl-cli --release

# The binary will be at target/release/pctrl
```

### Option 2: Install from Cargo (coming soon)

```bash
cargo install pctrl-cli
```

## First Run

### CLI Mode (Default)

```bash
# Show help
cargo run -p pctrl-cli -- --help

# List SSH connections
cargo run -p pctrl-cli -- ssh list

# List Docker containers
cargo run -p pctrl-cli -- docker list <host-id>
```

### TUI Mode (Terminal UI)

```bash
cargo run -p pctrl-cli -- --mode tui
```

Use arrow keys to navigate and 'q' to quit.

### GUI Mode (Desktop App)

```bash
cd apps/desktop
npm install
npm run tauri:dev
```

## Configuration

Create a configuration file at `~/.config/pctrl/config.yml`:

```yaml
database:
  path: "~/.config/pctrl/pctrl.db"
  encryption_enabled: true

ssh_connections:
  - id: "my-server"
    name: "My Server"
    host: "example.com"
    port: 22
    username: "user"
    auth_method:
      type: "public_key"
      key_path: "~/.ssh/id_rsa"
```

See `config.example.yml` for more examples.

## Common Tasks

### Managing SSH Connections

```bash
# List all connections
pctrl ssh list

# Execute a command
pctrl ssh exec my-server "uptime"
```

### Managing Docker Containers

```bash
# List containers on a host
pctrl docker list local-docker

# Start a container
pctrl docker start local-docker container-id

# Stop a container
pctrl docker stop local-docker container-id
```

### Managing Git Releases

```bash
# List releases in a repository
pctrl git list my-repo

# Create a new release
pctrl git create my-repo v1.0.0 "Release version 1.0.0"

# Push tags to remote
pctrl git push my-repo
```

### Managing Coolify Deployments

```bash
# List deployments
pctrl coolify list my-instance

# Deploy a project
pctrl coolify deploy my-instance project-id
```

## Development

### Running Tests

```bash
cargo test
```

### Building for Production

```bash
# CLI
cargo build -p pctrl-cli --release

# Desktop App
cd apps/desktop
npm run tauri:build

# Landing Page
cd apps/landing
npm run build
```

## Troubleshooting

### "Command not found: pctrl"

Make sure the binary is in your PATH:

```bash
export PATH="$PATH:$HOME/.cargo/bin"
```

### Database Errors

If you encounter database errors, try removing the database file:

```bash
rm ~/.config/pctrl/pctrl.db
```

The database will be recreated on next run.

### SSH Connection Issues

Ensure your SSH keys are properly configured:

```bash
ssh-add ~/.ssh/id_rsa
```

## Next Steps

- Read the [full documentation](https://github.com/Degi100/pctrl/wiki)
- Check out [examples](https://github.com/Degi100/pctrl/tree/main/examples)
- Join the [community discussions](https://github.com/Degi100/pctrl/discussions)

## Getting Help

- 📖 [Documentation](https://github.com/Degi100/pctrl/wiki)
- 💬 [Discussions](https://github.com/Degi100/pctrl/discussions)
- 🐛 [Issue Tracker](https://github.com/Degi100/pctrl/issues)
- 📧 [Contact](https://github.com/Degi100)
