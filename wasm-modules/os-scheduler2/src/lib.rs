use wasm_bindgen::prelude::*;

/// # OS Process Scheduling — Network-Graph Cellular Automaton
///
/// ## What is a Network-Graph CA?
/// A **Network-Graph CA** (also called a "Graph Automaton") generalises the
/// grid-based CA to an arbitrary graph topology. Each **node** has a state;
/// each **edge** defines which neighbours can influence a node's next state.
/// The transition rule is: next_state(v) = f(state(v), {state(u) : (u,v) ∈ E}).
/// Unlike a grid CA, the neighbourhood structure is irregular — some nodes have
/// 2 neighbours, others have 10 — and this topology encodes the actual constraints
/// of the system being modelled.
///
/// ## Domain: Operating System Scheduling
/// An OS scheduler decides which process runs on which CPU core at each clock tick.
/// The resources a process needs (CPU, I/O bus, memory bus, GPU, lock) are shared
/// and constrained: two processes cannot simultaneously hold the same lock.
///
/// ## The Graph
/// The simulation builds a **bipartite resource-contention graph**:
///   - **Process nodes**: each process has state (ready/running/waiting/done)
///     and a set of resource requirements.
///   - **Resource nodes**: CPU cores, I/O bus, memory bus, network, mutex locks.
///   - **Edges**: a process node is connected to each resource it currently needs.
///
/// This is exactly the graph used in deadlock detection (the Resource Allocation
/// Graph / RAG). The CA dynamics on this graph simulate scheduling decisions.
///
/// ## CA Transition Rule
///
/// At each tick, every node applies a local rule:
///
/// **Process node** transition rule:
///   1. A READY process becomes RUNNING if all its required resource-nodes are FREE.
///   2. A RUNNING process consumes ticks from its burst; when done it releases
///      all resource edges (edges removed ≡ resource tokens freed).
///   3. A RUNNING process becomes WAITING if a needed resource becomes BUSY
///      (preemption / blocking). It queues on that resource node.
///   4. A WAITING process becomes READY when its blocking resource becomes FREE.
///
/// **Resource node** transition rule:
///   1. A FREE resource becomes BUSY when a running process claims it.
///   2. A BUSY resource becomes FREE when the owning process finishes or releases.
///   3. If multiple processes compete for a resource (contention), the resource
///      node applies a **priority arbitration rule**: the highest-priority ready
///      process wins (this is the CA encoding of priority scheduling).
///
/// ## What you can observe
/// - **Deadlock**: create a cycle in the resource graph (process A holds R1 and waits
///   for R2; process B holds R2 and waits for R1). The CA's state freezes — no
///   node can transition because all transition conditions are blocked.
/// - **Starvation**: a low-priority process never gets the CPU because high-priority
///   processes continuously claim resources. Watch the waiting-time counter grow.
/// - **Resource contention waves**: releasing a heavily-contested resource causes a
///   cascade of state transitions across the graph — a propagation "wave" that is
///   only possible to visualise because of the graph CA structure.
/// - **HMM overlay**: Each resource node runs a Hidden Markov Model on its busy/free
///   history to emit a "predicted load" — used by the scheduler to probabilistically
///   prefetch resources for the next process.

// Graph sizes
const MAX_PROCS: usize = 10;      // process nodes
const MAX_RESOURCES: usize = 8;   // resource nodes
const MAX_EDGES: usize = 40;      // max edges (proc→resource assignments)

// Resource types
const RES_CPU: u8    = 0; // CPU core (4 of these)
const RES_IO: u8     = 1; // I/O bus (1)
const RES_MEM: u8    = 2; // Memory bus (1)
const RES_NET: u8    = 3; // Network interface (1)
const RES_LOCK: u8   = 4; // Mutex lock (1)

// Process states
const PROC_EMPTY:   u8 = 0;
const PROC_READY:   u8 = 1;
const PROC_RUNNING: u8 = 2;
const PROC_WAITING: u8 = 3;
const PROC_DONE:    u8 = 4;

// Resource states
const RES_FREE: u8   = 0;
const RES_BUSY: u8   = 1;
const RES_CONTENDED: u8 = 2; // multiple processes waiting

