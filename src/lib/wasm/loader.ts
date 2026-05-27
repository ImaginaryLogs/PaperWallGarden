// src/lib/wasm/loader.ts

const wasmCompilationCache = new Map<string, Promise<any>>();

/**
 * Handles browser-native WebAssembly initialization layers, bypassing 
 * compilation constraints imposed on the public assets folder by Vite.
 * * @param targetPath Absolute browser request URI (e.g. '/wasm/quantum_sim.js')
 */
export async function loadWasmModule<T>(targetPath: string): Promise<T> {
  if (wasmCompilationCache.has(targetPath)) {
    return wasmCompilationCache.get(targetPath)!;
  }

  const loadTask = (async () => {
    try {
      // Pre-flight check: Ensure the target file actually exists in the public directory
      const response = await fetch(targetPath, { method: 'HEAD' });
      if (!response.ok) {
        throw new Error(`Network asset not found at destination path: ${targetPath} (Status: ${response.status})`);
      }

      // Bypass Vite's bundle-time AST analyzer by invoking a standard, runtime browser import
      const importRef = new Function('path', 'return import(path)');
      const moduleGlue = await importRef(targetPath);

      // Execute the auto-generated wasm-bindgen linear memory init step
      if (moduleGlue && typeof moduleGlue.default === 'function') {
        await moduleGlue.default();
      }

      return moduleGlue as T;
    } catch (error) {
      wasmCompilationCache.delete(targetPath); // Evict cache records to allow retries
      console.error(`[Wasm Native Loader Core] Operational crash during initialization:`, error);
      throw error;
    }
  })();

  wasmCompilationCache.set(targetPath, loadTask);
  return loadTask;
}