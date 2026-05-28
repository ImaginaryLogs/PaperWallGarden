import type { 
  QuartzPageTypePlugin, 
  PageMatcher, 
  VirtualPage, 
  ProcessedContent, 
  BuildCtx,
  FullSlug
} from "@quartz-community/types"
import { slugifyFilePath } from "@quartz-community/utils/path"
import { join } from "path"
import { WasmPlaygroundBody } from "./WasmPlayground.js"

export interface WasmPlaygroundOptions {
  wasmModulesDir?: string;
  publicPrefix?: string;
  frontmatterKey?: string;
}

const wasmPageMatcher: PageMatcher = ({ fileData }) => {
  return fileData && "wasmModuleData" in fileData
}

export const WasmPlaygroundPage: QuartzPageTypePlugin<WasmPlaygroundOptions> = (opts) => {
  const config = {
    wasmModulesDir: opts?.wasmModulesDir ?? "./wasm-modules",
    publicPrefix: opts?.publicPrefix ?? "/wasm",
    frontmatterKey: opts?.frontmatterKey ?? "wasm-module",
  }

  return {
    name: "WasmPlaygroundPage",
    priority: 30,
    fileExtensions: [".wasm", ".wat"],
    
    // 1. Fulfills check: "layout" in instance
    layout: "content", 
    
    // 2. Fulfills check: "body" in instance
    body: WasmPlaygroundBody({ publicPrefix: config.publicPrefix, frontmatterKey: config.frontmatterKey }),

    // 3. Fulfills check: "match" in instance
    match: wasmPageMatcher,

    generate({ ctx }) {
      const virtualPages: VirtualPage[] = []

      const wasmSourceFiles = ctx.allFiles.filter(
        (filePath) => filePath.endsWith(".wasm") || filePath.endsWith(".wat")
      )

      for (const filePath of wasmSourceFiles) {
        const baseName = filePath.split("/").pop() ?? "WASM Module"
        
        // Nominal Type Branding: Cast raw path slug to branded FullSlug format
        const slug = slugifyFilePath(filePath as any) as FullSlug

        virtualPages.push({
          slug,
          title: baseName,
          data: {
            frontmatter: { 
              title: baseName, 
              tags: ["wasm-playground", "webassembly"],
            },
            wasmModuleData: {
              sourcePath: filePath,
              publicUrl: join(config.publicPrefix, baseName),
              config: config
            }
          }
        })
      }

      return virtualPages
    },

    shouldPublish(_ctx: BuildCtx, content: ProcessedContent) {
      const relativePath = content[1]?.data?.relativePath ?? ""
      return relativePath.endsWith(".wasm") || relativePath.endsWith(".wat")
    }
  }
}