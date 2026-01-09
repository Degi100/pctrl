// Unified seed script that syncs all markdown files to the API
// Usage: bun run seed-all.ts
//
// This script:
// 1. Parses ROADMAP.md → /roadmap API
// 2. Parses CHANGELOG.md → /changelog API
// 3. Parses docs (README, QUICKSTART, ARCHITECTURE, CONTRIBUTING) → /docs API

import { readFileSync, existsSync } from 'fs';
import { resolve } from 'path';

const API_URL = process.env.API_URL || `http://localhost:${process.env.PORT || 3000}`;
const API_KEY = process.env.API_KEY;

if (!API_KEY) {
  console.error('Error: API_KEY environment variable is required');
  console.error('Set it in .env or run: API_KEY=your-key bun run seed-all.ts');
  process.exit(1);
}

const PROJECT_ROOT = resolve(__dirname, '../..');

// ============================================================================
// Types
// ============================================================================

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

type SectionName = 'planned' | 'added' | 'changed' | 'deprecated' | 'removed' | 'fixed' | 'security';

interface ChangelogEntry {
  version: string;
  date: string | null;
  order: number;
  sections: Partial<Record<SectionName, string[]>>;
}

interface Doc {
  slug: string;
  title: string;
  category: string;
  content: string;
  order: number;
}

// ============================================================================
// Roadmap Parser
// ============================================================================

