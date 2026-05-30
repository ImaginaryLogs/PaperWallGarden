---
title: Federated Learning Convergence Simulator
tags: [simulation, federated-learning, machine-learning, wasm]
---

# Federated Learning Simulator

Simulates **FedAvg** (McMahan et al. 2017) over {{CLIENT_COUNT}} client nodes holding
non-IID private datasets. Watch how the global model converges across rounds
and how client divergence, dropout, and aggregation strategy affect training.

<div id="fl-controls" style="margin:0.5rem 0 1rem;display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;font-size:0.875rem;">
  <button id="btn-round"  style="padding:6px 14px;cursor:pointer;">Run 1 Round</button>
  <button id="btn-10"     style="padding:6px 14px;cursor:pointer;">Run 10 Rounds</button>
  <button id="btn-run"    style="padding:6px 14px;cursor:pointer;">▶ Auto</button>
  <button id="btn-pause"  style="padding:6px 14px;cursor:pointer;" disabled>⏸ Pause</button>
  <button id="btn-reset"  style="padding:6px 14px;cursor:pointer;">↺ Reset</button>
  <label>LR <input id="inp-lr"      type="range" min="1" max="20" value="5" style="width:70px"> <span id="lbl-lr">0.05</span></label>
  <label>Dropout <input id="inp-drop" type="range" min="0" max="50" value="10" style="width:70px"> <span id="lbl-drop">10%</span></label>
  <select id="sel-agg" style="padding:5px 8px;font-size:0.85rem;">
    <option value="0">FedAvg</option>
    <option value="1">FedProx</option>
    <option value="2">Median</option>
  </select>
</div>

<div style="display:grid;grid-template-columns:1fr 1fr;gap:1rem;margin-bottom:1rem;">
  <canvas id="canvas-loss" width="400" height="220"
    style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;">
  </canvas>
  <canvas id="canvas-clients" width="400" height="220"
    style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;">
  </canvas>
</div>

<canvas id="canvas-weights" width="800" height="120"
  style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;">
</canvas>

<div id="fl-stats" style="margin-top:0.75rem;font-size:0.82rem;opacity:0.65;font-family:monospace;display:flex;gap:2rem;flex-wrap:wrap;"></div>

<script type="module">
const isGitHubPages = window.location.hostname.includes('github.io');
const basePath = isGitHubPages ? '/PaperWallGarden' : '';
const WASM_JS = `${basePath}/wasm/federated_sim.js`;
const WASM_BG = `${basePath}/wasm/federated_sim_bg.wasm`;

const CLIENT_COLORS = ['#3b82f6','#8b5cf6','#ec4899','#f59e0b','#10b981','#ef4444'];
const CLIENT_LABELS = ['Hospital A','Hospital B','Clinic C','Clinic D','Lab E','Lab F'];

