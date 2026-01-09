// Sync from GitHub - runs at startup to seed database from GitHub raw files

import { getDB } from './db';

const GITHUB_RAW = 'https://raw.githubusercontent.com/Degi100/pctrl/main';

interface Feature { name: string; done: boolean; }
interface Category { name: string; features: Feature[]; }
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

function parseRoadmap(content: string): Phase[] {
  const phases: Phase[] = [];
  const lines = content.split('\n');
  let currentPhase: Phase | null = null;
  let currentCategory: Category | null = null;
  let inPhase = false;

  for (const line of lines) {
    const phaseMatch = line.match(/^## Phase (\d+): (.+) \[(\w+)\]/);
    if (phaseMatch) {
      if (currentPhase) {
        if (currentCategory?.features.length) {
          currentPhase.categories = currentPhase.categories || [];
          currentPhase.categories.push(currentCategory);
        }
        phases.push(currentPhase);
      }
      const title = phaseMatch[2].replace(/\s*[✅🚧📋].*/g, '').trim();
      const statusRaw = phaseMatch[3].toLowerCase();
      let status: Phase['status'] = 'planned', statusLabel = 'Planned';
      if (statusRaw === 'done') { status = 'done'; statusLabel = 'Complete'; }
      else if (statusRaw === 'current') { status = 'current'; statusLabel = 'Current'; }
      currentPhase = { phaseId: parseInt(phaseMatch[1]), version: '', title, status, statusLabel, description: '' };
      currentCategory = null;
      inPhase = true;
      continue;
    }

    if (line.startsWith('## Future') || line.startsWith('## Contributing') || line.startsWith('## Release')) {
      if (currentPhase) {
        if (currentCategory?.features.length) {
          currentPhase.categories = currentPhase.categories || [];
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
    if (goalMatch) { currentPhase.description = goalMatch[1].trim(); continue; }

    const releaseMatch = line.match(/^\*\*(?:Target )?Release:\*\* (v[\d.]+)/);
    if (releaseMatch) {
      const v = releaseMatch[1].match(/v(\d+)\.(\d+)/);
      currentPhase.version = v ? `v${v[1]}.${v[2]}.x` : releaseMatch[1];
      continue;
    }

    const catMatch = line.match(/^### (.+)/);
    if (catMatch) {
      if (currentCategory?.features.length) {
        currentPhase.categories = currentPhase.categories || [];
        currentPhase.categories.push(currentCategory);
      }
      currentCategory = { name: catMatch[1].trim(), features: [] };
      continue;
    }

    const featMatch = line.match(/^- .+? (.+)/);
    if (featMatch && (line.includes('✅') || line.includes('📋'))) {
      const feature = { name: featMatch[1].trim(), done: line.includes('✅') };
      if (currentCategory) currentCategory.features.push(feature);
      else { currentPhase.features = currentPhase.features || []; currentPhase.features.push(feature); }
    }
  }

  if (currentPhase) {
    if (currentCategory?.features.length) {
      currentPhase.categories = currentPhase.categories || [];
      currentPhase.categories.push(currentCategory);
    }
    phases.push(currentPhase);
  }
  return phases;
}

function parseChangelog(content: string): ChangelogEntry[] {
  const entries: ChangelogEntry[] = [];
  const lines = content.split('\n');
  let entry: ChangelogEntry | null = null;
  let section: SectionName | null = null;
  let order = 1000;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const verMatch = line.match(/^## \[([^\]]+)\](?:\s*-\s*(\d{4}-\d{2}-\d{2}))?/);
    if (verMatch) {
      if (entry) entries.push(entry);
      entry = { version: verMatch[1], date: verMatch[2] || null, order: order--, sections: {} };
      section = null;
      continue;
    }
    if (line.startsWith('## Project Links')) { if (entry) { entries.push(entry); entry = null; } break; }
    if (!entry) continue;

    const secMatch = line.match(/^### (\w+)/);
    if (secMatch) {
      const name = secMatch[1].toLowerCase() as SectionName;
      if (['planned', 'added', 'changed', 'deprecated', 'removed', 'fixed', 'security'].includes(name)) {
        section = name;
        entry.sections[section] = entry.sections[section] || [];
      } else section = null;
      continue;
    }

    if (section && line.match(/^-\s+.+/)) {
      let item = line.replace(/^-\s+/, '').trim();
      let j = i + 1;
      while (j < lines.length && lines[j].match(/^\s{2,}-\s+/)) {
        item += ` | ${lines[j].replace(/^\s+-\s+/, '').trim()}`;
        j++;
      }
      entry.sections[section]!.push(item);
    }
  }
  if (entry) entries.push(entry);
  return entries;
}

export async function syncFromGitHub(): Promise<void> {
  console.log('[Sync] Starting GitHub sync...');
  const db = await getDB();

  try {
    // Roadmap
    const roadmapRes = await fetch(`${GITHUB_RAW}/ROADMAP.md`);
    if (roadmapRes.ok) {
      const phases = parseRoadmap(await roadmapRes.text());
      console.log(`[Sync] Parsed ${phases.length} phases from ROADMAP.md`);
      for (const phase of phases) {
        await db.collection('roadmap').updateOne(
          { phaseId: phase.phaseId },
          { $set: { ...phase, updatedAt: new Date() }, $setOnInsert: { createdAt: new Date() } },
          { upsert: true }
        );
      }
    }

    // Changelog
    const changelogRes = await fetch(`${GITHUB_RAW}/CHANGELOG.md`);
    if (changelogRes.ok) {
      const entries = parseChangelog(await changelogRes.text());
      console.log(`[Sync] Parsed ${entries.length} changelog entries`);
      for (const entry of entries) {
        await db.collection('changelog').updateOne(
          { version: entry.version },
          { $set: { ...entry, updatedAt: new Date() }, $setOnInsert: { createdAt: new Date() } },
          { upsert: true }
        );
      }
    }

    // Docs
    const docConfigs = [
      { file: 'README.md', slug: 'overview', title: 'Overview', category: 'getting-started', order: 1 },
      { file: 'QUICKSTART.md', slug: 'quickstart', title: 'Quick Start', category: 'getting-started', order: 2 },
      { file: 'ARCHITECTURE.md', slug: 'architecture', title: 'Architecture', category: 'guides', order: 10 },
      { file: 'CONTRIBUTING.md', slug: 'contributing', title: 'Contributing', category: 'guides', order: 20 },
    ];

    for (const cfg of docConfigs) {
      const res = await fetch(`${GITHUB_RAW}/${cfg.file}`);
      if (res.ok) {
        const content = await res.text();
        const titleMatch = content.match(/^#\s+(.+)/m);
        await db.collection('docs').updateOne(
          { slug: cfg.slug },
          { $set: {
            slug: cfg.slug,
            title: titleMatch?.[1]?.trim() || cfg.title,
            category: cfg.category,
            content,
            order: cfg.order,
            updatedAt: new Date(),
          }, $setOnInsert: { createdAt: new Date() } },
          { upsert: true }
        );
      }
    }

    console.log('[Sync] GitHub sync complete!');
  } catch (err) {
    console.error('[Sync] Error:', err);
  }
}
