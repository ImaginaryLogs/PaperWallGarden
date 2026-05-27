import React from 'react';

interface BlochSphereProps {
  quantumApi: any;
}

export default function BlochSphere({ quantumApi }: BlochSphereProps) {
  return (
    <div style={{ padding: '1rem', background: '#111827', borderRadius: '6px', border: '1px solid #334155' }}>
      <h4>🔮 Bloch Sphere Visualization</h4>
      <p style={{ fontSize: '0.85rem', color: '#94a3b8' }}>Rendering engine ready.</p>
    </div>
  );
}