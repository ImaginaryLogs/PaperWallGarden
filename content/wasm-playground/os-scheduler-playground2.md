---
title: OS Process Scheduling — Resource Allocation Graph Automaton
tags: [simulation, operating-systems, graph-ca, network-automata, hmm, wasm]
---

# OS Process Scheduling — Network-Graph Cellular Automaton

This simulation models **OS process scheduling** as a **Network-Graph Cellular Automaton**
on a **Resource Allocation Graph (RAG)**. The graph has two types of nodes —
**process nodes** and **resource nodes** — connected by directed edges representing
claims and requests. Each node evolves according to a local rule that looks only at
its immediate neighbours in the graph.

**Why a graph CA?** Unlike a grid CA (where every cell has exactly 8 neighbours),
a graph CA has irregular topology — processes have different resource needs, resources
have different numbers of claimants. The topology *encodes the constraints of the system*.
A cycle in the graph is a deadlock. A resource with many incoming request edges is a bottleneck.

The HMM (Hidden Markov Model) on each resource estimates its **hidden load state**
(LOW/MED/HIGH) from observed busy/free history, used to predict future contention.

<div id="os-controls" style="margin:0.5rem 0 1rem;display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;font-size:0.875rem;">
  <button id="btn-tick"   style="padding:6px 14px;cursor:pointer;">Tick ×1</button>
  <button id="btn-t10"    style="padding:6px 14px;cursor:pointer;">×10</button>
  <button id="btn-run"    style="padding:6px 14px;cursor:pointer;">▶ Run</button>
  <button id="btn-pause"  style="padding:6px 14px;cursor:pointer;" disabled>⏸ Pause</button>
  <button id="btn-spawn"  style="padding:6px 14px;cursor:pointer;">+ Spawn Process</button>
  <button id="btn-reset"  style="padding:6px 14px;cursor:pointer;">↺ Reset</button>
</div>

<canvas id="graph-canvas" width="700" height="420"
  style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;margin-bottom:0.75rem;">
</canvas>

<div style="display:grid;grid-template-columns:1fr 1fr;gap:1rem;margin-bottom:0.75rem;">
  <canvas id="trace-canvas" width="360" height="140"
    style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;">
  </canvas>
  <canvas id="hmm-canvas" width="360" height="140"
    style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;">
  </canvas>
</div>

<div id="os-stats" style="font-size:0.82rem;opacity:0.65;font-family:monospace;display:flex;gap:2rem;flex-wrap:wrap;"></div>

<script type="module">
const isGitHubPages = window.location.hostname.includes('github.io');
const basePath = isGitHubPages ? '/PaperWallGarden' : '';
const WASM_JS = `${basePath}/wasm/os_scheduler2.js`;
const WASM_BG = `${basePath}/wasm/os_scheduler2_bg.wasm`;

const PROC_COLORS = ['#3b82f6','#8b5cf6','#ec4899','#f59e0b','#10b981','#ef4444','#06b6d4','#f97316'];
const STATE_NAMES = ['empty','ready','running','waiting','done'];
const STATE_COLORS = {'empty':'#1e293b','ready':'#3b82f6','running':'#22c55e','waiting':'#f59e0b','done':'#475569'};
const RES_NAMES = ['CPU0','CPU1','CPU2','CPU3','I/O','Mem','Net','Lock'];
const RES_TYPE_COLORS = ['#ef4444','#ef4444','#ef4444','#ef4444','#3b82f6','#8b5cf6','#06b6d4','#f59e0b'];
const RES_STATE_COLORS = {'0':'#1e293b','1':'#22c55e','2':'#f59e0b'};
const HMM_COLORS = ['#22c55e','#f59e0b','#ef4444'];

