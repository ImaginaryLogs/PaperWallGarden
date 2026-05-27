import { defineConfig } from 'astro/config';
import mdx from '@astrojs/mdx';
import react from '@astrojs/react';
import remarkWikiLink from 'remark-wiki-link';
import remarkObsidianCallout from 'remark-obsidian-callout';
import remarkMath from 'remark-math';
import rehypeKatex from 'rehype-katex';
import { resolve } from 'path';

export default defineConfig({
  site: 'https://YOUR-USERNAME.github.io',
  base: '/YOUR-REPO-NAME',

  integrations: [
    react(),   // For canvas + playground components
    mdx({
      remarkPlugins: [
        remarkMath,
        [remarkWikiLink, {
          // Resolve [[note-name]] to /garden/note-name
          pageResolver: (name) => [name.replace(/\s+/g, '-').toLowerCase()],
          hrefTemplate: (permalink) => `/garden/${permalink}`,
        }],
        remarkObsidianCallout,
      ],
      rehypePlugins: [rehypeKatex],  // LaTeX math rendering
    }),
  ],

  vite: {
    // Allow Wasm modules
    plugins: [],
    optimizeDeps: {
      exclude: ['@sqlite.org/sqlite-wasm'], // example heavy wasm
    },
    // Ensure wasm files are served correctly
    assetsInclude: ['**/*.wasm'],
    resolve: {
      alias: {
        // Allow vault symlink to resolve correctly on all platforms
        '@vault': resolve('./obsidian'),
        '@components': resolve('./src/components'),
        '@lib': resolve('./src/lib'),
      },
    },
    server: {
      fs: { allow: ['..'] }, // Allow serving from outside /src
    },
  },

  build: {
    assets: '_assets',
  },
});