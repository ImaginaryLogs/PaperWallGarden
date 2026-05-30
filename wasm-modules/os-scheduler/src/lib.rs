use wasm_bindgen::prelude::*;

/// Operating System Process Scheduler Simulator
///
/// Simulates a preemptive multi-level feedback queue (MLFQ) scheduler —
/// the algorithm underlying most real-world OS schedulers (Linux CFS simplified,
/// Windows NTFS priority queues, macOS GCD).
///
/// Architecture:
///   - MAX_PROCS process slots (PCB array)
///   - 4 priority queues (Q0 = highest, Q3 = lowest / background)
///   - Round-robin within each queue with adaptive time quantum
///   - Processes demoted after using their full quantum (CPU-bound behaviour)
///   - Processes promoted after sleeping / blocking (I/O-bound behaviour)
///   - New processes always enter Q0 (optimistic: assume interactive)
///
/// PCB memory layout (per process, flat f32 array for zero-copy rendering):
///   [0] state        (0=empty, 1=ready, 2=running, 3=waiting/IO, 4=done)
///   [1] priority     (0-3, queue level)
///   [2] cpu_usage    (normalised 0.0-1.0, used for bar chart)
///   [3] wait_time    (total ticks spent waiting)
///   [4] burst_left   (remaining CPU burst in ticks)
///   [5] io_wait      (ticks until I/O completes, 0 = not blocked)
///   [6] pid          (process id, cast from u32)
///   [7] color_id     (0-7, stable colour assigned at spawn)

const MAX_PROCS: usize = 12;
const PCB_FIELDS: usize = 8;
const PCB_SIZE: usize = MAX_PROCS * PCB_FIELDS;
const NUM_QUEUES: usize = 4;

/// Time quantum per queue level (ticks)
const QUANTUM: [u32; NUM_QUEUES] = [4, 8, 16, 32];

// State constants (stored as f32 in PCB)
const STATE_EMPTY:   f32 = 0.0;
const STATE_READY:   f32 = 1.0;
const STATE_RUNNING: f32 = 2.0;
const STATE_WAITING: f32 = 3.0;
const STATE_DONE:    f32 = 4.0;

// PCB field indices
const F_STATE:     usize = 0;
const F_PRIORITY:  usize = 1;
const F_CPU_USAGE: usize = 2;
const F_WAIT_TIME: usize = 3;
const F_BURST:     usize = 4;
const F_IO_WAIT:   usize = 5;
const F_PID:       usize = 6;
const F_COLOR:     usize = 7;

// ---------------------------------------------------------------------------
// LCG RNG
// ---------------------------------------------------------------------------
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
    fn range(&mut self, lo: u32, hi: u32) -> u32 {
        lo + (self.next() as u32 % (hi - lo + 1))
    }
    fn f32(&mut self) -> f32 {
        ((self.next() >> 33) as f32) / (u32::MAX as f32)
    }
}

// ---------------------------------------------------------------------------
// Gantt chart entry: records which PID ran on which tick
// ---------------------------------------------------------------------------
#[derive(Clone, Copy)]
struct GanttEntry {
    pid: i32,      // -1 = idle
    priority: u8,
}

const GANTT_LEN: usize = 200; // last N ticks of history

// ---------------------------------------------------------------------------
// Scheduler struct
// ---------------------------------------------------------------------------
#[wasm_bindgen]
pub struct Scheduler {
    /// Process Control Blocks, flat f32 array [MAX_PROCS × PCB_FIELDS]
    pcb: Vec<f32>,
    /// Gantt chart entries (ring buffer)
    gantt_pids: Vec<i32>,
    gantt_priorities: Vec<u8>,
    gantt_head: usize,
    /// Per-queue runnable count (for dashboard)
    queue_lengths: Vec<u32>,
    /// Currently running process slot (-1 = idle)
    running: i32,
    /// Ticks consumed by current process in this quantum
    quantum_used: u32,
    /// Promotion boost: ticks after I/O wake-up with elevated priority
    io_boost_ticks: Vec<u32>,
    /// Total elapsed ticks
    tick: u64,
    /// Next PID to assign
    next_pid: u32,
    /// Context switch overhead counter (resets after each switch)
    ctx_overhead: u32,
    /// Throughput: completed processes per 100 ticks
    completed: u32,
    rng: Rng,
    /// Scheduling algorithm: 0=MLFQ, 1=Round-Robin, 2=Priority (no demotion)
    algorithm: u8,
}

