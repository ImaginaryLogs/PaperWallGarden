/**
 * Converts an Obsidian [[wikilink]] target to a URL-safe slug.
 * Handles: bare names, paths, aliases.
 */
export function wikilinkToSlug(target: string): string {
  // Strip any heading anchor (#section)
  const base = target.split('#')[0].trim();
  // Take just the filename (last path segment), no extension
  const name = base.split('/').pop()!.replace(/\.md$/i, '');
  return name.toLowerCase().replace(/\s+/g, '-').replace(/[^\w-]/g, '');
}

export function wikilinkToHref(target: string, base = '/garden/'): string {
  return `${base}${wikilinkToSlug(target)}`;
}

/**
 * Build a reverse index: slug → array of slugs that link to it.
 * Pass in all (slug, rawContent) pairs from the vault.
 */
export function buildBacklinkIndex(
  pages: Array<{ slug: string; content: string }>
): Map<string, string[]> {
  const index = new Map<string, string[]>();
  const regex = /\[\[([^\]|#]+)(?:[|#][^\]]*)?\]\]/g;

  for (const { slug, content } of pages) {
    let m: RegExpExecArray | null;
    while ((m = regex.exec(content)) !== null) {
      const target = wikilinkToSlug(m[1]);
      if (!index.has(target)) index.set(target, []);
      index.get(target)!.push(slug);
    }
  }
  return index;
}