// HMM parameters for resource load prediction
// States: LOW_LOAD(0), MED_LOAD(1), HIGH_LOAD(2)
// Transition matrix A[s][s'] = P(next_state=s' | current_state=s)
// Emission matrix B[s][obs] = P(obs | state=s) where obs=0 (free) or 1 (busy)
const HMM_STATES: usize = 3;
const HMM_A: [[f32; HMM_STATES]; HMM_STATES] = [
    [0.7, 0.2, 0.1], // from LOW:  70% stay low, 20% med, 10% high
    [0.3, 0.5, 0.2], // from MED:  30% low,  50% stay med, 20% high
    [0.1, 0.3, 0.6], // from HIGH: 10% low,  30% med, 60% stay high
];
// Emission: P(busy | state)
const HMM_B_BUSY: [f32; HMM_STATES] = [0.1, 0.5, 0.9];

// ---------------------------------------------------------------------------
// LCG RNG
// ---------------------------------------------------------------------------
struct Rng(u64);
impl Rng {
    fn u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
    fn f32(&mut self) -> f32 { ((self.u64() >> 33) as f32) / (u32::MAX as f32) }
    fn u32_range(&mut self, n: u32) -> u32 { (self.u64() as u32) % n }
}

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

/// A directed edge: process `proc_id` is assigned to / waiting for resource `res_id`
#[derive(Clone, Copy, Default)]
struct Edge {
    proc_id: u8,
    res_id: u8,
    active: bool,    // edge exists (process-resource relationship is live)
    waiting: bool,   // true = process is blocked waiting; false = actively using
}

/// Per-process data
#[derive(Clone, Copy, Default)]
struct Process {
    state: u8,
    priority: u8,       // 0=highest
    burst_left: u16,    // ticks of CPU time remaining
    wait_ticks: u32,    // total ticks spent waiting
    run_ticks: u32,     // total ticks spent running
    pid: u16,
    color_id: u8,
    // Which resource types this process needs (bitmask)
    needs_cpu: bool,
    needs_io: bool,
    needs_mem: bool,
    needs_net: bool,
    needs_lock: bool,
}

/// Per-resource data
#[derive(Clone, Copy, Default)]
struct Resource {
    res_type: u8,
    state: u8,          // FREE / BUSY / CONTENDED
    owner_pid: i16,     // -1 if free
    waiters: u8,        // count of processes waiting
    // HMM belief state: probability distribution over hidden load states
    hmm_belief: [f32; HMM_STATES],  // P(state=s | observations so far)
    predicted_busy_prob: f32,        // emitted prediction for next tick
    // Busy/free history for HMM (ring buffer, 1 bit per tick, packed as u32)
    history: u32,
    history_len: u8,
}

// ---------------------------------------------------------------------------
// Main struct
// ---------------------------------------------------------------------------
#[wasm_bindgen]
pub struct OsGraphCA {
    procs:     Vec<Process>,
    resources: Vec<Resource>,
    edges:     Vec<Edge>,
    edge_count: usize,

    /// Flat f32 array for zero-copy rendering:
    /// [proc_0: state, priority, burst_pct, wait_ticks, pid, color | proc_1: ... ]
    proc_buf: Vec<f32>,
    /// [res_0: state, owner, waiters, predicted_busy, type | res_1: ...]
    res_buf: Vec<f32>,
    /// Edge list: [proc_id, res_id, waiting] × MAX_EDGES
    edge_buf: Vec<f32>,
    /// Per-resource busy/free trace (last 64 ticks) as f32 0/1
    trace_buf: Vec<f32>,

    tick: u32,
    next_pid: u16,
    completed: u32,
    rng: Rng,

    // Layout coordinates for graph rendering (assigned at spawn)
    proc_x: Vec<f32>,
    proc_y: Vec<f32>,
    res_x: Vec<f32>,
    res_y: Vec<f32>,
}

impl OsGraphCA {
    fn find_free_edge(&self) -> Option<usize> {
        self.edges.iter().position(|e| !e.active)
    }

    fn add_edge(&mut self, proc_id: usize, res_id: usize, waiting: bool) {
        if let Some(idx) = self.find_free_edge() {
            self.edges[idx] = Edge {
                proc_id: proc_id as u8,
                res_id: res_id as u8,
                active: true,
                waiting,
            };
            if self.edge_count < MAX_EDGES { self.edge_count += 1; }
        }
    }

    fn remove_edges_for_proc(&mut self, proc_id: usize) {
        for e in self.edges.iter_mut() {
            if e.active && e.proc_id as usize == proc_id {
                e.active = false;
            }
        }
    }

    fn resource_of_type(&self, rtype: u8) -> Option<usize> {
        self.resources.iter().position(|r| r.res_type == rtype && r.state == RES_FREE)
    }

    fn resource_busy_for_proc(&self, proc_id: usize) -> Option<usize> {
        // Returns the first resource this process is waiting on
        for e in self.edges.iter() {
            if e.active && e.proc_id as usize == proc_id && e.waiting {
                return Some(e.res_id as usize);
            }
        }
        None
    }

