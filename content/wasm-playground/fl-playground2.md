---
title: Federated Learning — Grid Cellular Automaton
tags: [simulation, federated-learning, cellular-automata, grid-ca, wasm]
---

# Federated Learning as a Grid CA

This simulation models **Federated Learning (FL)** as a **Grid-Based Cellular Automaton**.
Each cell is a device (hospital, phone, IoT node) holding a private model parameter
and private data it is trying to fit. Devices interact *only with immediate neighbours* —
there is no central server. Global model consensus **emerges from local neighbourhood
gossip**, exactly like Conway's Game of Life, but the "alive/dead" state is replaced
by a continuous model weight and a loss value.

**Why is this a CA?** Each cell's next state is determined entirely by (a) its own current
state and (b) the states of its 8 Moore-neighbourhood cells. No global view. No server.
Yet the grid converges — watch the colour gradients stabilise.

**Colour map:** blue = converged (low loss) · green = training · orange = aggregating · red = drifted/diverged

<div id="fl-controls" style="margin:0.5rem 0 1rem;display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;font-size:0.875rem;">
  <button id="btn-step"   style="padding:6px 14px;cursor:pointer;">Step ×1</button>
  <button id="btn-step10" style="padding:6px 14px;cursor:pointer;">Step ×10</button>
  <button id="btn-run"    style="padding:6px 14px;cursor:pointer;">▶ Run</button>
  <button id="btn-pause"  style="padding:6px 14px;cursor:pointer;" disabled>⏸ Pause</button>
  <button id="btn-reset"  style="padding:6px 14px;cursor:pointer;">↺ Reset</button>
  <label>Gossip weight
    <input id="inp-gossip" type="range" min="0" max="95" value="40" style="width:80px">
    <span id="lbl-gossip">0.40</span>
  </label>
  <label>LR
    <input id="inp-lr" type="range" min="1" max="50" value="8" style="width:60px">
    <span id="lbl-lr">0.08</span>
  </label>
  <span style="font-size:0.8rem;opacity:0.6;">Click grid to inject Byzantine node</span>
</div>

<div style="display:grid;grid-template-columns:2fr 1fr;gap:1rem;margin-bottom:1rem;">
  <canvas id="fl-grid" width="512" height="384"
    style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;cursor:crosshair;">
  </canvas>
  <canvas id="fl-loss-chart" width="280" height="384"
    style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;">
  </canvas>
</div>

<div id="fl-stats" style="font-size:0.82rem;opacity:0.65;font-family:monospace;display:flex;gap:2rem;flex-wrap:wrap;"></div>

<script type="module">
const isGitHubPages = window.location.hostname.includes('github.io');
const basePath = isGitHubPages ? '/PaperWallGarden' : '';
const WASM_JS = `${basePath}/wasm/federated_sim2.js`;
const WASM_BG = `${basePath}/wasm/federated_sim2_bg.wasm`;

// State colours: converged=blue, aggregating=cyan, training=green, idle=grey, drifted=red
const STATE_COLORS = ['#475569','#22d3ee','#4ade80','#fbbf24','#ef4444'];

