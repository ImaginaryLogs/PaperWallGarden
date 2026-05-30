---
title: Quantum Communication – BB84 QKD Simulator
tags: [simulation, quantum-communication, bb84, qkd, wasm]
---

# Quantum Communication Simulator

Implements the **BB84 Quantum Key Distribution** protocol and a single-qubit
gate playground. Prepare qubits, apply Pauli/Hadamard/phase gates, run BB84
rounds, toggle an eavesdropper (Eve), and watch the **Quantum Bit Error Rate
(QBER)** spike past the 11% security threshold.

<div style="display:grid;grid-template-columns:1fr 1fr;gap:1rem;margin-bottom:1rem;">
  <!-- Left: Gate playground -->
  <div>
    <h3 style="margin-bottom:0.5rem;font-size:0.95rem;font-weight:600;">Gate Playground — Qubit 0</h3>
    <div style="display:flex;gap:6px;flex-wrap:wrap;margin-bottom:0.75rem;">
      <button class="gate-btn" data-gate="0" style="padding:5px 12px;cursor:pointer;font-family:monospace;">X</button>
      <button class="gate-btn" data-gate="1" style="padding:5px 12px;cursor:pointer;font-family:monospace;">Y</button>
      <button class="gate-btn" data-gate="2" style="padding:5px 12px;cursor:pointer;font-family:monospace;">Z</button>
      <button class="gate-btn" data-gate="3" style="padding:5px 12px;cursor:pointer;font-family:monospace;">H</button>
      <button class="gate-btn" data-gate="4" style="padding:5px 12px;cursor:pointer;font-family:monospace;">S</button>
      <button class="gate-btn" data-gate="5" style="padding:5px 12px;cursor:pointer;font-family:monospace;">T</button>
      <button id="btn-measure" style="padding:5px 12px;cursor:pointer;background:#1d4ed8;color:#fff;border:none;border-radius:4px;">Measure</button>
      <button id="btn-reset0"  style="padding:5px 12px;cursor:pointer;">Reset |0⟩</button>
    </div>
    <canvas id="canvas-bloch" width="280" height="280"
      style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;">
    </canvas>
    <div id="bloch-state" style="margin-top:0.5rem;font-size:0.8rem;font-family:monospace;opacity:0.65;"></div>
  </div>

  <!-- Right: Multi-qubit register -->
  <div>
    <h3 style="margin-bottom:0.5rem;font-size:0.95rem;font-weight:600;">Qubit Register</h3>
    <div style="margin-bottom:0.6rem;display:flex;gap:6px;flex-wrap:wrap;">
      <button id="btn-bell0" style="padding:5px 10px;cursor:pointer;font-size:0.8rem;">Bell |Φ+⟩</button>
      <button id="btn-bell1" style="padding:5px 10px;cursor:pointer;font-size:0.8rem;">|Φ-⟩</button>
      <button id="btn-bell2" style="padding:5px 10px;cursor:pointer;font-size:0.8rem;">|Ψ+⟩</button>
      <button id="btn-bell3" style="padding:5px 10px;cursor:pointer;font-size:0.8rem;">|Ψ-⟩</button>
    </div>
    <canvas id="canvas-register" width="340" height="260"
      style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;">
    </canvas>
  </div>
</div>

