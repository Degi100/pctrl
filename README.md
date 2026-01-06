# pctrl

Mission Control for Self-Hosters & Indie Devs - Manage servers, containers, deployments and releases from one place.

## Quick Start

Get started with pctrl in 3 steps:

```bash
# 1. Clone and build
git clone https://github.com/Degi100/pctrl.git
cd pctrl
cargo build --release -p pctrl-cli

# 2. Run the CLI
./target/release/pctrl --help

# 3. Try different modes
./target/release/pctrl ssh list              # CLI mode
./target/release/pctrl --mode tui            # TUI mode
cd apps/desktop && npm install && npm run tauri:dev  # GUI mode
```

**📚 For detailed instructions, see [QUICKSTART.md](QUICKSTART.md)**

## Features

- **🔐 SSH Management**: Connect to and manage remote servers securely
- **🐳 Docker Control**: Monitor and manage Docker containers across multiple hosts
- **🚀 Coolify Integration**: Deploy and manage applications on Coolify instances
- **📦 Git Releases**: Create and manage Git releases for your repositories
- **💻 Three Modes**: CLI (clap), TUI (ratatui), and GUI (Tauri + React)
- **🔒 Encrypted Storage**: Local-first data with encrypted SQLite database
- **📱 Mobile App**: Expo-based mobile application (coming soon)
- **🌐 Landing Page**: Astro-powered project website with auto-synced roadmap and changelog

## Architecture

This is a monorepo containing:

- **apps/cli**: Command-line and terminal UI application (Rust)
- **apps/desktop**: Desktop GUI application (Tauri + React)
- **apps/landing**: Project website (Astro)
- **apps/mobile**: Mobile application (Expo + React Native)
- **crates/core**: Core types and configuration
- **crates/database**: Encrypted SQLite database layer
- **crates/ssh**: SSH connection management
- **crates/docker**: Docker container management
- **crates/coolify**: Coolify deployment integration
- **crates/git**: Git release management

## Getting Started

### Prerequisites

- Rust 1.70+ and Cargo
- Node.js 18+ and npm
- For desktop app: Tauri prerequisites for your platform

### Installation

1. Clone the repository:
```bash
git clone https://github.com/Degi100/pctrl.git
cd pctrl
```

2. Build the Rust workspace:
```bash
cargo build
```

3. Install Node.js dependencies:
```bash
npm install
```

### Running the Applications

#### CLI/TUI Mode

```bash
# CLI mode (default)
cargo run -p pctrl-cli

# TUI mode
cargo run -p pctrl-cli -- --mode tui

# Show help
cargo run -p pctrl-cli -- --help

# Examples
cargo run -p pctrl-cli -- ssh list
cargo run -p pctrl-cli -- docker list <host-id>
cargo run -p pctrl-cli -- git list <repo-id>
```

#### Desktop GUI

```bash
cd apps/desktop
npm install
npm run tauri:dev
```

#### Landing Page

```bash
cd apps/landing
npm install
npm run dev
```

#### Mobile App

```bash
cd apps/mobile
npm install
npm start
```

## Usage

### SSH Connections

```bash
# List connections
pctrl ssh list

# Execute command
pctrl ssh exec <id> "ls -la"
```

### Docker Containers

```bash
# List containers
pctrl docker list <host-id>

# Start container
pctrl docker start <host-id> <container-id>

# Stop container
pctrl docker stop <host-id> <container-id>
```

### Coolify Deployments

```bash
# List deployments
pctrl coolify list <instance-id>

# Deploy project
pctrl coolify deploy <instance-id> <project-id>
```

### Git Releases

```bash
# List releases
pctrl git list <repo-id>

# Create release
pctrl git create <repo-id> <tag> <message>

# Push tags
pctrl git push <repo-id>
```

## Configuration

Configuration is stored in an encrypted SQLite database. The database path can be configured via environment variables or config file.

## Development

### Building

```bash
# Build all Rust crates
cargo build --release

# Build desktop app
cd apps/desktop
npm run build
npm run tauri:build

# Build landing page
cd apps/landing
npm run build
```

### Testing

```bash
# Run Rust tests
cargo test

# Run Node.js tests
npm test
```

## Roadmap

- [x] Core architecture
- [x] CLI interface with clap
- [x] TUI interface with ratatui
- [x] GUI with Tauri and React
- [x] Basic SSH, Docker, Coolify, Git support
- [ ] Advanced SSH connection management
- [ ] Real-time container monitoring
- [ ] Deployment pipelines
- [ ] Mobile app functionality
- [ ] Plugin system
- [ ] Cloud sync (optional)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details.

## Links

- [GitHub Repository](https://github.com/Degi100/pctrl)
- [Issue Tracker](https://github.com/Degi100/pctrl/issues)
- [Documentation](https://github.com/Degi100/pctrl/wiki)
