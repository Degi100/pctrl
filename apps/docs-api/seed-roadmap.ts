// Seed script to populate roadmap phases
// Uses environment variables from .env file (Bun loads automatically)
const API_URL = process.env.API_URL || `http://localhost:${process.env.PORT || 3000}`;
const API_KEY = process.env.API_KEY;

if (!API_KEY) {
  console.error('Error: API_KEY environment variable is required');
  console.error('Set it in .env or run: API_KEY=your-key bun run seed-roadmap.ts');
  process.exit(1);
}

interface Feature {
  name: string;
  done: boolean;
}

interface Category {
  name: string;
  features: Feature[];
}

interface Phase {
  phaseId: number;
  version: string;
  title: string;
  status: 'done' | 'current' | 'planned';
  statusLabel: string;
  description: string;
  features?: Feature[];
  categories?: Category[];
}

const phases: Phase[] = [
  {
    phaseId: 1,
    version: 'v0.1.x',
    title: 'Foundation',
    status: 'done',
    statusLabel: 'Complete',
    description: 'Establish core architecture and basic functionality',
    features: [
      { name: 'Monorepo structure with Rust workspace', done: true },
      { name: 'Core types and configuration system', done: true },
      { name: 'Encrypted SQLite database (AES-256-GCM)', done: true },
      { name: 'CLI interface with clap', done: true },
      { name: 'TUI interface with ratatui', done: true },
      { name: 'GUI scaffold with Tauri + React', done: true },
      { name: 'SSH connection management (public key auth)', done: true },
      { name: 'Docker container management (list, start, stop)', done: true },
      { name: 'Coolify API client integration', done: true },
      { name: 'Git release management (tags, push)', done: true },
      { name: 'Project documentation (README, QUICKSTART, ARCHITECTURE)', done: true },
      { name: 'CI/CD pipeline with GitHub Actions', done: true },
      { name: 'Unit tests for core library', done: true },
    ],
  },
  {
    phaseId: 2,
    version: 'v0.2.x',
    title: 'Project Registry',
    status: 'done',
    statusLabel: 'Complete',
    description: 'Implement project-centric architecture (MASTERPLAN v6)',
    categories: [
      {
        name: 'Core Data Model',
        features: [
          { name: 'Project entity as central organizing unit', done: true },
          { name: 'Server entity with types (vps, dedicated, local, cloud)', done: true },
          { name: 'Server specs auto-detection via SSH (CPU, RAM, Disk)', done: true },
          { name: 'Domain entity with types (root, subdomain, wildcard)', done: true },
          { name: 'DatabaseCredentials entity with secure storage', done: true },
          { name: 'Container entity for Docker tracking', done: true },
          { name: 'Script entity for automation', done: true },
          { name: 'ProjectResource linking (many-to-many)', done: true },
        ],
      },
      {
        name: 'Database Schema',
        features: [
          { name: 'projects table', done: true },
          { name: 'servers table', done: true },
          { name: 'domains table', done: true },
          { name: 'databases table', done: true },
          { name: 'containers table', done: true },
          { name: 'scripts table', done: true },
          { name: 'project_resources table', done: true },
          { name: 'discovery_cache table (for Phase 3)', done: true },
        ],
      },
      {
        name: 'CLI Commands',
        features: [
          { name: 'pctrl project - list, add, show, remove, link, unlink', done: true },
          { name: 'pctrl server - list, add, show, remove', done: true },
          { name: 'pctrl domain - list, add, show, remove', done: true },
          { name: 'pctrl db - list, add, show, get, remove', done: true },
          { name: 'pctrl script - list, add, show, run, remove', done: true },
        ],
      },
      {
        name: 'TUI Enhancements',
        features: [
          { name: 'Projects panel in sidebar', done: true },
          { name: 'Project listing with status indicators', done: true },
          { name: 'Add project form', done: true },
          { name: 'Status-colored display (dev/staging/live/archived)', done: true },
        ],
      },
    ],
  },
  {
    phaseId: 3,
    version: 'v0.3.x',
    title: 'Auto-Discovery',
    status: 'current',
    statusLabel: 'Current',
    description: 'Automatic detection and mapping of resources',
    categories: [
      {
        name: 'Discovery Features',
        features: [
          { name: 'DNS lookup for domain verification', done: false },
          { name: 'Port scanning for service detection', done: false },
          { name: 'Docker container inspection', done: false },
          { name: 'Environment variable extraction', done: false },
          { name: 'Coolify project synchronization', done: false },
          { name: 'Git remote linking', done: false },
        ],
      },
      {
        name: 'Discovery Workflow',
        features: [
          { name: 'pctrl discover command', done: false },
          { name: 'Suggestion review interface', done: false },
          { name: 'Auto-link confirmed resources', done: false },
          { name: 'Discovery cache management', done: false },
        ],
      },
      {
        name: 'CLI/TUI Improvements',
        features: [
          { name: 'Interactive configuration wizard', done: false },
          { name: 'Colored output and progress indicators', done: true },
          { name: 'Shell completion scripts (bash, zsh, fish)', done: false },
          { name: 'Configuration file validation', done: false },
        ],
      },
    ],
  },
  {
    phaseId: 4,
    version: 'v0.4.x',
    title: 'Infrastructure View',
    status: 'planned',
    statusLabel: 'Planned',
    description: 'Server-centric management and monitoring',
    categories: [
      {
        name: 'Infrastructure Dashboard',
        features: [
          { name: 'Server-grouped resource view', done: false },
          { name: 'Real-time container status', done: false },
          { name: 'Resource usage metrics (CPU, memory, disk)', done: false },
          { name: 'Container logs viewing', done: false },
          { name: 'Health check indicators', done: false },
        ],
      },
      {
        name: 'SSH Enhancements',
        features: [
          { name: 'Password authentication support', done: false },
          { name: 'SSH agent integration', done: false },
          { name: 'Connection history and favorites', done: false },
          { name: 'Port forwarding management', done: false },
          { name: 'SFTP file transfer', done: false },
        ],
      },
      {
        name: 'Docker Advanced Features',
        features: [
          { name: 'Real-time container logs', done: false },
          { name: 'Container statistics (CPU, memory, network)', done: false },
          { name: 'Docker Compose support', done: false },
          { name: 'Image management (pull, push, remove)', done: false },
          { name: 'Volume and network management', done: false },
        ],
      },
      {
        name: 'Coolify Integration',
        features: [
          { name: 'Deployment status monitoring', done: false },
          { name: 'Environment variable management', done: false },
          { name: 'Build logs viewing', done: false },
          { name: 'Service configuration updates', done: false },
        ],
      },
    ],
  },
  {
    phaseId: 5,
    version: 'v0.5.x',
    title: 'Desktop & Mobile',
    status: 'planned',
    statusLabel: 'Planned',
    description: 'Complete cross-platform experience',
    categories: [
      {
        name: 'Desktop GUI (Tauri)',
        features: [
          { name: 'Tauri commands for all entities', done: false },
          { name: 'Complete React UI implementation', done: false },
          { name: 'Dashboard with project overview', done: false },
          { name: 'Real-time updates and notifications', done: false },
          { name: 'Theme support (light/dark mode)', done: false },
          { name: 'Multi-window support', done: false },
          { name: 'System tray integration', done: false },
          { name: 'Keyboard shortcuts', done: false },
        ],
      },
      {
        name: 'Mobile App (Expo)',
        features: [
          { name: 'React Native implementation', done: false },
          { name: 'Push notifications', done: false },
          { name: 'Biometric authentication', done: false },
          { name: 'Offline mode with sync', done: false },
          { name: 'iOS App Store release', done: false },
          { name: 'Android Play Store release', done: false },
        ],
      },
      {
        name: 'Landing Page',
        features: [
          { name: 'Auto-sync from ROADMAP.md', done: true },
          { name: 'Blog/news section', done: false },
          { name: 'Documentation site integration', done: false },
          { name: 'Community showcase', done: false },
        ],
      },
    ],
  },
  {
    phaseId: 6,
    version: 'v0.6.x',
    title: 'Automation & Scripts',
    status: 'planned',
    statusLabel: 'Planned',
    description: 'Script execution and automation features',
    categories: [
      {
        name: 'Script Execution',
        features: [
          { name: 'Run scripts via CLI (pctrl script run)', done: true },
          { name: 'Script output capture (stdout/stderr)', done: true },
          { name: 'Exit code handling (ScriptResult)', done: true },
          { name: 'Script variables and templating', done: false },
        ],
      },
      {
        name: 'Automation',
        features: [
          { name: 'Task scheduling (cron-like)', done: false },
          { name: 'Deployment pipelines', done: false },
          { name: 'Automated backups', done: false },
          { name: 'Webhooks support', done: false },
        ],
      },
      {
        name: 'Git Features',
        features: [
          { name: 'Automatic changelog generation', done: false },
          { name: 'Release notes templating', done: false },
          { name: 'GitHub/GitLab release creation', done: false },
          { name: 'Multi-repository support', done: false },
        ],
      },
    ],
  },
  {
    phaseId: 7,
    version: 'v0.7.x',
    title: 'Monitoring & Alerts',
    status: 'planned',
    statusLabel: 'Planned',
    description: 'Real-time monitoring and alerting',
    categories: [
      {
        name: 'Monitoring',
        features: [
          { name: 'Real-time server monitoring', done: false },
          { name: 'Container health checks', done: false },
          { name: 'Resource usage alerts', done: false },
          { name: 'Custom alert rules', done: false },
          { name: 'Metrics dashboard', done: false },
        ],
      },
      {
        name: 'Notifications',
        features: [
          { name: 'Email notifications', done: false },
          { name: 'SMS/Slack/Discord notifications', done: false },
          { name: 'Push notifications (mobile)', done: false },
          { name: 'Webhook integrations', done: false },
        ],
      },
      {
        name: 'Security',
        features: [
          { name: 'Two-factor authentication', done: false },
          { name: 'Role-based access control', done: false },
          { name: 'Audit logging', done: false },
          { name: 'Secret management integration', done: false },
        ],
      },
    ],
  },
  {
    phaseId: 8,
    version: 'v0.8.x',
    title: 'Extensibility',
    status: 'planned',
    statusLabel: 'Planned',
    description: 'Plugin system and third-party integrations',
    categories: [
      {
        name: 'Plugin System',
        features: [
          { name: 'Plugin API design', done: false },
          { name: 'Plugin discovery and installation', done: false },
          { name: 'Plugin marketplace', done: false },
          { name: 'Documentation for plugin developers', done: false },
          { name: 'Example plugins', done: false },
        ],
      },
      {
        name: 'Third-party Integrations',
        features: [
          { name: 'Kubernetes management', done: false },
          { name: 'AWS/Azure/GCP support', done: false },
          { name: 'Terraform integration', done: false },
          { name: 'Ansible integration', done: false },
          { name: 'Prometheus/Grafana', done: false },
          { name: 'CI/CD platforms (Jenkins, GitHub Actions, GitLab CI)', done: false },
        ],
      },
      {
        name: 'Developer Tools',
        features: [
          { name: 'REST API', done: false },
          { name: 'WebSocket API for real-time updates', done: false },
          { name: 'CLI plugin system', done: false },
          { name: 'SDK for other languages', done: false },
        ],
      },
    ],
  },
  {
    phaseId: 9,
    version: 'v1.0.0',
    title: 'Enterprise Features',
    status: 'planned',
    statusLabel: 'Planned',
    description: 'Team collaboration and enterprise-grade features',
    categories: [
      {
        name: 'Team Collaboration',
        features: [
          { name: 'Multi-user support', done: false },
          { name: 'Team workspaces', done: false },
          { name: 'Shared configurations', done: false },
          { name: 'Activity feed', done: false },
          { name: 'Comments and annotations', done: false },
        ],
      },
      {
        name: 'Cloud Sync (Optional)',
        features: [
          { name: 'End-to-end encrypted cloud sync', done: false },
          { name: 'Backup and restore', done: false },
          { name: 'Device synchronization', done: false },
          { name: 'Conflict resolution', done: false },
        ],
      },
      {
        name: 'Enterprise',
        features: [
          { name: 'SSO integration (SAML, OAuth)', done: false },
          { name: 'Advanced audit logs', done: false },
          { name: 'Compliance reporting', done: false },
          { name: 'Custom branding', done: false },
          { name: 'Priority support', done: false },
        ],
      },
    ],
  },
];

