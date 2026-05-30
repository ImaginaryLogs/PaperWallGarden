---
title: System Automata — Smart Coffee Shop Pipeline
tags: [simulation, queuing-theory, petri-net, wasm]
---

# Smart Coffee Shop — Queuing Theory & Feedback Automata

This simulation models a simplified business process: **A Coffee Shop Barista Pipeline**. 
Unlike static code, this system features an embedded **Feedback Loop**: if the `Order_Queue` backs up past 5 people, new customers stop walking in, and current customers' patience expires, funneling them into the `Frustrated_Walkouts` bucket.

---

### Operations Controller

<div id="cafe-controls" style="margin: 1rem 0; display: flex; gap: 0.75rem; flex-wrap: wrap; background: #161b22; padding: 1rem; border-radius: 8px; border: 1px solid #30363d;">
  <button id="btn-tick" style="padding: 8px 16px; background: #21262d; color: #c9d1d9; border: 1px solid #30363d; border-radius: 6px; cursor: pointer; font-weight: 600;">Tick ×1</button>
  <button id="btn-run"  style="padding: 8px 16px; background: #238636; color: #ffffff; border: none; border-radius: 6px; cursor: pointer; font-weight: 600;">▶ Auto Run</button>
  <button id="btn-pause" style="padding: 8px 16px; background: #da3633; color: #ffffff; border: none; border-radius: 6px; cursor: pointer; font-weight: 600;" disabled>⏸ Pause</button>
  <button id="btn-reset" style="padding: 8px 16px; background: #21262d; color: #c9d1d9; border: 1px solid #30363d; border-radius: 6px; cursor: pointer; font-weight: 600;">↺ Reset Shift</button>
  
  <div style="display: flex; align-items: center; gap: 0.75rem; margin-left: auto; padding-left: 1rem; border-left: 1px solid #30363d;">
    <label for="slide-rate" style="font-weight: 600; font-size: 0.85rem; color: #8b949e;">Customer Arrival Rate:</label>
    <input id="slide-rate" type="range" min="0.5" max="8.0" step="0.5" value="2.5" style="width: 140px; accent-color: #58a6ff;">
  </div>
</div>

<div style="position: relative; margin-bottom: 1rem; border: 1px solid #30363d; border-radius: 8px; overflow: hidden; background: #0d1117;">
  <canvas id="cafe-canvas" style="width: 100%; height: 480px; display: block;"></canvas>
</div>

<div id="cafe-stats" style="font-size: 0.9rem; font-family: monospace; display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1rem; padding: 1.25rem; background: #161b22; border: 1px solid #30363d; border-radius: 8px; color: #c9d1d9;"></div>

<script type="module">
const isGitHubPages = window.location.hostname.includes('github.io');
const basePath = isGitHubPages ? '/PaperWallGarden' : '';

const PLACE_NAMES = ['Order_Queue', 'Beans_Hopper', 'Completed_Drinks', 'Frustrated_Walkouts'];
const TRANS_NAMES = ['T0: Customer_Arrival', 'T1: Grind_Beans', 'T2: Brew_Espresso', 'T3: Patience_Expire'];

function setupHQCanvas(canvas, logicalWidth, logicalHeight) {
  const dpr = window.devicePixelRatio || 1;
  canvas.width = logicalWidth * dpr; canvas.height = logicalHeight * dpr;
  canvas.style.width = `${logicalWidth}px`; canvas.style.height = `${logicalHeight}px`;
  const ctx = canvas.getContext('2d'); ctx.scale(dpr, dpr); return ctx;
}

