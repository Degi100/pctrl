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
    path.resolve(process.cwd(), '../../ROADMAP.md'),  // Monorepo: apps/landing -> root
    path.resolve(process.cwd(), 'ROADMAP.md'),        // If copied to landing
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
      description: 'Core CLI with database and basic commands',
    },
    {
      id: 2,
      version: 'v0.2.x',
      title: 'Expansion',
      status: 'current',
      statusLabel: 'Current',
      description: 'Extended functionality and integrations',
    },
    {
      id: 3,
      version: 'v0.3.x',
      title: 'Automation',
      status: 'planned',
      statusLabel: 'Planned',
      description: 'Scripts, scheduling, and workflows',
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
