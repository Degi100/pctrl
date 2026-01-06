# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure and documentation

## [0.1.0] - 2026-01-06

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
