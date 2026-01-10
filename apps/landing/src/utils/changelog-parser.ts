/**
 * Fetches changelog data from the API
 * Data is stored in MongoDB and managed via the docs-api
 */

const API_URL = import.meta.env.PUBLIC_DOCS_API_URL || 'http://localhost:3000';

export interface ChangelogSections {
  planned?: string[];
  added?: string[];
  changed?: string[];
  deprecated?: string[];
  removed?: string[];
  fixed?: string[];
  security?: string[];
}

export interface ChangelogEntry {
  version: string;
  date: string | null;
  order: number;
  sections: ChangelogSections;
}

interface ChangelogResponse {
  entries: ChangelogEntry[];
}

/**
 * Fetch changelog data from the API
 */
export async function parseChangelog(): Promise<ChangelogEntry[]> {
  try {
    const response = await fetch(`${API_URL}/changelog`);
    if (!response.ok) {
      console.warn(`Changelog API error: ${response.status}, using fallback`);
      return getFallbackChangelog();
    }

    const data: ChangelogResponse = await response.json();
    return data.entries;
  } catch (error) {
    console.error('Failed to fetch changelog:', error);
    return getFallbackChangelog();
  }
}

/**
 * Get the latest released version
 */
export async function getLatestVersion(): Promise<ChangelogEntry | null> {
  try {
    const response = await fetch(`${API_URL}/changelog/latest`);
    if (!response.ok) {
      return null;
    }

    const data = await response.json();
    return data.entry;
  } catch (error) {
    console.error('Failed to fetch latest version:', error);
    return null;
  }
}

/**
 * Count total items in a changelog entry
 */
export function countSections(entry: ChangelogEntry): number {
  const sections = entry.sections;
  if (!sections) return 0;
  return (
    (sections.planned?.length || 0) +
    (sections.added?.length || 0) +
    (sections.changed?.length || 0) +
    (sections.deprecated?.length || 0) +
    (sections.removed?.length || 0) +
    (sections.fixed?.length || 0) +
    (sections.security?.length || 0)
  );
}

/**
 * Fallback changelog when API is not available
 */
function getFallbackChangelog(): ChangelogEntry[] {
  return [
    {
      version: 'Unreleased',
      date: null,
      order: 999,
      sections: {
        added: [
          'Database Schema Migrations - Automatic schema versioning',
          'pctrl migrate command - Interactive migration from legacy to v6',
        ],
        deprecated: [
          'Legacy commands (pctrl ssh, docker, coolify, git) - use v6 commands instead',
        ],
        planned: [
          'TUI detail views and interactive actions',
          'Desktop GUI functionality',
          'Real-time container monitoring',
        ],
      },
    },
    {
      version: '0.1.2',
      date: '2025-01-06',
      order: 2,
      sections: {
        added: [
          'Full CRUD Commands for all entities',
          'Styled CLI Output with ASCII banner',
          'Database Persistence for all entity types',
          'TUI Improvements with fixed Windows navigation',
        ],
        fixed: [
          'Config loading now includes all entity types',
          'Database URL now uses ?mode=rwc for auto-create',
          'TUI no longer skips menu items on Windows',
        ],
      },
    },
    {
      version: '0.1.0',
      date: '2025-01-06',
      order: 1,
      sections: {
        added: [
          'Core Architecture - Rust workspace with 6 modular crates',
          'CLI Interface using clap',
          'TUI Interface using ratatui',
          'Encrypted SQLite database (AES-256-GCM)',
          'SSH Management with public key auth',
          'Docker Container Management',
          'Coolify API Integration',
          'Git Release Management',
          'Desktop GUI scaffold with Tauri + React',
          'Landing Page with Astro',
          'Mobile App scaffold with Expo',
          'CI/CD pipeline with GitHub Actions',
        ],
        security: [
          'AES-256-GCM encryption for sensitive data',
          'Argon2 password hashing',
          'Cryptographically secure random number generation',
        ],
      },
    },
  ];
}
