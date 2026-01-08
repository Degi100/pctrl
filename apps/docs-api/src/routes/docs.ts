import { Hono } from 'hono';
import { getDB } from '../db';

const docs = new Hono();

// Doc schema:
// {
//   _id: ObjectId,
//   slug: string,
//   category: string,
//   title: string,
//   content: string (markdown),
//   order: number,
//   version: string,
//   createdAt: Date,
//   updatedAt: Date
// }

// GET /docs - List all docs (titles only)
docs.get('/', async (c) => {
  const db = await getDB();
  const documents = await db
    .collection('docs')
    .find({}, { projection: { slug: 1, category: 1, title: 1, order: 1 } })
    .sort({ category: 1, order: 1 })
    .toArray();

  return c.json({ docs: documents });
});

// GET /docs/categories - List all categories
docs.get('/categories', async (c) => {
  const db = await getDB();
  const categories = await db
    .collection('docs')
    .distinct('category');

  return c.json({ categories });
});

// GET /docs/:category - List docs in a category
docs.get('/category/:category', async (c) => {
  const category = c.req.param('category');
  const db = await getDB();
  const documents = await db
    .collection('docs')
    .find({ category }, { projection: { slug: 1, title: 1, order: 1 } })
    .sort({ order: 1 })
    .toArray();

  return c.json({ docs: documents });
});

// GET /docs/:slug - Get a single doc by slug
docs.get('/:slug', async (c) => {
  const slug = c.req.param('slug');
  const db = await getDB();
  const doc = await db.collection('docs').findOne({ slug });

  if (!doc) {
    return c.json({ error: 'Document not found' }, 404);
  }

  return c.json({ doc });
});

// POST /docs - Create a new doc (requires auth header)
docs.post('/', async (c) => {
  const authHeader = c.req.header('Authorization');
  const apiKey = process.env.API_KEY;

  if (!apiKey || authHeader !== `Bearer ${apiKey}`) {
    return c.json({ error: 'Unauthorized' }, 401);
  }

  const body = await c.req.json();
  const { slug, category, title, content, order = 0, version = '0.1.0' } = body;

  if (!slug || !category || !title || !content) {
    return c.json({ error: 'Missing required fields' }, 400);
  }

  const db = await getDB();
  const now = new Date();

  const result = await db.collection('docs').insertOne({
    slug,
    category,
    title,
    content,
    order,
    version,
    createdAt: now,
    updatedAt: now,
  });

  return c.json({ id: result.insertedId, slug }, 201);
});

// PUT /docs/:slug - Update a doc
docs.put('/:slug', async (c) => {
  const authHeader = c.req.header('Authorization');
  const apiKey = process.env.API_KEY;

  if (!apiKey || authHeader !== `Bearer ${apiKey}`) {
    return c.json({ error: 'Unauthorized' }, 401);
  }

  const slug = c.req.param('slug');
  const body = await c.req.json();
  const { category, title, content, order, version } = body;

  const db = await getDB();
  const updateFields: Record<string, unknown> = { updatedAt: new Date() };

  if (category) updateFields.category = category;
  if (title) updateFields.title = title;
  if (content) updateFields.content = content;
  if (order !== undefined) updateFields.order = order;
  if (version) updateFields.version = version;

  const result = await db.collection('docs').updateOne(
    { slug },
    { $set: updateFields }
  );

  if (result.matchedCount === 0) {
    return c.json({ error: 'Document not found' }, 404);
  }

  return c.json({ updated: true });
});

// DELETE /docs/:slug - Delete a doc
docs.delete('/:slug', async (c) => {
  const authHeader = c.req.header('Authorization');
  const apiKey = process.env.API_KEY;

  if (!apiKey || authHeader !== `Bearer ${apiKey}`) {
    return c.json({ error: 'Unauthorized' }, 401);
  }

  const slug = c.req.param('slug');
  const db = await getDB();
  const result = await db.collection('docs').deleteOne({ slug });

  if (result.deletedCount === 0) {
    return c.json({ error: 'Document not found' }, 404);
  }

  return c.json({ deleted: true });
});

export default docs;
