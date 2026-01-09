// Seed script that parses ROADMAP.md and syncs to the API
// Usage: bun run seed-roadmap.ts [path-to-roadmap.md]

import { readFileSync } from 'fs';
import { resolve } from 'path';

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

/**
 * Parse ROADMAP.md and extract phase data
 */
function parseRoadmapMd(content: string): Phase[] {
  const phases: Phase[] = [];
  const lines = content.split('\n');

  let currentPhase: Phase | null = null;
  let currentCategory: Category | null = null;
  let inPhase = false;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // Match phase header: ## Phase N: Title STATUS [status-label]
    // Examples:
    //   ## Phase 1: Foundation ✅ [done]
    //   ## Phase 3: Auto-Discovery 🚧 [current]
    //   ## Phase 4: Infrastructure View 📋 [planned]
    // Match everything up to the [status] bracket, then extract title by removing trailing emoji
    const phaseMatch = line.match(/^## Phase (\d+): (.+) \[(\w+)\]/);
    if (phaseMatch) {
      // Save previous phase
      if (currentPhase) {
        if (currentCategory && currentCategory.features.length > 0) {
          if (!currentPhase.categories) currentPhase.categories = [];
          currentPhase.categories.push(currentCategory);
        }
        phases.push(currentPhase);
      }

      const phaseId = parseInt(phaseMatch[1]);
      // Remove trailing emoji from title (✅, 🚧, 📋)
      const title = phaseMatch[2].replace(/\s*[✅🚧📋].*/g, '').trim();
      const statusRaw = phaseMatch[3].toLowerCase();

      // Map status
      let status: 'done' | 'current' | 'planned' = 'planned';
      let statusLabel = 'Planned';
      if (statusRaw === 'done') {
        status = 'done';
        statusLabel = 'Complete';
      } else if (statusRaw === 'current') {
        status = 'current';
        statusLabel = 'Current';
      }

      currentPhase = {
        phaseId,
        version: '', // Will be filled from Release line
        title,
        status,
        statusLabel,
        description: '',
      };
      currentCategory = null;
      inPhase = true;
      continue;
    }

    // Stop parsing phases at certain sections
    if (line.startsWith('## Future Considerations') ||
        line.startsWith('## Contributing') ||
        line.startsWith('## Release Schedule')) {
      if (currentPhase) {
        if (currentCategory && currentCategory.features.length > 0) {
          if (!currentPhase.categories) currentPhase.categories = [];
          currentPhase.categories.push(currentCategory);
        }
        phases.push(currentPhase);
        currentPhase = null;
      }
      inPhase = false;
      continue;
    }

    if (!inPhase || !currentPhase) continue;

    // Match goal/description: **Goal:** Description text
    const goalMatch = line.match(/^\*\*Goal:\*\* (.+)/);
    if (goalMatch) {
      currentPhase.description = goalMatch[1].trim();
      continue;
    }

    // Match release version: **Release:** v0.1.0 or **Target Release:** v0.3.0
    const releaseMatch = line.match(/^\*\*(?:Target )?Release:\*\* (v[\d.]+)/);
    if (releaseMatch) {
      currentPhase.version = releaseMatch[1];
      // Convert v0.1.0 to v0.1.x format
      const versionParts = releaseMatch[1].match(/v(\d+)\.(\d+)/);
      if (versionParts) {
        currentPhase.version = `v${versionParts[1]}.${versionParts[2]}.x`;
      }
      continue;
    }

    // Match category header: ### Category Name
    const categoryMatch = line.match(/^### (.+)/);
    if (categoryMatch) {
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

    // Match feature: - ✅ Feature name (done) or - 📋 Feature name (planned)
    const featureMatch = line.match(/^- ([✅📋]) (.+)/);
    if (featureMatch) {
      const done = featureMatch[1] === '✅';
      const name = featureMatch[2].trim();

      const feature: Feature = { name, done };

      if (currentCategory) {
        currentCategory.features.push(feature);
      } else {
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

  return phases;
}

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
  // Get roadmap file path from args or use default
  const roadmapPath = process.argv[2] || resolve(__dirname, '../../ROADMAP.md');

  console.log(`Reading roadmap from: ${roadmapPath}\n`);

  let content: string;
  try {
    content = readFileSync(roadmapPath, 'utf-8');
  } catch (err) {
    console.error(`Error reading file: ${roadmapPath}`);
    console.error(err);
    process.exit(1);
  }

  const phases = parseRoadmapMd(content);

  console.log(`Parsed ${phases.length} phases from ROADMAP.md\n`);
  console.log('Syncing to API (upsert mode)...\n');

  let created = 0, updated = 0, errors = 0;

  for (const phase of phases) {
    try {
      const result = await upsertPhase(phase);

      if (result === 'created') {
        console.log(`+ Created: Phase ${phase.phaseId} - ${phase.title} (${phase.version})`);
        created++;
      } else if (result === 'updated') {
        console.log(`~ Updated: Phase ${phase.phaseId} - ${phase.title} (${phase.version})`);
        updated++;
      } else {
        console.log(`x Error: Phase ${phase.phaseId} - ${phase.title}`);
        errors++;
      }
    } catch (err) {
      console.log(`x Error: Phase ${phase.phaseId} - ${err}`);
      errors++;
    }
  }

  console.log(`\nDone! Created: ${created}, Updated: ${updated}, Errors: ${errors}\n`);

  // Fetch stats
  try {
    const statsResponse = await fetch(`${API_URL}/roadmap`);
    if (statsResponse.ok) {
      const data = await statsResponse.json();
      console.log(`Total phases: ${data.stats.phaseCount}`);
      console.log(`Total features: ${data.stats.total}`);
      console.log(`Completed: ${data.stats.completed}`);
      console.log(`Progress: ${Math.round((data.stats.completed / data.stats.total) * 100)}%`);
    }
  } catch {
    // Stats fetch failed, ignore
  }
}

seedRoadmap();
