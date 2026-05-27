import fs from 'node:fs/promises';
import path from 'node:path';
import { glob } from 'glob';

export interface BacklinkIndex {
  [targetSlug: string]: { title: string; slug: string }[];
}

export async function buildBacklinks(vaultPath: string): Promise<BacklinkIndex> {
  const index: BacklinkIndex = {};
  const files = await glob(`${vaultPath}/**/*.md`);

  for (const file of files) {
    try {
      const content = await fs.readFile(file, 'utf-8');
      const sourceSlug = path.basename(file, '.md').replace(/\s+/g, '-').toLowerCase();
      const sourceTitle = path.basename(file, '.md');

      // Regular expression matching default [[Note Destination]] wiki targets
      const wikiLinkRegex = /\[\[([^\]|]+)(?:\|[^\]]+)?\]\]/g;
      let match;

      while ((match = wikiLinkRegex.exec(content)) !== null) {
        const targetName = match[1].trim();
        const targetSlug = targetName.replace(/\s+/g, '-').toLowerCase();

        if (!index[targetSlug]) {
          index[targetSlug] = [];
        }
        
        if (!index[targetSlug].some(item => item.slug === sourceSlug)) {
          index[targetSlug].push({ title: sourceTitle, slug: sourceSlug });
        }
      }
    } catch (e) {
      console.error(`Could not parse backlinks for file: ${file}`, e);
    }
  }

  return index;
}