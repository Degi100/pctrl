import { Hono } from 'hono';
import { getDB } from '../db';

const roadmap = new Hono();

// Phase schema:
// {
//   _id: ObjectId,
//   phaseId: number,
//   version: string,
//   title: string,
//   status: 'done' | 'current' | 'planned',
//   statusLabel: string,
//   description: string,
//   categories: [{ name: string, features: [{ name: string, done: boolean }] }],
//   features: [{ name: string, done: boolean }],
//   updatedAt: Date
// }

// GET /roadmap - Get all phases with features
roadmap.get('/', async (c) => {
  const db = await getDB();
  const phases = await db
    .collection('roadmap')
    .find({})
    .sort({ phaseId: 1 })
    .toArray();

  // Calculate stats
  let total = 0;
  let completed = 0;

  for (const phase of phases) {
    if (phase.features) {
      total += phase.features.length;
      completed += phase.features.filter((f: { done: boolean }) => f.done).length;
    }
    if (phase.categories) {
      for (const cat of phase.categories) {
        total += cat.features.length;
        completed += cat.features.filter((f: { done: boolean }) => f.done).length;
      }
    }
  }

  return c.json({
    phases,
    stats: { total, completed, phaseCount: phases.length },
  });
});

// GET /roadmap/stats - Get stats only
roadmap.get('/stats', async (c) => {
  const db = await getDB();
  const phases = await db.collection('roadmap').find({}).toArray();

  let total = 0;
  let completed = 0;

  for (const phase of phases) {
    if (phase.features) {
      total += phase.features.length;
      completed += phase.features.filter((f: { done: boolean }) => f.done).length;
    }
    if (phase.categories) {
      for (const cat of phase.categories) {
        total += cat.features.length;
        completed += cat.features.filter((f: { done: boolean }) => f.done).length;
      }
    }
  }

  return c.json({ total, completed, phaseCount: phases.length });
});

// GET /roadmap/:phaseId - Get a single phase
roadmap.get('/:phaseId', async (c) => {
  const phaseId = parseInt(c.req.param('phaseId'));
  const db = await getDB();
  const phase = await db.collection('roadmap').findOne({ phaseId });

  if (!phase) {
    return c.json({ error: 'Phase not found' }, 404);
  }

  return c.json({ phase });
});

// POST /roadmap - Create a new phase (requires auth)
roadmap.post('/', async (c) => {
  const authHeader = c.req.header('Authorization');
  const apiKey = process.env.API_KEY;

  if (!apiKey || authHeader !== `Bearer ${apiKey}`) {
    return c.json({ error: 'Unauthorized' }, 401);
  }

  const body = await c.req.json();
  const { phaseId, version, title, status, statusLabel, description, features, categories } = body;

  if (!phaseId || !title || !status) {
    return c.json({ error: 'Missing required fields (phaseId, title, status)' }, 400);
  }

  const db = await getDB();

  // Check if phase already exists
  const existing = await db.collection('roadmap').findOne({ phaseId });
  if (existing) {
    return c.json({ error: 'Phase already exists' }, 409);
  }

  const result = await db.collection('roadmap').insertOne({
    phaseId,
    version: version || `v0.${phaseId}.x`,
    title,
    status,
    statusLabel: statusLabel || status.charAt(0).toUpperCase() + status.slice(1),
    description: description || '',
    features: features || [],
    categories: categories || [],
    updatedAt: new Date(),
  });

  return c.json({ id: result.insertedId, phaseId }, 201);
});

// PUT /roadmap/:phaseId - Update a phase
roadmap.put('/:phaseId', async (c) => {
  const authHeader = c.req.header('Authorization');
  const apiKey = process.env.API_KEY;

  if (!apiKey || authHeader !== `Bearer ${apiKey}`) {
    return c.json({ error: 'Unauthorized' }, 401);
  }

  const phaseId = parseInt(c.req.param('phaseId'));
  const body = await c.req.json();

  const db = await getDB();
  const updateFields: Record<string, unknown> = { updatedAt: new Date() };

  // Allow updating any field
  const allowedFields = ['version', 'title', 'status', 'statusLabel', 'description', 'features', 'categories'];
  for (const field of allowedFields) {
    if (body[field] !== undefined) {
      updateFields[field] = body[field];
    }
  }

  const result = await db.collection('roadmap').updateOne(
    { phaseId },
    { $set: updateFields }
  );

  if (result.matchedCount === 0) {
    return c.json({ error: 'Phase not found' }, 404);
  }

  return c.json({ updated: true });
});

// PUT /roadmap/:phaseId/feature - Update a specific feature's done status
roadmap.put('/:phaseId/feature', async (c) => {
  const authHeader = c.req.header('Authorization');
  const apiKey = process.env.API_KEY;

  if (!apiKey || authHeader !== `Bearer ${apiKey}`) {
    return c.json({ error: 'Unauthorized' }, 401);
  }

  const phaseId = parseInt(c.req.param('phaseId'));
  const body = await c.req.json();
  const { featureName, done, categoryName } = body;

  if (!featureName || done === undefined) {
    return c.json({ error: 'Missing required fields (featureName, done)' }, 400);
  }

  const db = await getDB();
  const phase = await db.collection('roadmap').findOne({ phaseId });

  if (!phase) {
    return c.json({ error: 'Phase not found' }, 404);
  }

  let updated = false;

  // Update in categories if categoryName provided
  if (categoryName && phase.categories) {
    for (const cat of phase.categories) {
      if (cat.name === categoryName) {
        for (const feature of cat.features) {
          if (feature.name === featureName) {
            feature.done = done;
            updated = true;
            break;
          }
        }
      }
    }
  }

  // Update in direct features
  if (!updated && phase.features) {
    for (const feature of phase.features) {
      if (feature.name === featureName) {
        feature.done = done;
        updated = true;
        break;
      }
    }
  }

  if (!updated) {
    return c.json({ error: 'Feature not found' }, 404);
  }

  await db.collection('roadmap').updateOne(
    { phaseId },
    { $set: { features: phase.features, categories: phase.categories, updatedAt: new Date() } }
  );

  return c.json({ updated: true });
});

// DELETE /roadmap/:phaseId - Delete a phase
roadmap.delete('/:phaseId', async (c) => {
  const authHeader = c.req.header('Authorization');
  const apiKey = process.env.API_KEY;

  if (!apiKey || authHeader !== `Bearer ${apiKey}`) {
    return c.json({ error: 'Unauthorized' }, 401);
  }

  const phaseId = parseInt(c.req.param('phaseId'));
  const db = await getDB();
  const result = await db.collection('roadmap').deleteOne({ phaseId });

  if (result.deletedCount === 0) {
    return c.json({ error: 'Phase not found' }, 404);
  }

  return c.json({ deleted: true });
});

export default roadmap;
