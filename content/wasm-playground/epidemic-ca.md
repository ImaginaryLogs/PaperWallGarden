---
title: System Automata — Viral Graph Cellular Automaton
tags: [simulation, cellular-automata, complex-systems, wasm]
---

# Epidemic Spread — Graph Cellular Automaton (SIR Model)

Unlike Petri Nets which move *tokens*, a **Graph-Based Cellular Automaton** updates the *state* of nodes synchronously based on their neighbors. 

This models the classic **SIR Epidemic Model** across an irregular social network containing hubs and isolated clusters. Adjusting the transmission and recovery rates lets you visualize concepts like "Flattening the Curve" and Herd Immunity.

---

### Biological Controller

<div id="ca-controls" style="margin: 1rem 0; display: flex; gap: 0.75rem; flex-wrap: wrap; background: #161b22; padding: 1rem; border-radius: 8px; border: 1px solid #30363d;">
  <button id="btn-tick" style="padding: 8px 16px; background: #21262d; color: #c9d1d9; border: 1px solid #30363d; border-radius: 6px; cursor: pointer; font-weight: 600;">Tick ×1</button>
  <button id="btn-run"  style="padding: 8px 16px; background: #238636; color: #ffffff; border: none; border-radius: 6px; cursor: pointer; font-weight: 600;">▶ Auto Run</button>
  <button id="btn-pause" style="padding: 8px 16px; background: #da3633; color: #ffffff; border: none; border-radius: 6px; cursor: pointer; font-weight: 600;" disabled>⏸ Pause</button>
  <button id="btn-reset" style="padding: 8px 16px; background: #21262d; color: #c9d1d9; border: 1px solid #30363d; border-radius: 6px; cursor: pointer; font-weight: 600;">↺ Outbreak Reset</button>
  
  <div style="display: flex; align-items: center; gap: 1rem; margin-left: auto; padding-left: 1rem; border-left: 1px solid #30363d;">
    <label style="font-weight: 600; font-size: 0.85rem; color: #8b949e; display:flex; flex-direction:column;">
      Transmission (β):
      <input id="slide-trans" type="range" min="0.01" max="0.5" step="0.01" value="0.15" style="width: 100px; accent-color: #f85149;">
    </label>
    <label style="font-weight: 600; font-size: 0.85rem; color: #8b949e; display:flex; flex-direction:column;">
      Recovery (γ):
      <input id="slide-recov" type="range" min="0.01" max="0.2" step="0.01" value="0.05" style="width: 100px; accent-color: #2ea44f;">
    </label>
  </div>
</div>

<div style="display: grid; grid-template-columns: 2fr 1fr; gap: 1rem; margin-bottom: 1rem;">
  <div style="position: relative; border: 1px solid #30363d; border-radius: 8px; overflow: hidden; background: #0d1117;">
    <div style="position: absolute; top: 12px; left: 14px; font-family: monospace; font-size: 0.75rem; color: #8b949e; pointer-events: none; text-transform: uppercase; font-weight: 600; z-index: 10;">Social Topology Graph (Drag to Pan, Scroll to Zoom)</div>
    <canvas id="ca-canvas" style="width: 100%; height: 380px; display: block; cursor: grab;"></canvas>
  </div>
  
  <div style="position: relative; border: 1px solid #30363d; border-radius: 8px; overflow: hidden; background: #0d1117;">
    <div style="position: absolute; top: 12px; left: 14px; font-family: monospace; font-size: 0.75rem; color: #8b949e; pointer-events: none; text-transform: uppercase; font-weight: 600;">SIR Distribution Curve</div>
    <canvas id="chart-canvas" style="width: 100%; height: 380px; display: block;"></canvas>
  </div>
</div>

<script type="module">
const isGitHubPages = window.location.hostname.includes('github.io');
const basePath = isGitHubPages ? '/PaperWallGarden' : '';

function setupHQCanvas(canvas, logicalWidth, logicalHeight) {
  const dpr = window.devicePixelRatio || 1;
  canvas.width = logicalWidth * dpr; canvas.height = logicalHeight * dpr;
  canvas.style.width = `${logicalWidth}px`; canvas.style.height = `${logicalHeight}px`;
  const ctx = canvas.getContext('2d'); ctx.scale(dpr, dpr); return ctx;
}

const COLORS = ['#58a6ff', '#f85149', '#3fb950']; // S: Blue, I: Red, R: Green

