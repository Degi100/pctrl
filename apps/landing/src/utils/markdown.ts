// Enhanced markdown to HTML converter with syntax highlighting support

export interface Heading {
  id: string;
  text: string;
  level: number;
}

export interface MarkdownResult {
  html: string;
  headings: Heading[];
}

function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/(^-|-$)/g, '');
}

export function markdownToHtml(md: string): string {
  return parseMarkdown(md).html;
}

export function parseMarkdown(md: string): MarkdownResult {
  const headings: Heading[] = [];

  let html = md
    // Code blocks with language - add language label
    .replace(/```(\w+)?\n([\s\S]*?)```/g, (_, lang, code) => {
      const escaped = code
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .trim();
      const langClass = lang ? `language-${lang}` : '';
      const langLabel = lang ? `<span class="code-lang">${lang}</span>` : '';
      return `<div class="code-block">${langLabel}<pre><code class="${langClass}">${escaped}</code></pre></div>`;
    })
    // Inline code
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    // Blockquotes with special types (Tip, Warning, Note, etc.)
    .replace(/^>\s*\*\*Tip:\*\*\s*(.+)$/gm, '<div class="callout callout-tip"><span class="callout-icon">💡</span><div class="callout-content"><strong>Tip</strong><p>$1</p></div></div>')
    .replace(/^>\s*\*\*Warning:\*\*\s*(.+)$/gm, '<div class="callout callout-warning"><span class="callout-icon">⚠️</span><div class="callout-content"><strong>Warning</strong><p>$1</p></div></div>')
    .replace(/^>\s*\*\*Note:\*\*\s*(.+)$/gm, '<div class="callout callout-note"><span class="callout-icon">📝</span><div class="callout-content"><strong>Note</strong><p>$1</p></div></div>')
    .replace(/^>\s*\*\*Info:\*\*\s*(.+)$/gm, '<div class="callout callout-info"><span class="callout-icon">ℹ️</span><div class="callout-content"><strong>Info</strong><p>$1</p></div></div>')
    // Regular blockquotes
    .replace(/^>\s*(.+)$/gm, '<blockquote>$1</blockquote>')
    // Headers with IDs for anchor links
    .replace(/^### (.+)$/gm, (_, text) => {
      const id = slugify(text);
      headings.push({ id, text, level: 3 });
      return `<h3 id="${id}">${text}</h3>`;
    })
    .replace(/^## (.+)$/gm, (_, text) => {
      const id = slugify(text);
      headings.push({ id, text, level: 2 });
      return `<h2 id="${id}">${text}</h2>`;
    })
    .replace(/^# (.+)$/gm, (_, text) => {
      const id = slugify(text);
      headings.push({ id, text, level: 1 });
      return `<h1 id="${id}">${text}</h1>`;
    })
    // Bold
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    // Italic
    .replace(/\*([^*]+)\*/g, '<em>$1</em>')
    // Links
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>')
    // Horizontal rule
    .replace(/^---$/gm, '<hr>')
    // Tables
    .replace(/^\|(.+)\|$/gm, (_, content) => {
      const cells = content.split('|').map((c: string) => c.trim());
      // Skip separator rows
      if (cells.every((c: string) => /^[-:]+$/.test(c))) {
        return '<!-- table-sep -->';
      }
      return `<tr>${cells.map((c: string) => `<td>${c}</td>`).join('')}</tr>`;
    });

  // Wrap table rows
  html = html.replace(/((?:<tr>.*<\/tr>\n?)+)/g, '<table>$1</table>');
  html = html.replace(/<!-- table-sep -->\n?/g, '');

  // First table row should be header
  html = html.replace(/<table><tr>(.*?)<\/tr>/g, '<table><thead><tr>$1</tr></thead><tbody>');
  html = html.replace(/<\/table>/g, '</tbody></table>');
  html = html.replace(/<tbody><\/tbody><\/table>/g, '</table>');

  // Convert td to th in thead
  html = html.replace(/<thead><tr>(.*?)<\/tr><\/thead>/g, (_, content) => {
    return `<thead><tr>${content.replace(/<td>/g, '<th>').replace(/<\/td>/g, '</th>')}</tr></thead>`;
  });

  // Merge consecutive blockquotes
  html = html.replace(/(<blockquote>.*?<\/blockquote>\n?)+/g, (match) => {
    const content = match.replace(/<\/?blockquote>/g, '').trim();
    return `<blockquote>${content}</blockquote>`;
  });

  // Lists - unordered
  html = html.replace(/^- (.+)$/gm, '<li>$1</li>');
  html = html.replace(/((?:<li>.*<\/li>\n?)+)/g, '<ul>$1</ul>');

  // Numbered lists
  html = html.replace(/^\d+\.\s+(.+)$/gm, '<oli>$1</oli>');
  html = html.replace(/((?:<oli>.*<\/oli>\n?)+)/g, (match) => {
    return '<ol>' + match.replace(/<\/?oli>/g, (tag) => tag === '<oli>' ? '<li>' : '</li>') + '</ol>';
  });

  // Paragraphs - wrap text that isn't already in a block element
  const lines = html.split('\n');
  const result: string[] = [];
  let inParagraph = false;

  for (const line of lines) {
    const trimmed = line.trim();

    // Skip empty lines
    if (!trimmed) {
      if (inParagraph) {
        result.push('</p>');
        inParagraph = false;
      }
      result.push('');
      continue;
    }

    // Check if line starts with a block element
    const isBlockElement = /^<(h[1-6]|pre|ul|ol|li|table|thead|tbody|tr|td|th|hr|blockquote|div)/.test(trimmed);

    if (isBlockElement) {
      if (inParagraph) {
        result.push('</p>');
        inParagraph = false;
      }
      result.push(line);
    } else if (!inParagraph && trimmed) {
      result.push('<p>' + line);
      inParagraph = true;
    } else {
      result.push(line);
    }
  }

  if (inParagraph) {
    result.push('</p>');
  }

  return {
    html: result.join('\n'),
    headings
  };
}
