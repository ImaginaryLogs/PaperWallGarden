import { defineConfig } from "tsup"

const SINGLETON_EXTERNALS = [
  "preact",
  "preact/hooks",
  "preact/jsx-runtime",
  "preact/compat",
  "@jackyzha0/quartz",
  "@jackyzha0/quartz/*",
  "vfile",
  "unified",
  "globby",       // <-- Protect globby from inline compiling
  "fs/promises",  // <-- Native module flag
  "path",         // <-- Native module flag
]

export default defineConfig({
  entry: ["src/index.ts", "src/frames/index.ts"],
  format: ["esm"],
  dts: true,
  splitting: false,
  clean: true,
  noExternal: [/^((?!(globby|fs|path|preact|@jackyzha0\/quartz|vfile|unified)).)*$/],
  external: SINGLETON_EXTERNALS,
})