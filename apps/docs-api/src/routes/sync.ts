// Sync route - fetches markdown from GitHub and seeds the database
// POST /webhook/sync - triggers a full sync from GitHub

import { Hono } from 'hono';
import { getDb } from '../db';

const sync = new Hono();

// GitHub raw URLs
const GITHUB_RAW_BASE = 'https://raw.githubusercontent.com/Degi100/pctrl/main';
const GITHUB_FILES = {
  roadmap: `${GITHUB_RAW_BASE}/ROADMAP.md`,
  changelog: `${GITHUB_RAW_BASE}/CHANGELOG.md`,
  readme: `${GITHUB_RAW_BASE}/README.md`,
  quickstart: `${GITHUB_RAW_BASE}/QUICKSTART.md`,
  architecture: `${GITHUB_RAW_BASE}/ARCHITECTURE.md`,
  contributing: `${GITHUB_RAW_BASE}/CONTRIBUTING.md`,
};

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
// Parsers (same as seed-all.ts)
// ============================================================================

function parseRoadmapMd(content: string): Phase[] {
  const phases: Phase[] = [];
  const lines = content.split('\n');

  let currentPhase: Phase | null = null;
  let currentCategory: Category | null = null;
  let inPhase = false;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

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
// Sync Endpoint
// ============================================================================

sync.post('/', async (c) => {
  const startTime = Date.now();
  const results = {
    roadmap: { phases: 0, created: 0, updated: 0, errors: 0 },
    changelog: { entries: 0, created: 0, updated: 0, errors: 0 },
    docs: { docs: 0, created: 0, updated: 0, errors: 0 },
  };

  try {
    const db = getDb();

    // 1. Sync Roadmap
    console.log('[Sync] Fetching ROADMAP.md from GitHub...');
    const roadmapRes = await fetch(GITHUB_FILES.roadmap);
    if (roadmapRes.ok) {
      const roadmapContent = await roadmapRes.text();
      const phases = parseRoadmapMd(roadmapContent);
      results.roadmap.phases = phases.length;

      for (const phase of phases) {
        try {
          const existing = await db.collection('roadmap').findOne({ phaseId: phase.phaseId });
          if (existing) {
            await db.collection('roadmap').updateOne(
              { phaseId: phase.phaseId },
              { $set: { ...phase, updatedAt: new Date() } }
            );
            results.roadmap.updated++;
          } else {
            await db.collection('roadmap').insertOne({
              ...phase,
              createdAt: new Date(),
              updatedAt: new Date(),
            });
            results.roadmap.created++;
          }
        } catch (err) {
          console.error(`[Sync] Error upserting phase ${phase.phaseId}:`, err);
          results.roadmap.errors++;
        }
      }
    }

    // 2. Sync Changelog
    console.log('[Sync] Fetching CHANGELOG.md from GitHub...');
    const changelogRes = await fetch(GITHUB_FILES.changelog);
    if (changelogRes.ok) {
      const changelogContent = await changelogRes.text();
      const entries = parseChangelogMd(changelogContent);
      results.changelog.entries = entries.length;

      for (const entry of entries) {
        try {
          const existing = await db.collection('changelog').findOne({ version: entry.version });
          if (existing) {
            await db.collection('changelog').updateOne(
              { version: entry.version },
              { $set: { ...entry, updatedAt: new Date() } }
            );
            results.changelog.updated++;
          } else {
            await db.collection('changelog').insertOne({
              ...entry,
              createdAt: new Date(),
              updatedAt: new Date(),
            });
            results.changelog.created++;
          }
        } catch (err) {
          console.error(`[Sync] Error upserting changelog ${entry.version}:`, err);
          results.changelog.errors++;
        }
      }
    }

    // 3. Sync Docs
    console.log('[Sync] Fetching documentation from GitHub...');
    const docConfigs = [
      { url: GITHUB_FILES.readme, slug: 'overview', title: 'Overview', category: 'getting-started', order: 1 },
      { url: GITHUB_FILES.quickstart, slug: 'quickstart', title: 'Quick Start', category: 'getting-started', order: 2 },
      { url: GITHUB_FILES.architecture, slug: 'architecture', title: 'Architecture', category: 'guides', order: 10 },
      { url: GITHUB_FILES.contributing, slug: 'contributing', title: 'Contributing', category: 'guides', order: 20 },
    ];

    for (const config of docConfigs) {
      try {
        const res = await fetch(config.url);
        if (!res.ok) continue;

        const content = await res.text();
        const titleMatch = content.match(/^#\s+(.+)/m);

        const doc: Doc = {
          slug: config.slug,
          title: titleMatch ? titleMatch[1].trim() : config.title,
          category: config.category,
          content,
          order: config.order,
        };

        results.docs.docs++;

        const existing = await db.collection('docs').findOne({ slug: doc.slug });
        if (existing) {
          await db.collection('docs').updateOne(
            { slug: doc.slug },
            { $set: { ...doc, updatedAt: new Date() } }
          );
          results.docs.updated++;
        } else {
          await db.collection('docs').insertOne({
            ...doc,
            createdAt: new Date(),
            updatedAt: new Date(),
          });
          results.docs.created++;
        }
      } catch (err) {
        console.error(`[Sync] Error fetching ${config.slug}:`, err);
        results.docs.errors++;
      }
    }

    const duration = Date.now() - startTime;
    console.log(`[Sync] Complete in ${duration}ms`);

    return c.json({
      success: true,
      duration: `${duration}ms`,
      results,
    });
  } catch (error) {
    console.error('[Sync] Error:', error);
    return c.json({ error: 'Sync failed', details: String(error) }, 500);
  }
});

// GET /webhook/sync - check last sync status
sync.get('/', async (c) => {
  return c.json({
    endpoint: 'POST /webhook/sync',
    description: 'Triggers a full sync from GitHub to MongoDB',
    source: 'https://github.com/Degi100/pctrl',
  });
});

export default sync;