// ---------------------------------------------------------------------------
// Helper: read/write PCB field for process slot `s`
// ---------------------------------------------------------------------------
#[inline]
fn pcb_get(pcb: &[f32], s: usize, field: usize) -> f32 {
    pcb[s * PCB_FIELDS + field]
}
#[inline]
fn pcb_set(pcb: &mut [f32], s: usize, field: usize, val: f32) {
    pcb[s * PCB_FIELDS + field] = val;
}

#[wasm_bindgen]
impl Scheduler {
    // -----------------------------------------------------------------------
    // Constructor
    // -----------------------------------------------------------------------
    pub fn new() -> Self {
        let mut pcb = vec![STATE_EMPTY; PCB_SIZE];
        let mut rng = Rng(2024);

        // Spawn 4 initial processes
        let mut next_pid = 0u32;
        for s in 0..4 {
            let pid = next_pid;
            next_pid += 1;
            pcb_set(&mut pcb, s, F_STATE, STATE_READY);
            pcb_set(&mut pcb, s, F_PRIORITY, 0.0);
            pcb_set(&mut pcb, s, F_CPU_USAGE, 0.0);
            pcb_set(&mut pcb, s, F_WAIT_TIME, 0.0);
            pcb_set(&mut pcb, s, F_BURST, rng.range(10, 50) as f32);
            pcb_set(&mut pcb, s, F_IO_WAIT, 0.0);
            pcb_set(&mut pcb, s, F_PID, pid as f32);
            pcb_set(&mut pcb, s, F_COLOR, (s % 8) as f32);
        }

        Scheduler {
            pcb,
            gantt_pids: vec![-1i32; GANTT_LEN],
            gantt_priorities: vec![0u8; GANTT_LEN],
            gantt_head: 0,
            queue_lengths: vec![0u32; NUM_QUEUES],
            running: -1,
            quantum_used: 0,
            io_boost_ticks: vec![0u32; MAX_PROCS],
            tick: 0,
            next_pid,
            ctx_overhead: 0,
            completed: 0,
            rng,
            algorithm: 0,
        }
    }

