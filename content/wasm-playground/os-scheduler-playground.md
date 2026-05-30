---
title: OS Process Scheduler – MLFQ Simulator
tags: [simulation, operating-systems, scheduler, mlfq, wasm]
---

# OS Process Scheduler

Simulates a **Multi-Level Feedback Queue (MLFQ)** — the algorithm at the heart
of Linux, Windows, and macOS schedulers. Processes enter Queue 0 (highest priority)
and are demoted when they exhaust their time quantum (CPU-bound behaviour),
promoted after I/O completion (interactive behaviour), and aged to prevent starvation.

<div id="os-controls" style="margin:0.5rem 0 1rem;display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;font-size:0.875rem;">
  <button id="btn-tick"   style="padding:6px 14px;cursor:pointer;">Tick ×1</button>
  <button id="btn-t10"    style="padding:6px 14px;cursor:pointer;">×10</button>
  <button id="btn-run"    style="padding:6px 14px;cursor:pointer;">▶ Run</button>
  <button id="btn-pause"  style="padding:6px 14px;cursor:pointer;" disabled>⏸ Pause</button>
  <button id="btn-spawn"  style="padding:6px 14px;cursor:pointer;">+ Spawn Process</button>
  <button id="btn-reset"  style="padding:6px 14px;cursor:pointer;">↺ Reset</button>
  <select id="sel-algo" style="padding:5px 8px;font-size:0.85rem;">
    <option value="0">MLFQ</option>
    <option value="1">Round-Robin</option>
    <option value="2">Priority</option>
  </select>
</div>

<!-- Top row: PCB table + queue bars -->
<div style="display:grid;grid-template-columns:2fr 1fr;gap:1rem;margin-bottom:1rem;">
  <canvas id="canvas-pcb"    width="560" height="260"
    style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;">
  </canvas>
  <canvas id="canvas-queues" width="220" height="260"
    style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;">
  </canvas>
</div>

<!-- Gantt chart -->
<canvas id="canvas-gantt" width="800" height="100"
  style="width:100%;border:1px solid rgba(128,128,128,0.25);border-radius:6px;background:#0d1117;margin-bottom:0.5rem;">
</canvas>

<div id="os-stats" style="font-size:0.82rem;opacity:0.65;font-family:monospace;display:flex;gap:2rem;flex-wrap:wrap;"></div>

<script type="module">
const isGitHubPages = window.location.hostname.includes('github.io');
const basePath = isGitHubPages ? '/PaperWallGarden' : '';
const WASM_JS = `${basePath}/wasm/os_scheduler.js`;
const WASM_BG = `${basePath}/wasm/os_scheduler_bg.wasm`;

// Per-process colour palette (8 colours, indexed by color_id field)
const PROC_COLORS = [
  '#3b82f6','#8b5cf6','#ec4899','#f59e0b',
  '#10b981','#ef4444','#06b6d4','#f97316'
];
const QUEUE_COLORS = ['#ef4444','#f59e0b','#3b82f6','#6b7280'];
const STATE_NAMES  = ['empty','ready','running','waiting','done'];

// PCB field indices (mirror Rust consts)
const F_STATE=0,F_PRIO=1,F_CPU=2,F_WAIT=3,F_BURST=4,F_IO=5,F_PID=6,F_COL=7;

