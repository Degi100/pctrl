// Clean script - removes all docs from database
// Uses environment variables from .env file

const MONGODB_URI = process.env.MONGODB_URI;

if (!MONGODB_URI) {
  console.error('Error: MONGODB_URI environment variable is required');
  process.exit(1);
}

import { MongoClient } from 'mongodb';

async function cleanDocs() {
  console.log('Connecting to MongoDB...');

  const client = new MongoClient(MONGODB_URI!);

  try {
    await client.connect();
    const db = client.db();

    const countBefore = await db.collection('docs').countDocuments();
    console.log(`Found ${countBefore} docs in database`);

    if (countBefore === 0) {
      console.log('Nothing to clean!');
      return;
    }

    const result = await db.collection('docs').deleteMany({});
    console.log(`Deleted ${result.deletedCount} docs`);

    console.log('\nDatabase cleaned! Run "bun run seed" to re-populate.');
  } catch (err) {
    console.error('Error:', err);
  } finally {
    await client.close();
  }
}

cleanDocs();