<!-- BB84 QKD section -->
<div style="border:1px solid rgba(128,128,128,0.25);border-radius:8px;padding:1rem;margin-bottom:1rem;">
  <h3 style="margin:0 0 0.75rem;font-size:0.95rem;font-weight:600;">BB84 Key Distribution Protocol</h3>
  <div style="display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;margin-bottom:0.75rem;font-size:0.875rem;">
    <button id="btn-bb84-round" style="padding:6px 14px;cursor:pointer;">Run 1 Round</button>
    <button id="btn-bb84-10"    style="padding:6px 14px;cursor:pointer;">Run 10 Rounds</button>
    <button id="btn-bb84-auto"  style="padding:6px 14px;cursor:pointer;">▶ Auto</button>
    <button id="btn-bb84-pause" style="padding:6px 14px;cursor:pointer;" disabled>⏸ Pause</button>
    <button id="btn-bb84-reset" style="padding:6px 14px;cursor:pointer;">↺ Reset</button>
    <label style="display:flex;align-items:center;gap:6px;">
      <input type="checkbox" id="chk-eve"> 👁 Eavesdropper (Eve)
    </label>
    <label>Noise <input id="inp-noise" type="range" min="0" max="30" value="2" style="width:70px">
      <span id="lbl-noise">2%</span></label>
  </div>
  <div style="display:grid;grid-template-columns:2fr 1fr;gap:1rem;">
    <canvas id="canvas-qber" width="480" height="180"
      style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;">
    </canvas>
    <div id="qkd-status" style="font-size:0.82rem;font-family:monospace;padding:0.5rem;border:1px solid rgba(128,128,128,0.2);border-radius:6px;"></div>
  </div>
  <div id="key-display" style="margin-top:0.75rem;font-size:0.75rem;font-family:monospace;word-break:break-all;opacity:0.65;"></div>
</div>

<script type="module">
const isGitHubPages = window.location.hostname.includes('github.io');
const basePath = isGitHubPages ? '/PaperWallGarden' : '';
const WASM_JS = `${basePath}/wasm/qcomm_sim.js`;
const WASM_BG = `${basePath}/wasm/qcomm_sim_bg.wasm`;

const GATE_NAMES = ['X (NOT)','Y','Z (Phase flip)','H (Hadamard)','S (π/2)','T (π/4)'];

