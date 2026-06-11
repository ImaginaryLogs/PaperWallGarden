---
title: Quantum Communication — Dynamic Petri Net Workspace
tags: [simulation, quantum-communication, petri-net, wasm]
---

# Quantum Communication — Dynamic Petri Net Simulation

Drag your mouse across the canvas space to **Pan** through the network layout; use your scroll wheel to **Zoom**.

---

### Real-Time Pipeline Controller

<div id="qcomm-controls" style="margin: 1rem 0; display: flex; gap: 0.75rem; flex-wrap: wrap; background: #161b22; padding: 1rem; border-radius: 8px; border: 1px solid #30363d;">
  <button id="btn-tick" style="padding: 8px 16px; background: #21262d; color: #c9d1d9; border: 1px solid #30363d; border-radius: 6px; cursor: pointer; font-weight: 600;">Tick ×1</button>
  <button id="btn-run"  style="padding: 8px 16px; background: #238636; color: #ffffff; border: none; border-radius: 6px; cursor: pointer; font-weight: 600;">▶ Auto Run</button>
  <button id="btn-pause" style="padding: 8px 16px; background: #da3633; color: #ffffff; border: none; border-radius: 6px; cursor: pointer; font-weight: 600;" disabled>⏸ Pause</button>
  <button id="btn-reset" style="padding: 8px 16px; background: #21262d; color: #c9d1d9; border: 1px solid #30363d; border-radius: 6px; cursor: pointer; font-weight: 600;">↺ Reset</button>
</div>

<div style="position: relative; margin-bottom: 1rem; border: 1px solid #30363d; border-radius: 8px; overflow: hidden; background: #0d1117;">
  <canvas id="petri-canvas" style="width: 100%; height: 550px; display: block; cursor: grab;"></canvas>
</div>

<div style="background: #161b22; border: 1px solid #30363d; border-radius: 8px; padding: 1.25rem; margin-bottom: 1rem; font-family: monospace;">
  <h4 style="margin: 0 0 0.75rem 0; color: #58a6ff; font-size: 0.9rem;">Live Gate Transition Status Logs</h4>
  <div id="gate-debug-panel" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 0.75rem; font-size: 0.8rem; color: #c9d1d9;"></div>
</div>

<div id="qcomm-stats" style="font-size: 0.85rem; font-family: monospace; display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1rem; padding: 1.25rem; background: #161b22; border: 1px solid #30363d; border-radius: 8px; color: #c9d1d9;"></div>

<script type="module">
const isGitHubPages = window.location.hostname.includes('github.io');
const basePath = isGitHubPages ? '/PaperWallGarden' : '';

const PLACE_NAMES = [
  'EPR_A_Attempts', 'Alice_Memory_Ready', 'Repeater_Memory_L',
  'EPR_B_Attempts', 'Repeater_Memory_R', 'Bob_Memory_Ready',
  'Bell_Measure_Ready', 'Classical_Msg', 'Entangled_Alice_Bob',
  'Error_Detected', 'QEC_Recovery', 'Fidelity_Pool'
];

const TRANS_NAMES = [
  'T0: EPR_Gen_A', 'T1: Distribute_A', 'T2: EPR_Gen_B', 'T3: Distribute_B',
  'T4: Bell_Measure', 'T5: Classical_Send', 'T6: Entanglement_Done',
  'T7: Decoherence', 'T8: Error_Correct', 'T9: Purification'
];

