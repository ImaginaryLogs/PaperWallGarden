---
title: Business Process — Stochastic Petri Net + HMM Process Mining
tags: [simulation, business-processes, petri-net, stochastic, hmm, process-mining, wasm]
---

# Business Process: Stochastic Petri Net with HMM Process Mining

This simulation models a **Loan Application workflow** as a **Stochastic Petri Net (SPN)**
with a real-time **Hidden Markov Model** for process mining.

**What makes it stochastic**: every transition has a firing rate λ. When multiple
transitions are simultaneously enabled (true concurrency), they compete in an
**exponential race**: the winner is drawn proportionally to its rate. This models
real-world variability — sometimes credit checks are fast, sometimes staff are busy.
The resulting dynamics are a **Continuous-Time Markov Chain** over Petri Net markings.

**The HMM process mining layer**: the model observes *which transition fires* each step
and uses the Forward Algorithm to maintain a belief distribution over 4 hidden process
states: NORMAL, BACKLOG, BOTTLENECK, CLEARED. This is exactly what process mining tools
(ProM, Disco) do when analysing event logs — except here it runs live on the SPN.

<div id="bp-controls" style="margin:0.5rem 0 1rem;display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;font-size:0.875rem;">
  <button id="btn-step"    style="padding:6px 14px;cursor:pointer;">Step ×1</button>
  <button id="btn-step20"  style="padding:6px 14px;cursor:pointer;">Step ×20</button>
  <button id="btn-run"     style="padding:6px 14px;cursor:pointer;">▶ Run</button>
  <button id="btn-pause"   style="padding:6px 14px;cursor:pointer;" disabled>⏸ Pause</button>
  <button id="btn-inject"  style="padding:6px 14px;cursor:pointer;">+ Application</button>
  <button id="btn-reset"   style="padding:6px 14px;cursor:pointer;">↺ Reset</button>
</div>

<canvas id="petri-canvas" width="800" height="420"
  style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;margin-bottom:0.75rem;">
</canvas>

<div style="display:grid;grid-template-columns:1fr 1fr;gap:1rem;margin-bottom:0.75rem;">
  <canvas id="hmm-canvas" width="380" height="150"
    style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;">
  </canvas>
  <canvas id="throughput-canvas" width="380" height="150"
    style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;">
  </canvas>
</div>

<div id="bp-stats" style="font-size:0.82rem;opacity:0.65;font-family:monospace;display:flex;gap:2rem;flex-wrap:wrap;"></div>

<script type="module">
const isGitHubPages = window.location.hostname.includes('github.io');
const basePath = isGitHubPages ? '/PaperWallGarden' : '';
const WASM_JS = `${basePath}/wasm/petri_sim2.js`;
const WASM_BG = `${basePath}/wasm/petri_sim2_bg.wasm`;

const PLACE_NAMES = [
  'Inbox','Under\nAssess.','Complete','Incomplete',
  'Docs\nRequested','Docs\nReceived','Timeout',
  'Credit\nCheck','Credit\nApproved','Credit\nRefused',
  'Risk\nAssess.','Final\nApproved','Final\nRejected',
  'Contract\nSigning','Funds\nDisburse','Done'
];
const TRANS_NAMES = [
  'Assess\nComplete.','Route\nOK','Route\nFail','Request\nDocs',
  'Docs\nReceived','Docs\nTimeout','Credit\nCheck',
  'Credit\nApproved','Credit\nRefused','Risk\nAssess.',
  'Final\nApprove','Final\nReject'
];
const LAMBDA = [2.0,2.5,1.5,3.0,1.5,0.3,2.0,1.8,0.8,1.5,1.2,0.4];
const HMM_LABELS = ['NORMAL','BACKLOG','BOTTLENECK','CLEARED'];
const HMM_COLORS = ['#22c55e','#f59e0b','#ef4444','#3b82f6'];

