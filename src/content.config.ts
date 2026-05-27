import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

const garden = defineCollection({
  loader: glob({ pattern: '**/*.md', base: './obsidian' }),
  schema: z.object({
    title:    z.string().optional(),
    tags:     z.array(z.string()).optional(),
    aliases:  z.array(z.string()).optional(),
    created:  z.coerce.date().optional(),
    modified: z.coerce.date().optional(),
  }).passthrough(),
});

const playground = defineCollection({
  loader: glob({ pattern: '**/*.mdx', base: './src/content/playground' }),
  schema: z.object({
    title:       z.string(),
    description: z.string(),
    tags:        z.array(z.string()).default([]),
    wasmModules: z.array(z.string()).optional(),
    crossLinks:  z.array(z.object({
      garden:     z.string().optional(),
      playground: z.string().optional(),
    })).optional(),
  }),
});

export const collections = { garden, playground };