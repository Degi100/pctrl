/**
 * Parses ROADMAP.md and extracts phases with features
 * This enables auto-sync between ROADMAP.md and the landing page
 */

import fs from 'node:fs';
import path from 'node:path';

export interface Feature {
  name: string;
  done: boolean;
}

export interface Category {
  name: string;
  features: Feature[];
}

export interface Phase {
  id: number;
  version: string;
  title: string;
  status: 'done' | 'current' | 'planned';
  statusLabel: string;
  description: string;
  features?: Feature[];
  categories?: Category[];
}

/**
 * Parse the ROADMAP.md file and extract structured phase data
 */
export async function parseRoadmap(): Promise<Phase[]> {
  // Try to read ROADMAP.md from project root (works in monorepo dev)
  // Falls back to empty if not found (Docker build with isolated context)
  const possiblePaths = [
    path.resolve(process.cwd(), 'ROADMAP.md'),        // Docker build: copied to /app
    path.resolve(process.cwd(), '../../ROADMAP.md'),  // Monorepo dev: apps/landing -> root
    '/ROADMAP.md',                                     // Absolute fallback
  ];

  let content = '';
  for (const p of possiblePaths) {
    try {
      if (fs.existsSync(p)) {
        content = fs.readFileSync(p, 'utf-8');
        break;
      }
    } catch {
      // Continue to next path
    }
  }

  // Return empty array if no ROADMAP.md found
  if (!content) {
    console.warn('ROADMAP.md not found, using fallback data');
    return getFallbackPhases();
  }

  const phases: Phase[] = [];
  const lines = content.split('\n');

  let currentPhase: Phase | null = null;
  let currentCategory: Category | null = null;
  let phaseId = 0;

  for (const line of lines) {
    // Match phase headers: ## Phase 1: Foundation ✅ [done]
    const phaseMatch = line.match(/^## Phase (\d+): (.+?)\s*(✅|🚧|📋)\s*\[(\w+)\]/);
    if (phaseMatch) {
      // Save previous phase
      if (currentPhase) {
        if (currentCategory && currentPhase.categories) {
          // Don't add empty categories
          if (currentCategory.features.length > 0) {
            currentPhase.categories.push(currentCategory);
          }
        }
        phases.push(currentPhase);
      }

      phaseId++;
      const [, , title, emoji, statusText] = phaseMatch;

      let status: 'done' | 'current' | 'planned';
      let statusLabel: string;

      switch (emoji) {
        case '✅':
          status = 'done';
          statusLabel = 'Complete';
          break;
        case '🚧':
          status = 'current';
          statusLabel = 'Current';
          break;
        default:
          status = 'planned';
          statusLabel = 'Planned';
      }

      // Extract version from release schedule or use default
      const version = getVersionForPhase(phaseId);

      currentPhase = {
        id: phaseId,
        version,
        title: title.trim(),
        status,
        statusLabel,
        description: '',
        categories: [],
      };
      currentCategory = null;
      continue;
    }

    // Match goal/description: **Goal:** ...
    const goalMatch = line.match(/^\*\*Goal:\*\*\s*(.+)/);
    if (goalMatch && currentPhase) {
      currentPhase.description = goalMatch[1].trim();
      continue;
    }

    // Match category headers: ### Category Name
    const categoryMatch = line.match(/^### (.+)/);
    if (categoryMatch && currentPhase) {
      // Save previous category
      if (currentCategory && currentCategory.features.length > 0) {
        if (!currentPhase.categories) currentPhase.categories = [];
        currentPhase.categories.push(currentCategory);
      }

      currentCategory = {
        name: categoryMatch[1].trim(),
        features: [],
      };
      continue;
    }

    // Match features: - ✅ Feature name or - 📋 Feature name
    const featureMatch = line.match(/^- (✅|📋|🚧)\s+(.+)/);
    if (featureMatch && currentPhase) {
      const [, emoji, name] = featureMatch;
      const feature: Feature = {
        name: name.trim(),
        done: emoji === '✅',
      };

      if (currentCategory) {
        currentCategory.features.push(feature);
      } else {
        // Feature without category - add directly to phase
        if (!currentPhase.features) currentPhase.features = [];
        currentPhase.features.push(feature);
      }
      continue;
    }
  }

  // Don't forget the last phase
  if (currentPhase) {
    if (currentCategory && currentCategory.features.length > 0) {
      if (!currentPhase.categories) currentPhase.categories = [];
      currentPhase.categories.push(currentCategory);
    }
    phases.push(currentPhase);
  }

  // Clean up: remove empty categories arrays, convert features-only phases
  for (const phase of phases) {
    if (phase.categories && phase.categories.length === 0) {
      delete phase.categories;
    }
    if (phase.features && phase.features.length === 0) {
      delete phase.features;
    }
  }

  return phases;
}

/**
 * Get version string for a phase number
 */
function getVersionForPhase(phaseId: number): string {
  const versions: Record<number, string> = {
    1: 'v0.1.x',
    2: 'v0.2.x',
    3: 'v0.3.x',
    4: 'v0.4.x',
    5: 'v0.5.x',
    6: 'v0.6.x',
    7: 'v0.7.x',
    8: 'v0.8.x',
    9: 'v1.0.0',
  };
  return versions[phaseId] || `v0.${phaseId}.x`;
}

/**
 * Fallback phases when ROADMAP.md is not available
 */
function getFallbackPhases(): Phase[] {
  return [
    {
      id: 1,
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
        { name: 'Project documentation', done: true },
        { name: 'CI/CD pipeline with GitHub Actions', done: true },
        { name: 'Unit tests for core library', done: true },
      ],
    },
    {
      id: 2,
      version: 'v0.2.x',
      title: 'Project Registry',
      status: 'done',
      statusLabel: 'Complete',
      description: 'Implement project-centric architecture (MASTERPLAN v6)',
      features: [
        { name: 'Project entity as central organizing unit', done: true },
        { name: 'Server entity with types', done: true },
        { name: 'Server specs auto-detection via SSH', done: true },
        { name: 'Domain entity with types', done: true },
        { name: 'DatabaseCredentials entity', done: true },
        { name: 'Container entity for Docker tracking', done: true },
        { name: 'Script entity for automation', done: true },
        { name: 'ProjectResource linking', done: true },
        { name: 'Database schema (all tables)', done: true },
        { name: 'CLI Commands (project, server, domain, db, script)', done: true },
        { name: 'TUI Enhancements', done: true },
      ],
    },
    {
      id: 3,
      version: 'v0.3.x',
      title: 'Auto-Discovery',
      status: 'current',
      statusLabel: 'Current',
      description: 'Automatic detection and mapping of resources',
      features: [
        { name: 'DNS lookup for domain verification', done: false },
        { name: 'Port scanning for service detection', done: false },
        { name: 'Docker container inspection', done: false },
        { name: 'Coolify project synchronization', done: false },
        { name: 'pctrl discover command', done: false },
        { name: 'Interactive configuration wizard', done: false },
        { name: 'Colored output and progress indicators', done: true },
        { name: 'Shell completion scripts', done: false },
      ],
    },
    {
      id: 4,
      version: 'v0.4.x',
      title: 'Monitoring & Health',
      status: 'planned',
      statusLabel: 'Planned',
      description: 'Resource monitoring and health checks',
      features: [
        { name: 'Server health monitoring', done: false },
        { name: 'Docker container health checks', done: false },
        { name: 'Service uptime tracking', done: false },
        { name: 'Alert system', done: false },
      ],
    },
    {
      id: 5,
      version: 'v0.5.x',
      title: 'Backup & Restore',
      status: 'planned',
      statusLabel: 'Planned',
      description: 'Backup management and disaster recovery',
      features: [
        { name: 'Database backup automation', done: false },
        { name: 'Configuration export/import', done: false },
        { name: 'Restore workflows', done: false },
      ],
    },
    {
      id: 6,
      version: 'v1.0.0',
      title: 'Production Ready',
      status: 'planned',
      statusLabel: 'Planned',
      description: 'Stable release with full feature set',
      features: [
        { name: 'Complete GUI application', done: false },
        { name: 'Mobile companion app', done: false },
        { name: 'Full documentation', done: false },
        { name: 'Plugin system', done: false },
      ],
    },
  ];
}

/**
 * Calculate statistics from phases
 */
export function calculateStats(phases: Phase[]): { total: number; completed: number } {
  let total = 0;
  let completed = 0;

  for (const phase of phases) {
    if (phase.features) {
      total += phase.features.length;
      completed += phase.features.filter(f => f.done).length;
    }
    if (phase.categories) {
      for (const cat of phase.categories) {
        total += cat.features.length;
        completed += cat.features.filter(f => f.done).length;
      }
    }
  }

  return { total, completed };
}
