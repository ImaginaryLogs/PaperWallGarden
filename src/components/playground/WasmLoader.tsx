// src/components/playground/WasmLoader.tsx
import { useState, useEffect, useRef } from 'react';

interface WasmLoaderProps<T> {
  modulePath: string;                       // e.g. '/wasm/quantum_sim.js'
  initFn: (mod: T) => void;
  children: (api: T | null, ready: boolean) => React.ReactNode;
}

export function WasmLoader<T>({ modulePath, initFn, children }: WasmLoaderProps<T>) {
  const [api, setApi] = useState<T | null>(null);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      const mod = await import(/* @vite-ignore */ modulePath);
      await mod.default(); // wasm-bindgen init()
      if (!cancelled) {
        initFn(mod);
        setApi(mod as T);
        setReady(true);
      }
    })();
    return () => { cancelled = true; };
  }, [modulePath]);

  return <>{children(api, ready)}</>;
}