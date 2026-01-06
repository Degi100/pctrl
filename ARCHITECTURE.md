# Architecture

## Overview

pctrl follows a modular monorepo architecture with clear separation between the core functionality and various user interfaces.

## Structure

```
pctrl/
├── apps/
│   ├── cli/              # CLI & TUI application (Rust)
│   │   ├── src/
│   │   │   ├── main.rs   # Entry point, mode selection
│   │   │   ├── cli.rs    # CLI command handlers
│   │   │   └── tui.rs    # TUI interface
│   │   └── Cargo.toml
│   │
│   ├── desktop/          # Desktop GUI (Tauri + React)
│   │   ├── src/          # React frontend
│   │   ├── src-tauri/    # Rust backend
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
│   ├── database/         # Encrypted SQLite database
│   ├── ssh/              # SSH connection management
│   ├── docker/           # Docker container management
│   ├── coolify/          # Coolify API client
│   └── git/              # Git operations
│
└── scripts/              # Automation scripts
    └── sync-website.sh   # Sync roadmap/changelog to website
```

## Component Interaction

```
┌─────────────────────────────────────────────────────────────┐
│                        User Interfaces                       │
├──────────────┬──────────────┬──────────────┬────────────────┤
│   CLI        │   TUI        │   GUI        │   Mobile       │
│   (clap)     │  (ratatui)   │  (Tauri)     │   (Expo)       │
└──────────────┴──────────────┴──────────────┴────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        Core Library                          │
│              (Types, Config, Error Handling)                 │
└─────────────────────────────────────────────────────────────┘
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
└──────────────┘    │  └────────┘  │    └──────────────┘
                    └──────────────┘
```

## Data Flow

### CLI/TUI Mode

1. User runs command via CLI
2. CLI parser (clap) processes arguments
3. Command handler invokes appropriate manager
4. Manager performs operation (SSH, Docker, etc.)
5. Results are formatted and displayed to user

### GUI Mode (Tauri)

1. User interacts with React frontend
2. Frontend calls Tauri commands
3. Rust backend invokes appropriate manager
4. Results are returned to frontend
5. React components update UI

### Data Persistence

1. Application state is stored in encrypted SQLite
2. Database crate handles encryption/decryption
3. Configuration is loaded on startup
4. Changes are persisted immediately

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
- Secure storage of credentials

### Authentication
- SSH public key authentication
- API key storage in encrypted database
- No plaintext passwords in configuration

## Performance

### Optimization Strategies
- Async/await for I/O operations
- Connection pooling for database
- Lazy loading of resources
- Caching where appropriate

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

## Future Enhancements

1. **Plugin System**: Allow third-party extensions
2. **Cloud Sync**: Optional cloud backup (encrypted)
3. **Team Features**: Multi-user support
4. **Advanced Monitoring**: Real-time metrics and alerts
5. **AI Assistant**: Natural language command interface
6. **Multi-platform**: iOS, Android native apps