async function startPipeline() {
  let module, wasm, sim;
  try {
    module = await import(WASM_JS);
    wasm   = await module.default(WASM_BG);
    sim    = module.FederatedSim.new();
  } catch (err) {
    document.getElementById('fl-stats').textContent = `WASM load failed: ${err}`;
    return;
  }

  const lossCanvas    = document.getElementById('canvas-loss');
  const clientCanvas  = document.getElementById('canvas-clients');
  const weightsCanvas = document.getElementById('canvas-weights');
  const lctx = lossCanvas.getContext('2d');
  const cctx = clientCanvas.getContext('2d');
  const wctx = weightsCanvas.getContext('2d');
  const statsEl = document.getElementById('fl-stats');

  // ============================================================
  // Zero-copy views
  // ============================================================
  function getViews() {
    const mem = wasm.memory.buffer;
    return {
      lossHistory:   new Float32Array(mem, sim.loss_history_ptr(),    sim.max_rounds()),
      clientLosses:  new Float32Array(mem, sim.client_losses_ptr(),   sim.client_count()),
      participation: new Uint8Array(mem,   sim.participation_ptr(),   sim.client_count()),
      globalWeights: new Float32Array(mem, sim.global_weights_ptr(),  sim.model_size()),
    };
  }

  // ============================================================
  // Rendering
  // ============================================================
  function drawLossChart(v) {
    const W = lossCanvas.width, H = lossCanvas.height;
    const round = sim.current_round();
    lctx.clearRect(0,0,W,H);

    // Axis labels
    lctx.fillStyle = 'rgba(148,163,184,0.6)';
    lctx.font = '11px monospace';
    lctx.fillText('Global Loss per Round', 10, 18);

    if (round === 0) return;

    const pad = {l:40, r:10, t:28, b:30};
    const losses = Array.from(v.lossHistory.slice(0, round)).filter(x => !isNaN(x));
    const maxL   = Math.max(...losses, 0.01);

    lctx.strokeStyle = 'rgba(148,163,184,0.2)';
    lctx.lineWidth   = 0.5;
    [0.25,0.5,0.75,1.0].forEach(frac => {
      const y = pad.t + (H - pad.t - pad.b) * (1 - frac / 1.0 * (maxL / maxL));
      // grid line at loss = frac*maxL
      const ly = pad.t + (H - pad.t - pad.b) * (1 - frac);
      lctx.beginPath(); lctx.moveTo(pad.l, ly); lctx.lineTo(W - pad.r, ly); lctx.stroke();
      lctx.fillStyle = 'rgba(148,163,184,0.4)';
      lctx.font = '9px monospace';
      lctx.textAlign = 'right';
      lctx.fillText((maxL * frac).toFixed(3), pad.l - 4, ly + 3);
    });

    // Loss curve
    lctx.beginPath();
    losses.forEach((l, i) => {
      const x = pad.l + i * (W - pad.l - pad.r) / (sim.max_rounds() - 1);
      const y = pad.t + (H - pad.t - pad.b) * (1 - l / maxL);
      i === 0 ? lctx.moveTo(x, y) : lctx.lineTo(x, y);
    });
    lctx.strokeStyle = '#60a5fa';
    lctx.lineWidth   = 1.5;
    lctx.stroke();

    // Fill under curve
    lctx.lineTo(pad.l + (losses.length-1)*(W-pad.l-pad.r)/(sim.max_rounds()-1), H-pad.b);
    lctx.lineTo(pad.l, H-pad.b);
    lctx.closePath();
    lctx.fillStyle = 'rgba(96,165,250,0.1)';
    lctx.fill();

    lctx.fillStyle = 'rgba(148,163,184,0.5)';
    lctx.font = '10px monospace';
    lctx.textAlign = 'center';
    lctx.fillText(`Round ${round} / ${sim.max_rounds()}`, W/2, H - 6);
  }

  function drawClients(v) {
    const W = clientCanvas.width, H = clientCanvas.height;
    cctx.clearRect(0,0,W,H);
    cctx.fillStyle = 'rgba(148,163,184,0.6)';
    cctx.font = '11px monospace';
    cctx.fillText('Client Status', 10, 18);

    const n = sim.client_count();
    const barW = Math.floor((W - 60) / n) - 8;
    const maxLoss = Math.max(...Array.from(v.clientLosses), 0.1);

    for (let c = 0; c < n; c++) {
      const loss  = v.clientLosses[c];
      const alive = v.participation[c] === 1;
      const divergence = sim.client_divergence(c);
      const x    = 30 + c * ((W - 60) / n);
      const barH = Math.max(4, (H - 80) * (loss / maxLoss));
      const y    = H - 40 - barH;

      // Bar
      cctx.fillStyle = alive ? CLIENT_COLORS[c % CLIENT_COLORS.length] : 'rgba(100,100,100,0.3)';
      cctx.globalAlpha = alive ? 0.85 : 0.3;
      cctx.fillRect(x, y, barW, barH);
      cctx.globalAlpha = 1.0;

      // Dropout indicator
      if (!alive) {
        cctx.fillStyle = '#ef4444';
        cctx.font = '14px monospace';
        cctx.textAlign = 'center';
        cctx.fillText('✕', x + barW/2, y - 4);
      }

      // Labels
      cctx.fillStyle = 'rgba(226,232,240,0.6)';
      cctx.font = '9px monospace';
      cctx.textAlign = 'center';
      cctx.fillText(CLIENT_LABELS[c].split(' ')[1], x + barW/2, H - 26);
      cctx.fillText(`L:${loss.toFixed(3)}`, x + barW/2, H - 14);
      cctx.fillStyle = 'rgba(251,191,36,0.7)';
      cctx.fillText(`d:${divergence.toFixed(2)}`, x + barW/2, H - 2);
    }
  }

  function drawWeights(v) {
    const W = weightsCanvas.width, H = weightsCanvas.height;
    wctx.clearRect(0,0,W,H);
    wctx.fillStyle = 'rgba(148,163,184,0.6)';
    wctx.font = '11px monospace';
    wctx.fillText('Global Model Weights Heatmap', 10, 16);

    const n = v.globalWeights.length;
    const cellW = (W - 80) / n;
    const maxAbs = Math.max(...Array.from(v.globalWeights).map(Math.abs), 0.01);

    for (let i = 0; i < n; i++) {
      const w = v.globalWeights[i];
      const norm = w / maxAbs; // -1..1
      const r = norm > 0 ? Math.round(norm * 200) : 0;
      const b = norm < 0 ? Math.round(-norm * 200) : 0;
      const x = 40 + i * cellW;
      wctx.fillStyle = `rgb(${r},60,${b})`;
      wctx.fillRect(x, 28, Math.max(1, cellW - 1), H - 48);
      if (cellW > 14) {
        wctx.fillStyle = 'rgba(255,255,255,0.7)';
        wctx.font = '8px monospace';
        wctx.textAlign = 'center';
        wctx.fillText(w.toFixed(2), x + cellW/2, H - 10);
      }
    }

    wctx.fillStyle = 'rgba(148,163,184,0.4)';
    wctx.font = '9px monospace';
    wctx.textAlign = 'left';
    wctx.fillText('↑+', 2, H/2);
    wctx.fillText('↓−', 2, H/2 + 14);
    const norm = sim.global_model_norm();
    wctx.textAlign = 'right';
    wctx.fillText(`‖W‖₁: ${norm.toFixed(4)}`, W - 4, H - 4);
  }

  function updateStats(v) {
    statsEl.innerHTML = [
      `Round: <b>${sim.current_round()}</b>`,
      `Avg Loss: <b>${(Array.from(v.clientLosses).reduce((a,b)=>a+b,0)/sim.client_count()).toFixed(4)}</b>`,
      `Active Clients: <b>${Array.from(v.participation).filter(x=>x).length} / ${sim.client_count()}</b>`,
      `‖W‖: <b>${sim.global_model_norm().toFixed(4)}</b>`,
    ].map(s => `<span>${s}</span>`).join('');
  }

  function renderAll() {
    const v = getViews();
    drawLossChart(v);
    drawClients(v);
    drawWeights(v);
    updateStats(v);
  }

  renderAll();

  // ============================================================
  // Controls
  // ============================================================
  let autoRunning = false, rafId = null;

  function autoLoop() {
    sim.run_round();
    renderAll();
    if (autoRunning && sim.current_round() < sim.max_rounds()) {
      rafId = setTimeout(autoLoop, 80);
    } else {
      autoRunning = false;
      document.getElementById('btn-run').disabled   = false;
      document.getElementById('btn-pause').disabled = true;
    }
  }

  document.getElementById('btn-round').onclick = () => { sim.run_round(); renderAll(); };
  document.getElementById('btn-10').onclick    = () => { sim.run_rounds(10); renderAll(); };

  document.getElementById('btn-run').onclick = () => {
    autoRunning = true;
    document.getElementById('btn-run').disabled   = true;
    document.getElementById('btn-pause').disabled = false;
    autoLoop();
  };
  document.getElementById('btn-pause').onclick = () => {
    autoRunning = false;
    clearTimeout(rafId);
    document.getElementById('btn-run').disabled   = false;
    document.getElementById('btn-pause').disabled = true;
  };
  document.getElementById('btn-reset').onclick = () => {
    autoRunning = false; clearTimeout(rafId);
    sim.reset();
    document.getElementById('btn-run').disabled   = false;
    document.getElementById('btn-pause').disabled = true;
    renderAll();
  };

  document.getElementById('inp-lr').oninput = function() {
    const v = this.value / 100;
    sim.set_lr(v);
    document.getElementById('lbl-lr').textContent = v.toFixed(2);
  };
  document.getElementById('inp-drop').oninput = function() {
    const v = this.value / 100;
    sim.set_dropout(v);
    document.getElementById('lbl-drop').textContent = `${this.value}%`;
  };
  document.getElementById('sel-agg').onchange = function() {
    sim.set_agg_method(parseInt(this.value));
  };
}

startPipeline();
</script>