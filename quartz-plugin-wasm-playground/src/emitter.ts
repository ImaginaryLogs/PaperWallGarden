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

    // This method fulfills the runtime validation criteria: "emit" in instance
    async emit(ctx: BuildCtx, _content: ProcessedContent[], _resources): Promise<FilePath[]> {
      const emittedFiles: FilePath[] = []
      
      const outputDir = join(ctx.argv.output, config.publicPrefix)
      await fs.mkdir(outputDir, { recursive: true })

      let filesToEmit: string[] = []
      try {
        filesToEmit = await fs.readdir(config.wasmModulesDir)
      } catch (err) {
        // If the workspace target dir isn't ready yet, skip gracefully
        return []
      }

      for (const fileName of filesToEmit) {
        if (fileName.endsWith(".wasm") || fileName.endsWith(".wat")) {
          const sourceFilePath = join(config.wasmModulesDir, fileName)
          const destinationPath = join(outputDir, fileName)

          try {
            const binaryBuffer = await fs.readFile(sourceFilePath)
            await fs.writeFile(destinationPath, binaryBuffer)
            
            // Nominal Type Branding: Cast string explicitly to branded FilePath format
            const trackedPath = join(config.publicPrefix, fileName) as FilePath
            emittedFiles.push(trackedPath)
          } catch (error) {
            console.error(`[WASM EMITTER ERROR] Failed to copy asset ${fileName}:`, error)
          }
        }
      }

      return emittedFiles
    }
  }
}