async function start() {
  const wasmModule = await import(`${basePath}/wasm/epidemic_ca.js`);
  const wasm = await wasmModule.default(`${basePath}/wasm/epidemic_ca_bg.wasm`);
  const engine = wasmModule.EpidemicCA.new();

  const canvasNode = document.getElementById('ca-canvas');
  const cNode = setupHQCanvas(canvasNode, 600, 380);
  const cChart = setupHQCanvas(document.getElementById('chart-canvas'), 300, 380);
  
  let running = false, rafId = null;

  // --- VIEW TRANSFORMS FOR PAN & ZOOM ---
  let panX = -120; // Center graph layout inside canvas bounds initially
  let panY = 0;
  let zoom = 1.0;
  let isDragging = false;
  let startX = 0, startY = 0;

  // Drag to Pan Event Listeners
  canvasNode.addEventListener('mousedown', (e) => {
    isDragging = true;
    canvasNode.style.cursor = 'grabbing';
    startX = e.clientX - panX;
    startY = e.clientY - panY;
  });

  window.addEventListener('mousemove', (e) => {
    if (!isDragging) return;
    panX = e.clientX - startX;
    panY = e.clientY - startY;
    render();
  });

  window.addEventListener('mouseup', () => {
    if (isDragging) {
      isDragging = false;
      canvasNode.style.cursor = 'grab';
    }
  });

  // Wheel to Zoom Event Listener
  canvasNode.addEventListener('wheel', (e) => {
    e.preventDefault();
    const zoomFactor = 1.1;
    
    // Get mouse position relative to canvas
    const rect = canvasNode.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    // Calculate mouse position in workspace coordinates before zoom change
    const worldX = (mouseX - panX) / zoom;
    const worldY = (mouseY - panY) / zoom;

    if (e.deltaY < 0) {
      zoom *= zoomFactor;
    } else {
      zoom /= zoomFactor;
    }
    // Constrain zoom levels
    zoom = Math.max(0.3, Math.min(zoom, 4.0));

    // Recalculate pan positions so the cursor location remains pinned during scaling
    panX = mouseX - worldX * zoom;
    panY = mouseY - worldY * zoom;

    render();
  });

  function render() {
    const mem = wasm.memory.buffer;
    const nN = engine.num_nodes();
    const nE = engine.num_edges();
    const hLen = engine.hist_len();

    const states = new Uint8Array(mem, engine.states_ptr(), nN);
    const pX = new Float32Array(mem, engine.node_x_ptr(), nN);
    const pY = new Float32Array(mem, engine.node_y_ptr(), nN);
    const edges = new Uint32Array(mem, engine.edges_ptr(), nE * 2);
    
    const hS = new Uint32Array(mem, engine.hist_s_ptr(), hLen);
    const hI = new Uint32Array(mem, engine.hist_i_ptr(), hLen);
    const hR = new Uint32Array(mem, engine.hist_r_ptr(), hLen);

    // --- RENDER NETWORK GRAPH ---
    cNode.clearRect(0, 0, 600, 380);
    cNode.fillStyle = '#0d1117'; cNode.fillRect(0, 0, 600, 380);

    // Push view transform matrix state
    cNode.save();
    cNode.translate(panX, panY);
    cNode.scale(zoom, zoom);

    // Draw Edges (Contagion Vectors)
    for (let e = 0; e < nE; e++) {
      const n1 = edges[e * 2];
      const n2 = edges[e * 2 + 1];
      
      const isThreat = (states[n1] === 1 && states[n2] === 0) || (states[n1] === 0 && states[n2] === 1);
      cNode.strokeStyle = isThreat ? 'rgba(248, 81, 73, 0.6)' : 'rgba(48, 54, 61, 0.8)';
      cNode.lineWidth = isThreat ? 2.5 / zoom : 1.5 / zoom; // Maintain sharp visibility across zooms

      cNode.beginPath(); cNode.moveTo(pX[n1], pY[n1]); cNode.lineTo(pX[n2], pY[n2]); cNode.stroke();
    }

    // Draw Nodes (Individuals)
    for (let i = 0; i < nN; i++) {
      cNode.fillStyle = COLORS[states[i]];
      cNode.strokeStyle = '#0d1117';
      cNode.lineWidth = 3 / zoom;
      cNode.beginPath(); cNode.arc(pX[i], pY[i], 12, 0, Math.PI * 2); cNode.fill(); cNode.stroke();
      
      cNode.fillStyle = '#ffffff';
      cNode.beginPath(); cNode.arc(pX[i], pY[i], 3, 0, Math.PI * 2); cNode.fill();
    }

    // Pop view transform matrix state
    cNode.restore();

    // --- RENDER SIR CHART ---
    cChart.clearRect(0, 0, 300, 380);
    cChart.fillStyle = '#161b22'; cChart.fillRect(0, 0, 300, 380);
    
    if (hLen > 0) {
      const stepX = 300 / 100;
      const scaleY = 320 / nN;

      const drawLine = (dataArray, color) => {
        cChart.strokeStyle = color; cChart.lineWidth = 3;
        cChart.beginPath();
        for (let i = 0; i < hLen; i++) {
          const x = i * stepX;
          const y = 360 - (dataArray[i] * scaleY);
          if (i === 0) cChart.moveTo(x, y); else cChart.lineTo(x, y);
        }
        cChart.stroke();
      };

      drawLine(hR, COLORS[2]);
      drawLine(hS, COLORS[0]);
      drawLine(hI, COLORS[1]);
      
      cChart.fillStyle = COLORS[0]; cChart.font = '11px monospace'; cChart.fillText(`S: ${hS[hLen-1]}`, 10, 370);
      cChart.fillStyle = COLORS[1]; cChart.fillText(`I: ${hI[hLen-1]}`, 60, 370);
      cChart.fillStyle = COLORS[2]; cChart.fillText(`R: ${hR[hLen-1]}`, 110, 370);
    }
  }

  document.getElementById('btn-tick').onclick = () => { engine.step(); render(); };
  document.getElementById('btn-run').onclick = () => {
    if (running) return; running = true;
    document.getElementById('btn-run').disabled = true; document.getElementById('btn-pause').disabled = false;
    (function loop() { if (!running) return; engine.step(); render(); rafId = setTimeout(loop, 150); })();
  };
  document.getElementById('btn-pause').onclick = () => { running = false; clearTimeout(rafId); document.getElementById('btn-run').disabled = false; document.getElementById('btn-pause').disabled = true; };
  document.getElementById('btn-reset').onclick = () => { engine.reset(); render(); };
  
  document.getElementById('slide-trans').oninput = (e) => { engine.set_transmission(parseFloat(e.target.value)); render(); };
  document.getElementById('slide-recov').oninput = (e) => { engine.set_recovery(parseFloat(e.target.value)); render(); };

  render();
}
start();
</script>