async function start() {
  let module, wasm, sim;
  try {
    module = await import(WASM_JS);
    wasm   = await module.default(WASM_BG);
    sim    = module.BusinessProcessSPN.new();
  } catch(e) {
    document.getElementById('bp-stats').textContent = `WASM failed: ${e}`;
    return;
  }

  const pCanvas = document.getElementById('petri-canvas');
  const hCanvas = document.getElementById('hmm-canvas');
  const tCanvas = document.getElementById('throughput-canvas');
  const pctx = pCanvas.getContext('2d');
  const hctx = hCanvas.getContext('2d');
  const tctx = tCanvas.getContext('2d');

  const NP = sim.num_places(), NT = sim.num_transitions();
  let running = false, rafId = null;
  const tpHistory = [];

  function getViews() {
    return {
      tokens:      new Uint32Array  (wasm.memory.buffer, sim.tokens_ptr(),     NP),
      place_x:     new Float32Array (wasm.memory.buffer, sim.place_x_ptr(),    NP),
      place_y:     new Float32Array (wasm.memory.buffer, sim.place_y_ptr(),    NP),
      trans_x:     new Float32Array (wasm.memory.buffer, sim.trans_x_ptr(),    NT),
      trans_y:     new Float32Array (wasm.memory.buffer, sim.trans_y_ptr(),    NT),
      fired:       new Uint8Array   (wasm.memory.buffer, sim.fired_ptr(),      NT),
      fire_count:  new Uint32Array  (wasm.memory.buffer, sim.fire_count_ptr(), NT),
      anim:        new Float32Array (wasm.memory.buffer, sim.anim_ptr(),       NT),
      hmm_belief:  new Float32Array (wasm.memory.buffer, sim.hmm_belief_ptr(), 4),
    };
  }

  const ARCS_PT = [[0,0],[1,1],[1,2],[3,3],[4,4],[4,5],[2,6],[5,6],[7,7],[7,8],[8,9],[10,10],[10,11]];
  const ARCS_TP = [[0,1],[1,2],[2,3],[3,4],[4,5],[5,12],[6,7],[7,8],[8,9],[9,10],[10,11],[11,12]];

  function renderNet(v) {
    pctx.fillStyle = '#0d1117';
    pctx.fillRect(0, 0, pCanvas.width, pCanvas.height);
    const sw = pCanvas.width / 820, sh = pCanvas.height / 420;

    // Arcs place→transition
    for (const [pi, ti] of ARCS_PT) {
      if (pi >= NP || ti >= NT) continue;
      pctx.beginPath();
      pctx.moveTo(v.place_x[pi]*sw, v.place_y[pi]*sh);
      pctx.lineTo(v.trans_x[ti]*sw, v.trans_y[ti]*sh);
      pctx.strokeStyle = '#334155';
      pctx.lineWidth = 1.2;
      pctx.stroke();
    }
    // Arcs transition→place
    for (const [ti, pi] of ARCS_TP) {
      if (ti >= NT || pi >= NP) continue;
      pctx.beginPath();
      pctx.moveTo(v.trans_x[ti]*sw, v.trans_y[ti]*sh);
      pctx.lineTo(v.place_x[pi]*sw, v.place_y[pi]*sh);
      pctx.strokeStyle = '#475569';
      pctx.lineWidth = 1.2;
      pctx.stroke();
    }

    // Transitions
    for (let t = 0; t < NT; t++) {
      const tx = v.trans_x[t]*sw, ty = v.trans_y[t]*sh;
      const fired = v.fired[t] > 0;
      // Width proportional to lambda (firing rate visualisation)
      const rw = 16 + LAMBDA[t] * 4;
      pctx.fillStyle = fired ? '#22d3ee' : '#1e3a5f';
      pctx.fillRect(tx - rw/2, ty - 8, rw, 16);
      pctx.strokeStyle = fired ? '#67e8f9' : '#3b82f6';
      pctx.lineWidth = fired ? 2 : 1;
      pctx.strokeRect(tx - rw/2, ty - 8, rw, 16);
      // Token animation dot
      if (v.anim[t] < 1.0 && v.anim[t] > 0) {
        pctx.beginPath();
        pctx.arc(tx, ty + 14 + v.anim[t] * 10, 3, 0, Math.PI*2);
        pctx.fillStyle = '#fbbf24';
        pctx.fill();
      }
      // Fire count badge
      if (v.fire_count[t] > 0) {
        pctx.fillStyle = '#475569';
        pctx.font = '8px monospace';
        pctx.textAlign = 'center';
        pctx.fillText(`×${v.fire_count[t]}`, tx, ty + 22);
      }
    }

    // Places
    for (let p = 0; p < NP; p++) {
      const px = v.place_x[p]*sw, py = v.place_y[p]*sh;
      const tok = v.tokens[p];
      const hasTokens = tok > 0;
      pctx.beginPath();
      pctx.arc(px, py, 14, 0, Math.PI*2);
      pctx.fillStyle = hasTokens ? '#1e3a5f' : '#0f172a';
      pctx.fill();
      pctx.strokeStyle = hasTokens ? '#3b82f6' : '#334155';
      pctx.lineWidth = hasTokens ? 2 : 1;
      pctx.stroke();
      // Token count
      if (tok > 0) {
        pctx.fillStyle = '#fbbf24';
        pctx.font = `bold ${tok > 9 ? 9 : 11}px monospace`;
        pctx.textAlign = 'center';
        pctx.fillText(tok.toString(), px, py + 4);
      }
      // Place name
      pctx.fillStyle = '#64748b';
      pctx.font = '7px sans-serif';
      pctx.textAlign = 'center';
      const lines = PLACE_NAMES[p].split('\n');
      lines.forEach((line, i) => pctx.fillText(line, px, py + 22 + i * 8));
    }
    pctx.textAlign = 'left';
  }

  function renderHMM(v) {
    hctx.fillStyle = '#0d1117';
    hctx.fillRect(0, 0, hCanvas.width, hCanvas.height);
    hctx.fillStyle = '#94a3b8';
    hctx.font = '10px monospace';
    hctx.fillText('HMM Process State Belief', 6, 14);
    const barW = (hCanvas.width - 20) / 4;
    for (let h = 0; h < 4; h++) {
      const belief = v.hmm_belief[h];
      const barH = (hCanvas.height - 50) * belief;
      const x = 10 + h * barW;
      const y = hCanvas.height - 25 - barH;
      hctx.fillStyle = HMM_COLORS[h];
      hctx.fillRect(x + 4, y, barW - 8, barH);
      hctx.fillStyle = '#94a3b8';
      hctx.font = '9px monospace';
      hctx.textAlign = 'center';
      hctx.fillText(HMM_LABELS[h], x + barW/2, hCanvas.height - 10);
      hctx.fillText((belief*100).toFixed(1)+'%', x + barW/2, y - 3);
    }
    // Bottleneck indicator
    const bp = sim.bottleneck_prob();
    hctx.textAlign = 'right';
    hctx.fillStyle = bp > 0.5 ? '#ef4444' : '#94a3b8';
    hctx.font = '10px monospace';
    hctx.fillText(`Bottleneck risk: ${(bp*100).toFixed(1)}%`, hCanvas.width - 6, hCanvas.height - 30);
    hctx.textAlign = 'left';
  }

  function renderThroughput() {
    tctx.fillStyle = '#0d1117';
    tctx.fillRect(0, 0, tCanvas.width, tCanvas.height);
    tctx.fillStyle = '#94a3b8';
    tctx.font = '10px monospace';
    tctx.fillText('Throughput (done/step)', 6, 14);
    if (tpHistory.length < 2) return;
    const pts = tpHistory.slice(-120);
    const maxV = Math.max(...pts, 0.01);
    const pad = {l:10, r:8, t:22, b:18};
    const gw = tCanvas.width - pad.l - pad.r;
    const gh = tCanvas.height - pad.t - pad.b;
    tctx.beginPath();
    tctx.strokeStyle = '#22c55e';
    tctx.lineWidth = 1.5;
    pts.forEach((val, i) => {
      const x = pad.l + (i/(pts.length-1))*gw;
      const y = pad.t + gh - (val/maxV)*gh;
      i === 0 ? tctx.moveTo(x,y) : tctx.lineTo(x,y);
    });
    tctx.stroke();
    tctx.strokeStyle = '#334155';
    tctx.lineWidth = 1;
    tctx.strokeRect(pad.l, pad.t, gw, gh);
  }

  function updateStats() {
    document.getElementById('bp-stats').innerHTML =
      `Step: <b>${sim.step_count()}</b> &nbsp;|&nbsp;
       In: <b>${sim.applications_in()}</b> &nbsp;|&nbsp;
       Approved: <b>${sim.applications_done()}</b> &nbsp;|&nbsp;
       Rejected: <b>${sim.applications_rejected()}</b> &nbsp;|&nbsp;
       Throughput: <b>${sim.throughput().toFixed(3)}</b>/step &nbsp;|&nbsp;
       Sojourn: <b>~${sim.mean_sojourn().toFixed(1)}</b> steps`;
  }

  function tick(n=1) {
    sim.step_n(n);
    tpHistory.push(sim.throughput());
    const v = getViews();
    renderNet(v);
    renderHMM(v);
    renderThroughput();
    updateStats();
  }

  document.getElementById('btn-step').onclick    = () => tick(1);
  document.getElementById('btn-step20').onclick  = () => tick(20);
  document.getElementById('btn-inject').onclick  = () => { sim.inject_application(); if(!running) tick(0); };
  document.getElementById('btn-reset').onclick   = () => {
    running=false; cancelAnimationFrame(rafId); sim.reset(); tpHistory.length=0;
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

## Petri Net concepts demonstrated

**Concurrency**: multiple loan applications (tokens) flow through the net simultaneously.
They are independent — one application's credit check does not block another's risk
assessment (no shared place between those paths). This is Petri Net concurrency in action.

**Probabilistic branching**: transition T1 (Route Complete) and T2 (Route Incomplete) are
simultaneously enabled after T0 fires. Their rates λ=2.5 and λ=1.5 mean T1 wins ~62.5%
of the time — a stochastic branch. The "probability" emerges from the ratio of rates,
not from an explicit if/else. This is the stochastic Petri Net's elegant model of
real-world uncertainty.

**Deadlock impossibility here**: the topology of this net is acyclic — tokens always
flow forward. But try following a token: what happens when T5 (Docs Timeout) fires
but T7 (Decoherence) also fires? The credit check path gets blocked. That's not deadlock
but **starvation** — the HMM should detect this as BOTTLENECK.

**Process mining**: the HMM belief chart shows real-time inference. When you inject many
applications at once (`+ Application` repeatedly), watch BACKLOG probability rise, then
BOTTLENECK as the credit check and risk assessment stages saturate. This is the same
inference a process mining engine runs on production event logs.