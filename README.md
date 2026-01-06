# pctrl

```
  ██████╗  ██████╗████████╗██████╗ ██╗
  ██╔══██╗██╔════╝╚══██╔══╝██╔══██╗██║
  ██████╔╝██║        ██║   ██████╔╝██║
  ██╔═══╝ ██║        ██║   ██╔══██╗██║
  ██║     ╚██████╗   ██║   ██║  ██║███████╗
  ╚═╝      ╚═════╝   ╚═╝   ╚═╝  ╚═╝╚══════╝

  Mission Control for Self-Hosters & Indie Devs
```

Manage SSH connections, Docker containers, Coolify deployments and Git releases from one unified interface.

## Quick Start

```bash
# Install from source
git clone https://github.com/Degi100/pctrl.git
cd pctrl
cargo install --path apps/cli

# Run pctrl
pctrl              # Shows styled status banner
pctrl --help       # List all commands
pctrl -m tui       # Terminal UI mode
```

## Features

| Feature | CLI | TUI | GUI |
|---------|-----|-----|-----|
| SSH Connections | ✅ | ✅ | 🔄 |
| Docker Hosts | ✅ | ✅ | 🔄 |
| Coolify Instances | ✅ | ✅ | 🔄 |
| Git Repositories | ✅ | ✅ | 🔄 |

**Legend:** ✅ Working | 🔄 In Progress | ❌ Not Started

### Core Features

- **🔐 SSH Management** - Add, remove, connect and execute commands on remote servers
- **🐳 Docker Control** - Manage Docker hosts and containers across multiple machines
- **🚀 Coolify Integration** - Deploy and monitor applications on Coolify instances
- **📦 Git Releases** - Create tags and manage releases for your repositories
- **💻 Three Modes** - CLI, TUI (Terminal UI), and GUI (Tauri + React)
- **🔒 Encrypted Storage** - AES-256-GCM encrypted SQLite database with Argon2 key derivation

## Usage

### SSH Management

```bash
# List all SSH connections
pctrl ssh list

# Add a new SSH connection
pctrl ssh add "My Server" 192.168.1.100 -u root -p 22 -k ~/.ssh/id_rsa

# Remove a connection
pctrl ssh remove my-server

# Connect to a host
pctrl ssh connect my-server

# Execute command on remote host
pctrl ssh exec my-server "ls -la"
```

### Docker Management

```bash
# List configured Docker hosts
pctrl docker hosts

# Add a new Docker host
pctrl docker add "Local Docker"                    # Uses default socket
pctrl docker add "Remote" -u tcp://10.0.0.1:2375   # Remote Docker

# Remove a host
pctrl docker remove local-docker

# List containers on a host
pctrl docker list local-docker

# Start/stop containers
pctrl docker start local-docker container-id
pctrl docker stop local-docker container-id
```

### Coolify Management

```bash
# List Coolify instances
pctrl coolify instances

# Add a Coolify instance
pctrl coolify add "Production" -u https://coolify.example.com -t your-api-token

# Remove an instance
pctrl coolify remove production

# List deployments
pctrl coolify list production

# Deploy a project
pctrl coolify deploy production project-id
```

### Git Release Management

```bash
# List configured repositories
pctrl git repos

# Add a Git repository
pctrl git add "My Project" -p /path/to/repo

# Remove a repository
pctrl git remove my-project

# List releases/tags
pctrl git list my-project

# Create a new release
pctrl git create my-project v1.0.0 "Initial release"

# Push tags to remote
pctrl git push my-project
```

### Terminal UI Mode

```bash
# Launch TUI
pctrl -m tui

# Navigation:
# ↑/↓ or j/k  - Navigate menu
# Enter       - Select (coming soon)
# q or Esc    - Quit
```

## Architecture

```
pctrl/
├── apps/
│   ├── cli/        # CLI + TUI (Rust, clap, ratatui)
│   ├── desktop/    # GUI (Tauri + React)
│   ├── landing/    # Website (Astro)
│   └── mobile/     # Mobile (Expo + React Native)
├── crates/
│   ├── core/       # Types, Config, Error handling
│   ├── database/   # Encrypted SQLite (sqlx, aes-gcm, argon2)
│   ├── ssh/        # SSH connections (ssh2)
│   ├── docker/     # Docker API (bollard)
│   ├── coolify/    # Coolify API (reqwest)
│   └── git/        # Git operations (git2)
```

## Installation

### Prerequisites

- Rust 1.70+ and Cargo
- Node.js 18+ (for desktop/mobile apps)

### From Source

```bash
git clone https://github.com/Degi100/pctrl.git
cd pctrl
cargo install --path apps/cli
```

### Database Location

The database is stored at:
- **Linux/macOS**: `~/.local/share/pctrl/pctrl.db`
- **Windows**: `%LOCALAPPDATA%\pctrl\pctrl.db`

Custom path: `pctrl --db /path/to/custom.db`

## Development

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Build release
cargo build --release

# Run CLI directly
cargo run -p pctrl-cli -- ssh list

# Run TUI directly
cargo run -p pctrl-cli -- -m tui
```

### Desktop GUI (Tauri)

```bash
cd apps/desktop
npm install
npm run tauri dev
```

### Landing Page (Astro)

```bash
cd apps/landing
npm install
npm run dev
```

## Roadmap

### v0.1.x - Foundation ✅
- [x] Core architecture with modular crates
- [x] CLI with full CRUD for all entities
- [x] TUI with navigation
- [x] Encrypted database persistence
- [x] Styled CLI output

### v0.2.x - Enhanced Features 🔄
- [ ] TUI detail views and actions
- [ ] Desktop GUI functionality
- [ ] Real-time container monitoring
- [ ] SSH password authentication

### v1.0.0 - Production Ready
- [ ] Mobile app functionality
- [ ] Plugin system
- [ ] Cloud sync (optional)
- [ ] Deployment pipelines

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT License - see [LICENSE](LICENSE) for details.

## Links

- [GitHub Repository](https://github.com/Degi100/pctrl)
- [Issue Tracker](https://github.com/Degi100/pctrl/issues)
- [Documentation](QUICKSTART.md)
