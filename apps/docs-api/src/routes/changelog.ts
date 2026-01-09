import { Hono } from 'hono';
import { getDB } from '../db';

const changelog = new Hono();

// Changelog entry schema:
// {
//   _id: ObjectId,
//   version: string,         // e.g., "0.1.2", "Unreleased"
//   date: string | null,     // e.g., "2025-01-06" or null for unreleased
//   order: number,           // For sorting (higher = newer)
//   sections: {
//     planned?: string[],
//     added?: string[],
//     changed?: string[],
//     deprecated?: string[],
//     removed?: string[],
//     fixed?: string[],
//     security?: string[]
//   },
//   createdAt: Date,
//   updatedAt: Date
// }

// GET /changelog - List all changelog entries
changelog.get('/', async (c) => {
  const db = await getDB();
  const entries = await db
    .collection('changelog')
    .find({})
    .sort({ order: -1 })
    .toArray();

  return c.json({ entries });
});

// GET /changelog/latest - Get the latest released version
changelog.get('/latest', async (c) => {
  const db = await getDB();
  const entry = await db
    .collection('changelog')
    .findOne(
      { version: { $ne: 'Unreleased' } },
      { sort: { order: -1 } }
    );

  if (!entry) {
    return c.json({ error: 'No releases found' }, 404);
  }

  return c.json({ entry });
});

// GET /changelog/:version - Get a specific version
changelog.get('/:version', async (c) => {
  const version = c.req.param('version');
  const db = await getDB();
  const entry = await db.collection('changelog').findOne({ version });

  if (!entry) {
    return c.json({ error: 'Version not found' }, 404);
  }

  return c.json({ entry });
});

// POST /changelog - Create a new changelog entry (requires auth)
changelog.post('/', async (c) => {
  const authHeader = c.req.header('Authorization');
  const apiKey = process.env.API_KEY;

  if (!apiKey || authHeader !== `Bearer ${apiKey}`) {
    return c.json({ error: 'Unauthorized' }, 401);
  }

  const body = await c.req.json();
  const { version, date, order, sections } = body;

  if (!version) {
    return c.json({ error: 'Missing required field: version' }, 400);
  }

  const db = await getDB();
  const now = new Date();

  // Check if version already exists
  const existing = await db.collection('changelog').findOne({ version });
  if (existing) {
    return c.json({ error: 'Version already exists' }, 409);
  }

  const result = await db.collection('changelog').insertOne({
    version,
    date: date || null,
    order: order ?? 0,
    sections: sections || {},
    createdAt: now,
    updatedAt: now,
  });

  return c.json({ id: result.insertedId, version }, 201);
});

// PUT /changelog/:version - Update a changelog entry
changelog.put('/:version', async (c) => {
  const authHeader = c.req.header('Authorization');
  const apiKey = process.env.API_KEY;

  if (!apiKey || authHeader !== `Bearer ${apiKey}`) {
    return c.json({ error: 'Unauthorized' }, 401);
  }

  const version = c.req.param('version');
  const body = await c.req.json();
  const { date, order, sections } = body;

  const db = await getDB();
  const updateFields: Record<string, unknown> = { updatedAt: new Date() };

  if (date !== undefined) updateFields.date = date;
  if (order !== undefined) updateFields.order = order;
  if (sections) updateFields.sections = sections;

  const result = await db.collection('changelog').updateOne(
    { version },
    { $set: updateFields }
  );

  if (result.matchedCount === 0) {
    return c.json({ error: 'Version not found' }, 404);
  }

  return c.json({ updated: true });
});

// DELETE /changelog/:version - Delete a changelog entry
changelog.delete('/:version', async (c) => {
  const authHeader = c.req.header('Authorization');
  const apiKey = process.env.API_KEY;

  if (!apiKey || authHeader !== `Bearer ${apiKey}`) {
    return c.json({ error: 'Unauthorized' }, 401);
  }

  const version = c.req.param('version');
  const db = await getDB();
  const result = await db.collection('changelog').deleteOne({ version });

  if (result.deletedCount === 0) {
    return c.json({ error: 'Version not found' }, 404);
  }

  return c.json({ deleted: true });
});

export default changelog;