    // -----------------------------------------------------------------------
    // Advance simulation by one tick
    // -----------------------------------------------------------------------
    pub fn tick(&mut self) {
        self.tick += 1;

        // --- Phase 1: I/O completion ---
        for s in 0..MAX_PROCS {
            if pcb_get(&self.pcb, s, F_STATE) == STATE_WAITING {
                let io = pcb_get(&self.pcb, s, F_IO_WAIT);
                if io <= 1.0 {
                    // I/O done: wake up, boost priority
                    pcb_set(&mut self.pcb, s, F_IO_WAIT, 0.0);
                    pcb_set(&mut self.pcb, s, F_STATE, STATE_READY);
                    if self.algorithm == 0 { // MLFQ promotes on I/O wake
                        let p = pcb_get(&self.pcb, s, F_PRIORITY);
                        pcb_set(&mut self.pcb, s, F_PRIORITY, (p - 1.0).max(0.0));
                    }
                    self.io_boost_ticks[s] = 4;
                } else {
                    pcb_set(&mut self.pcb, s, F_IO_WAIT, io - 1.0);
                }
            }
        }

        // --- Phase 2: Increment wait time for READY processes ---
        for s in 0..MAX_PROCS {
            if pcb_get(&self.pcb, s, F_STATE) == STATE_READY {
                let w = pcb_get(&self.pcb, s, F_WAIT_TIME);
                pcb_set(&mut self.pcb, s, F_WAIT_TIME, w + 1.0);

                // Ageing: promote if waiting too long (anti-starvation)
                if self.algorithm == 0 && w > 0.0 && (w as u32 % 40 == 0) {
                    let p = pcb_get(&self.pcb, s, F_PRIORITY);
                    pcb_set(&mut self.pcb, s, F_PRIORITY, (p - 1.0).max(0.0));
                }
            }
        }

        // --- Phase 3: Scheduling decision ---
        if self.running == -1 || pcb_get(&self.pcb, self.running as usize, F_STATE) != STATE_RUNNING {
            self.running = self.select_next();
            self.quantum_used = 0;
            self.ctx_overhead = 2; // context switch costs 2 ticks
        }

        // --- Phase 4: Run the selected process ---
        if let Some(s) = (self.running as usize).checked_add(0).filter(|&s| s < MAX_PROCS) {
            if pcb_get(&self.pcb, s, F_STATE) == STATE_RUNNING || self.running >= 0 {
                self.quantum_used += 1;

                let burst = pcb_get(&self.pcb, s, F_BURST);
                if burst <= 0.0 {
                    // Process done
                    pcb_set(&mut self.pcb, s, F_STATE, STATE_DONE);
                    pcb_set(&mut self.pcb, s, F_CPU_USAGE, 1.0);
                    self.running = -1;
                    self.completed += 1;
                    // Spawn replacement after brief pause
                    if self.tick % 3 == 0 { self.spawn_process(); }
                } else {
                    // Execute one tick
                    pcb_set(&mut self.pcb, s, F_STATE, STATE_RUNNING);
                    pcb_set(&mut self.pcb, s, F_BURST, burst - 1.0);
                    let total_burst = burst + 1.0; // approx original
                    pcb_set(&mut self.pcb, s, F_CPU_USAGE, 1.0 - (burst - 1.0) / total_burst.max(1.0));

                    let prio = pcb_get(&self.pcb, s, F_PRIORITY) as usize;
                    let quantum = QUANTUM[prio.min(NUM_QUEUES - 1)];

                    // Check for I/O interrupt (random, more likely for interactive processes)
                    let io_prob = if prio == 0 { 0.08 } else { 0.03 };
                    if self.rng.f32() < io_prob {
                        let io_duration = self.rng.range(3, 12) as f32;
                        pcb_set(&mut self.pcb, s, F_STATE, STATE_WAITING);
                        pcb_set(&mut self.pcb, s, F_IO_WAIT, io_duration);
                        self.running = -1;
                    } else if self.quantum_used >= quantum {
                        // Quantum expired: preempt and demote (MLFQ only)
                        pcb_set(&mut self.pcb, s, F_STATE, STATE_READY);
                        if self.algorithm == 0 {
                            let p = pcb_get(&self.pcb, s, F_PRIORITY);
                            pcb_set(&mut self.pcb, s, F_PRIORITY, (p + 1.0).min((NUM_QUEUES - 1) as f32));
                        }
                        self.running = -1;
                    }
                }
            }
        }

        // --- Phase 5: Record Gantt entry ---
        let entry_pid = if self.running >= 0 {
            pcb_get(&self.pcb, self.running as usize, F_PID) as i32
        } else { -1 };
        let entry_prio = if self.running >= 0 {
            pcb_get(&self.pcb, self.running as usize, F_PRIORITY) as u8
        } else { 255 };
        self.gantt_pids[self.gantt_head] = entry_pid;
        self.gantt_priorities[self.gantt_head] = entry_prio;
        self.gantt_head = (self.gantt_head + 1) % GANTT_LEN;

        // --- Phase 6: Update queue length counters ---
        for q in 0..NUM_QUEUES { self.queue_lengths[q] = 0; }
        for s in 0..MAX_PROCS {
            let st = pcb_get(&self.pcb, s, F_STATE);
            if st == STATE_READY || st == STATE_RUNNING {
                let p = pcb_get(&self.pcb, s, F_PRIORITY) as usize;
                self.queue_lengths[p.min(NUM_QUEUES - 1)] += 1;
            }
        }

        // --- Phase 7: Stochastically spawn new processes ---
        let active = (0..MAX_PROCS)
            .filter(|&s| {
                let st = pcb_get(&self.pcb, s, F_STATE);
                st == STATE_READY || st == STATE_RUNNING || st == STATE_WAITING
            })
            .count();
        if active < 3 || (active < 8 && self.rng.f32() < 0.05) {
            self.spawn_process();
        }
    }