async function startPipeline() {
  let module, wasm, sched;
  try {
    module = await import(WASM_JS);
    wasm   = await module.default(WASM_BG);
    sched  = module.Scheduler.new();
  } catch (err) {
    document.getElementById('os-stats').textContent = `WASM load failed: ${err}`;
    return;
  }

  const pcbCanvas    = document.getElementById('canvas-pcb');
  const qCanvas      = document.getElementById('canvas-queues');
  const ganttCanvas  = document.getElementById('canvas-gantt');
  const pctx = pcbCanvas.getContext('2d');
  const qctx = qCanvas.getContext('2d');
  const gctx = ganttCanvas.getContext('2d');
  const statsEl = document.getElementById('os-stats');

  const NPROC    = sched.max_procs();
  const NFIELDS  = sched.pcb_fields();
  const NGANTT   = sched.gantt_len();
  const NQUEUES  = sched.num_queues();

  // Zero-copy views
  function getViews() {
    const mem = wasm.memory.buffer;
    return {
      pcb:        new Float32Array(mem, sched.pcb_ptr(),          NPROC * NFIELDS),
      ganttPids:  new Int32Array(mem,   sched.gantt_pids_ptr(),   NGANTT),
      ganttPrios: new Uint8Array(mem,   sched.gantt_prios_ptr(),  NGANTT),
      queueLens:  new Uint32Array(mem,  sched.queue_lengths_ptr(),NQUEUES),
    };
  }

  // ============================================================
  // PCB Table renderer
  // ============================================================
  function drawPCB(v) {
    const W = pcbCanvas.width, H = pcbCanvas.height;
    pctx.clearRect(0,0,W,H);

    const cols    = ['PID','State','Priority','Burst','Wait','CPU%','I/O'];
    const colW    = [42,60,60,52,52,52,52];
    const rowH    = 20;
    const startY  = 28;
    let  x0       = 8;

    // Header
    pctx.fillStyle = 'rgba(148,163,184,0.5)';
    pctx.font      = '11px monospace';
    cols.forEach((c, i) => {
      const cx = x0 + colW.slice(0,i).reduce((a,b)=>a+b,0);
      pctx.fillText(c, cx, 16);
    });
    pctx.strokeStyle = 'rgba(148,163,184,0.2)';
    pctx.lineWidth = 0.5;
    pctx.beginPath(); pctx.moveTo(0,20); pctx.lineTo(W,20); pctx.stroke();

    // Rows
    for (let s = 0; s < NPROC; s++) {
      const off   = s * NFIELDS;
      const state = v.pcb[off + F_STATE];
      if (state === 0) continue; // empty slot

      const pid    = v.pcb[off + F_PID]   | 0;
      const prio   = v.pcb[off + F_PRIO]  | 0;
      const burst  = v.pcb[off + F_BURST] | 0;
      const wait   = v.pcb[off + F_WAIT]  | 0;
      const cpuPct = (v.pcb[off + F_CPU] * 100) | 0;
      const io     = v.pcb[off + F_IO]    | 0;
      const colId  = v.pcb[off + F_COL]   | 0;
      const y      = startY + s * rowH;

      // Row highlight for running process
      if (state === 2) {
        pctx.fillStyle = `rgba(${parseInt(PROC_COLORS[colId%8].slice(1,3),16)},${parseInt(PROC_COLORS[colId%8].slice(3,5),16)},${parseInt(PROC_COLORS[colId%8].slice(5,7),16)},0.12)`;
        pctx.fillRect(0, y - 13, W, rowH);
      }

      const vals = [pid, STATE_NAMES[state|0], `Q${prio}`, burst, wait, `${cpuPct}%`, io>0?io:'—'];
      pctx.fillStyle = state === 2 ? PROC_COLORS[colId % 8] :
                       state === 3 ? '#f59e0b' :
                       state === 4 ? 'rgba(100,116,139,0.5)' : 'rgba(226,232,240,0.75)';
      pctx.font = state === 2 ? 'bold 11px monospace' : '11px monospace';

      vals.forEach((val, i) => {
        const cx = x0 + colW.slice(0,i).reduce((a,b)=>a+b,0);
        pctx.fillText(String(val), cx, y);
      });

      // CPU progress bar
      const barX = x0 + colW.slice(0,5).reduce((a,b)=>a+b,0);
      pctx.fillStyle = 'rgba(255,255,255,0.08)';
      pctx.fillRect(barX, y - 10, colW[5] - 4, 12);
      pctx.fillStyle = PROC_COLORS[colId % 8];
      pctx.globalAlpha = 0.6;
      pctx.fillRect(barX, y - 10, (colW[5] - 4) * v.pcb[off + F_CPU], 12);
      pctx.globalAlpha = 1.0;
    }
  }

  // ============================================================
  // Queue bar renderer
  // ============================================================
  function drawQueues(v) {
    const W = qCanvas.width, H = qCanvas.height;
    qctx.clearRect(0,0,W,H);
    qctx.fillStyle = 'rgba(148,163,184,0.6)';
    qctx.font = '11px monospace';
    qctx.fillText('Ready queues', 10, 16);

    const maxQ = Math.max(...Array.from(v.queueLens), 1);
    const barH = (H - 60) / NQUEUES - 8;
    const labels = ['Q0 (highest)','Q1','Q2','Q3 (lowest)'];

    for (let q = 0; q < NQUEUES; q++) {
      const y    = 28 + q * ((H - 50) / NQUEUES);
      const len  = v.queueLens[q];
      const barW = Math.max(4, (W - 90) * len / Math.max(maxQ, 1));

      qctx.fillStyle = 'rgba(255,255,255,0.05)';
      qctx.fillRect(80, y, W - 90, barH);
      qctx.fillStyle = QUEUE_COLORS[q];
      qctx.globalAlpha = 0.8;
      qctx.fillRect(80, y, barW, barH);
      qctx.globalAlpha = 1.0;

      qctx.fillStyle = 'rgba(148,163,184,0.6)';
      qctx.font = '10px monospace';
      qctx.textAlign = 'right';
      qctx.fillText(labels[q], 76, y + barH/2 + 4);
      qctx.textAlign = 'left';
      qctx.fillStyle = 'rgba(226,232,240,0.8)';
      qctx.fillText(len, 84 + barW, y + barH/2 + 4);
    }
  }

  // ============================================================
  // Gantt chart renderer
  // ============================================================
  function drawGantt(v) {
    const W = ganttCanvas.width, H = ganttCanvas.height;
    gctx.clearRect(0,0,W,H);
    gctx.fillStyle = 'rgba(148,163,184,0.5)';
    gctx.font = '10px monospace';
    gctx.fillText('CPU Gantt (last 200 ticks →)', 6, 12);

    const head   = sched.gantt_head();
    const cellW  = (W - 10) / NGANTT;

    for (let i = 0; i < NGANTT; i++) {
      // Reorder ring buffer so oldest is at left
      const idx  = (head + i) % NGANTT;
      const pid  = v.ganttPids[idx];
      const prio = v.ganttPrios[idx];
      const x    = 5 + i * cellW;

      if (pid < 0) {
        gctx.fillStyle = 'rgba(255,255,255,0.04)';
      } else {
        const colorIdx = pid % 8;
        const hex = PROC_COLORS[colorIdx];
        const r = parseInt(hex.slice(1,3),16);
        const g = parseInt(hex.slice(3,5),16);
        const b = parseInt(hex.slice(5,7),16);
        gctx.fillStyle = `rgba(${r},${g},${b},0.75)`;
      }
      gctx.fillRect(x, 18, Math.max(1, cellW - 0.5), H - 30);
    }

    // Tick label at right
    gctx.fillStyle = 'rgba(148,163,184,0.4)';
    gctx.font = '9px monospace';
    gctx.textAlign = 'right';
    gctx.fillText(`tick ${sched.current_tick()}`, W - 6, H - 4);
  }

  function updateStats() {
    const util = (sched.cpu_util() * 100).toFixed(1);
    statsEl.innerHTML = [
      `Tick: <b>${sched.current_tick()}</b>`,
      `CPU util: <b>${util}%</b>`,
      `Completed: <b>${sched.completed_count()}</b>`,
      `Running: <b>PID ${sched.running_slot() >= 0 ? 'active' : 'idle'}</b>`,
    ].map(s=>`<span>${s}</span>`).join('');
  }

  function renderAll() {
    const v = getViews();
    drawPCB(v);
    drawQueues(v);
    drawGantt(v);
    updateStats();
  }

  renderAll();

  // ============================================================
  // Controls
  // ============================================================
  let running = false, rafId = null;

  function runLoop() {
    sched.tick();
    renderAll();
    if (running) rafId = requestAnimationFrame(runLoop);
  }

  document.getElementById('btn-tick').onclick  = () => { sched.tick(); renderAll(); };
  document.getElementById('btn-t10').onclick   = () => { sched.advance(10); renderAll(); };
  document.getElementById('btn-spawn').onclick = () => { sched.spawn_new(); renderAll(); };
  document.getElementById('btn-reset').onclick = () => {
    running = false; cancelAnimationFrame(rafId);
    sched.reset();
    document.getElementById('btn-run').disabled   = false;
    document.getElementById('btn-pause').disabled = true;
    renderAll();
  };
  document.getElementById('btn-run').onclick = () => {
    running = true;
    document.getElementById('btn-run').disabled   = true;
    document.getElementById('btn-pause').disabled = false;
    rafId = requestAnimationFrame(runLoop);
  };
  document.getElementById('btn-pause').onclick = () => {
    running = false; cancelAnimationFrame(rafId);
    document.getElementById('btn-run').disabled   = false;
    document.getElementById('btn-pause').disabled = true;
  };
  document.getElementById('sel-algo').onchange = function() {
    sched.set_algorithm(parseInt(this.value));
  };
}

startPipeline();
</script>