import React from 'react';
import { WasmLoader } from './WasmLoader';
import CircuitEditor from './CircuitEditor';
import BlochSphere from './BlochSphere';

interface Props {
  modulePath: string;
}

export default function QuantumPlaygroundContainer({ modulePath }: Props) {
  return (
    <WasmLoader 
      modulePath={modulePath} 
      render={(api, ready) => 
        ready && api ? (
          <div className="circuit-playground">
            <CircuitEditor quantumApi={api} />
            <BlochSphere quantumApi={api} />
          </div>
        ) : (
          <div className="loading-wasm">⚙️ Mounting WebAssembly container allocations...</div>
        )
      }
    />
  );
}