async function start() {
  const wasmModule = await import(`${basePath}/wasm/qcomm_sim2.js`);
  const wasm = await wasmModule.default(`${basePath}/wasm/qcomm_sim2_bg.wasm`);
  const qc = wasmModule.QCommPetri.new();

  const canvas = document.getElementById('petri-canvas');
  const pctx = canvas.getContext('2d');
  
  let transform = { x: 50, y: 40, scale: 0.55 };
  let isDragging = false;
  let dragStart = { x: 0, y: 0 };

  function resize() {
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    pctx.scale(dpr, dpr);
  }
  window.addEventListener('resize', resize);
  resize();

  canvas.onmousedown = (e) => { isDragging = true; dragStart.x = e.clientX - transform.x; dragStart.y = e.clientY - transform.y; };
  window.onmouseup = () => { isDragging = false; };
  canvas.onmousemove = (e) => { if (isDragging) { transform.x = e.clientX - dragStart.x; transform.y = e.clientY - dragStart.y; renderAll(); } };
  canvas.onwheel = (e) => {
    e.preventDefault();
    const factor = Math.exp((e.deltaY < 0 ? 1 : -1) * 0.05);
    transform.scale = Math.max(0.15, Math.min(transform.scale * factor, 2.5));
    renderAll();
  };

  function renderAll() {
    const mem = wasm.memory.buffer;
    const numPlaces = qc.num_places();
    const numTrans = qc.num_transitions();

    // Map raw shared memory allocations directly from WebAssembly
    const tokens = new Uint32Array(mem, qc.tokens_ptr(), numPlaces);
    const placeX = new Float32Array(mem, qc.place_x_ptr(), numPlaces);
    const placeY = new Float32Array(mem, qc.place_y_ptr(), numPlaces);
    const transX = new Float32Array(mem, qc.trans_x_ptr(), numTrans);
    const transY = new Float32Array(mem, qc.trans_y_ptr(), numTrans);
    const inputs = new Int32Array(mem, qc.inputs_ptr(), numTrans * 3);
    const outputs = new Int32Array(mem, qc.outputs_ptr(), numTrans * 4);

    pctx.clearRect(0, 0, canvas.width, canvas.height);
    pctx.fillStyle = '#0d1117'; pctx.fillRect(0, 0, canvas.width, canvas.height);

    pctx.save();
    pctx.translate(transform.x, transform.y);
    pctx.scale(transform.scale, transform.scale);

    // 1. Structural Line Rendering (Reading dynamic pointers)
    pctx.lineWidth = 2.5;
    for (let t = 0; t < numTrans; t++) {
      for (let i = 0; i < 3; i++) {
        const p = inputs[t * 3 + i];
        if (p >= 0) {
          pctx.strokeStyle = tokens[p] > 0 ? '#58a6ff' : '#30363d';
          pctx.beginPath(); pctx.moveTo(placeX[p], placeY[p]); pctx.lineTo(transX[t], transY[t]); pctx.stroke();
        }
      }
      for (let o = 0; o < 4; o++) {
        const p = outputs[t * 4 + o];
        if (p >= 0) {
          pctx.strokeStyle = 'rgba(56, 139, 253, 0.2)';
          pctx.beginPath(); pctx.moveTo(transX[t], transY[t]); pctx.lineTo(placeX[p], placeY[p]); pctx.stroke();
        }
      }
    }

    // 2. Draw Token Repositories (Places)
    for (let p = 0; p < numPlaces; p++) {
      pctx.fillStyle = '#161b22'; pctx.strokeStyle = '#30363d'; pctx.lineWidth = 3;
      pctx.beginPath(); pctx.arc(placeX[p], placeY[p], 24, 0, Math.PI * 2); pctx.fill(); pctx.stroke();

      if (tokens[p] > 0) {
        pctx.fillStyle = '#58a6ff';
        pctx.beginPath(); pctx.arc(placeX[p], placeY[p], 8, 0, Math.PI * 2); pctx.fill();
        pctx.fillStyle = '#ffffff'; pctx.font = 'bold 12px monospace'; pctx.textAlign = 'center'; pctx.textBaseline = 'middle';
        pctx.fillText(tokens[p], placeX[p], placeY[p]);
      }
      pctx.fillStyle = '#8b949e'; pctx.font = '13px monospace'; pctx.textAlign = 'center';
      pctx.fillText(PLACE_NAMES[p], placeX[p], placeY[p] - 36);
    }

    // 3. System Gate Components (Transitions)
    let debugHTML = "";
    for (let t = 0; t < numTrans; t++) {
      let missingInputs = [];
      for (let i = 0; i < 3; i++) {
        const p = inputs[t * 3 + i];
        if (p >= 0 && tokens[p] === 0) missingInputs.push(PLACE_NAMES[p]);
      }

      const isReady = missingInputs.length === 0;
      pctx.fillStyle = isReady ? '#238636' : '#21262d';
      pctx.strokeStyle = isReady ? '#58a6ff' : '#30363d';
      if (t === 7) { pctx.fillStyle = '#8e1519'; pctx.strokeStyle = '#f85149'; }

      pctx.fillRect(transX[t] - 12, transY[t] - 24, 24, 48);
      pctx.strokeRect(transX[t] - 12, transY[t] - 24, 24, 48);

      pctx.fillStyle = '#ffffff'; pctx.font = '12px monospace'; pctx.textAlign = 'center';
      pctx.fillText(`T${t}`, transX[t], transY[t] + 4);

      pctx.fillStyle = '#c9d1d9'; pctx.font = '11px monospace'; pctx.textAlign = 'left';
      pctx.fillText(TRANS_NAMES[t].split(': ')[1], transX[t] + 18, transY[t] + 4);

      const statusColor = isReady ? '#238636' : (t === 7 ? '#da3633' : '#30363d');
      const statusText = isReady ? 'READY 🟢' : (t === 7 ? 'DECOHERENCE INTERRUPT ACTIVE' : `STALLED (Missing: ${missingInputs.map(s => s.replace('_Ready','')).join(', ')})`);
      debugHTML += `<div style="padding:0.5rem; background:#0d1117; border-radius:4px; border-left:4px solid ${statusColor};">
        <strong>${TRANS_NAMES[t]}</strong><br/><span style="color:#8b949e; font-size:0.75rem;">${statusText}</span>
      </div>`;
    }

    pctx.restore();
    document.getElementById('gate-debug-panel').innerHTML = debugHTML;

    document.getElementById('qcomm-stats').innerHTML = `
      <div><span style="color:#8b949e">Simulation Iterations:</span> ${qc.step_count()}</div>
      <div><span style="color:#8b949e">Links Built (T6):</span> <span style="color:#238636; font-weight:bold">${qc.entanglements_established()}</span></div>
      <div><span style="color:#8b949e">Decoherence Crashes (T7):</span> <span style="color:#f85149">${qc.decoherence_events()}</span></div>
      <div><span style="color:#8b949e">Fidelity Index Rating:</span> ${(qc.fidelity_score() * 100).toFixed(1)}%</div>
    `;
  }

  let running = false, rafId = null;
  document.getElementById('btn-tick').onclick = () => { qc.step(); renderAll(); };
  document.getElementById('btn-run').onclick = () => {
    if (running) return; running = true;
    document.getElementById('btn-run').disabled = true; document.getElementById('btn-pause').disabled = false;
    (function loop() { if (!running) return; qc.step(); renderAll(); rafId = requestAnimationFrame(loop); })();
  };
  document.getElementById('btn-pause').onclick = () => { running = false; cancelAnimationFrame(rafId); document.getElementById('btn-run').disabled = false; document.getElementById('btn-pause').disabled = true; };
  document.getElementById('btn-reset').onclick = () => { qc.reset(); renderAll(); };

  renderAll();
}
start();
</script>