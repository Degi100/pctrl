// Seed script to populate documentation
const API_URL = 'http://localhost:3000';
const API_KEY = process.env.API_KEY || 'pctrl-docs-api-2026-secure-key';

interface Doc {
  slug: string;
  title: string;
  category: string;
  content: string;
  order: number;
}

const docs: Doc[] = [
  {
    slug: 'installation',
    title: 'Installation',
    category: 'getting-started',
    order: 1,
    content: `# Installation

## Prerequisites

- **Rust** 1.70 or higher ([install](https://rustup.rs/))
- **Node.js** 18 or higher (for desktop/mobile apps)

## From Source

\`\`\`bash
# Clone the repository
git clone https://github.com/Degi100/pctrl.git
cd pctrl

# Install the CLI
cargo install --path apps/cli

# Verify installation
pctrl --version
\`\`\`

## Database Location

Your data is stored in an encrypted SQLite database:

| Platform | Location |
|----------|----------|
| Linux/macOS | \`~/.local/share/pctrl/pctrl.db\` |
| Windows | \`%LOCALAPPDATA%\\pctrl\\pctrl.db\` |

### Custom Database Path

\`\`\`bash
pctrl --db /custom/path/pctrl.db ssh list
\`\`\`

### Reset Database

\`\`\`bash
# Linux/macOS
rm ~/.local/share/pctrl/pctrl.db

# Windows
del %LOCALAPPDATA%\\pctrl\\pctrl.db
\`\`\`
`
  },
  {
    slug: 'quickstart',
    title: 'Quick Start',
    category: 'getting-started',
    order: 2,
    content: `# Quick Start

Get up and running with pctrl in minutes!

## First Run

\`\`\`bash
# Show the status dashboard
pctrl

# Show all available commands
pctrl --help
\`\`\`

You'll see the styled pctrl banner with your current configuration status.

## Three Modes

### CLI Mode (Default)

Fast, scriptable command-line interface:

\`\`\`bash
pctrl ssh list
pctrl docker hosts
pctrl --help
\`\`\`

### TUI Mode (Terminal Dashboard)

Beautiful terminal UI with keyboard navigation:

\`\`\`bash
pctrl -m tui

# Navigation:
# ↑/↓ or j/k  - Move between items
# Enter       - Select
# q or Esc    - Quit
\`\`\`

### GUI Mode (Desktop App)

Native desktop application:

\`\`\`bash
cd apps/desktop
npm install
npm run tauri dev
\`\`\`
`
  },
  {
    slug: 'ssh-commands',
    title: 'SSH Commands',
    category: 'commands',
    order: 3,
    content: `# SSH Commands

Manage SSH connections to your servers.

## Add a Connection

\`\`\`bash
# Basic usage
pctrl ssh add "Production" 10.0.0.1 -u root -p 22

# With custom SSH key
pctrl ssh add "Staging" 10.0.0.2 -u deploy -k ~/.ssh/staging_key
\`\`\`

## List Connections

\`\`\`bash
pctrl ssh list
\`\`\`

## Connect to Host

\`\`\`bash
pctrl ssh connect production
\`\`\`

## Execute Remote Command

\`\`\`bash
pctrl ssh exec production "hostname"
pctrl ssh exec production "ls -la /var/www"
\`\`\`

## Remove Connection

\`\`\`bash
pctrl ssh remove production
\`\`\`

## Command Reference

| Command | Description |
|---------|-------------|
| \`pctrl ssh list\` | List all connections |
| \`pctrl ssh add <name> <host> -u <user> [-p <port>] [-k <key>]\` | Add connection |
| \`pctrl ssh remove <id>\` | Remove connection |
| \`pctrl ssh connect <id>\` | Connect to host |
| \`pctrl ssh exec <id> "<command>"\` | Execute command |
`
  },
  {
    slug: 'docker-commands',
    title: 'Docker Commands',
    category: 'commands',
    order: 4,
    content: `# Docker Commands

Manage Docker hosts and containers.

## Add a Docker Host

\`\`\`bash
# Local Docker (default socket)
pctrl docker add "Local"

# Remote Docker host
pctrl docker add "Remote Server" -u tcp://10.0.0.1:2375
\`\`\`

## List Hosts

\`\`\`bash
pctrl docker hosts
\`\`\`

## List Containers

\`\`\`bash
pctrl docker list local
\`\`\`

## Start/Stop Containers

\`\`\`bash
pctrl docker start local container-id
pctrl docker stop local container-id
\`\`\`

## Remove Host

\`\`\`bash
pctrl docker remove local
\`\`\`

## Command Reference

| Command | Description |
|---------|-------------|
| \`pctrl docker hosts\` | List configured hosts |
| \`pctrl docker add <name> [-u <url>]\` | Add Docker host |
| \`pctrl docker remove <id>\` | Remove host |
| \`pctrl docker list <host-id>\` | List containers |
| \`pctrl docker start <host-id> <container-id>\` | Start container |
| \`pctrl docker stop <host-id> <container-id>\` | Stop container |
`
  },
  {
    slug: 'coolify-commands',
    title: 'Coolify Commands',
    category: 'commands',
    order: 5,
    content: `# Coolify Commands

Manage Coolify deployments.

## Add a Coolify Instance

\`\`\`bash
pctrl coolify add "Production" -u https://coolify.example.com -t your-api-token
\`\`\`

## List Instances

\`\`\`bash
pctrl coolify instances
\`\`\`

## List Deployments

\`\`\`bash
pctrl coolify list production
\`\`\`

## Deploy a Project

\`\`\`bash
pctrl coolify deploy production project-id
\`\`\`

## Remove Instance

\`\`\`bash
pctrl coolify remove production
\`\`\`

## Command Reference

| Command | Description |
|---------|-------------|
| \`pctrl coolify instances\` | List instances |
| \`pctrl coolify add <name> -u <url> -t <token>\` | Add instance |
| \`pctrl coolify remove <id>\` | Remove instance |
| \`pctrl coolify list <instance-id>\` | List deployments |
| \`pctrl coolify deploy <instance-id> <project-id>\` | Deploy project |
`
  },
  {
    slug: 'git-commands',
    title: 'Git Commands',
    category: 'commands',
    order: 6,
    content: `# Git Commands

Manage Git repositories and releases.

## Add a Repository

\`\`\`bash
pctrl git add "My Project" -p /path/to/repo
\`\`\`

## List Repositories

\`\`\`bash
pctrl git repos
\`\`\`

## List Releases/Tags

\`\`\`bash
pctrl git list my-project
\`\`\`

## Create a Release

\`\`\`bash
pctrl git create my-project v1.0.0 "First release"
\`\`\`

## Push Tags to Remote

\`\`\`bash
pctrl git push my-project
\`\`\`

## Remove Repository

\`\`\`bash
pctrl git remove my-project
\`\`\`

## Command Reference

| Command | Description |
|---------|-------------|
| \`pctrl git repos\` | List repositories |
| \`pctrl git add <name> -p <path>\` | Add repository |
| \`pctrl git remove <id>\` | Remove repository |
| \`pctrl git list <repo-id>\` | List releases/tags |
| \`pctrl git create <repo-id> <tag> <message>\` | Create release |
| \`pctrl git push <repo-id>\` | Push tags |
`
  },
  {
    slug: 'tui-guide',
    title: 'Terminal UI Guide',
    category: 'guides',
    order: 7,
    content: `# Terminal UI Guide

The TUI (Terminal User Interface) provides a beautiful dashboard for managing your resources.

## Starting the TUI

\`\`\`bash
pctrl -m tui
# or
pctrl tui
\`\`\`

## Navigation

| Key | Action |
|-----|--------|
| \`↑\` / \`k\` | Move up |
| \`↓\` / \`j\` | Move down |
| \`Enter\` | Select item |
| \`Tab\` | Switch panel |
| \`q\` / \`Esc\` | Quit |
| \`r\` | Refresh |
| \`?\` | Help |

## Panels

The TUI is divided into panels:

- **Sidebar** - Navigation menu (SSH, Docker, Coolify, Git, Projects)
- **Main** - List of items for selected category
- **Detail** - Details of selected item
- **Status** - Connection status indicators

## Features

- Real-time status updates
- Colored indicators (green = connected, red = error)
- Keyboard-driven navigation
- Quick actions on selected items
`
  },
  {
    slug: 'troubleshooting',
    title: 'Troubleshooting',
    category: 'guides',
    order: 8,
    content: `# Troubleshooting

Common issues and solutions.

## "pctrl: command not found"

Add Cargo bin to your PATH:

\`\`\`bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$PATH:$HOME/.cargo/bin"
\`\`\`

## SSH Connection Errors

1. Check your SSH key is loaded:
\`\`\`bash
ssh-add -l
ssh-add ~/.ssh/id_rsa
\`\`\`

2. Verify the connection works with standard SSH:
\`\`\`bash
ssh user@host
\`\`\`

3. Check the key file permissions:
\`\`\`bash
chmod 600 ~/.ssh/id_rsa
\`\`\`

## Docker Connection Errors

1. Check Docker is running:
\`\`\`bash
docker ps
\`\`\`

2. For remote hosts, ensure the Docker API is exposed

3. Verify the URL format:
   - Local: \`unix:///var/run/docker.sock\`
   - Remote: \`tcp://host:2375\`

## Database Locked

If you see "database locked" errors, ensure only one pctrl instance is running.

\`\`\`bash
# Kill any running instances
pkill pctrl
\`\`\`

## Coolify API Errors

1. Verify your API token is correct
2. Check the Coolify instance URL (must include https://)
3. Ensure your token has the required permissions

## Getting Help

- [GitHub Issues](https://github.com/Degi100/pctrl/issues)
- [Documentation](https://pctrl.dev/docs)
`
  },
  {
    slug: 'project-commands',
    title: 'Project Commands',
    category: 'commands',
    order: 9,
    content: `# Project Commands

Projects are the central organizing unit in pctrl. Link servers, databases, domains, and other resources to projects.

## Add a Project

\`\`\`bash
pctrl project add "My App" -s live
# Status options: dev, staging, live, archived
\`\`\`

## List Projects

\`\`\`bash
pctrl project list
\`\`\`

## Show Project Details

\`\`\`bash
pctrl project show my-app
\`\`\`

## Link Resources

\`\`\`bash
# Link a server to a project
pctrl project link my-app server production-server

# Link a database
pctrl project link my-app database postgres-main

# Link a domain
pctrl project link my-app domain example.com
\`\`\`

## Unlink Resources

\`\`\`bash
pctrl project unlink my-app server production-server
\`\`\`

## Remove Project

\`\`\`bash
pctrl project remove my-app
\`\`\`

## Command Reference

| Command | Description |
|---------|-------------|
| \`pctrl project list\` | List all projects |
| \`pctrl project add <name> -s <status>\` | Add project |
| \`pctrl project show <id>\` | Show project details |
| \`pctrl project remove <id>\` | Remove project |
| \`pctrl project link <project> <type> <resource>\` | Link resource |
| \`pctrl project unlink <project> <type> <resource>\` | Unlink resource |
`
  }
];

async function seedDocs() {
  console.log('Seeding documentation...\n');

  for (const doc of docs) {
    try {
      const response = await fetch(`${API_URL}/docs`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${API_KEY}`
        },
        body: JSON.stringify(doc)
      });

      if (response.ok) {
        const result = await response.json();
        console.log(`✓ Created: ${doc.title} (${doc.slug})`);
      } else {
        const error = await response.text();
        console.log(`✗ Failed: ${doc.title} - ${error}`);
      }
    } catch (err) {
      console.log(`✗ Error: ${doc.title} - ${err}`);
    }
  }

  console.log('\nDone! Checking results...\n');

  const listResponse = await fetch(`${API_URL}/docs`);
  const { docs: allDocs } = await listResponse.json();
  console.log(`Total docs in database: ${allDocs.length}`);

  const catResponse = await fetch(`${API_URL}/docs/categories`);
  const { categories } = await catResponse.json();
  console.log(`Categories: ${categories.join(', ')}`);
}

seedDocs();