async function startPipeline() {
  let module, wasm, qc;
  try {
    module = await import(WASM_JS);
    wasm   = await module.default(WASM_BG);
    qc     = module.QuantumChannel.new();
  } catch (err) {
    document.getElementById('bloch-state').textContent = `WASM load failed: ${err}`;
    return;
  }

  const blochCanvas    = document.getElementById('canvas-bloch');
  const registerCanvas = document.getElementById('canvas-register');
  const qberCanvas     = document.getElementById('canvas-qber');
  const bctx = blochCanvas.getContext('2d');
  const rctx = registerCanvas.getContext('2d');
  const qctx = qberCanvas.getContext('2d');

  const N_Q = qc.max_qubits();
  const N_ROUNDS = 64;

  function getViews() {
    const mem = wasm.memory.buffer;
    return {
      bloch:       new Float32Array(mem, qc.bloch_coords_ptr(),    N_Q * 3),
      states:      new Float32Array(mem, qc.qubit_states_ptr(),    N_Q * 4),
      qberHistory: new Float32Array(mem, qc.qber_history_ptr(),    N_ROUNDS),
      keyBits:     new Uint8Array(mem,   qc.key_bits_ptr(),        64),
    };
  }

  // ============================================================
  // Bloch sphere renderer (2D projection, isometric-ish)
  // ============================================================
  function drawBloch(v) {
    const W = blochCanvas.width, H = blochCanvas.height;
    const cx = W/2, cy = H/2 + 10;
    const R  = Math.min(W, H) / 2 - 20;
    bctx.clearRect(0,0,W,H);

    // Sphere outline
    bctx.strokeStyle = 'rgba(148,163,184,0.25)';
    bctx.lineWidth   = 1;
    bctx.beginPath();
    bctx.arc(cx, cy, R, 0, Math.PI*2);
    bctx.stroke();

    // Equator ellipse (x-y plane projection)
    bctx.beginPath();
    bctx.ellipse(cx, cy, R, R * 0.35, 0, 0, Math.PI*2);
    bctx.strokeStyle = 'rgba(148,163,184,0.15)';
    bctx.stroke();

    // Axes
    const axes = [
      {dx:0,   dy:-R,    label:'|0⟩', col:'rgba(96,165,250,0.7)'},
      {dx:0,   dy:R,     label:'|1⟩', col:'rgba(96,165,250,0.4)'},
      {dx:R*0.7, dy:R*0.35*0.7, label:'|+⟩', col:'rgba(167,139,250,0.5)'},
      {dx:-R*0.7,dy:-R*0.35*0.7,label:'|−⟩', col:'rgba(167,139,250,0.3)'},
    ];
    axes.forEach(({dx,dy,label,col}) => {
      bctx.strokeStyle = col;
      bctx.lineWidth = 0.75;
      bctx.setLineDash([3,4]);
      bctx.beginPath(); bctx.moveTo(cx,cy); bctx.lineTo(cx+dx,cy+dy); bctx.stroke();
      bctx.setLineDash([]);
      bctx.fillStyle = col;
      bctx.font = '11px monospace';
      bctx.textAlign = 'center';
      bctx.fillText(label, cx+dx*1.12, cy+dy*1.12+4);
    });

    // Bloch vector for qubit 0
    const bx = v.bloch[0], by = v.bloch[1], bz = v.bloch[2];
    // 2D projection: x maps to screen-right, z maps to screen-up, y gives depth
    const sx = cx + R * (bx * 0.85 + by * 0.25);
    const sy = cy - R * (bz + 0.0 * by * 0.18);

    // Shadow on equator
    const shadowX = cx + R * bx * 0.85;
    const shadowY = cy + R * 0.18;
    bctx.beginPath();
    bctx.arc(shadowX, shadowY, 4, 0, Math.PI*2);
    bctx.fillStyle = 'rgba(251,191,36,0.2)';
    bctx.fill();

    // Vector line
    bctx.strokeStyle = '#fbbf24';
    bctx.lineWidth = 2.5;
    bctx.beginPath();
    bctx.moveTo(cx, cy);
    bctx.lineTo(sx, sy);
    bctx.stroke();

    // Arrowhead
    const angle = Math.atan2(sy-cy, sx-cx);
    bctx.save();
    bctx.translate(sx, sy);
    bctx.rotate(angle);
    bctx.beginPath();
    bctx.moveTo(0,0); bctx.lineTo(-8,-4); bctx.lineTo(-8,4);
    bctx.fillStyle = '#fbbf24';
    bctx.fill();
    bctx.restore();

    // State label
    const re_a = v.states[0], im_a = v.states[1];
    const re_b = v.states[2], im_b = v.states[3];
    const p0   = re_a*re_a + im_a*im_a;
    const stateStr = `α=(${re_a.toFixed(3)},${im_a.toFixed(3)}i)  β=(${re_b.toFixed(3)},${im_b.toFixed(3)}i)`;
    document.getElementById('bloch-state').textContent =
      stateStr + `  P(|0⟩)=${p0.toFixed(4)}  P(|1⟩)=${(1-p0).toFixed(4)}`;

    // Coords overlay
    bctx.fillStyle = 'rgba(148,163,184,0.5)';
    bctx.font = '10px monospace';
    bctx.textAlign = 'left';
    bctx.fillText(`x:${bx.toFixed(3)}  y:${by.toFixed(3)}  z:${bz.toFixed(3)}`, 8, H - 8);
  }

  // ============================================================
  // Register renderer (probability bars + Bloch z-component)
  // ============================================================
  function drawRegister(v) {
    const W = registerCanvas.width, H = registerCanvas.height;
    rctx.clearRect(0,0,W,H);
    rctx.fillStyle = 'rgba(148,163,184,0.6)';
    rctx.font = '11px monospace';
    rctx.fillText('Qubit register  P(|0⟩) / Bloch z', 8, 16);

    const barW = (W - 60) / N_Q - 6;
    for (let q = 0; q < N_Q; q++) {
      const p0 = v.states[q*4]*v.states[q*4] + v.states[q*4+1]*v.states[q*4+1];
      const bz = v.bloch[q*3 + 2];
      const x  = 20 + q * ((W-40)/N_Q);

      // P(|0⟩) bar
      const bh = (H - 70) * p0;
      rctx.fillStyle = 'rgba(255,255,255,0.06)';
      rctx.fillRect(x, H-50, barW, -(H-70));
      rctx.fillStyle = p0 > 0.9 ? '#3b82f6' : p0 > 0.5 ? '#8b5cf6' : '#ec4899';
      rctx.fillRect(x, H-50, barW, -bh);

      // Bloch z dot
      const dotY = H/2 + 10 - (bz * (H/2 - 30));
      rctx.beginPath();
      rctx.arc(x + barW/2, dotY, 4, 0, Math.PI*2);
      rctx.fillStyle = '#fbbf24';
      rctx.fill();

      rctx.fillStyle = 'rgba(226,232,240,0.6)';
      rctx.font = '10px monospace';
      rctx.textAlign = 'center';
      rctx.fillText(`q${q}`, x + barW/2, H - 36);
      rctx.fillText(p0.toFixed(2), x + barW/2, H - 22);
    }

    rctx.fillStyle = 'rgba(251,191,36,0.5)';
    rctx.font = '9px monospace';
    rctx.textAlign = 'left';
    rctx.fillText('● Bloch z', 8, H - 4);
  }

  // ============================================================
  // QBER chart renderer
  // ============================================================
  function drawQBER(v) {
    const W = qberCanvas.width, H = qberCanvas.height;
    qctx.clearRect(0,0,W,H);
    const rounds = qc.qkd_rounds();

    qctx.fillStyle = 'rgba(148,163,184,0.6)';
    qctx.font = '11px monospace';
    qctx.fillText('QBER per round', 8, 16);

    // Security threshold line at 11%
    const pad = {l:40, r:10, t:24, b:28};
    const gH  = H - pad.t - pad.b;
    const threshold = 0.11;
    const thY = pad.t + gH * (1 - threshold);
    qctx.strokeStyle = 'rgba(239,68,68,0.4)';
    qctx.lineWidth   = 1;
    qctx.setLineDash([4,4]);
    qctx.beginPath(); qctx.moveTo(pad.l, thY); qctx.lineTo(W-pad.r, thY); qctx.stroke();
    qctx.setLineDash([]);
    qctx.fillStyle = 'rgba(239,68,68,0.5)';
    qctx.font = '9px monospace';
    qctx.textAlign = 'left';
    qctx.fillText('11% security limit', pad.l + 4, thY - 3);

    if (rounds === 0) return;

    // QBER bars
    const barW = Math.max(3, (W - pad.l - pad.r) / Math.min(rounds, N_ROUNDS) - 1);
    for (let r = 0; r < Math.min(rounds, N_ROUNDS); r++) {
      const q = v.qberHistory[r];
      if (isNaN(q)) continue;
      const x  = pad.l + r * ((W - pad.l - pad.r) / N_ROUNDS);
      const bh = gH * Math.min(q, 0.5);
      qctx.fillStyle = q > threshold ? '#ef4444' : '#10b981';
      qctx.fillRect(x, H - pad.b - bh, barW, bh);
    }

    // Y axis ticks
    [0, 0.1, 0.2, 0.3].forEach(v => {
      const y = pad.t + gH * (1 - v);
      qctx.fillStyle = 'rgba(148,163,184,0.35)';
      qctx.font = '9px monospace';
      qctx.textAlign = 'right';
      qctx.fillText(`${(v*100).toFixed(0)}%`, pad.l - 4, y + 3);
    });
  }

  function updateQKDStatus() {
    const avgQ   = qc.average_qber();
    const secure = qc.is_secure();
    const eve    = qc.eve_active();
    document.getElementById('qkd-status').innerHTML = `
      Rounds: <b>${qc.qkd_rounds()}</b><br>
      Avg QBER: <b style="color:${avgQ > 0.11 ? '#ef4444':'#10b981'}">${(avgQ*100).toFixed(2)}%</b><br>
      Eve active: <b style="color:${eve?'#ef4444':'#10b981'}">${eve ? 'YES ⚠' : 'No'}</b><br>
      Key length: <b>${qc.key_len()} bits</b><br>
      Security: <b style="color:${secure?'#10b981':'#ef4444'}">${secure ? '✓ Secure' : '✗ Compromised'}</b>
    `;
    const bits = Array.from(document.getElementById('key-display').dataset.bits || '');
    const keyArr = new Uint8Array(wasm.memory.buffer, qc.key_bits_ptr(), 64);
    const keyStr = Array.from(keyArr.slice(0, qc.key_len())).join('');
    document.getElementById('key-display').textContent = `Sifted key: ${keyStr || '(no key yet)'}`;
  }

  function renderAll() {
    const v = getViews();
    drawBloch(v);
    drawRegister(v);
    drawQBER(v);
    updateQKDStatus();
  }

  renderAll();

  // ============================================================
  // Gate buttons
  // ============================================================
  document.querySelectorAll('.gate-btn').forEach(btn => {
    btn.onclick = () => {
      qc.apply_gate(0, parseInt(btn.dataset.gate));
      renderAll();
    };
  });

  document.getElementById('btn-measure').onclick = () => {
    const r = qc.measure(0);
    const v = getViews();
    drawBloch(v); drawRegister(v);
    document.getElementById('bloch-state').textContent += `  → Measured: |${r}⟩`;
  };
  document.getElementById('btn-reset0').onclick = () => {
    qc.reset_qubit(0); renderAll();
  };

  [0,1,2,3].forEach(s => {
    document.getElementById(`btn-bell${s}`).onclick = () => {
      qc.prepare_bell_state(s); renderAll();
    };
  });

  // ============================================================
  // BB84 controls
  // ============================================================
  let bbRunning = false, bbTimer = null;

  function bbLoop() {
    qc.run_bb84_round();
    renderAll();
    if (bbRunning && qc.qkd_rounds() < N_ROUNDS) {
      bbTimer = setTimeout(bbLoop, 120);
    } else {
      bbRunning = false;
      document.getElementById('btn-bb84-auto').disabled  = false;
      document.getElementById('btn-bb84-pause').disabled = true;
    }
  }

  document.getElementById('btn-bb84-round').onclick = () => { qc.run_bb84_round(); renderAll(); };
  document.getElementById('btn-bb84-10').onclick    = () => {
    for (let i=0;i<10;i++) qc.run_bb84_round();
    renderAll();
  };
  document.getElementById('btn-bb84-auto').onclick = () => {
    bbRunning = true;
    document.getElementById('btn-bb84-auto').disabled  = true;
    document.getElementById('btn-bb84-pause').disabled = false;
    bbLoop();
  };
  document.getElementById('btn-bb84-pause').onclick = () => {
    bbRunning = false; clearTimeout(bbTimer);
    document.getElementById('btn-bb84-auto').disabled  = false;
    document.getElementById('btn-bb84-pause').disabled = true;
  };
  document.getElementById('btn-bb84-reset').onclick = () => {
    bbRunning = false; clearTimeout(bbTimer);
    qc.reset_all();
    document.getElementById('btn-bb84-auto').disabled  = false;
    document.getElementById('btn-bb84-pause').disabled = true;
    renderAll();
  };
  document.getElementById('chk-eve').onchange = function() {
    qc.set_eve(this.checked); renderAll();
  };
  document.getElementById('inp-noise').oninput = function() {
    qc.set_noise(this.value / 100);
    document.getElementById('lbl-noise').textContent = `${this.value}%`;
  };
}

startPipeline();
</script>