async function start() {
  let module, wasm, sim;
  try {
    module = await import(WASM_JS);
    wasm   = await module.default(WASM_BG);
    sim    = module.FederatedGridCA.new();
  } catch(e) {
    document.getElementById('fl-stats').textContent = `WASM load failed: ${e}`;
    return;
  }

  const gridCanvas = document.getElementById('fl-grid');
  const chartCanvas = document.getElementById('fl-loss-chart');
  const gctx = gridCanvas.getContext('2d');
  const cctx = chartCanvas.getContext('2d');

  const W = sim.width(), H = sim.height(), N = sim.n();
  const cellW = gridCanvas.width / W;
  const cellH = gridCanvas.height / H;

  const lossHistory = [];
  let running = false, rafId = null;

  function getViews() {
    return {
      model:     new Float32Array(wasm.memory.buffer, sim.model_ptr(),     N),
      loss:      new Float32Array(wasm.memory.buffer, sim.loss_ptr(),      N),
      state:     new Uint8Array  (wasm.memory.buffer, sim.state_ptr(),     N),
      trust:     new Float32Array(wasm.memory.buffer, sim.trust_ptr(),     N),
      byzantine: new Uint8Array  (wasm.memory.buffer, sim.byzantine_ptr(), N),
    };
  }

  function renderGrid() {
    const v = getViews();
    for (let i = 0; i < N; i++) {
      const col = i % W, row = Math.floor(i / W);
      const x = col * cellW, y = row * cellH;
      // Background: model value as heatmap
      const m = v.model[i];
      const r = Math.round(255 * m);
      const b = Math.round(255 * (1 - m));
      gctx.fillStyle = `rgb(${r},80,${b})`;
      gctx.fillRect(x, y, cellW, cellH);
      // Overlay: state colour with alpha
      gctx.fillStyle = STATE_COLORS[v.state[i]] + '80';
      gctx.fillRect(x, y, cellW, cellH);
      // Byzantine highlight
      if (v.byzantine[i]) {
        gctx.strokeStyle = '#fbbf24';
        gctx.lineWidth = 2;
        gctx.strokeRect(x+1, y+1, cellW-2, cellH-2);
      }
      // Trust: dim low-trust cells
      if (v.trust[i] < 0.5) {
        gctx.fillStyle = `rgba(0,0,0,${0.5 - v.trust[i]})`;
        gctx.fillRect(x, y, cellW, cellH);
      }
    }
  }

  function renderChart() {
    const cw = chartCanvas.width, ch = chartCanvas.height;
    cctx.fillStyle = '#0d1117';
    cctx.fillRect(0, 0, cw, ch);
    // Title
    cctx.fillStyle = '#94a3b8';
    cctx.font = '11px monospace';
    cctx.fillText('Mean Loss per Round', 8, 18);
    if (lossHistory.length < 2) return;
    const maxLoss = 0.5;
    const padL = 12, padR = 8, padT = 28, padB = 20;
    const gw = cw - padL - padR, gh = ch - padT - padB;
    const pts = lossHistory.slice(-200);
    cctx.beginPath();
    cctx.strokeStyle = '#3b82f6';
    cctx.lineWidth = 1.5;
    pts.forEach((v, i) => {
      const x = padL + (i / (pts.length - 1)) * gw;
      const y = padT + gh - (v / maxLoss) * gh;
      i === 0 ? cctx.moveTo(x, y) : cctx.lineTo(x, y);
    });
    cctx.stroke();
    // Axes
    cctx.strokeStyle = '#334155';
    cctx.lineWidth = 1;
    cctx.strokeRect(padL, padT, gw, gh);
  }

  function updateStats() {
    const el = document.getElementById('fl-stats');
    el.innerHTML = `Round: <b>${sim.round()}</b> &nbsp;|&nbsp;
      Mean loss: <b>${sim.mean_loss().toFixed(4)}</b> &nbsp;|&nbsp;
      Converged: <b>${sim.converged_count()}/${N}</b> &nbsp;|&nbsp;
      Byzantine: <b>${sim.byzantine_count()}</b> &nbsp;|&nbsp;
      Gossip: <b>${sim.gossip_weight().toFixed(2)}</b> &nbsp;|&nbsp;
      LR: <b>${sim.lr().toFixed(3)}</b>`;
  }

  function tick(n = 1) {
    sim.step_n(n);
    const histPtr = sim.loss_history_ptr();
    const hist = new Float32Array(wasm.memory.buffer, histPtr, 200);
    lossHistory.length = 0;
    for (let i = 0; i < sim.round() && i < 200; i++) {
      if (!isNaN(hist[i])) lossHistory.push(hist[i]);
    }
    renderGrid();
    renderChart();
    updateStats();
  }

  // Controls
  document.getElementById('btn-step').onclick   = () => tick(1);
  document.getElementById('btn-step10').onclick = () => tick(10);
  document.getElementById('btn-run').onclick = () => {
    if (running) return;
    running = true;
    document.getElementById('btn-run').disabled = true;
    document.getElementById('btn-pause').disabled = false;
    function loop() { tick(1); rafId = requestAnimationFrame(loop); }
    rafId = requestAnimationFrame(loop);
  };
  document.getElementById('btn-pause').onclick = () => {
    running = false;
    cancelAnimationFrame(rafId);
    document.getElementById('btn-run').disabled = false;
    document.getElementById('btn-pause').disabled = true;
  };
  document.getElementById('btn-reset').onclick = () => {
    running = false; cancelAnimationFrame(rafId);
    sim.reset(); lossHistory.length = 0;
    document.getElementById('btn-run').disabled = false;
    document.getElementById('btn-pause').disabled = true;
    tick(0);
  };

  const gossipSlider = document.getElementById('inp-gossip');
  gossipSlider.oninput = () => {
    const v = +gossipSlider.value / 100;
    sim.set_gossip_weight(v);
    document.getElementById('lbl-gossip').textContent = v.toFixed(2);
  };
  const lrSlider = document.getElementById('inp-lr');
  lrSlider.oninput = () => {
    const v = +lrSlider.value / 100;
    sim.set_lr(v);
    document.getElementById('lbl-lr').textContent = v.toFixed(3);
  };

  // Click to inject Byzantine node
  gridCanvas.onclick = (e) => {
    const rect = gridCanvas.getBoundingClientRect();
    const scaleX = gridCanvas.width / rect.width;
    const scaleY = gridCanvas.height / rect.height;
    const px = (e.clientX - rect.left) * scaleX;
    const py = (e.clientY - rect.top) * scaleY;
    const col = Math.floor(px / cellW);
    const row = Math.floor(py / cellH);
    sim.inject_byzantine(row, col);
    if (!running) tick(0);
  };

  tick(0);
}

start();
</script>

---

## What to look for

**Data heterogeneity zones**: the grid is divided into thirds vertically. The top third
of devices has private targets near 0.2 (shown in blue-purple); the bottom third
near 0.8 (red-purple). Watch how the middle zone struggles — it is pulled in both
directions by its neighbours, exactly replicating the *gradient conflict* that
makes FL with non-IID data hard.

**Convergence waves**: start with low gossip weight (0.1). Convergence is slow and
patchy — local clusters form. Increase gossip to 0.8: the grid snaps to a compromise
value quickly but accuracy per zone is lower (the global model is a compromise).
This reproduces the **precision-vs-consensus tradeoff** of FL.

**Byzantine fault isolation**: click any cell to mark it Byzantine (yellow border).
Its model value oscillates wildly. Watch its trust score drop (cell darkens) and
observe that the corruption does not propagate — neighbouring cells discount its
influence. This is **gossip-based Byzantine fault tolerance** as a CA phenomenon.