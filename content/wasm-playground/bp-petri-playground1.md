---
title: Business Process Petri Net Simulator
tags: [simulation, business-processes, petri-net, wasm]
---

# Business Process Petri Net

This playground simulates an **Order Fulfilment** workflow modelled as a Petri net.
Places (circles) hold tokens; transitions (bars) fire when all input places are marked,
consuming and producing tokens according to the process flow.

<div id="petri-controls" style="margin:0.5rem 0 1rem;display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;">
  <button id="btn-step"    style="padding:6px 14px;cursor:pointer;">Step ×1</button>
  <button id="btn-run"     style="padding:6px 14px;cursor:pointer;">▶ Run</button>
  <button id="btn-pause"   style="padding:6px 14px;cursor:pointer;" disabled>⏸ Pause</button>
  <button id="btn-order"   style="padding:6px 14px;cursor:pointer;">+ Inject Order</button>
  <button id="btn-reset"   style="padding:6px 14px;cursor:pointer;">↺ Reset</button>
  <span id="petri-stats"   style="font-size:0.85rem;opacity:0.7;"></span>
</div>

<canvas id="petri-canvas" width="600" height="360"
  style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;">
</canvas>

<div id="petri-log" style="margin-top:0.75rem;font-size:0.78rem;opacity:0.65;font-family:monospace;height:4rem;overflow-y:auto;"></div>

<script type="module">
// ============================================================
// 1. Dynamic path resolution (localhost vs GitHub Pages)
// ============================================================
const isGitHubPages = window.location.hostname.includes('github.io');
const basePath = isGitHubPages ? '/PaperWallGarden' : '';

const WASM_JS  = `${basePath}/wasm/petri_sim1.js`;
const WASM_BG  = `${basePath}/wasm/petri_sim1_bg.wasm`;

// ============================================================
// 2. Constants mirroring Rust (place/transition names)
// ============================================================
const PLACE_NAMES = [
  'Order\nReceived', 'Order\nValidated', 'In\nStock', 'Out of\nStock',
  'Packed', 'Restocked', 'Shipped', 'Delivered'
];
const TRANS_NAMES = [
  'Validate', 'Check\nStock', 'Pick &\nPack', 'Reorder', 'Restock\n→Pack', 'Ship', 'Deliver'
];

// Colour palette for places and transitions
const PLACE_COL   = '#3b82f6'; // blue
const TRANS_COL   = '#6366f1'; // indigo
const TOKEN_COL   = '#fbbf24'; // amber
const FIRED_COL   = '#34d399'; // green flash
const IDLE_COL    = '#475569'; // slate

// Edges: [from_place, to_transition] or [from_transition, to_place]
// Using indices matching Rust's INCIDENCE matrix
const ARCS_PT = [ // place → transition
  [0,0],[1,1],[2,2],[3,3],[5,4],[4,5],[6,6]
];
const ARCS_TP = [ // transition → place
  [0,1],[1,2],[1,3],[2,4],[3,5],[4,4],[5,6],[6,7]
];

