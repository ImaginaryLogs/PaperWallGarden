import { defineCollection, z } from 'astro:content';

// ODG collection — reads from the vault symlink
const garden = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string().optional(),
    tags: z.array(z.string()).optional(),
    aliases: z.array(z.string()).optional(),
    created: z.date().optional(),
    modified: z.date().optional(),
    // Obsidian frontmatter is loosely typed
  }).passthrough(),
});

// IDP collection — structured MDX entries
const playground = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    description: z.string(),
    tags: z.array(z.string()),
    wasmModules: z.array(z.string()).optional(),  // Wasm deps to preload
    crossLinks: z.array(z.object({
      garden: z.string().optional(),   // path in ODG
      playground: z.string().optional(), // path in IDP
    })).optional(),
  }),
});

export const collections = { garden, playground };