async function start() {
  let module, wasm, sim;
  try {
    module = await import(WASM_JS);
    wasm   = await module.default(WASM_BG);
    sim    = module.OsGraphCA.new();
  } catch(e) {
    document.getElementById('os-stats').textContent = `WASM failed: ${e}`;
    return;
  }

  const gCanvas = document.getElementById('graph-canvas');
  const tCanvas = document.getElementById('trace-canvas');
  const hCanvas = document.getElementById('hmm-canvas');
  const gctx = gCanvas.getContext('2d');
  const tctx = tCanvas.getContext('2d');
  const hctx = hCanvas.getContext('2d');

  const MAX_PROCS = sim.max_procs(), MAX_RES = sim.max_resources();
  const MAX_EDGES = sim.max_edges();
  const PS = sim.proc_buf_stride(), RS = sim.res_buf_stride(), ES = sim.edge_buf_stride();

  let running = false, rafId = null;

  function getViews() {
    return {
      procs:  new Float32Array(wasm.memory.buffer, sim.proc_buf_ptr(),  MAX_PROCS * PS),
      res:    new Float32Array(wasm.memory.buffer, sim.res_buf_ptr(),   MAX_RES * RS),
      edges:  new Float32Array(wasm.memory.buffer, sim.edge_buf_ptr(),  MAX_EDGES * ES),
      trace:  new Float32Array(wasm.memory.buffer, sim.trace_buf_ptr(), MAX_RES * 64),
      proc_x: new Float32Array(wasm.memory.buffer, sim.proc_x_ptr(),    MAX_PROCS),
      proc_y: new Float32Array(wasm.memory.buffer, sim.proc_y_ptr(),    MAX_PROCS),
      res_x:  new Float32Array(wasm.memory.buffer, sim.res_x_ptr(),     MAX_RES),
      res_y:  new Float32Array(wasm.memory.buffer, sim.res_y_ptr(),     MAX_RES),
    };
  }

  function renderGraph(v) {
    gctx.clearRect(0, 0, gCanvas.width, gCanvas.height);
    gctx.fillStyle = '#0d1117';
    gctx.fillRect(0, 0, gCanvas.width, gCanvas.height);

    // Draw edges (proc → resource arcs)
    for (let ei = 0; ei < MAX_EDGES; ei++) {
      const base = ei * ES;
      if (v.edges[base] < 0) continue; // inactive
      const pid = v.edges[base] | 0;
      const rid = v.edges[base+1] | 0;
      const waiting = v.edges[base+2] > 0;
      const px = v.proc_x[pid], py = v.proc_y[pid];
      const rx = v.res_x[rid],  ry = v.res_y[rid];
      gctx.beginPath();
      gctx.moveTo(px, py);
      gctx.lineTo(rx, ry);
      gctx.strokeStyle = waiting ? '#f59e0b80' : '#22c55e80';
      gctx.lineWidth = waiting ? 1 : 2;
      gctx.setLineDash(waiting ? [4, 4] : []);
      gctx.stroke();
      gctx.setLineDash([]);
      // Arrowhead
      const dx = rx - px, dy = ry - py;
      const len = Math.hypot(dx, dy);
      if (len > 0) {
        const mx = px + dx * 0.65, my = py + dy * 0.65;
        const nx = -dy/len*5, ny = dx/len*5;
        gctx.beginPath();
        gctx.moveTo(mx + nx, my + ny);
        gctx.lineTo(mx - nx, my - ny);
        gctx.lineTo(mx + dx/len*8, my + dy/len*8);
        gctx.fillStyle = waiting ? '#f59e0b' : '#22c55e';
        gctx.fill();
      }
    }

    // Draw resource nodes (rectangles at top)
    for (let r = 0; r < MAX_RES; r++) {
      const base = r * RS;
      const state = v.res[base] | 0;
      const rx = v.res_x[r], ry = v.res_y[r];
      const pred = v.res[base+3]; // HMM predicted busy prob
      gctx.fillStyle = ['#1e293b','#22c55e','#f59e0b'][state] || '#1e293b';
      gctx.beginPath();
      gctx.roundRect(rx - 28, ry - 16, 56, 32, 4);
      gctx.fill();
      gctx.strokeStyle = RES_TYPE_COLORS[r];
      gctx.lineWidth = 2;
      gctx.stroke();
      // Label
      gctx.fillStyle = '#e2e8f0';
      gctx.font = 'bold 10px monospace';
      gctx.textAlign = 'center';
      gctx.fillText(RES_NAMES[r], rx, ry - 2);
      // HMM prediction bar
      gctx.fillStyle = `rgba(239,68,68,${pred})`;
      gctx.fillRect(rx - 26, ry + 6, 52 * pred, 6);
      gctx.strokeStyle = '#475569';
      gctx.lineWidth = 1;
      gctx.strokeRect(rx - 26, ry + 6, 52, 6);
    }

    // Draw process nodes (circles)
    for (let p = 0; p < MAX_PROCS; p++) {
      const base = p * PS;
      const state = STATE_NAMES[v.procs[base] | 0] || 'empty';
      if (state === 'empty') continue;
      const pid = v.procs[base+4] | 0;
      const color_id = v.procs[base+5] | 0;
      const px = v.proc_x[p], py = v.proc_y[p];
      const r = 20;
      gctx.beginPath();
      gctx.arc(px, py, r, 0, Math.PI * 2);
      gctx.fillStyle = STATE_COLORS[state] || '#1e293b';
      gctx.fill();
      gctx.strokeStyle = PROC_COLORS[color_id % 8];
      gctx.lineWidth = 2.5;
      gctx.stroke();
      // CPU usage arc
      const cpu = v.procs[base+2];
      gctx.beginPath();
      gctx.arc(px, py, r + 4, -Math.PI/2, -Math.PI/2 + cpu * Math.PI * 2);
      gctx.strokeStyle = PROC_COLORS[color_id % 8] + 'aa';
      gctx.lineWidth = 3;
      gctx.stroke();
      gctx.fillStyle = '#f1f5f9';
      gctx.font = '9px monospace';
      gctx.textAlign = 'center';
      gctx.fillText(`P${pid}`, px, py + 3);
    }
    gctx.textAlign = 'left';
  }

  function renderTrace(v) {
    tctx.fillStyle = '#0d1117';
    tctx.fillRect(0, 0, tCanvas.width, tCanvas.height);
    tctx.fillStyle = '#94a3b8';
    tctx.font = '10px monospace';
    tctx.fillText('Resource Busy/Free Timeline (last 64 ticks)', 6, 14);
    const rowH = (tCanvas.height - 22) / MAX_RES;
    for (let r = 0; r < MAX_RES; r++) {
      const y0 = 18 + r * rowH;
      tctx.fillStyle = '#334155';
      tctx.fillText(RES_NAMES[r], 2, y0 + rowH * 0.7);
      for (let t = 0; t < 64; t++) {
        const busy = v.trace[r * 64 + t] > 0;
        tctx.fillStyle = busy ? RES_TYPE_COLORS[r] : '#1e293b';
        tctx.fillRect(32 + t * 4, y0 + 1, 3, rowH - 2);
      }
    }
  }

  function renderHMM() {
    hctx.fillStyle = '#0d1117';
    hctx.fillRect(0, 0, hCanvas.width, hCanvas.height);
    hctx.fillStyle = '#94a3b8';
    hctx.font = '10px monospace';
    hctx.fillText('HMM Predicted Load per Resource', 6, 14);
    const labels = ['LOW','MED','HIGH'];
    for (let r = 0; r < MAX_RES; r++) {
      const pred = sim.hmm_prediction(r);
      const x = 10 + r * (hCanvas.width / MAX_RES - 4);
      const barH = (hCanvas.height - 50) * pred;
      const y = hCanvas.height - 20 - barH;
      hctx.fillStyle = HMM_COLORS[Math.round(pred * 2)] || '#22c55e';
      hctx.fillRect(x, y, 30, barH);
      hctx.strokeStyle = '#475569';
      hctx.lineWidth = 1;
      hctx.strokeRect(x, hCanvas.height - 20 - (hCanvas.height-50), 30, hCanvas.height-50);
      hctx.fillStyle = '#94a3b8';
      hctx.font = '9px monospace';
      hctx.fillText(RES_NAMES[r], x - 4, hCanvas.height - 6);
      hctx.fillText((pred*100).toFixed(0)+'%', x + 2, y - 2);
    }
  }

  function updateStats() {
    const el = document.getElementById('os-stats');
    el.innerHTML = `Tick: <b>${sim.current_tick()}</b> &nbsp;|&nbsp;
      Active processes: <b>${sim.active_proc_count()}</b> &nbsp;|&nbsp;
      Completed: <b>${sim.completed()}</b>`;
  }

  function tick(n = 1) {
    sim.advance(n);
    const v = getViews();
    renderGraph(v);
    renderTrace(v);
    renderHMM();
    updateStats();
  }

  document.getElementById('btn-tick').onclick   = () => tick(1);
  document.getElementById('btn-t10').onclick    = () => tick(10);
  document.getElementById('btn-spawn').onclick  = () => { sim.spawn_new(); if(!running) tick(0); };
  document.getElementById('btn-reset').onclick  = () => {
    running=false; cancelAnimationFrame(rafId);
    sim.reset();
    document.getElementById('btn-run').disabled=false;
    document.getElementById('btn-pause').disabled=true;
    tick(0);
  };
  document.getElementById('btn-run').onclick = () => {
    if(running) return; running=true;
    document.getElementById('btn-run').disabled=true;
    document.getElementById('btn-pause').disabled=false;
    function loop(){tick(1);rafId=requestAnimationFrame(loop);}
    rafId=requestAnimationFrame(loop);
  };
  document.getElementById('btn-pause').onclick = () => {
    running=false; cancelAnimationFrame(rafId);
    document.getElementById('btn-run').disabled=false;
    document.getElementById('btn-pause').disabled=true;
  };

  tick(0);
}

start();
</script>

---

## Automata concepts demonstrated

**Graph structure**: the resource nodes (top row) and process nodes (circle ring) are the
graph vertices. Solid green edges = process **owns** resource; dashed yellow edges =
process **waiting for** resource. The graph topology changes every tick as processes
acquire and release resources.

**CA transition rule**: each process node's state transition (READY→RUNNING→WAITING→READY)
is a function of the states of all resource nodes it is connected to. Each resource node's
state (FREE/BUSY/CONTENDED) is a function of the process nodes connected to it.
The entire graph updates synchronously — a true parallel automaton.

**HMM layer**: the bottom-right panel shows the HMM's predicted busy probability for each
resource. The HMM hidden states (LOW/MED/HIGH load) are inferred from the busy/free
trace (bottom-left panel). In a real OS, this prediction would be used to pre-warm
caches or pre-allocate resources before a process needs them.