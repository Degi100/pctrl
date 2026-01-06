# Roadmap

## Legend
- ✅ **[done]** - Completed and available
- 🚧 **[current]** - Currently in development
- 📋 **[planned]** - Planned for future releases

---

## Phase 1: Foundation ✅ [done]

**Goal:** Establish core architecture and basic functionality

- ✅ Monorepo structure with Rust workspace
- ✅ Core types and configuration system
- ✅ Encrypted SQLite database (AES-256-GCM)
- ✅ CLI interface with clap
- ✅ TUI interface with ratatui
- ✅ GUI scaffold with Tauri + React
- ✅ SSH connection management (public key auth)
- ✅ Docker container management (list, start, stop)
- ✅ Coolify API client integration
- ✅ Git release management (tags, push)
- ✅ Project documentation (README, QUICKSTART, ARCHITECTURE)
- ✅ CI/CD pipeline with GitHub Actions
- ✅ Unit tests for core library

**Release:** v0.1.0 (Initial Release)

---

## Phase 2: Enhanced Features 🚧 [current]

**Goal:** Improve usability and add advanced functionality

### CLI/TUI Improvements
- 📋 Interactive configuration wizard
- 📋 Colored output and progress indicators
- 📋 Shell completion scripts (bash, zsh, fish)
- 📋 Configuration file validation

### SSH Enhancements
- 📋 Password authentication support
- 📋 SSH agent integration
- 📋 Connection history and favorites
- 📋 Port forwarding management
- 📋 SFTP file transfer

### Docker Advanced Features
- 📋 Real-time container logs
- 📋 Container statistics (CPU, memory, network)
- 📋 Docker Compose support
- 📋 Image management (pull, push, remove)
- 📋 Volume management
- 📋 Network management

### Coolify Integration
- 📋 Deployment status monitoring
- 📋 Environment variable management
- 📋 Build logs viewing
- 📋 Service configuration updates

### Git Features
- 📋 Automatic changelog generation
- 📋 Release notes templating
- 📋 GitHub/GitLab release creation
- 📋 Multi-repository support

**Target Release:** v0.2.0

---

## Phase 3: Desktop & Mobile 📋 [planned]

**Goal:** Complete cross-platform experience

### Desktop GUI
- 📋 Complete React UI implementation
- 📋 Dashboard with overview widgets
- 📋 Real-time updates and notifications
- 📋 Theme support (light/dark mode)
- 📋 Multi-window support
- 📋 System tray integration
- 📋 Keyboard shortcuts

### Mobile App
- 📋 React Native implementation
- 📋 Push notifications
- 📋 Biometric authentication
- 📋 Offline mode with sync
- 📋 iOS app store release
- 📋 Android Play Store release

### Landing Page
- 📋 Auto-sync from database
- 📋 Blog/news section
- 📋 Documentation site integration
- 📋 Community showcase

**Target Release:** v0.3.0

---

## Phase 4: Advanced Management 📋 [planned]

**Goal:** Add monitoring, automation, and advanced features

### Monitoring & Alerts
- 📋 Real-time server monitoring
- 📋 Container health checks
- 📋 Resource usage alerts
- 📋 Custom alert rules
- 📋 Email/SMS/Slack notifications
- 📋 Metrics dashboard

### Automation
- 📋 Task scheduling (cron-like)
- 📋 Deployment pipelines
- 📋 Automated backups
- 📋 Script execution
- 📋 Webhooks support

### Security
- 📋 Two-factor authentication
- 📋 Role-based access control
- 📋 Audit logging
- 📋 Secret management integration (Vault, AWS Secrets Manager)
- 📋 Security scanning integration

**Target Release:** v0.4.0

---

## Phase 5: Extensibility 📋 [planned]

**Goal:** Plugin system and community integrations

### Plugin System
- 📋 Plugin API design
- 📋 Plugin discovery and installation
- 📋 Plugin marketplace
- 📋 Documentation for plugin developers
- 📋 Example plugins

### Third-party Integrations
- 📋 Kubernetes management
- 📋 AWS/Azure/GCP support
- 📋 Terraform integration
- 📋 Ansible integration
- 📋 Prometheus/Grafana
- 📋 CI/CD platforms (Jenkins, GitHub Actions, GitLab CI)

### Developer Tools
- 📋 REST API
- 📋 WebSocket API for real-time updates
- 📋 CLI plugin system
- 📋 SDK for other languages

**Target Release:** v0.5.0

---

## Phase 6: Enterprise Features 📋 [planned]

**Goal:** Team collaboration and enterprise-grade features

### Team Collaboration
- 📋 Multi-user support
- 📋 Team workspaces
- 📋 Shared configurations
- 📋 Activity feed
- 📋 Comments and annotations

### Cloud Sync (Optional)
- 📋 End-to-end encrypted cloud sync
- 📋 Backup and restore
- 📋 Device synchronization
- 📋 Conflict resolution

### Enterprise
- 📋 SSO integration (SAML, OAuth)
- 📋 Advanced audit logs
- 📋 Compliance reporting
- 📋 Custom branding
- 📋 Priority support

**Target Release:** v1.0.0

---

## Future Considerations

### AI & Automation
- 📋 Natural language command interface
- 📋 Intelligent resource optimization
- 📋 Anomaly detection
- 📋 Predictive maintenance

### Performance
- 📋 Performance optimization
- 📋 Caching improvements
- 📋 Parallel operations
- 📋 Database optimization

### Developer Experience
- 📋 Better error messages
- 📋 More comprehensive documentation
- 📋 Video tutorials
- 📋 Interactive playground

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

To suggest new features or changes to the roadmap:
1. Open a [GitHub Discussion](https://github.com/Degi100/pctrl/discussions)
2. Submit a feature request [issue](https://github.com/Degi100/pctrl/issues)
3. Join our community chat

---

## Release Schedule

- **v0.1.x** - Monthly maintenance releases
- **v0.2.0** - Q2 2026
- **v0.3.0** - Q3 2026
- **v0.4.0** - Q4 2026
- **v1.0.0** - 2027

*Note: This roadmap is subject to change based on community feedback and priorities.*
