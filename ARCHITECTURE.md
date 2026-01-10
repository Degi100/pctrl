# Architecture

## Overview

pctrl (pilotCtrl) follows a **project-centric architecture** where projects are the core entity. All resources (servers, containers, databases, domains, scripts) are linked to projects, enabling a unified view of your infrastructure.

## Core Philosophy

> "Mission Control for Self-Hosters & Indie Devs"

- **Project-First**: Projects are the central organizing entity
- **Dual-View System**: Project View for daily work, Infrastructure View for admin tasks
- **Local-First**: All data stored locally with encryption
- **Auto-Discovery**: Automatic detection and mapping of resources (planned)

## Structure

```
pctrl/
├── apps/
│   ├── cli/              # CLI & TUI application (Rust)
│   │   ├── src/
│   │   │   ├── main.rs   # Entry point, command parsing
│   │   │   ├── cli.rs    # CLI command handlers
│   │   │   └── tui.rs    # TUI interface with Project View
│   │   └── Cargo.toml
│   │
│   ├── desktop/          # Desktop GUI (Tauri + React)
│   │   ├── src/          # React frontend
│   │   ├── src-tauri/    # Rust backend with Tauri commands
│   │   └── package.json
│   │
│   ├── landing/          # Project website (Astro)
│   │   ├── src/
│   │   │   ├── pages/
│   │   │   └── layouts/
│   │   └── package.json
│   │
│   └── mobile/           # Mobile app (Expo)
│       ├── App.tsx
│       └── package.json
│
├── crates/
│   ├── core/             # Core types and configuration
│   │   └── src/lib.rs    # Project, Server, Domain, Database, Script types
│   ├── database/         # Encrypted SQLite database
│   │   └── src/lib.rs    # CRUD for all entities + project_resources
│   ├── ssh/              # SSH connection management
│   ├── docker/           # Docker container management
│   ├── coolify/          # Coolify API client
│   └── git/              # Git operations
│
└── scripts/              # Automation scripts
    └── sync-website.sh   # Sync roadmap/changelog to website
```

## Data Model (v6)

### Core Entities

```
┌─────────────────────────────────────────────────────────────────┐
│                         PROJECTS                                 │
│  (Central organizing entity for all resources)                   │
│                                                                   │
│  - id, name, description, stack[], status, color, icon, notes   │
│  - Status: Dev | Staging | Live | Archived                       │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                    ┌───────────┴───────────┐
                    │   project_resources   │
                    │  (Many-to-Many Link)  │
                    └───────────┬───────────┘
                                │
    ┌───────────┬───────────┬───┴───┬───────────┬───────────┐
    ▼           ▼           ▼       ▼           ▼           ▼
┌───────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌───────┐ ┌───────┐
│Server │ │Container│ │Database │ │ Domain  │ │ Git   │ │Script │
│       │ │         │ │  Creds  │ │         │ │ Repo  │ │       │
└───────┘ └─────────┘ └─────────┘ └─────────┘ └───────┘ └───────┘
```

### Entity Definitions

| Entity | Description | Key Fields |
|--------|-------------|------------|
| **Project** | Central organizing unit | name, stack[], status, color |
| **Server** | VPS, dedicated, or local server | host, type, provider, ssh_connection_id |
| **Domain** | Domain with DNS records | name, type, server_id, ssl_status |
| **DatabaseCredentials** | Database connection info | type, host, port, username, password |
| **Container** | Docker container | name, image, server_id, status |
| **Script** | Automation scripts | name, type, content, project_id |
| **ProjectResource** | Links projects to resources | project_id, resource_type, resource_id, role |

## Component Interaction

```
┌─────────────────────────────────────────────────────────────────┐
│                        User Interfaces                           │
├──────────────┬──────────────┬──────────────┬────────────────────┤
│   CLI        │   TUI        │   GUI        │   Mobile           │
│   (clap)     │  (ratatui)   │  (Tauri)     │   (Expo)           │
│              │              │              │                    │
│  project     │  ┌────────┐  │  Dashboard   │   Quick Actions    │
│  server      │  │Projects│  │  Projects    │   Status View      │
│  domain      │  │  SSH   │  │  Resources   │                    │
│  db          │  │ Docker │  │              │                    │
│  script      │  │Coolify │  │              │                    │
│              │  │  Git   │  │              │                    │
│              │  └────────┘  │              │                    │
└──────────────┴──────────────┴──────────────┴────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Core Library                              │
│  Types: Project, Server, Domain, DatabaseCredentials, Script     │
│  Config, Error Handling, Serialization                           │
└─────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Database   │    │  Managers    │    │   External   │
│              │    │              │    │   Services   │
│  ┌────────┐  │    │  ┌────────┐  │    │  ┌────────┐  │
│  │ SQLite │  │    │  │  SSH   │  │    │  │  SSH   │  │
│  │Encrypted│ │    │  │ Docker │  │    │  │ Docker │  │
│  └────────┘  │    │  │Coolify │  │    │  │Coolify │  │
│              │    │  │  Git   │  │    │  │  Git   │  │
│  Tables:     │    │  └────────┘  │    └──────────────┘
│  - projects  │    └──────────────┘
│  - servers   │
│  - domains   │
│  - databases │
│  - containers│
│  - scripts   │
│  - project_  │
│    resources │
└──────────────┘
```

## Dual-View System

### Project View (Default)
- **Focus**: Daily development work
- **Shows**: Projects with linked resources
- **Use Case**: "Show me everything related to my SaaS app"

