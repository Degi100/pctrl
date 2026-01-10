# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **TUI v6 Update**
  - Replaced legacy panels (SSH, Docker, Coolify, Git) with v6 entities
  - New panels: Projects, Servers, Domains, Databases, Scripts
  - Add forms for all v6 entity types
  - Removed legacy connection health checks

- **Desktop GUI v6 Update**
  - Tauri backend with v6 DTOs and Commands
  - React frontend with v6 entity management
  - Tabs: Projects, Servers, Domains, Databases, Scripts
  - Add forms for all v6 entity types
  - Color-coded status badges
  - Shared database with CLI/TUI

- **Database Schema Migrations**
  - Automatic schema versioning in metadata table
  - Auto-migration on database startup
  - `migrations.rs` module for managing schema changes
  - Migration v1→v2: Added `exit_code` and `last_output` columns to scripts table

- **Landing Page: Changelog**
  - New `/changelog` page displaying all releases
  - Fetches data from docs-api with fallback
  - Color-coded sections (Added, Changed, Fixed, etc.)
  - Stats overview (releases count, changes, latest version)

### Removed
- **Legacy Commands** - Completed Phase 2.5 cleanup:
  - `pctrl ssh` - Use `pctrl server` instead
  - `pctrl docker` - Use containers via project resources
  - `pctrl coolify` - Use project deployments
  - `pctrl git` - Use project resources
  - `pctrl migrate` - No longer needed
- **Legacy Database Tables**:
  - `ssh_connections`
  - `docker_hosts`
  - `coolify_instances`
  - `git_repos`
- **Legacy CLI Handlers**: Removed `handlers/legacy/` folder
- **Legacy TUI Panels**: Migration warning removed
- **Legacy Desktop Warning**: Migration banner removed

### Fixed
- **Desktop database path**: Desktop now uses `data_local_dir()` to match CLI/TUI
  - Previously used `data_dir()` which pointed to different folder on Windows
  - All apps now share `%LOCALAPPDATA%\pctrl\pctrl.db`

### Planned
- TUI detail views and item selection
- TUI edit/delete functionality
- Real-time container monitoring
- SSH password authentication

## [0.1.2] - 2025-01-06

### Added
- **Full CRUD Commands** for all entities
  - `pctrl ssh add/remove` - Manage SSH connections
  - `pctrl docker add/remove/hosts` - Manage Docker hosts
  - `pctrl coolify add/remove/instances` - Manage Coolify instances
  - `pctrl git add/remove/repos` - Manage Git repositories

- **Styled CLI Output**
  - ASCII banner with pctrl logo
  - Colored status display with indicators (● for active, ○ for empty)
  - Consistent formatting across all commands

- **Database Persistence**
  - Full load/save for all entity types (SSH, Docker, Coolify, Git)
  - Auto-creation of database directory and file
  - Duplicate ID detection with helpful error messages

- **TUI Improvements**
  - Fixed navigation on Windows (KeyEventKind filter)
  - Added ESC key to quit
  - Enter key handler prepared for future detail views

### Fixed
- Config loading now includes Docker hosts, Coolify instances, and Git repos
- Database URL now uses `?mode=rwc` for auto-create
- TUI no longer skips menu items on Windows due to key repeat

## [0.1.0] - 2025-01-06

### Added
- **Core Architecture**
  - Rust workspace with 6 modular crates (core, database, ssh, docker, coolify, git)
  - Monorepo structure with Turborepo for efficient builds
  - Comprehensive error handling with custom error types
  
- **CLI Interface**
  - Command-line interface using clap
  - Subcommands for SSH, Docker, Coolify, and Git management
  - Help text and version information
  - Mode selection (CLI, TUI, GUI)
  
- **TUI Interface**
  - Terminal user interface using ratatui
  - Keyboard navigation with arrow keys
  - Sidebar menu for feature selection
  - Empty state placeholders for all features
  
- **Database**
  - Encrypted SQLite database with AES-256-GCM
  - Argon2 key derivation for password-based encryption
  - Cryptographically secure random nonce generation
  - Database schema for SSH, Docker, Coolify, Git, Changelog, and Roadmap
  
- **SSH Management**
  - SSH connection configuration
  - Public key authentication support
  - Remote command execution
  - Connection listing
  
- **Docker Management**
  - Docker host configuration
  - Container listing via bollard
  - Container start/stop operations
  - Support for multiple Docker hosts
  
- **Coolify Integration**
  - Coolify instance configuration
  - Deployment listing via API
  - Project deployment trigger
  - API authentication with API keys
  
- **Git Management**
  - Git repository configuration
  - Release/tag listing using libgit2
  - Tag creation with messages
  - Tag pushing to remote
  
- **Desktop GUI**
  - Tauri application scaffold
  - React frontend with TypeScript
  - Vite build configuration
  - Tab-based navigation UI
  - Empty state placeholders
  
- **Landing Page**
  - Astro static site generator setup
  - Project homepage with feature showcase
  - Roadmap section
  - Changelog section
  - Responsive design
  
- **Mobile App**
  - Expo project structure
  - React Native components
  - Feature placeholders for all management types
  
- **Documentation**
  - Comprehensive README with usage examples
  - QUICKSTART guide for new users
  - ARCHITECTURE document explaining system design
  - CONTRIBUTING guidelines
  - IMPLEMENTATION_SUMMARY with project statistics
  - Example configuration file
  
- **CI/CD**
  - GitHub Actions workflow
  - Rust formatting and linting checks
  - Build verification for CLI
  - Landing page build process
  
- **Automation**
  - Website sync script for roadmap/changelog updates
  
- **Testing**
  - Unit tests for core library
  - Integration test structure
  - Test coverage for configuration and types

### Security
- AES-256-GCM encryption for sensitive data
- Argon2 password hashing
- Cryptographically secure random number generation
- Proper nonce handling (unique per encryption operation)
- Security warnings in documentation

### Changed
- N/A (initial release)

### Deprecated
- N/A (initial release)

### Removed
- N/A (initial release)

### Fixed
- N/A (initial release)

## Project Links

- [GitHub Repository](https://github.com/Degi100/pctrl)
- [Issue Tracker](https://github.com/Degi100/pctrl/issues)
- [Discussions](https://github.com/Degi100/pctrl/discussions)

[Unreleased]: https://github.com/Degi100/pctrl/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Degi100/pctrl/releases/tag/v0.1.0
