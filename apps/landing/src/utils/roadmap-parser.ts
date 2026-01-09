/**
 * Fetches roadmap data from the API
 * Data is stored in MongoDB and managed via the docs-api
 */

const API_URL = import.meta.env.PUBLIC_DOCS_API_URL || 'http://localhost:3000';

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

interface ApiPhase {
  phaseId: number;
  version: string;
  title: string;
  status: 'done' | 'current' | 'planned';
  statusLabel: string;
  description: string;
  features?: Feature[];
  categories?: Category[];
}

interface RoadmapResponse {
  phases: ApiPhase[];
  stats: {
    total: number;
    completed: number;
    phaseCount: number;
  };
}

/**
 * Fetch roadmap data from the API
 */
export async function parseRoadmap(): Promise<Phase[]> {
  try {
    const response = await fetch(`${API_URL}/roadmap`);
    if (!response.ok) {
      console.warn(`Roadmap API error: ${response.status}, using fallback`);
      return getFallbackPhases();
    }

    const data: RoadmapResponse = await response.json();

    // Transform API phases to match existing interface (phaseId -> id)
    return data.phases.map(phase => ({
      id: phase.phaseId,
      version: phase.version,
      title: phase.title,
      status: phase.status,
      statusLabel: phase.statusLabel,
      description: phase.description,
      features: phase.features,
      categories: phase.categories,
    }));
  } catch (error) {
    console.error('Failed to fetch roadmap:', error);
    return getFallbackPhases();
  }
}

/**
 * Fallback phases when API is not available
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