### Infrastructure View (Admin)
- **Focus**: Server/resource management
- **Shows**: All servers, containers, domains grouped by server
- **Use Case**: "Show me all containers running on prod-server"

## CLI Commands

### Project-Centric Commands (v6)
```bash
# Projects
pctrl project list                    # List all projects
pctrl project add <name>              # Add new project
pctrl project show <name>             # Show project with linked resources
pctrl project link <project> <type> <id>  # Link resource to project

# Servers
pctrl server list
pctrl server add <name> <host> [-t vps|dedicated|local]

# Domains
pctrl domain list
pctrl domain add <name> [-t root|subdomain|wildcard]

# Databases (with quick lookup)
pctrl db list
pctrl db add <name> -t postgres -H localhost -u admin
pctrl db get <name> user              # Quick lookup: get specific field

# Scripts
pctrl script list
pctrl script add <name> -t deploy|backup|health-check
```

## Architecture

**Project-Centric Design**:
```
           PROJECT
              │
    ┌─────────┼─────────┐
    ▼         ▼         ▼
  Server    Domain   Database
    │
    ├── SSH (integrated)
    ├── Containers (linked)
    └── Scripts (execute here)
```

## Data Flow

### CLI/TUI Mode

1. User runs command via CLI (e.g., `pctrl project show myapp`)
2. CLI parser (clap) processes arguments
3. Command handler queries database for project and linked resources
4. Results are formatted and displayed to user

### GUI Mode (Tauri)

1. User interacts with React frontend
2. Frontend calls Tauri commands
3. Rust backend queries database
4. Results are returned to frontend
5. React components update UI

### Data Persistence

1. All entities stored in encrypted SQLite
2. Database crate handles encryption/decryption (AES-256-GCM)
3. Project-resource links stored in `project_resources` table
4. Changes are persisted immediately

## Database Schema (v6)

```sql
-- Core project entity
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    stack TEXT,                    -- JSON array
    status TEXT DEFAULT 'dev',     -- dev|staging|live|archived
    color TEXT,
    icon TEXT,
    notes TEXT,
    created_at DATETIME,
    updated_at DATETIME
);

-- Server definitions
CREATE TABLE servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    server_type TEXT DEFAULT 'vps',
    provider TEXT,
    ssh_connection_id TEXT,
    location TEXT,
    specs TEXT,                    -- JSON object
    notes TEXT,
    FOREIGN KEY (ssh_connection_id) REFERENCES ssh_connections(id)
);

-- Many-to-many project-resource links
CREATE TABLE project_resources (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    resource_type TEXT NOT NULL,   -- server|container|database|domain|git|script
    resource_id TEXT NOT NULL,
    role TEXT,                     -- primary|backup|staging|etc
    notes TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

-- Additional tables: domains, databases, containers, scripts, discovery_cache
```

## Key Technologies

### Backend (Rust)
- **clap**: CLI parsing and command handling
- **ratatui**: Terminal UI framework
- **Tauri**: Desktop application framework
- **sqlx**: Async SQLite database access
- **ssh2**: SSH protocol implementation
- **bollard**: Docker API client
- **git2**: Git operations
- **reqwest**: HTTP client for Coolify API
- **aes-gcm**: Encryption
- **argon2**: Key derivation
- **uuid**: Unique identifiers
- **serde**: Serialization/deserialization

### Frontend
- **React**: UI library for desktop and web
- **React Native**: Mobile UI framework
- **Astro**: Static site generator
- **TypeScript**: Type-safe JavaScript
- **Vite**: Build tool

## Security

### Encryption
- Database encryption using AES-256-GCM
- Key derivation using Argon2
- Secure storage of credentials (passwords, API keys)
- Cryptographically secure random nonces

### Authentication
- SSH public key authentication
- API key storage in encrypted database
- No plaintext passwords in configuration

## Future: Auto-Discovery (Phase 3)

```
┌─────────────────────────────────────────────────────────────────┐
│                     Auto-Discovery Engine                        │
├─────────────────────────────────────────────────────────────────┤
│  DNS Lookup     → Find domains pointing to servers               │
│  Port Scanner   → Detect services (22, 80, 443, 5432, etc.)     │
│  Docker Inspect → List containers, extract env vars              │
│  Coolify API    → Sync projects and deployments                  │
│  Git Remote     → Link repos to projects                         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │ discovery_cache │
                    │  (suggestions)  │
                    └─────────────────┘
```

## Performance

### Optimization Strategies
- Async/await for I/O operations
- Connection pooling for database
- Lazy loading of resources
- Caching where appropriate
- Efficient project-resource queries

## Extensibility

### Plugin System (Planned)
- Dynamic loading of additional managers
- Custom command implementations
- Third-party integrations

## Testing Strategy

### Unit Tests
- Test individual functions and modules
- Mock external dependencies
- Run via `cargo test`

### Integration Tests
- Test component interactions
- Use test fixtures for data
- Verify end-to-end workflows

### Manual Testing
- CLI command validation
- TUI interaction testing
- GUI functionality verification

## Deployment

### CLI Distribution
- Cargo crates.io publication
- GitHub releases with binaries
- Package manager support (Homebrew, etc.)

### Desktop App
- Platform-specific installers (dmg, exe, deb)
- Auto-update functionality
- Code signing for security

### Landing Page
- Static site deployment (Vercel, Netlify)
- Auto-sync from database
- CDN for performance

### Mobile App
- App Store / Play Store distribution
- Over-the-air updates via Expo
- Push notification support
