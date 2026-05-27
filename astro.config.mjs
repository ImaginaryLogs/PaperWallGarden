// @ts-check
import { defineConfig } from 'astro/config';
import react from '@astrojs/react';
import mdx from '@astrojs/mdx';
import remarkMath from 'remark-math';
import rehypeKatex from 'rehype-katex';
import { resolve } from 'path';
import wikiLinkPlugin from 'remark-wiki-link';

// Custom remark plugin to transform [[wikilinks]] into HTML anchors
function remarkWikiLinks() {
  return (tree, file) => {
    const { visit } = require('unist-util-visit');
    visit(tree, 'text', (node, index, parent) => {
      const regex = /\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g;
      let match;
      let lastIndex = 0;
      const parts = [];
      while ((match = regex.exec(node.value)) !== null) {
        if (match.index > lastIndex) {
          parts.push({ type: 'text', value: node.value.slice(lastIndex, match.index) });
        }
        const target = match[1].trim();
        const label  = match[2]?.trim() || target;
        const slug   = target.toLowerCase().replace(/\s+/g, '-').replace(/[^\w-]/g, '');
        parts.push({
          type: 'html',
          value: `<a href="/garden/${slug}" class="wikilink" data-target="${target}"><span class="wikilink-bracket">[[</span>${label}<span class="wikilink-bracket">]]</span></a>`,
        });
        lastIndex = match.index + match[0].length;
      }
      if (parts.length > 0) {
        if (lastIndex < node.value.length) {
          parts.push({ type: 'text', value: node.value.slice(lastIndex) });
        }
        parent.children.splice(index, 1, ...parts);
      }
    });
  };
}

export default defineConfig({
  site: 'https://yourusername.github.io',
  base: '/',
  integrations: [
    react(),
    mdx({
      remarkPlugins: [remarkMath],
      rehypePlugins: [rehypeKatex],
    }),
  ],
  vite: {
    assetsInclude: ['**/*.wasm'],
    resolve: {
      alias: {
        '@vault':      resolve('./obsidian-vault'),
        '@components': resolve('./src/components'),
        '@lib':        resolve('./src/lib'),
      },
    },
    server: { fs: { allow: ['..'] } },
  },
  markdown: {
    remarkPlugins: [
      [wikiLinkPlugin, {
        // Tell the plugin how to construct the URL
        pageResolver: (name) => [name.replace(/\s+/g, '-').toLowerCase()],
        hrefTemplate: (permalink) => `/garden/${permalink}`,
      }],
      remarkMath
    ],
    rehypePlugins: [
      rehypeKatex, // Renders LaTeX into math-friendly HTML
    ],
  },
});