// ============================================================
// 3. Async pipeline initialisation
// ============================================================
async function startPipeline() {
  let module, wasm, net;
  try {
    module = await import(WASM_JS);
    wasm   = await module.default(WASM_BG);
    net    = module.PetriNet.new();
  } catch (err) {
    document.getElementById('petri-log').textContent = `WASM load failed: ${err}`;
    return;
  }

  const canvas  = document.getElementById('petri-canvas');
  const ctx     = canvas.getContext('2d');
  const logEl   = document.getElementById('petri-log');
  const statsEl = document.getElementById('petri-stats');

  // ============================================================
  // 4. Zero-copy typed array views over WASM shared memory
  // ============================================================
  function getViews() {
    const mem = wasm.memory.buffer;
    return {
      tokens:  new Uint32Array(mem, net.tokens_ptr(),   net.num_places()),
      placeX:  new Float32Array(mem, net.place_x_ptr(), net.num_places()),
      placeY:  new Float32Array(mem, net.place_y_ptr(), net.num_places()),
      transX:  new Float32Array(mem, net.trans_x_ptr(), net.num_transitions()),
      transY:  new Float32Array(mem, net.trans_y_ptr(), net.num_transitions()),
      fired:   new Uint8Array(mem,   net.fired_ptr(),   net.num_transitions()),
      anim:    new Float32Array(mem, net.anim_ptr(),    net.num_transitions()),
    };
  }

  // ============================================================
  // 5. Render
  // ============================================================
  const W = canvas.width, H = canvas.height;
  // Scale Rust layout coords (0..480, 0..320) to canvas
  const SX = v => 20 + v * (W - 40) / 500;
  const SY = v => 20 + v * (H - 40) / 280;
  const PR = 22, TR_W = 16, TR_H = 44;

  function drawMultiline(ctx, text, x, y, lineH) {
    const lines = text.split('\n');
    const startY = y - ((lines.length - 1) * lineH) / 2;
    lines.forEach((l, i) => ctx.fillText(l, x, startY + i * lineH));
  }

  function render(views) {
    ctx.clearRect(0, 0, W, H);

    // Draw arcs place→transition
    ctx.strokeStyle = 'rgba(148,163,184,0.35)';
    ctx.lineWidth   = 1.5;
    ARCS_PT.forEach(([p, t]) => {
      ctx.beginPath();
      ctx.moveTo(SX(views.placeX[p]), SY(views.placeY[p]));
      ctx.lineTo(SX(views.transX[t]), SY(views.transY[t]));
      ctx.stroke();
    });
    // Draw arcs transition→place
    ARCS_TP.forEach(([t, p]) => {
      const tx = SX(views.transX[t]), ty = SY(views.transY[t]);
      const px = SX(views.placeX[p]),  py = SY(views.placeY[p]);
      // Arrowhead direction
      const dx = px - tx, dy = py - ty;
      const len = Math.hypot(dx, dy);
      const ux = dx / len, uy = dy / len;
      const endX = px - ux * (PR + 2), endY = py - uy * (PR + 2);
      ctx.beginPath();
      ctx.moveTo(tx, ty);
      ctx.lineTo(endX, endY);
      ctx.strokeStyle = 'rgba(148,163,184,0.35)';
      ctx.stroke();
      // Arrow tip
      const angle = Math.atan2(uy, ux);
      ctx.save();
      ctx.translate(endX, endY);
      ctx.rotate(angle);
      ctx.beginPath();
      ctx.moveTo(0,0); ctx.lineTo(-8,-4); ctx.lineTo(-8,4);
      ctx.fillStyle = 'rgba(148,163,184,0.5)';
      ctx.fill();
      ctx.restore();
    });

    // Draw transitions
    for (let t = 0; t < net.num_transitions(); t++) {
      const cx = SX(views.transX[t]), cy = SY(views.transY[t]);
      ctx.fillStyle = views.fired[t] ? FIRED_COL : TRANS_COL;
      ctx.globalAlpha = views.fired[t] ? 1.0 : 0.75;
      ctx.fillRect(cx - TR_W/2, cy - TR_H/2, TR_W, TR_H);
      ctx.globalAlpha = 1.0;
      ctx.fillStyle = '#fff';
      ctx.font = '10px monospace';
      ctx.textAlign = 'center';
      drawMultiline(ctx, TRANS_NAMES[t], cx, cy - 14 - (TR_H/2 + 12), 11);
    }

    // Draw places
    for (let p = 0; p < net.num_places(); p++) {
      const cx = SX(views.placeX[p]), cy = SY(views.placeY[p]);
      const tok = views.tokens[p];
      ctx.beginPath();
      ctx.arc(cx, cy, PR, 0, Math.PI * 2);
      ctx.fillStyle   = tok > 0 ? `rgba(59,130,246,0.25)` : `rgba(71,85,105,0.15)`;
      ctx.strokeStyle = tok > 0 ? PLACE_COL : IDLE_COL;
      ctx.lineWidth   = tok > 0 ? 2 : 1;
      ctx.fill(); ctx.stroke();

      // Tokens
      if (tok > 0) {
        if (tok === 1) {
          ctx.beginPath();
          ctx.arc(cx, cy, 5, 0, Math.PI * 2);
          ctx.fillStyle = TOKEN_COL;
          ctx.fill();
        } else {
          ctx.fillStyle = TOKEN_COL;
          ctx.font      = 'bold 12px monospace';
          ctx.textAlign = 'center';
          ctx.fillText(tok, cx, cy + 4);
        }
      }

      // Label
      ctx.fillStyle = 'rgba(226,232,240,0.75)';
      ctx.font      = '9px sans-serif';
      ctx.textAlign = 'center';
      drawMultiline(ctx, PLACE_NAMES[p], cx, cy + PR + 14, 11);
    }

    // Stats
    statsEl.textContent =
      `Steps: ${net.step_count()}  |  Delivered: ${net.delivered_total()}  |  Pending: ${net.pending_orders()}`;
  }

  // ============================================================
  // 6. Animation loop
  // ============================================================
  let running = false;
  let rafId   = null;

  function stepAndRender() {
    net.step();
    const views = getViews();

    // Log fired transitions
    for (let t = 0; t < net.num_transitions(); t++) {
      if (views.fired[t]) {
        const msg = `[${net.step_count()}] Transition "${TRANS_NAMES[t].replace('\n',' ')}" fired`;
        logEl.textContent = msg + '\n' + logEl.textContent;
      }
    }

    render(views);
  }

  function loop() {
    stepAndRender();
    if (running) rafId = requestAnimationFrame(loop);
  }

  // Initial render
  render(getViews());

  // ============================================================
  // 7. Controls
  // ============================================================
  document.getElementById('btn-step').onclick   = () => stepAndRender();
  document.getElementById('btn-order').onclick  = () => { net.inject_order(); render(getViews()); };
  document.getElementById('btn-reset').onclick  = () => {
    net.reset(); running = false;
    cancelAnimationFrame(rafId);
    document.getElementById('btn-run').disabled   = false;
    document.getElementById('btn-pause').disabled = true;
    logEl.textContent = '';
    render(getViews());
  };
  document.getElementById('btn-run').onclick = () => {
    running = true;
    document.getElementById('btn-run').disabled   = true;
    document.getElementById('btn-pause').disabled = false;
    rafId = requestAnimationFrame(loop);
  };
  document.getElementById('btn-pause').onclick = () => {
    running = false;
    cancelAnimationFrame(rafId);
    document.getElementById('btn-run').disabled   = false;
    document.getElementById('btn-pause').disabled = true;
  };
}

startPipeline();
</script>