// Docs API utility for fetching documentation from the API

const API_URL = import.meta.env.PUBLIC_DOCS_API_URL || 'http://localhost:3000';

export interface Doc {
  _id: string;
  slug: string;
  title: string;
  category: string;
  content: string;
  order: number;
  version?: string;
  createdAt?: string;
  updatedAt?: string;
}

export interface DocsListItem {
  _id: string;
  slug: string;
  title: string;
  category: string;
  order: number;
  sectionCount: number;
}

export async function fetchDocs(): Promise<DocsListItem[]> {
  try {
    const response = await fetch(`${API_URL}/api/docs`);
    if (!response.ok) {
      throw new Error(`API error: ${response.status}`);
    }
    const data = await response.json();
    return data.docs || [];
  } catch (error) {
    console.error('Failed to fetch docs:', error);
    return [];
  }
}

export async function fetchDoc(slug: string): Promise<Doc | null> {
  try {
    const response = await fetch(`${API_URL}/api/docs/${slug}`);
    if (!response.ok) {
      if (response.status === 404) {
        return null;
      }
      throw new Error(`API error: ${response.status}`);
    }
    const data = await response.json();
    return data.doc || null;
  } catch (error) {
    console.error(`Failed to fetch doc ${slug}:`, error);
    return null;
  }
}

export async function fetchCategories(): Promise<string[]> {
  try {
    const response = await fetch(`${API_URL}/api/docs/categories`);
    if (!response.ok) {
      throw new Error(`API error: ${response.status}`);
    }
    const data = await response.json();
    return data.categories || [];
  } catch (error) {
    console.error('Failed to fetch categories:', error);
    return [];
  }
}

export async function fetchDocsByCategory(category: string): Promise<DocsListItem[]> {
  try {
    const response = await fetch(`${API_URL}/api/docs/category/${category}`);
    if (!response.ok) {
      throw new Error(`API error: ${response.status}`);
    }
    const data = await response.json();
    return data.docs || [];
  } catch (error) {
    console.error(`Failed to fetch docs for category ${category}:`, error);
    return [];
  }
}

// Group docs by category
export function groupByCategory(docs: DocsListItem[]): Record<string, DocsListItem[]> {
  return docs.reduce((acc, doc) => {
    if (!acc[doc.category]) {
      acc[doc.category] = [];
    }
    acc[doc.category].push(doc);
    return acc;
  }, {} as Record<string, DocsListItem[]>);
}

// Category display names
export const categoryNames: Record<string, string> = {
  'getting-started': 'Getting Started',
  'commands': 'Commands',
  'guides': 'Guides',
};