async function start() {
  const wasmModule = await import(`${basePath}/wasm/cafe_sim.js`);
  const wasm = await wasmModule.default(`${basePath}/wasm/cafe_sim_bg.wasm`);
  const engine = wasmModule.CafePetri.new();

  const canvas = document.getElementById('cafe-canvas');
  const ctx = setupHQCanvas(canvas, 800, 480);
  let running = false, rafId = null;

  function renderAll() {
    const mem = wasm.memory.buffer;
    const nP = engine.num_places();
    const nT = engine.num_transitions();

    const tokens = new Uint32Array(mem, engine.tokens_ptr(), nP);
    const pX = new Float32Array(mem, engine.place_x_ptr(), nP);
    const pY = new Float32Array(mem, engine.place_y_ptr(), nP);
    const tX = new Float32Array(mem, engine.trans_x_ptr(), nT);
    const tY = new Float32Array(mem, engine.trans_y_ptr(), nT);
    const inputs = new Int32Array(mem, engine.inputs_ptr(), nT * 2);
    const outputs = new Int32Array(mem, engine.outputs_ptr(), nT * 2);
    const lambdas = new Float32Array(mem, engine.lambda_effective_ptr(), nT);

    ctx.clearRect(0, 0, 800, 480);
    ctx.fillStyle = '#0d1117'; ctx.fillRect(0, 0, 800, 480);

    // 1. Draw Network Edges
    ctx.lineWidth = 2.5;
    for (let t = 0; t < nT; t++) {
      for (let i = 0; i < 2; i++) {
        const p = inputs[t * 2 + i];
        if (p >= 0) {
          ctx.strokeStyle = tokens[p] > 0 ? '#58a6ff' : '#30363d';
          ctx.beginPath(); ctx.moveTo(pX[p], pY[p]); ctx.lineTo(tX[t], tY[t]); ctx.stroke();
        }
      }
      for (let o = 0; o < 2; o++) {
        const p = outputs[t * 2 + o];
        if (p >= 0) {
          ctx.strokeStyle = 'rgba(56, 139, 253, 0.2)';
          ctx.beginPath(); ctx.moveTo(tX[t], tY[t]); ctx.lineTo(pX[p], pY[p]); ctx.stroke();
        }
      }
    }

    // 2. Draw Network Nodes (Places)
    for (let p = 0; p < nP; p++) {
      ctx.fillStyle = '#161b22'; ctx.strokeStyle = '#30363d'; ctx.lineWidth = 3;
      ctx.beginPath(); ctx.arc(pX[p], pY[p], 26, 0, Math.PI * 2); ctx.fill(); ctx.stroke();

      if (tokens[p] > 0) {
        ctx.fillStyle = p === 3 ? '#f85149' : '#58a6ff'; // Walkouts are red
        ctx.beginPath(); ctx.arc(pX[p], pY[p], 9, 0, Math.PI * 2); ctx.fill();
        ctx.fillStyle = '#ffffff'; ctx.font = 'bold 13px monospace'; ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
        ctx.fillText(tokens[p], pX[p], pY[p]);
      }
      ctx.fillStyle = '#8b949e'; ctx.font = '13px monospace'; ctx.textAlign = 'center';
      ctx.fillText(PLACE_NAMES[p], pX[p], pY[p] - 40);
    }

    // 3. Draw Engine Gates (Transitions)
    for (let t = 0; t < nT; t++) {
      let isBlocked = false;
      for (let i = 0; i < 2; i++) {
        const p = inputs[t * 2 + i];
        if (p >= 0 && tokens[p] === 0) isBlocked = true;
      }

      ctx.fillStyle = isBlocked ? '#21262d' : '#238636';
      ctx.strokeStyle = isBlocked ? '#30363d' : '#58a6ff';
      if (t === 3 && !isBlocked) { ctx.fillStyle = '#8e1519'; ctx.strokeStyle = '#f85149'; } // Walkout actively firing

      ctx.fillRect(tX[t] - 14, tY[t] - 28, 28, 56);
      ctx.strokeRect(tX[t] - 14, tY[t] - 28, 28, 56);

      ctx.fillStyle = '#ffffff'; ctx.font = '12px monospace'; ctx.textAlign = 'center';
      ctx.fillText(`T${t}`, tX[t], tY[t] + 4);
      
      ctx.fillStyle = '#c9d1d9'; ctx.font = '12px monospace'; ctx.textAlign = 'left';
      ctx.fillText(TRANS_NAMES[t].split(': ')[1], tX[t] + 24, tY[t] + 4);
      
      // Show Dynamic Rates (Feedback System)
      ctx.fillStyle = '#8b949e'; ctx.font = '10px monospace';
      ctx.fillText(`λ = ${lambdas[t].toFixed(1)}`, tX[t] + 24, tY[t] + 20);
    }

    document.getElementById('cafe-stats').innerHTML = `
      <div><span style="color:#8b949e">Simulation Ticks:</span> ${engine.step_count()}</div>
      <div><span style="color:#8b949e">Current Queue Length:</span> <span style="color:${tokens[0] > 5 ? '#f85149' : '#58a6ff'}; font-weight:bold">${tokens[0]}</span></div>
      <div><span style="color:#8b949e">Completed Drinks:</span> <span style="color:#238636; font-weight:bold">${tokens[2]}</span></div>
      <div><span style="color:#8b949e">Lost Customers:</span> <span style="color:#f85149; font-weight:bold">${tokens[3]}</span></div>
    `;
  }

  document.getElementById('btn-tick').onclick = () => { engine.step(); renderAll(); };
  document.getElementById('btn-run').onclick = () => {
    if (running) return; running = true;
    document.getElementById('btn-run').disabled = true; document.getElementById('btn-pause').disabled = false;
    (function loop() { if (!running) return; engine.step(); renderAll(); rafId = requestAnimationFrame(loop); })();
  };
  document.getElementById('btn-pause').onclick = () => { running = false; cancelAnimationFrame(rafId); document.getElementById('btn-run').disabled = false; document.getElementById('btn-pause').disabled = true; };
  document.getElementById('btn-reset').onclick = () => { engine.reset(); renderAll(); };
  document.getElementById('slide-rate').oninput = (e) => { engine.set_arrival_rate(parseFloat(e.target.value)); renderAll(); };

  renderAll();
}
start();
</script>