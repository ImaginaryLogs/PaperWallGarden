import path from "path"
import fs   from "fs/promises"
import { globby } from "globby"
import type {
  QuartzEmitterPluginInstance,
  QuartzPageTypePluginInstance,
  BuildCtx,
  ProcessedContent,
  StaticResources,
  FilePath,
  FullSlug,
} from "@quartz-community/types"
import { WasmPlaygroundBody } from "./WasmPlayground"
// Re-export factories directly
export { WasmPlaygroundPage } from "./pageType.js";
export { WasmModuleEmitter } from "./emitter.js";

import type { WasmPlaygroundOptions } from './pageType'

// Re-export type option interfaces explicitly to expose them to your quartz.config.yaml options loops
export type { WasmPlaygroundOptions } from "./pageType.js";
export type { WasmEmitterOptions } from "./emitter.js";
// ─── Options ────────────────────────────────────────────────────────────────



// ─── Emitter: copies .wasm + JS glue into the output directory ───────────────

function WasmAssetsEmitter(opts: WasmPlaygroundOptions = {}): QuartzEmitterPluginInstance {
  const wasmDir     = opts.wasmModulesDir ?? path.resolve(process.cwd(), "wasm-modules")
  const publicPfx   = opts.publicPrefix   ?? "/wasm"

  return {
    name: "WasmAssetsEmitter",
    getQuartzComponents: () => [],

    async emit(ctx: BuildCtx, _content: ProcessedContent[], _res: StaticResources) {
      const outBase = path.join(ctx.argv.output, publicPfx.replace(/^\//, ""))
      await fs.mkdir(outBase, { recursive: true })

      // Discover all pkg/ directories produced by wasm-pack
      const pkgDirs = await globby("*/pkg", {
        cwd: wasmDir,
        onlyDirectories: true,
        absolute: true,
      })

      const written: FilePath[] = []

      for (const pkgDir of pkgDirs) {
        const moduleName = path.basename(path.dirname(pkgDir))
        const destDir    = path.join(outBase, moduleName)
        await fs.mkdir(destDir, { recursive: true })

        // Copy .wasm and .js glue files only (skip .d.ts, package.json, etc.)
        const assets = await globby(["*.wasm", "*.js", "!*_bg.js"], {
          cwd: pkgDir,
          absolute: true,
        })

        for (const asset of assets) {
          const dest = path.join(destDir, path.basename(asset))
          await fs.copyFile(asset, dest)
          written.push(dest as FilePath)
        }
      }

      return written
    },
  }
}

// ─── Page Type: routes content/wasm-playground/*.md to the playground UI ─────

function WasmPlaygroundPageType(opts: WasmPlaygroundOptions = {}): QuartzPageTypePluginInstance {
  const fmKey      = opts.frontmatterKey ?? "wasm-module"
  const publicPfx  = opts.publicPrefix   ?? "/wasm"

  return {
    name: "WasmPlayground",
    frame: "playground-frame",
    layout: "content",
    priority: 10,

    match({ slug, fileData }: { slug: FullSlug; fileData: any; [key: string]: any }) {
    // Match any page under wasm-playground/ that has the frontmatter key
    return (
        slug.startsWith("wasm-playground/") &&
        !!fileData?.frontmatter?.[fmKey]
    )
    },

    body: WasmPlaygroundBody({ publicPrefix: publicPfx, frontmatterKey: fmKey }),
  }
}

// ─── Combined plugin export ───────────────────────────────────────────────────

export function WasmPlayground(opts: WasmPlaygroundOptions = {}) {
  return [WasmAssetsEmitter(opts), WasmPlaygroundPageType(opts)]
}