async function upsertPhase(phase: Phase): Promise<'created' | 'updated' | 'error'> {
  const headers = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${API_KEY}`
  };

  // Check if phase exists
  const checkResponse = await fetch(`${API_URL}/roadmap/${phase.phaseId}`);

  if (checkResponse.ok) {
    // Phase exists - update it
    const updateResponse = await fetch(`${API_URL}/roadmap/${phase.phaseId}`, {
      method: 'PUT',
      headers,
      body: JSON.stringify(phase)
    });
    return updateResponse.ok ? 'updated' : 'error';
  } else {
    // Phase doesn't exist - create it
    const createResponse = await fetch(`${API_URL}/roadmap`, {
      method: 'POST',
      headers,
      body: JSON.stringify(phase)
    });
    return createResponse.ok ? 'created' : 'error';
  }
}

async function seedRoadmap() {
  console.log('Seeding roadmap phases (upsert mode)...\n');

  let created = 0, updated = 0, errors = 0;

  for (const phase of phases) {
    try {
      const result = await upsertPhase(phase);

      if (result === 'created') {
        console.log(`+ Created: Phase ${phase.phaseId} - ${phase.title}`);
        created++;
      } else if (result === 'updated') {
        console.log(`~ Updated: Phase ${phase.phaseId} - ${phase.title}`);
        updated++;
      } else {
        console.log(`✗ Error: Phase ${phase.phaseId} - ${phase.title}`);
        errors++;
      }
    } catch (err) {
      console.log(`✗ Error: Phase ${phase.phaseId} - ${err}`);
      errors++;
    }
  }

  console.log(`\nDone! Created: ${created}, Updated: ${updated}, Errors: ${errors}\n`);

  // Fetch stats
  const statsResponse = await fetch(`${API_URL}/roadmap/stats`);
  if (statsResponse.ok) {
    const stats = await statsResponse.json();
    console.log(`Total phases: ${stats.phaseCount}`);
    console.log(`Total features: ${stats.total}`);
    console.log(`Completed: ${stats.completed}`);
    console.log(`Progress: ${Math.round((stats.completed / stats.total) * 100)}%`);
  }
}

seedRoadmap();