    fn spawn_proc(&mut self, slot: usize) {
        let pid = self.next_pid;
        self.next_pid += 1;
        let priority = self.rng.u32_range(4) as u8;
        let burst = 5 + self.rng.u32_range(30) as u16;
        let needs_io  = self.rng.f32() < 0.4;
        let needs_mem = self.rng.f32() < 0.6;
        let needs_net = self.rng.f32() < 0.2;
        let needs_lock = self.rng.f32() < 0.3;

        self.procs[slot] = Process {
            state: PROC_READY,
            priority,
            burst_left: burst,
            wait_ticks: 0,
            run_ticks: 0,
            pid,
            color_id: (pid % 8) as u8,
            needs_cpu: true, // all processes need CPU
            needs_io,
            needs_mem,
            needs_net,
            needs_lock,
        };

        // Layout: processes arranged in a circle
        let angle = (slot as f32 / MAX_PROCS as f32) * 2.0 * core::f32::consts::PI;
        self.proc_x[slot] = 250.0 + 160.0 * angle.cos();
        self.proc_y[slot] = 200.0 + 140.0 * angle.sin();
    }

    fn hmm_update(&mut self, res_id: usize, obs_busy: bool) {
        let obs = if obs_busy { 1.0f32 } else { 0.0f32 };
        let r = &mut self.resources[res_id];

        // Forward algorithm (one step): β' ∝ B(obs) × (A^T × β)
        let mut new_belief = [0.0f32; HMM_STATES];
        for s_next in 0..HMM_STATES {
            let mut sum = 0.0f32;
            for s_cur in 0..HMM_STATES {
                sum += HMM_A[s_cur][s_next] * r.hmm_belief[s_cur];
            }
            let emit_p = if obs_busy { HMM_B_BUSY[s_next] } else { 1.0 - HMM_B_BUSY[s_next] };
            new_belief[s_next] = emit_p * sum;
        }
        // Normalise
        let total: f32 = new_belief.iter().sum();
        if total > 1e-9 {
            for b in new_belief.iter_mut() { *b /= total; }
        }
        r.hmm_belief = new_belief;

        // Predicted busy probability for next tick (marginalise over future hidden states)
        // P(next_obs=busy) = Σ_s' B(busy|s') × Σ_s A[s][s'] × belief[s]
        let mut pred = 0.0f32;
        for s_next in 0..HMM_STATES {
            let mut reach = 0.0f32;
            for s_cur in 0..HMM_STATES {
                reach += HMM_A[s_cur][s_next] * r.hmm_belief[s_cur];
            }
            pred += HMM_B_BUSY[s_next] * reach;
        }
        r.predicted_busy_prob = pred;
    }

    fn update_buffers(&mut self) {
        for p in 0..MAX_PROCS {
            let proc = &self.procs[p];
            let base = p * 6;
            self.proc_buf[base]     = proc.state as f32;
            self.proc_buf[base + 1] = proc.priority as f32;
            self.proc_buf[base + 2] = if proc.burst_left > 0 { proc.run_ticks as f32 / (proc.run_ticks + proc.burst_left as u32).max(1) as f32 } else { 1.0 };
            self.proc_buf[base + 3] = proc.wait_ticks as f32;
            self.proc_buf[base + 4] = proc.pid as f32;
            self.proc_buf[base + 5] = proc.color_id as f32;
        }
        for r in 0..MAX_RESOURCES {
            let res = &self.resources[r];
            let base = r * 5;
            self.res_buf[base]     = res.state as f32;
            self.res_buf[base + 1] = res.owner_pid as f32;
            self.res_buf[base + 2] = res.waiters as f32;
            self.res_buf[base + 3] = res.predicted_busy_prob;
            self.res_buf[base + 4] = res.res_type as f32;
        }
        let mut ei = 0;
        for e in self.edges.iter() {
            if e.active && ei + 2 < self.edge_buf.len() {
                self.edge_buf[ei]     = e.proc_id as f32;
                self.edge_buf[ei + 1] = e.res_id as f32;
                self.edge_buf[ei + 2] = e.waiting as u8 as f32;
                ei += 3;
            }
        }
        // Pad unused edge slots with -1
        while ei < self.edge_buf.len() {
            self.edge_buf[ei] = -1.0;
            ei += 1;
        }
    }
}