    // -----------------------------------------------------------------------
    // Select next process to run (scheduler policy)
    // -----------------------------------------------------------------------
    fn select_next(&mut self) -> i32 {
        match self.algorithm {
            1 => {
                // Pure round-robin: cycle through READY slots
                let start = (self.running + 1).max(0) as usize % MAX_PROCS;
                for i in 0..MAX_PROCS {
                    let s = (start + i) % MAX_PROCS;
                    if pcb_get(&self.pcb, s, F_STATE) == STATE_READY {
                        pcb_set(&mut self.pcb, s, F_STATE, STATE_RUNNING);
                        return s as i32;
                    }
                }
                -1
            }
            2 => {
                // Strict priority: highest priority READY process
                let mut best = -1i32;
                let mut best_p = NUM_QUEUES as f32;
                for s in 0..MAX_PROCS {
                    if pcb_get(&self.pcb, s, F_STATE) == STATE_READY {
                        let p = pcb_get(&self.pcb, s, F_PRIORITY);
                        if p < best_p {
                            best_p = p;
                            best = s as i32;
                        }
                    }
                }
                if best >= 0 {
                    pcb_set(&mut self.pcb, best as usize, F_STATE, STATE_RUNNING);
                }
                best
            }
            _ => {
                // MLFQ: pick highest-priority non-empty queue, then FIFO within it
                for q in 0..NUM_QUEUES {
                    for s in 0..MAX_PROCS {
                        if pcb_get(&self.pcb, s, F_STATE) == STATE_READY
                            && pcb_get(&self.pcb, s, F_PRIORITY) as usize == q
                        {
                            pcb_set(&mut self.pcb, s, F_STATE, STATE_RUNNING);
                            return s as i32;
                        }
                    }
                }
                -1
            }
        }
    }

    // -----------------------------------------------------------------------
    // Spawn a new process into an empty slot
    // -----------------------------------------------------------------------
    fn spawn_process(&mut self) {
        for s in 0..MAX_PROCS {
            if pcb_get(&self.pcb, s, F_STATE) == STATE_EMPTY
                || pcb_get(&self.pcb, s, F_STATE) == STATE_DONE
            {
                let pid = self.next_pid;
                self.next_pid += 1;
                pcb_set(&mut self.pcb, s, F_STATE, STATE_READY);
                pcb_set(&mut self.pcb, s, F_PRIORITY, 0.0);
                pcb_set(&mut self.pcb, s, F_CPU_USAGE, 0.0);
                pcb_set(&mut self.pcb, s, F_WAIT_TIME, 0.0);
                pcb_set(&mut self.pcb, s, F_BURST, self.rng.range(8, 60) as f32);
                pcb_set(&mut self.pcb, s, F_IO_WAIT, 0.0);
                pcb_set(&mut self.pcb, s, F_PID, pid as f32);
                pcb_set(&mut self.pcb, s, F_COLOR, (pid % 8) as f32);
                return;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Multi-tick advance
    // -----------------------------------------------------------------------
    pub fn advance(&mut self, n: u32) {
        for _ in 0..n { self.tick(); }
    }

    // -----------------------------------------------------------------------
    // Configuration
    // -----------------------------------------------------------------------
    pub fn set_algorithm(&mut self, a: u8) { self.algorithm = a % 3; }
    pub fn spawn_new(&mut self) { self.spawn_process(); }

    pub fn reset(&mut self) {
        *self = Scheduler::new();
    }

    // -----------------------------------------------------------------------
    // Zero-copy memory accessors
    // -----------------------------------------------------------------------
    pub fn pcb_ptr(&self) -> *const f32 { self.pcb.as_ptr() }
    pub fn gantt_pids_ptr(&self) -> *const i32 { self.gantt_pids.as_ptr() }
    pub fn gantt_prios_ptr(&self) -> *const u8 { self.gantt_priorities.as_ptr() }
    pub fn queue_lengths_ptr(&self) -> *const u32 { self.queue_lengths.as_ptr() }

    // -----------------------------------------------------------------------
    // Scalar accessors
    // -----------------------------------------------------------------------
    pub fn max_procs(&self) -> usize { MAX_PROCS }
    pub fn pcb_fields(&self) -> usize { PCB_FIELDS }
    pub fn gantt_len(&self) -> usize { GANTT_LEN }
    pub fn gantt_head(&self) -> usize { self.gantt_head }
    pub fn num_queues(&self) -> usize { NUM_QUEUES }
    pub fn current_tick(&self) -> u32 { self.tick as u32 }
    pub fn running_slot(&self) -> i32 { self.running }
    pub fn completed_count(&self) -> u32 { self.completed }
    pub fn cpu_util(&self) -> f32 {
        // Fraction of non-idle Gantt entries in last 50 ticks
        let window = 50usize;
        let count = self.gantt_pids.iter().rev().take(window)
            .filter(|&&p| p >= 0).count();
        count as f32 / window as f32
    }
}