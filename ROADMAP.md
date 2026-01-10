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

## Phase 2: Project Registry ✅ [done]

**Goal:** Implement project-centric architecture (MASTERPLAN v6)

### Core Data Model
- ✅ Project entity as central organizing unit
- ✅ Server entity with types (vps, dedicated, local, cloud)
- ✅ Server specs auto-detection via SSH (CPU, RAM, Disk)
- ✅ Domain entity with types (root, subdomain, wildcard)
- ✅ DatabaseCredentials entity with secure storage
- ✅ Container entity for Docker tracking
- ✅ Script entity for automation
- ✅ ProjectResource linking (many-to-many)

### Database Schema
- ✅ projects table
- ✅ servers table
- ✅ domains table
- ✅ databases table
- ✅ containers table
- ✅ scripts table
- ✅ project_resources table
- ✅ discovery_cache table (for Phase 3)

### CLI Commands
- ✅ `pctrl project` - list, add, show, remove, link, unlink
- ✅ `pctrl server` - list, add, show, remove
- ✅ `pctrl domain` - list, add, show, remove
- ✅ `pctrl db` - list, add, show, get, remove
- ✅ `pctrl script` - list, add, show, run, remove

### TUI Enhancements
- ✅ Projects panel in sidebar
- ✅ Project listing with status indicators
- ✅ Add project form
- ✅ Status-colored display (dev/staging/live/archived)

**Release:** v0.2.0

---

## Phase 2.5: Legacy Migration ✅ [done]

**Goal:** Migrate from standalone commands to project-centric architecture

### Migration Tasks
- ✅ Add deprecation warnings to legacy commands
- ✅ Create `pctrl migrate` command for automatic data migration
- ✅ Update TUI to use v6 entities (Projects, Servers, Domains, etc.)
- ✅ Update Tauri desktop with v6 commands
- ✅ Remove legacy code paths

### Legacy Removed

| Removed | Replacement |
|---------|-------------|
| `pctrl ssh` | `pctrl server` (with SSH reference field) |
| `pctrl docker` | `pctrl server` + project resources |
| `pctrl coolify` | Project deployment layer (Phase 4) |
| `pctrl git` | Project git linking (Phase 4) |
| `pctrl migrate` | No longer needed |

### Database Cleanup
- ✅ Dropped `ssh_connections` table
- ✅ Dropped `docker_hosts` table
- ✅ Dropped `coolify_instances` table
- ✅ Dropped `git_repos` table

---

## Phase 3: Credentials & SSH 🚧 [current]

**Goal:** Secure credential management and SSH integration

### Credential System
- ✅ Credential entity (SshKey, SshAgent, ApiToken, BasicAuth, OAuth)
- ✅ CLI: `credential add/list/show/remove` commands
- ✅ Desktop: Credentials tab with full CRUD
- ✅ Desktop: SSH key generation (RSA-4096)
- ✅ Desktop: Test connection feature
- ✅ Desktop: Clipboard copy for public keys

### SSH Integration
- ✅ Server links to credentials
- ✅ SSH Agent authentication (ED25519 support)
- ✅ SSH Key authentication (RSA recommended)
- ✅ CLI: `server status` - Live stats via SSH
- ✅ CLI: `server exec` - Remote command execution
- ✅ Desktop: Server status button
- ✅ Auto-detect server specs on add (CPU, RAM, Disk)

### UX Improvements
- ✅ German tooltips and hints
- ✅ Auto-fill defaults (username, port, key path)
- ✅ File browser for SSH keys
- ✅ Colored output and progress indicators

**Target Release:** v0.3.0

---

## Phase 3.5: Auto-Discovery 📋 [next]

**Goal:** Automatic detection and mapping of resources

### Discovery Features
- 📋 DNS lookup for domain verification
- 📋 Port scanning for service detection
- 📋 Docker container inspection
- 📋 Environment variable extraction
- 📋 Coolify project synchronization
- 📋 Git remote linking

### Discovery Workflow
- 📋 `pctrl discover` command
- 📋 Suggestion review interface
- 📋 Auto-link confirmed resources
- 📋 Discovery cache management