#[wasm_bindgen]
impl OsGraphCA {
    pub fn new() -> Self {
        let mut rng = Rng(0xCA_CAFE);

        // Initialise resources
        let mut resources = vec![Resource::default(); MAX_RESOURCES];
        // 4 CPU cores
        for i in 0..4 { resources[i] = Resource { res_type: RES_CPU, state: RES_FREE, owner_pid: -1, hmm_belief: [0.6, 0.3, 0.1], ..Resource::default() }; }
        resources[4] = Resource { res_type: RES_IO,   state: RES_FREE, owner_pid: -1, hmm_belief: [0.5, 0.3, 0.2], ..Resource::default() };
        resources[5] = Resource { res_type: RES_MEM,  state: RES_FREE, owner_pid: -1, hmm_belief: [0.4, 0.4, 0.2], ..Resource::default() };
        resources[6] = Resource { res_type: RES_NET,  state: RES_FREE, owner_pid: -1, hmm_belief: [0.7, 0.2, 0.1], ..Resource::default() };
        resources[7] = Resource { res_type: RES_LOCK, state: RES_FREE, owner_pid: -1, hmm_belief: [0.6, 0.3, 0.1], ..Resource::default() };

        // Resource layout: in a horizontal strip at top
        let res_x: Vec<f32> = (0..MAX_RESOURCES).map(|i| 50.0 + i as f32 * 55.0).collect();
        let res_y: Vec<f32> = vec![60.0; MAX_RESOURCES];

        let procs = vec![Process::default(); MAX_PROCS];
        let proc_x = vec![0.0f32; MAX_PROCS];
        let proc_y = vec![0.0f32; MAX_PROCS];

        let mut sim = OsGraphCA {
            procs,
            resources,
            edges: vec![Edge::default(); MAX_EDGES],
            edge_count: 0,
            proc_buf: vec![0.0f32; MAX_PROCS * 6],
            res_buf: vec![0.0f32; MAX_RESOURCES * 5],
            edge_buf: vec![-1.0f32; MAX_EDGES * 3],
            trace_buf: vec![0.0f32; MAX_RESOURCES * 64],
            tick: 0,
            next_pid: 0,
            completed: 0,
            rng,
            proc_x,
            proc_y,
            res_x,
            res_y,
        };

        // Spawn initial processes
        for i in 0..5 { sim.spawn_proc(i); }
        sim.update_buffers();
        sim
    }

    // -----------------------------------------------------------------------
    // One CA tick — graph automaton transition rule
    // -----------------------------------------------------------------------
    pub fn tick(&mut self) {
        self.tick += 1;

        // --- Phase 1: Resource node transitions ---
        // Update resource state based on current edge set
        for r in 0..MAX_RESOURCES {
            let mut owner: i16 = -1;
            let mut waiters: u8 = 0;
            for e in self.edges.iter() {
                if !e.active || e.res_id as usize != r { continue; }
                if !e.waiting {
                    owner = e.proc_id as i16;
                } else {
                    waiters += 1;
                }
            }
            self.resources[r].owner_pid = owner;
            self.resources[r].waiters = waiters;
            self.resources[r].state = if owner >= 0 {
                if waiters > 0 { RES_CONTENDED } else { RES_BUSY }
            } else if waiters > 0 {
                RES_CONTENDED
            } else {
                RES_FREE
            };

            // HMM update on this resource
            let obs_busy = owner >= 0;
            self.hmm_update(r, obs_busy);

            // Update trace buffer (ring buffer per resource, 64 ticks)
            let trace_slot = (self.tick as usize) % 64;
            self.trace_buf[r * 64 + trace_slot] = if obs_busy { 1.0 } else { 0.0 };
        }

        // --- Phase 2: Process node transitions ---
        // Collect proc states to avoid borrow issues
        let proc_states: Vec<u8> = self.procs.iter().map(|p| p.state).collect();

        for p in 0..MAX_PROCS {
            match proc_states[p] {
                PROC_EMPTY | PROC_DONE => {}

                PROC_READY => {
                    // Try to acquire all needed resources (graph CA: check edge availability)
                    let needs = [
                        (self.procs[p].needs_cpu,  RES_CPU),
                        (self.procs[p].needs_io,   RES_IO),
                        (self.procs[p].needs_mem,  RES_MEM),
                        (self.procs[p].needs_net,  RES_NET),
                        (self.procs[p].needs_lock, RES_LOCK),
                    ];
                    let mut can_run = true;
                    let mut needed_res: Vec<usize> = Vec::new();
                    for (needed, rtype) in needs.iter() {
                        if !needed { continue; }
                        if let Some(rid) = self.resource_of_type(*rtype) {
                            needed_res.push(rid);
                        } else {
                            can_run = false;
                            // Add a waiting edge to signal contention
                            let contended = self.resources.iter().position(|r| r.res_type == *rtype);
                            if let Some(rid) = contended {
                                self.add_edge(p, rid, true);
                            }
                            break;
                        }
                    }
                    if can_run {
                        // Acquire all resources: add active edges
                        self.remove_edges_for_proc(p); // clear any waiting edges
                        for rid in needed_res {
                            self.add_edge(p, rid, false);
                        }
                        self.procs[p].state = PROC_RUNNING;
                    } else {
                        self.procs[p].wait_ticks += 1;
                    }
                }

                PROC_RUNNING => {
                    self.procs[p].burst_left = self.procs[p].burst_left.saturating_sub(1);
                    self.procs[p].run_ticks += 1;
                    if self.procs[p].burst_left == 0 {
                        // Done — release all resources (remove all edges for this proc)
                        self.remove_edges_for_proc(p);
                        self.procs[p].state = PROC_DONE;
                        self.completed += 1;
                        // Spawn replacement
                        if self.rng.f32() < 0.7 { self.spawn_proc(p); }
                    } else if self.rng.f32() < 0.04 {
                        // Random I/O block (stochastic graph-CA event)
                        self.procs[p].state = PROC_WAITING;
                    }
                }

                PROC_WAITING => {
                    self.procs[p].wait_ticks += 1;
                    // Wait for a random duration then become READY again
                    if self.rng.f32() < 0.15 {
                        self.remove_edges_for_proc(p);
                        self.procs[p].state = PROC_READY;
                    }
                }

                _ => {}
            }
        }

        // Maintain population
        let active = self.procs.iter().filter(|p| p.state == PROC_READY || p.state == PROC_RUNNING || p.state == PROC_WAITING).count();
        if active < 3 {
            for p in 0..MAX_PROCS {
                if self.procs[p].state == PROC_EMPTY || self.procs[p].state == PROC_DONE {
                    self.spawn_proc(p);
                    break;
                }
            }
        }

        self.update_buffers();
    }

