// Seed script that parses CHANGELOG.md and syncs to the API
// Usage: bun run seed-changelog.ts [path-to-changelog.md]

import { readFileSync } from 'fs';
import { resolve } from 'path';

const API_URL = process.env.API_URL || `http://localhost:${process.env.PORT || 3000}`;
const API_KEY = process.env.API_KEY;

if (!API_KEY) {
  console.error('Error: API_KEY environment variable is required');
  console.error('Set it in .env or run: API_KEY=your-key bun run seed-changelog.ts');
  process.exit(1);
}

interface ChangelogEntry {
  version: string;
  date: string | null;
  order: number;
  sections: {
    planned?: string[];
    added?: string[];
    changed?: string[];
    deprecated?: string[];
    removed?: string[];
    fixed?: string[];
    security?: string[];
  };
}

type SectionName = 'planned' | 'added' | 'changed' | 'deprecated' | 'removed' | 'fixed' | 'security';

/**
 * Parse CHANGELOG.md and extract changelog entries
 */
function parseChangelogMd(content: string): ChangelogEntry[] {
  const entries: ChangelogEntry[] = [];
  const lines = content.split('\n');

  let currentEntry: ChangelogEntry | null = null;
  let currentSection: SectionName | null = null;
  let orderCounter = 1000; // Higher = newer

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // Match version header: ## [Version] - Date or ## [Unreleased]
    // Examples:
    //   ## [Unreleased]
    //   ## [0.1.2] - 2025-01-06
    const versionMatch = line.match(/^## \[([^\]]+)\](?:\s*-\s*(\d{4}-\d{2}-\d{2}))?/);
    if (versionMatch) {
      // Save previous entry
      if (currentEntry) {
        entries.push(currentEntry);
      }

      const version = versionMatch[1];
      const date = versionMatch[2] || null;

      currentEntry = {
        version,
        date,
        order: orderCounter--,
        sections: {},
      };
      currentSection = null;
      continue;
    }

    // Stop at certain sections (like Project Links)
    if (line.startsWith('## Project Links')) {
      if (currentEntry) {
        entries.push(currentEntry);
        currentEntry = null;
      }
      break;
    }

    if (!currentEntry) continue;

    // Match section header: ### Added, ### Changed, etc.
    const sectionMatch = line.match(/^### (\w+)/);
    if (sectionMatch) {
      const sectionName = sectionMatch[1].toLowerCase() as SectionName;
      if (['planned', 'added', 'changed', 'deprecated', 'removed', 'fixed', 'security'].includes(sectionName)) {
        currentSection = sectionName;
        if (!currentEntry.sections[currentSection]) {
          currentEntry.sections[currentSection] = [];
        }
      } else {
        currentSection = null;
      }
      continue;
    }

    // Match list item: - Item text
    // Also handles nested items by combining them
    if (currentSection && line.match(/^-\s+.+/)) {
      let item = line.replace(/^-\s+/, '').trim();

      // Look ahead for nested items (indented with spaces)
      let j = i + 1;
      while (j < lines.length && lines[j].match(/^\s{2,}-\s+/)) {
        const nestedItem = lines[j].replace(/^\s+-\s+/, '').trim();
        item += ` | ${nestedItem}`;
        j++;
      }

      currentEntry.sections[currentSection]!.push(item);
      continue;
    }
  }

  // Don't forget the last entry
  if (currentEntry) {
    entries.push(currentEntry);
  }

  return entries;
}

async function upsertEntry(entry: ChangelogEntry): Promise<'created' | 'updated' | 'error'> {
  const headers = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${API_KEY}`
  };

  // Check if version exists
  const checkResponse = await fetch(`${API_URL}/changelog/${encodeURIComponent(entry.version)}`);

  if (checkResponse.ok) {
    // Entry exists - update it
    const updateResponse = await fetch(`${API_URL}/changelog/${encodeURIComponent(entry.version)}`, {
      method: 'PUT',
      headers,
      body: JSON.stringify(entry)
    });
    return updateResponse.ok ? 'updated' : 'error';
  } else {
    // Entry doesn't exist - create it
    const createResponse = await fetch(`${API_URL}/changelog`, {
      method: 'POST',
      headers,
      body: JSON.stringify(entry)
    });
    return createResponse.ok ? 'created' : 'error';
  }
}

async function seedChangelog() {
  // Get changelog file path from args or use default
  const changelogPath = process.argv[2] || resolve(__dirname, '../../CHANGELOG.md');

  console.log(`Reading changelog from: ${changelogPath}\n`);

  let content: string;
  try {
    content = readFileSync(changelogPath, 'utf-8');
  } catch (err) {
    console.error(`Error reading file: ${changelogPath}`);
    console.error(err);
    process.exit(1);
  }

  const entries = parseChangelogMd(content);

  console.log(`Parsed ${entries.length} versions from CHANGELOG.md\n`);
  console.log('Syncing to API (upsert mode)...\n');

  let created = 0, updated = 0, errors = 0;

  for (const entry of entries) {
    try {
      const result = await upsertEntry(entry);

      const dateStr = entry.date || 'unreleased';
      if (result === 'created') {
        console.log(`+ Created: ${entry.version} (${dateStr})`);
        created++;
      } else if (result === 'updated') {
        console.log(`~ Updated: ${entry.version} (${dateStr})`);
        updated++;
      } else {
        console.log(`x Error: ${entry.version}`);
        errors++;
      }
    } catch (err) {
      console.log(`x Error: ${entry.version} - ${err}`);
      errors++;
    }
  }

  console.log(`\nDone! Created: ${created}, Updated: ${updated}, Errors: ${errors}\n`);

  // Fetch and display summary
  try {
    const response = await fetch(`${API_URL}/changelog`);
    if (response.ok) {
      const data = await response.json();
      console.log(`Total versions in database: ${data.entries.length}`);
    }
  } catch {
    // Summary fetch failed, ignore
  }
}

seedChangelog();
