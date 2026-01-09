// Seed script that parses markdown files and syncs to the docs API
// Usage: bun run seed-docs.ts

import { readFileSync, existsSync } from 'fs';
import { resolve } from 'path';

const API_URL = process.env.API_URL || `http://localhost:${process.env.PORT || 3000}`;
const API_KEY = process.env.API_KEY;

if (!API_KEY) {
  console.error('Error: API_KEY environment variable is required');
  console.error('Set it in .env or run: API_KEY=your-key bun run seed-docs.ts');
  process.exit(1);
}

interface Doc {
  slug: string;
  title: string;
  category: string;
  content: string;
  order: number;
}

// Configuration: which markdown files to import and their metadata
interface DocConfig {
  file: string;           // Filename in project root
  slug: string;           // URL slug for the doc
  title: string;          // Display title
  category: string;       // Category for grouping
  order: number;          // Sort order within category
}

const docConfigs: DocConfig[] = [
  // Getting Started
  {
    file: 'README.md',
    slug: 'overview',
    title: 'Overview',
    category: 'getting-started',
    order: 1,
  },
  {
    file: 'QUICKSTART.md',
    slug: 'quickstart',
    title: 'Quick Start',
    category: 'getting-started',
    order: 2,
  },
  // Guides
  {
    file: 'ARCHITECTURE.md',
    slug: 'architecture',
    title: 'Architecture',
    category: 'guides',
    order: 10,
  },
  {
    file: 'CONTRIBUTING.md',
    slug: 'contributing',
    title: 'Contributing',
    category: 'guides',
    order: 20,
  },
];

/**
 * Read markdown file and extract title if present
 */
function readMarkdownFile(filePath: string): { title: string | null; content: string } | null {
  if (!existsSync(filePath)) {
    return null;
  }

  const content = readFileSync(filePath, 'utf-8');

  // Try to extract title from first line (# Title)
  const titleMatch = content.match(/^#\s+(.+)/m);
  const title = titleMatch ? titleMatch[1].trim() : null;

  return { title, content };
}

/**
 * Build docs array from config and markdown files
 */
function buildDocsFromFiles(projectRoot: string): Doc[] {
  const docs: Doc[] = [];

  for (const config of docConfigs) {
    const filePath = resolve(projectRoot, config.file);
    const result = readMarkdownFile(filePath);

    if (!result) {
      console.warn(`Warning: File not found: ${config.file}`);
      continue;
    }

    docs.push({
      slug: config.slug,
      title: result.title || config.title,
      category: config.category,
      content: result.content,
      order: config.order,
    });
  }

  return docs;
}

async function upsertDoc(doc: Doc): Promise<'created' | 'updated' | 'error'> {
  const headers = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${API_KEY}`
  };

  // Check if doc exists
  const checkResponse = await fetch(`${API_URL}/docs/${doc.slug}`);

  if (checkResponse.ok) {
    // Doc exists - update it
    const updateResponse = await fetch(`${API_URL}/docs/${doc.slug}`, {
      method: 'PUT',
      headers,
      body: JSON.stringify(doc)
    });
    return updateResponse.ok ? 'updated' : 'error';
  } else {
    // Doc doesn't exist - create it
    const createResponse = await fetch(`${API_URL}/docs`, {
      method: 'POST',
      headers,
      body: JSON.stringify(doc)
    });
    return createResponse.ok ? 'created' : 'error';
  }
}

async function seedDocs() {
  // Project root is two levels up from docs-api
  const projectRoot = resolve(__dirname, '../..');

  console.log(`Reading documentation from: ${projectRoot}\n`);
  console.log('Configured files:');
  for (const config of docConfigs) {
    console.log(`  - ${config.file} -> ${config.slug} (${config.category})`);
  }
  console.log('');

  const docs = buildDocsFromFiles(projectRoot);

  console.log(`Parsed ${docs.length} docs from markdown files\n`);
  console.log('Syncing to API (upsert mode)...\n');

  let created = 0, updated = 0, errors = 0;

  for (const doc of docs) {
    try {
      const result = await upsertDoc(doc);

      if (result === 'created') {
        console.log(`+ Created: ${doc.title} (${doc.slug})`);
        created++;
      } else if (result === 'updated') {
        console.log(`~ Updated: ${doc.title} (${doc.slug})`);
        updated++;
      } else {
        console.log(`x Error: ${doc.title}`);
        errors++;
      }
    } catch (err) {
      console.log(`x Error: ${doc.title} - ${err}`);
      errors++;
    }
  }

  console.log(`\nDone! Created: ${created}, Updated: ${updated}, Errors: ${errors}\n`);

  // Fetch summary
  try {
    const listResponse = await fetch(`${API_URL}/docs`);
    if (listResponse.ok) {
      const { docs: allDocs } = await listResponse.json();
      console.log(`Total docs in database: ${allDocs.length}`);
    }

    const catResponse = await fetch(`${API_URL}/docs/categories`);
    if (catResponse.ok) {
      const { categories } = await catResponse.json();
      console.log(`Categories: ${categories.join(', ')}`);
    }
  } catch {
    // Summary fetch failed, ignore
  }
}

seedDocs();
