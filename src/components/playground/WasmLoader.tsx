import { useState, useEffect } from 'react';
import React from 'react';

interface WasmLoaderProps<T> {
  modulePath: string;
  render: (api: T | null, ready: boolean) => React.ReactNode;
}

export function WasmLoader<T>({ modulePath, render }: WasmLoaderProps<T>) {
  const [api, setApi] = useState<T | null>(null);
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (typeof window === 'undefined') return;

    let cancelled = false;
    console.log(`[WASM DEBUG] Starting load cycle for path: "${modulePath}"`);
    
    async function loadWasm() {
      try {
        const targetUrl = new URL(modulePath, window.location.origin).href;
        console.log(`[WASM DEBUG] Resolved target URL: ${targetUrl}`);

        const mod = await import(/* @vite-ignore */ `${targetUrl}`);
        
        // Initialize the wasm-bindgen runtime memory
        const initializedWasmMemory = await mod.default(); 
        
        if (!cancelled) {
          console.group("🚀 [WASM DEBUG] Module Loaded Successfully");
          console.log("Raw Module Object:", mod);
          console.log("Exposed Exports Key-Map:", Object.keys(mod));
          if (mod.QuantumState) {
            console.log("QuantumState Prototype Methods:", Object.getOwnPropertyNames(mod.QuantumState.prototype));
          }
          if (initializedWasmMemory) {
            console.log(`WebAssembly Linear Memory Buffer Size: ${(initializedWasmMemory.memory.buffer.byteLength / 1024).toFixed(2)} KB`);
          }
          console.groupEnd();

          setApi(mod);
          setReady(true);
        }
      } catch (err) {
        if (!cancelled) {
          console.error("❌ [WASM DEBUG] Runtime execution failure:", err);
          setError(err instanceof Error ? err.message : 'Unknown Wasm error');
        }
      }
    }

    loadWasm();
    return () => { 
      cancelled = true; 
    };
  }, [modulePath]);

  if (error) {
    return <div className="wasm-error-fallback">❌ WebAssembly Execution Failed: {error}</div>;
  }

  if (typeof render !== 'function') {
    return (
      <div className="wasm-error-fallback" style={{ borderColor: '#f59e0b', color: '#fcd34d' }}>
        ⚠️ MDX Parsing Warning: The <code>render</code> attribute was expected to be a function but received <code>{typeof render}</code>.
      </div>
    );
  }

  return <>{render(api, ready)}</>;
}