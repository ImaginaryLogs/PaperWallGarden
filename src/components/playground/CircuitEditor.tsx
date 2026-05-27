import React, { useEffect, useRef, useState } from 'react';

interface CircuitEditorProps {
  quantumApi: any; // The API exposed from WasmLoader
}

export default function CircuitEditor({ quantumApi }: CircuitEditorProps) {
  const stateEngineRef = useRef<any>(null);
  
  // UI Reactive States
  const [probZero, setProbZero] = useState<number>(1.0);
  const [probOne, setProbOne] = useState<number>(0.0);
  const [rotationAngle, setRotationAngle] = useState<number>(90); 
  const [thetaDeg, setThetaDeg] = useState<number>(0);
  const [phiDeg, setPhiDeg] = useState<number>(0);

  useEffect(() => {
    console.log("[WASM DEBUG] CircuitEditor mounted. quantumApi available:", !!quantumApi);
    if (quantumApi) {
      console.log("[WASM DEBUG] QuantumState class prototype keys:", Object.getOwnPropertyNames(quantumApi.QuantumState?.prototype || {}));
    }

    if (quantumApi && !stateEngineRef.current) {
      try {
        stateEngineRef.current = new quantumApi.QuantumState();
        console.log("[WASM DEBUG] Instantiated new QuantumState. Memory pointer address:", stateEngineRef.current.__wbg_ptr);
        updateVisualMetrics();
      } catch (err) {
        console.error("[WASM DEBUG] Critical Error during QuantumState instantiation:", err);
      }
    }

    return () => {
      if (stateEngineRef.current && typeof stateEngineRef.current.free === 'function') {
        try {
          console.log("[WASM DEBUG] Cleaning up and freeing memory allocation for pointer:", stateEngineRef.current.__wbg_ptr);
          stateEngineRef.current.free();
          stateEngineRef.current = null;
        } catch (e) {
          console.warn("[WASM DEBUG] Prevented double-free allocation leak.");
        }
      }
    };
  }, [quantumApi]);

  // Synchronize state vector outputs directly with WASM memory allocations
  const updateVisualMetrics = () => {
    if (!stateEngineRef.current) {
      console.warn("[WASM DEBUG] updateVisualMetrics called but stateEngineRef.current is null.");
      return;
    }

    console.group("[WASM DEBUG] State Vector Sync Execution");
    console.log("Current Active Pointer Address:", stateEngineRef.current.__wbg_ptr);

    if (typeof stateEngineRef.current.bloch_coords === 'function') {
      const coords = stateEngineRef.current.bloch_coords(); // Returns [theta, phi] from Rust
      console.log("Raw Float64Array received from Wasm bloch_coords():", coords);
      
      if (coords && coords.length >= 1) {
        const theta = coords[0]; 
        const phi = coords.length > 1 ? coords[1] : 0;
        
        // Map to angle metrics
        const tDeg = (theta * 180) / Math.PI;
        const pDeg = (phi * 180) / Math.PI;
        console.log(`Parsed Coordinates -> Theta: ${tDeg.toFixed(2)}° (${theta.toFixed(4)} rad), Phi: ${pDeg.toFixed(2)}° (${phi.toFixed(4)} rad)`);

        // Compute true physical probabilities from theta
        const cosHalfTheta = Math.cos(theta / 2);
        const p0 = cosHalfTheta * cosHalfTheta;
        console.log(`Calculated Wavefunction Metrics -> P(|0⟩): ${(p0 * 100).toFixed(2)}%, P(|1⟩): ${((1.0 - p0) * 100).toFixed(2)}%`);

        setThetaDeg(tDeg);
        setPhiDeg(pDeg);
        setProbZero(parseFloat(p0.toFixed(5)));
        setProbOne(parseFloat((1.0 - p0).toFixed(5)));
      } else {
        console.error("[WASM DEBUG] bloch_coords() returned an empty or invalid array structure.");
      }
    } else {
      console.error("[WASM DEBUG] Critical Error: bloch_coords method does not exist on the loaded WASM prototype instance!");
    }
    console.groupEnd();
  };

  // Execution Handlers passing transforms straight over the Wasm boundaries
  const handleApplyHadamard = () => {
    if (stateEngineRef.current) {
      console.log("[WASM DEBUG] UI Click: Invoking stateEngine.hadamard()");
      if (typeof stateEngineRef.current.hadamard === 'function') {
        stateEngineRef.current.hadamard();
        updateVisualMetrics();
      } else {
        console.error("[WASM DEBUG] Method .hadamard() is missing on this Wasm binary!");
      }
    }
  };

  const handleSliderChange = (newDegrees: number) => {
    setRotationAngle(newDegrees);

    if (stateEngineRef.current) {
      // 1. Reset the qubit to ground state first so rotations aren't compounding on top of each other continuously
      if (typeof stateEngineRef.current.free === 'function') {
        stateEngineRef.current.free();
      }
      stateEngineRef.current = new quantumApi.QuantumState();

      // 2. Convert current slider angle to radians and pass directly to Wasm
      const radians = (newDegrees * Math.PI) / 180;
      if (typeof stateEngineRef.current.rotate_x === 'function') {
        stateEngineRef.current.rotate_x(radians);
      }
      
      // 3. Update dashboard metrics live
      updateVisualMetrics();
    }
  };

  const handleApplyPauliX = () => {
    if (stateEngineRef.current) {
      console.log("[WASM DEBUG] UI Click: Invoking stateEngine.pauli_x()");
      if (typeof stateEngineRef.current.pauli_x === 'function') {
        stateEngineRef.current.pauli_x();
        updateVisualMetrics();
      } else {
        console.error("[WASM DEBUG] Method .pauli_x() is missing on this Wasm binary!");
      }
    }
  };

  const handleApplyPauliZ = () => {
    if (stateEngineRef.current) {
      console.log("[WASM DEBUG] UI Click: Invoking stateEngine.pauli_z()");
      if (typeof stateEngineRef.current.pauli_z === 'function') {
        stateEngineRef.current.pauli_z();
        updateVisualMetrics();
      } else {
        console.error("[WASM DEBUG] Method .pauli_z() is missing on this Wasm binary!");
      }
    }
  };

  const handleApplyRotationX = () => {
    if (stateEngineRef.current) {
      const radians = (rotationAngle * Math.PI) / 180;
      console.log(`[WASM DEBUG] UI Click: Invoking stateEngine.rotate_x(theta: ${radians.toFixed(4)} rad) representing ${rotationAngle}°`);
      
      if (typeof stateEngineRef.current.rotate_x === 'function') {
        stateEngineRef.current.rotate_x(radians);
        updateVisualMetrics();
      } else {
        console.error("[WASM DEBUG] Method .rotate_x() is missing on this compiled Wasm binary! Verification failed.");
      }
    }
  };

  const handleResetState = () => {
    console.log("[WASM DEBUG] UI Click: Resetting State Vector Grounding context.");
    if (stateEngineRef.current) {
      if (typeof stateEngineRef.current.free === 'function') {
        stateEngineRef.current.free();
      }
      stateEngineRef.current = new quantumApi.QuantumState();
      updateVisualMetrics();
    }
  };

  return (
    <div className="editor-interface-wrapper" style={{ padding: '1rem', color: '#f8fafc' }}>
      <h3 style={{ marginTop: 0, color: '#3b82f6' }}>⚡ Qubit Rotation Operations</h3>
      
      {/* Probability Monitoring Vectors */}
      <div style={{ display: 'flex', gap: '1.5rem', margin: '1rem 0', background: '#0f172a', padding: '1rem', borderRadius: '6px', border: '1px solid #1e293b' }}>
        <div>
          <span style={{ color: '#94a3b8', fontSize: '0.8rem', textTransform: 'uppercase' }}>State |0⟩ Vector</span>
          <div style={{ fontSize: '1.5rem', fontWeight: 'bold', color: '#4ade80' }}>
            {(probZero * 100).toFixed(2)}%
          </div>
        </div>
        <div style={{ borderLeft: '1px solid #334155', paddingLeft: '1.5rem' }}>
          <span style={{ color: '#94a3b8', fontSize: '0.8rem', textTransform: 'uppercase' }}>State |1⟩ Vector</span>
          <div style={{ fontSize: '1.5rem', fontWeight: 'bold', color: '#60a5fa' }}>
            {(probOne * 100).toFixed(2)}%
          </div>
        </div>
      </div>

      {/* Complex Phase Geometry Dashboard */}
      <div style={{ margin: '1rem 0', padding: '1rem', background: '#0b0f19', borderRadius: '6px', border: '1px solid #3b82f644' }}>
        <span style={{ color: '#3b82f6', fontSize: '0.8rem', textTransform: 'uppercase', fontWeight: 'bold', display: 'block', marginBottom: '0.5rem' }}>
          🔮 Quantum Phase Matrix Monitoring
        </span>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem', fontFamily: 'monospace', fontSize: '0.9rem' }}>
          <div>
            <span style={{ color: '#64748b' }}>Polar Angle (θ):</span>{' '}
            <span style={{ color: '#f8fafc', fontWeight: 'bold' }}>{thetaDeg.toFixed(1)}°</span>
          </div>
          <div>
            <span style={{ color: '#64748b' }}>Phase Angle (ϕ):</span>{' '}
            <span style={{ color: '#a855f7', fontWeight: 'bold' }}>{phiDeg.toFixed(1)}°</span>
          </div>
        </div>

        {/* Dynamic Vector Wavefunction Readout */}
        <div style={{ marginTop: '0.75rem', paddingTop: '0.75rem', borderTop: '1px solid #1e293b', fontSize: '0.95rem', textAlign: 'center', fontFamily: 'monospace' }}>
          <span style={{ color: '#64748b' }}>|ψ⟩ = </span>
          <span style={{ color: '#4ade80' }}>{Math.sqrt(probZero).toFixed(3)}</span>
          <span style={{ color: '#64748b' }}>|0⟩ + </span>
          <span style={{ color: '#60a5fa' }}>
            {phiDeg > 0.1 ? `(e^i${(phiDeg * Math.PI / 180).toFixed(2)}) · ` : ''}
            {Math.sqrt(probOne).toFixed(3)}
          </span>
          <span style={{ color: '#64748b' }}>|1⟩</span>
        </div>
      </div>

      {/* Discrete Clifford Gates Panel */}
      <div style={{ marginBottom: '1.25rem' }}>
        <span style={{ fontSize: '0.85rem', color: '#94a3b8', display: 'block', marginBottom: '0.5rem' }}>Clifford Basis Matrix Transforms:</span>
        <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
          <button onClick={handleApplyHadamard} className="gate-btn">Hadamard (H)</button>
          <button onClick={handleApplyPauliX} className="gate-btn">Pauli-X (NOT)</button>
          <button onClick={handleApplyPauliZ} className="gate-btn">Pauli-Z (Phase)</button>
        </div>
      </div>

      {/* Continuous Arbitrary Rotations Parameters */}
      <div style={{ marginBottom: '1.5rem', padding: '0.75rem', background: 'rgba(255,255,255,0.02)', borderRadius: '4px', border: '1px solid #334155' }}>
        <label style={{ fontSize: '0.85rem', color: '#94a3b8', display: 'block', marginBottom: '0.5rem' }}>
          Arbitrary Rotation (Rx): <strong>{rotationAngle}°</strong>
        </label>
        <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
          <input 
            type="range" 
            min="0" 
            max="360" 
            value={rotationAngle} 
            onChange={(e) => handleSliderChange(Number(e.target.value))} 
            style={{ flex: 1, accentColor: '#3b82f6' }}
          />
          <button onClick={handleApplyRotationX} className="gate-btn-primary">Apply R_x</button>
        </div>
      </div>

      <button onClick={handleResetState} style={{ width: '100%', padding: '8px', background: '#df4747', border: 'none', borderRadius: '4px', color: '#fff', fontWeight: 600, cursor: 'pointer', transition: 'background 0.2s' }}>
        Reset Qubit State Vector Grounding
      </button>

      <style>{`
        .gate-btn {
          background: #1e293b;
          border: 1px solid #475569;
          color: #f8fafc;
          padding: 6px 12px;
          border-radius: 4px;
          cursor: pointer;
          font-family: monospace;
          transition: all 0.15s ease;
        }
        .gate-btn:hover {
          background: #334155;
          border-color: #3b82f6;
        }
        .gate-btn-primary {
          background: #3b82f6;
          border: none;
          color: #fff;
          padding: 6px 14px;
          border-radius: 4px;
          cursor: pointer;
          font-weight: 500;
        }
        .gate-btn-primary:hover {
          background: #2563eb;
        }
      `}</style>
    </div>
  );
}