### CLI/TUI Improvements
- 📋 Interactive configuration wizard
- 📋 Shell completion scripts (bash, zsh, fish)
- 📋 Configuration file validation

**Target Release:** v0.3.5

---

## Phase 4: Infrastructure View 📋 [planned]

**Goal:** Server-centric management and monitoring

### Infrastructure Dashboard
- 📋 Server-grouped resource view
- 📋 Real-time container status
- 📋 Resource usage metrics (CPU, memory, disk)
- 📋 Container logs viewing
- 📋 Health check indicators

### SSH Enhancements
- ✅ SSH agent integration (moved from Phase 3)
- 📋 Password authentication support
- 📋 Connection history and favorites
- 📋 Port forwarding management
- 📋 SFTP file transfer

### Docker Advanced Features
- 📋 Real-time container logs
- 📋 Container statistics (CPU, memory, network)
- 📋 Docker Compose support
- 📋 Image management (pull, push, remove)
- 📋 Volume and network management

### Coolify Integration
- 📋 Deployment status monitoring
- 📋 Trigger deployments via API (`pctrl coolify deploy`)
- 📋 Wait for deployment completion with status polling
- 📋 Environment variable management
- 📋 Build logs viewing
- 📋 Service configuration updates

**Target Release:** v0.4.0

---

## Phase 5: Desktop & Mobile 📋 [planned]

**Goal:** Complete cross-platform experience

### Desktop GUI (Tauri)
- 📋 Tauri commands for all entities
- 📋 Complete React UI implementation
- 📋 Dashboard with project overview
- 📋 Real-time updates and notifications
- 📋 Theme support (light/dark mode)
- 📋 Multi-window support
- 📋 System tray integration
- 📋 Keyboard shortcuts

### Mobile App (Expo)
- 📋 React Native implementation
- 📋 Push notifications
- 📋 Biometric authentication
- 📋 Offline mode with sync
- 📋 iOS App Store release
- 📋 Android Play Store release

### Landing Page
- ✅ Auto-sync from ROADMAP.md
- 📋 Blog/news section
- 📋 Documentation site integration
- 📋 Community showcase

**Target Release:** v0.5.0

---

## Phase 6: Automation & Scripts 📋 [planned]

**Goal:** Script execution and automation features

### Script Execution
- ✅ Run scripts via CLI (`pctrl script run`)
- ✅ Script output capture (stdout/stderr)
- ✅ Exit code handling (ScriptResult)
- 📋 Script variables and templating

### Automation
- 📋 Task scheduling (cron-like)
- 📋 Deployment pipelines
- 📋 Automated backups
- 📋 Webhooks support

### Git Features
- 📋 Automatic changelog generation
- 📋 Release notes templating
- 📋 GitHub/GitLab release creation
- 📋 Multi-repository support

**Target Release:** v0.6.0

---

## Phase 7: Monitoring & Alerts 📋 [planned]

**Goal:** Real-time monitoring and alerting

### Monitoring
- 📋 Real-time server monitoring
- 📋 Container health checks
- 📋 Resource usage alerts
- 📋 Custom alert rules
- 📋 Metrics dashboard

### Notifications
- 📋 Email notifications
- 📋 SMS/Slack/Discord notifications
- 📋 Push notifications (mobile)
- 📋 Webhook integrations

### Security
- 📋 Two-factor authentication
- 📋 Role-based access control
- 📋 Audit logging
- 📋 Secret management integration

**Target Release:** v0.7.0

---

## Phase 8: Extensibility 📋 [planned]

**Goal:** Plugin system and third-party integrations

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

**Target Release:** v0.8.0

---

## Phase 9: Enterprise Features 📋 [planned]

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

- **v0.1.x** - Foundation (completed)
- **v0.2.0** - Project Registry (completed)
- **v0.2.5** - Legacy Migration (completed)
- **v0.3.0** - Auto-Discovery (Q1 2026) ← next
- **v0.4.0** - Infrastructure View (Q2 2026)
- **v0.5.0** - Desktop & Mobile (Q3 2026)
- **v1.0.0** - Enterprise (2026/2027)

*Note: This roadmap is subject to change based on community feedback and priorities.*