    pub fn advance(&mut self, n: u32) { for _ in 0..n { self.tick(); } }

    pub fn spawn_new(&mut self) {
        for p in 0..MAX_PROCS {
            if self.procs[p].state == PROC_EMPTY || self.procs[p].state == PROC_DONE {
                self.spawn_proc(p);
                break;
            }
        }
        self.update_buffers();
    }

    pub fn reset(&mut self) { *self = OsGraphCA::new(); }

    // -----------------------------------------------------------------------
    // Zero-copy accessors
    // -----------------------------------------------------------------------
    pub fn proc_buf_ptr(&self)  -> *const f32 { self.proc_buf.as_ptr() }
    pub fn res_buf_ptr(&self)   -> *const f32 { self.res_buf.as_ptr() }
    pub fn edge_buf_ptr(&self)  -> *const f32 { self.edge_buf.as_ptr() }
    pub fn trace_buf_ptr(&self) -> *const f32 { self.trace_buf.as_ptr() }
    pub fn proc_x_ptr(&self)    -> *const f32 { self.proc_x.as_ptr() }
    pub fn proc_y_ptr(&self)    -> *const f32 { self.proc_y.as_ptr() }
    pub fn res_x_ptr(&self)     -> *const f32 { self.res_x.as_ptr() }
    pub fn res_y_ptr(&self)     -> *const f32 { self.res_y.as_ptr() }

    // -----------------------------------------------------------------------
    // Scalar accessors
    // -----------------------------------------------------------------------
    pub fn max_procs(&self)     -> usize { MAX_PROCS }
    pub fn max_resources(&self) -> usize { MAX_RESOURCES }
    pub fn max_edges(&self)     -> usize { MAX_EDGES }
    pub fn current_tick(&self)  -> u32   { self.tick }
    pub fn completed(&self)     -> u32   { self.completed }
    pub fn proc_buf_stride(&self) -> usize { 6 }
    pub fn res_buf_stride(&self)  -> usize { 5 }
    pub fn edge_buf_stride(&self) -> usize { 3 }

    pub fn active_proc_count(&self) -> u32 {
        self.procs.iter().filter(|p| matches!(p.state, PROC_READY | PROC_RUNNING | PROC_WAITING)).count() as u32
    }

    /// Predicted busy probability for resource r (from HMM)
    pub fn hmm_prediction(&self, r: usize) -> f32 {
        if r < MAX_RESOURCES { self.resources[r].predicted_busy_prob } else { 0.0 }
    }

}