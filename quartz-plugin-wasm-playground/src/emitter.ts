import type { QuartzEmitterPlugin, BuildCtx, ProcessedContent, FilePath } from "@quartz-community/types"
import { join } from "path"
import fs from "fs/promises"

export interface WasmEmitterOptions {
  wasmModulesDir?: string;
  publicPrefix?: string;
}

export const WasmModuleEmitter: QuartzEmitterPlugin<WasmEmitterOptions> = (opts) => {
  const config = {
    wasmModulesDir: opts?.wasmModulesDir ?? "./wasm-modules",
    publicPrefix: opts?.publicPrefix ?? "/wasm",
  }

  return {
    name: "WasmModuleEmitter",

    async emit(ctx: BuildCtx, _content: ProcessedContent[], _resources): Promise<FilePath[]> {
      const emittedFiles: FilePath[] = []
      const outputDir = join(ctx.argv.output, config.publicPrefix)
      await fs.mkdir(outputDir, { recursive: true })

      let workspaceItems: string[] = []
      try {
        workspaceItems = await fs.readdir(config.wasmModulesDir)
      } catch (err) {
        return [] // Skips gracefully if directory isn't initialized yet
      }

      // Loop through each subfolder inside wasm-modules (e.g., "quantum-sim")
      for (const item of workspaceItems) {
        console.log("emitter.ts: ", item)
        const pkgPath = join(config.wasmModulesDir, item, "pkg")
        
        try {
          // Check if this project has a compiled 'pkg' folder
          const pkgFiles = await fs.readdir(pkgPath)

          for (const fileName of pkgFiles) {
            // Target the required production bundle files (.wasm and JS bindings)
            if (fileName.endsWith(".wasm") || (fileName.endsWith(".js") && !fileName.endsWith("_bg.js"))) {
              const sourceFilePath = join(pkgPath, fileName)
              const destinationPath = join(outputDir, fileName)

              const binaryBuffer = await fs.readFile(sourceFilePath)
              await fs.writeFile(destinationPath, binaryBuffer)
              
              const trackedPath = join(config.publicPrefix, fileName) as FilePath
              emittedFiles.push(trackedPath)
            }
          }
        } catch {
          // If a subfolder doesn't have a /pkg folder, ignore it safely and continue
          continue
        }
      }

      return emittedFiles
    }
  }
}