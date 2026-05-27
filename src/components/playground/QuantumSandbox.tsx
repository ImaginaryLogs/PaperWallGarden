// src/components/playground/QuantumSandbox.tsx
import { WasmLoader } from './WasmLoader';

// Define a placeholder or import your real engine controls here
export default function QuantumSandbox() {
  return (
    <div className="w-full max-w-5xl mx-auto px-4 py-6 space-y-6 bg-slate-900/50 rounded-xl border border-slate-800 backdrop-blur-sm">
      <WasmLoader<any> 
        modulePath="/wasm/quantum_sim.js" 
        initFn={(mod) => console.log("WASM Quantum Core Loaded", mod)}
      >
        {(api, ready) => 
          ready ? (
            <div className="circuit-playground space-y-4">
              <div className="flex items-center space-x-2">
                <h4 className="text-lg font-semibold text-white">WASM Status:</h4>
                <span className="text-emerald-400 font-medium px-2 py-0.5 rounded bg-emerald-500/10 text-sm border border-emerald-500/20">
                  Active Engine
                </span>
              </div>
              <p className="text-slate-300 text-sm leading-relaxed">
                The system binary compiled with <code className="text-cyan-400 bg-slate-950 px-1.5 py-0.5 rounded font-mono text-xs">wasm-pack</code> is active in this browser environment.
              </p>
              <div className="pt-2">
                <button 
                  style={{ padding: '10px 16px', background: '#3b82f6', border: 'none', borderRadius: '6px', color: '#fff', cursor: 'pointer' }}
                  className="font-medium hover:bg-blue-600 transition-colors shadow-lg shadow-blue-500/10"
                  onClick={() => {
                    if (api) {
                      // Trigger your native Rust calls here safely
                      alert("Matrix transformation complete via WASM stack machine engine.");
                    }
                  }}
                >
                  Trigger Engine Math Call
                </button>
              </div>
            </div>
          ) : (
            <div className="loading-wasm flex items-center space-x-3 text-slate-400 py-4 font-medium">
              <span className="animate-spin text-blue-500">⚙</span>
              <span>Mounting WebAssembly container allocations...</span>
            </div>
          )
        }
      </WasmLoader>
    </div>
  );
}