function parseRoadmapMd(content: string): Phase[] {
  const phases: Phase[] = [];
  const lines = content.split('\n');

  let currentPhase: Phase | null = null;
  let currentCategory: Category | null = null;
  let inPhase = false;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // Match everything up to the [status] bracket
    const phaseMatch = line.match(/^## Phase (\d+): (.+) \[(\w+)\]/);
    if (phaseMatch) {
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

      let status: 'done' | 'current' | 'planned' = 'planned';
      let statusLabel = 'Planned';
      if (statusRaw === 'done') {
        status = 'done';
        statusLabel = 'Complete';
      } else if (statusRaw === 'current') {
        status = 'current';
        statusLabel = 'Current';
      }

      currentPhase = { phaseId, version: '', title, status, statusLabel, description: '' };
      currentCategory = null;
      inPhase = true;
      continue;
    }

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

    const goalMatch = line.match(/^\*\*Goal:\*\* (.+)/);
    if (goalMatch) {
      currentPhase.description = goalMatch[1].trim();
      continue;
    }

    const releaseMatch = line.match(/^\*\*(?:Target )?Release:\*\* (v[\d.]+)/);
    if (releaseMatch) {
      const versionParts = releaseMatch[1].match(/v(\d+)\.(\d+)/);
      currentPhase.version = versionParts ? `v${versionParts[1]}.${versionParts[2]}.x` : releaseMatch[1];
      continue;
    }

    const categoryMatch = line.match(/^### (.+)/);
    if (categoryMatch) {
      if (currentCategory && currentCategory.features.length > 0) {
        if (!currentPhase.categories) currentPhase.categories = [];
        currentPhase.categories.push(currentCategory);
      }
      currentCategory = { name: categoryMatch[1].trim(), features: [] };
      continue;
    }

    // Use flexible pattern because emojis have different unicode compositions
    const featureMatch = line.match(/^- .+? (.+)/);
    if (featureMatch && (line.includes('✅') || line.includes('📋'))) {
      const done = line.includes('✅');
      const name = featureMatch[1].trim();
      const feature: Feature = { name, done };

      if (currentCategory) {
        currentCategory.features.push(feature);
      } else {
        if (!currentPhase.features) currentPhase.features = [];
        currentPhase.features.push(feature);
      }
    }
  }

  if (currentPhase) {
    if (currentCategory && currentCategory.features.length > 0) {
      if (!currentPhase.categories) currentPhase.categories = [];
      currentPhase.categories.push(currentCategory);
    }
    phases.push(currentPhase);
  }

  return phases;
}

// ============================================================================
// Changelog Parser
// ============================================================================

function parseChangelogMd(content: string): ChangelogEntry[] {
  const entries: ChangelogEntry[] = [];
  const lines = content.split('\n');

  let currentEntry: ChangelogEntry | null = null;
  let currentSection: SectionName | null = null;
  let orderCounter = 1000;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    const versionMatch = line.match(/^## \[([^\]]+)\](?:\s*-\s*(\d{4}-\d{2}-\d{2}))?/);
    if (versionMatch) {
      if (currentEntry) entries.push(currentEntry);

      currentEntry = {
        version: versionMatch[1],
        date: versionMatch[2] || null,
        order: orderCounter--,
        sections: {},
      };
      currentSection = null;
      continue;
    }

    if (line.startsWith('## Project Links')) {
      if (currentEntry) {
        entries.push(currentEntry);
        currentEntry = null;
      }
      break;
    }

    if (!currentEntry) continue;

    const sectionMatch = line.match(/^### (\w+)/);
    if (sectionMatch) {
      const name = sectionMatch[1].toLowerCase() as SectionName;
      if (['planned', 'added', 'changed', 'deprecated', 'removed', 'fixed', 'security'].includes(name)) {
        currentSection = name;
        if (!currentEntry.sections[currentSection]) {
          currentEntry.sections[currentSection] = [];
        }
      } else {
        currentSection = null;
      }
      continue;
    }

    if (currentSection && line.match(/^-\s+.+/)) {
      let item = line.replace(/^-\s+/, '').trim();
      let j = i + 1;
      while (j < lines.length && lines[j].match(/^\s{2,}-\s+/)) {
        const nestedItem = lines[j].replace(/^\s+-\s+/, '').trim();
        item += ` | ${nestedItem}`;
        j++;
      }
      currentEntry.sections[currentSection]!.push(item);
    }
  }

  if (currentEntry) entries.push(currentEntry);
  return entries;
}

// ============================================================================
// Docs Config
// ============================================================================

const docConfigs = [
  { file: 'README.md', slug: 'overview', title: 'Overview', category: 'getting-started', order: 1 },
  { file: 'QUICKSTART.md', slug: 'quickstart', title: 'Quick Start', category: 'getting-started', order: 2 },
  { file: 'ARCHITECTURE.md', slug: 'architecture', title: 'Architecture', category: 'guides', order: 10 },
  { file: 'CONTRIBUTING.md', slug: 'contributing', title: 'Contributing', category: 'guides', order: 20 },
];

function buildDocs(): Doc[] {
  const docs: Doc[] = [];

  for (const config of docConfigs) {
    const filePath = resolve(PROJECT_ROOT, config.file);
    if (!existsSync(filePath)) {
      console.warn(`  Warning: ${config.file} not found`);
      continue;
    }

    const content = readFileSync(filePath, 'utf-8');
    const titleMatch = content.match(/^#\s+(.+)/m);

    docs.push({
      slug: config.slug,
      title: titleMatch ? titleMatch[1].trim() : config.title,
      category: config.category,
      content,
      order: config.order,
    });
  }

  return docs;
}

// ============================================================================
// API Functions
// ============================================================================

const headers = {
  'Content-Type': 'application/json',
  'Authorization': `Bearer ${API_KEY}`
};

async function upsertPhase(phase: Phase): Promise<'created' | 'updated' | 'error'> {
  const check = await fetch(`${API_URL}/roadmap/${phase.phaseId}`);
  if (check.ok) {
    const res = await fetch(`${API_URL}/roadmap/${phase.phaseId}`, {
      method: 'PUT', headers, body: JSON.stringify(phase)
    });
    return res.ok ? 'updated' : 'error';
  } else {
    const res = await fetch(`${API_URL}/roadmap`, {
      method: 'POST', headers, body: JSON.stringify(phase)
    });
    return res.ok ? 'created' : 'error';
  }
}

async function upsertChangelog(entry: ChangelogEntry): Promise<'created' | 'updated' | 'error'> {
  const check = await fetch(`${API_URL}/changelog/${encodeURIComponent(entry.version)}`);
  if (check.ok) {
    const res = await fetch(`${API_URL}/changelog/${encodeURIComponent(entry.version)}`, {
      method: 'PUT', headers, body: JSON.stringify(entry)
    });
    return res.ok ? 'updated' : 'error';
  } else {
    const res = await fetch(`${API_URL}/changelog`, {
      method: 'POST', headers, body: JSON.stringify(entry)
    });
    return res.ok ? 'created' : 'error';
  }
}

async function upsertDoc(doc: Doc): Promise<'created' | 'updated' | 'error'> {
  const check = await fetch(`${API_URL}/docs/${doc.slug}`);
  if (check.ok) {
    const res = await fetch(`${API_URL}/docs/${doc.slug}`, {
      method: 'PUT', headers, body: JSON.stringify(doc)
    });
    return res.ok ? 'updated' : 'error';
  } else {
    const res = await fetch(`${API_URL}/docs`, {
      method: 'POST', headers, body: JSON.stringify(doc)
    });
    return res.ok ? 'created' : 'error';
  }
}

// ============================================================================
// Main
// ============================================================================

async function seedAll() {
  console.log('='.repeat(60));
  console.log('  pctrl Seed All - Syncing markdown files to API');
  console.log('='.repeat(60));
  console.log(`\nAPI: ${API_URL}`);
  console.log(`Project root: ${PROJECT_ROOT}\n`);

  let totalCreated = 0, totalUpdated = 0, totalErrors = 0;

  // 1. Roadmap
  console.log('-'.repeat(60));
  console.log('ROADMAP.md -> /roadmap');
  console.log('-'.repeat(60));

  const roadmapPath = resolve(PROJECT_ROOT, 'ROADMAP.md');
  if (existsSync(roadmapPath)) {
    const phases = parseRoadmapMd(readFileSync(roadmapPath, 'utf-8'));
    console.log(`Parsed ${phases.length} phases\n`);

    for (const phase of phases) {
      const result = await upsertPhase(phase);
      if (result === 'created') { console.log(`+ Phase ${phase.phaseId}: ${phase.title}`); totalCreated++; }
      else if (result === 'updated') { console.log(`~ Phase ${phase.phaseId}: ${phase.title}`); totalUpdated++; }
      else { console.log(`x Phase ${phase.phaseId}: ERROR`); totalErrors++; }
    }
  } else {
    console.log('  ROADMAP.md not found');
  }

  // 2. Changelog
  console.log('\n' + '-'.repeat(60));
  console.log('CHANGELOG.md -> /changelog');
  console.log('-'.repeat(60));

  const changelogPath = resolve(PROJECT_ROOT, 'CHANGELOG.md');
  if (existsSync(changelogPath)) {
    const entries = parseChangelogMd(readFileSync(changelogPath, 'utf-8'));
    console.log(`Parsed ${entries.length} versions\n`);

    for (const entry of entries) {
      const result = await upsertChangelog(entry);
      const dateStr = entry.date || 'unreleased';
      if (result === 'created') { console.log(`+ ${entry.version} (${dateStr})`); totalCreated++; }
      else if (result === 'updated') { console.log(`~ ${entry.version} (${dateStr})`); totalUpdated++; }
      else { console.log(`x ${entry.version}: ERROR`); totalErrors++; }
    }
  } else {
    console.log('  CHANGELOG.md not found');
  }

  // 3. Docs
  console.log('\n' + '-'.repeat(60));
  console.log('Documentation -> /docs');
  console.log('-'.repeat(60));

  const docs = buildDocs();
  console.log(`Parsed ${docs.length} docs\n`);

  for (const doc of docs) {
    const result = await upsertDoc(doc);
    if (result === 'created') { console.log(`+ ${doc.title} (${doc.slug})`); totalCreated++; }
    else if (result === 'updated') { console.log(`~ ${doc.title} (${doc.slug})`); totalUpdated++; }
    else { console.log(`x ${doc.title}: ERROR`); totalErrors++; }
  }

  // Summary
  console.log('\n' + '='.repeat(60));
  console.log('  SUMMARY');
  console.log('='.repeat(60));
  console.log(`  Created: ${totalCreated}`);
  console.log(`  Updated: ${totalUpdated}`);
  console.log(`  Errors:  ${totalErrors}`);
  console.log('='.repeat(60));
}

seedAll();
