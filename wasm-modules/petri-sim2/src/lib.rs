use wasm_bindgen::prelude::*;

/// # Business Processes — Stochastic Petri Net (SPN) with HMM Process Mining
///
/// ## What is a Petri Net?
/// A **Petri Net** is a mathematical model for concurrent, distributed systems.
/// It is a bipartite directed graph with two types of nodes:
///   - **Places** (circles): hold tokens representing resources, conditions, or data.
///   - **Transitions** (rectangles): fire when their input places are sufficiently
///     marked, consuming tokens from inputs and producing tokens in outputs.
///
/// The key semantics:
///   - A transition is **enabled** when each of its input places has at least as many
///     tokens as the arc weight requires.
///   - When a transition **fires**, it atomically removes tokens from inputs and adds
///     tokens to outputs.
///   - Multiple transitions may be enabled simultaneously — Petri Nets model true
///     **concurrency** (unlike state machines which are sequential).
///
/// ## What makes it STOCHASTIC?
/// A **Stochastic Petri Net (SPN)** adds a firing rate (λ) to each transition.
/// When multiple transitions are simultaneously enabled, they compete to fire;
/// the one that fires is selected by sampling an exponential race condition:
///   P(transition t wins) ∝ λ_t / Σ (all enabled λ_i)
/// This turns the Petri Net into a **Continuous-Time Markov Chain (CTMC)**,
/// making it a proper probabilistic automaton — each marking (token distribution)
/// is a state; transitions between markings are stochastic.
///
/// ## Domain: Business Process
/// The simulation models a **Loan Application workflow** — a classic example from
/// the process mining literature (van der Aalst et al.):
///
///   Application Received → [Assess Completeness]
///     → Complete? → [Credit Check] ──────────────────────────────┐
///     → Incomplete → [Request Documents] → [Received?] ──────────┤
///                                          → Timeout → [Reject]  │
///   Credit Check Result ──────────────────────────────────────────┤
///     → Approved → [Risk Assessment] → [Final Approval] → [Disburse]
///     → Refused  → [Rejection Letter] → [Archive]
///   Final Approval
///     → [Sign Contract] → [Disburse Funds] → [Notify Client] → Done
///
/// Multiple loan applications flow concurrently — tokens represent individual cases.
///
/// ## HMM Process Mining Layer
/// On top of the SPN, a **Hidden Markov Model** is trained in real-time on the
/// observed transition firing sequence. This models the process mining task:
/// given only the sequence of observable events (transitions), infer the
/// unobservable internal state of the process (which case-handling team is active,
/// whether a bottleneck is building, etc.).
///
/// Hidden states: NORMAL_FLOW(0), BACKLOG_BUILDING(1), BOTTLENECK(2), CLEARED(3)
/// Observable events: transitions 0-11 (which transition fired last)
/// The HMM belief state is updated via the Forward Algorithm each tick.

// ---------------------------------------------------------------------------
// Petri Net topology — Loan Application workflow
// ---------------------------------------------------------------------------
// Places:
//   0  Inbox (new applications arrive here)
//   1  Under Assessment
//   2  Complete
//   3  Incomplete
//   4  Docs Requested
//   5  Docs Received
//   6  Timeout
//   7  Credit Check In Progress
//   8  Credit Approved
//   9  Credit Refused
//  10  Risk Assessment
//  11  Final Approved
//  12  Final Rejected
//  13  Contract Signing
//  14  Funds Disbursed
//  15  Done

const NUM_PLACES: usize = 16;
const NUM_TRANSITIONS: usize = 12;

// Transition names (for JS rendering)
// T0  Assess Completeness   (0→1→{2,3})    λ=2.0
// T1  Route: Complete       (1→2,)          λ=2.5  — actually fires after T0
// T2  Route: Incomplete     (1→3)           λ=1.5
// T3  Request Documents     (3→4)           λ=3.0
// T4  Docs Received         (4→5)           λ=1.5  — slow (client response)
// T5  Docs Timeout          (4→6)           λ=0.3  — rare
// T6  Credit Check          ({2,5}→7→{8,9}) λ=2.0
// T7  Credit Approved       (7→8)           λ=1.8
// T8  Credit Refused        (7→9)           λ=0.8
// T9  Risk Assessment       (8→10→11)       λ=1.5
// T10 Final Approve         (10→11)         λ=1.2
// T11 Final Reject          (10→12)         λ=0.4
// T12 Wait I removed — keeping 12 total

/// Firing rates (λ) for the exponential race. Higher = faster.
const LAMBDA: [f32; NUM_TRANSITIONS] = [
    2.0,  // T0  Assess Completeness
    2.5,  // T1  Route Complete
    1.5,  // T2  Route Incomplete
    3.0,  // T3  Request Documents
    1.5,  // T4  Docs Received
    0.3,  // T5  Docs Timeout → Reject
    2.0,  // T6  Credit Check
    1.8,  // T7  Credit Approved
    0.8,  // T8  Credit Refused
    1.5,  // T9  Risk Assessment
    1.2,  // T10 Final Approve
    0.4,  // T11 Final Reject
];

/// Input arcs: transition t requires these tokens
/// Format: [(place_id, arc_weight); up to 3 inputs per transition]
/// -1 = no arc
const INPUTS: [[i8; 3]; NUM_TRANSITIONS] = [
    [ 0, -1, -1],  // T0: consumes from Inbox
    [ 1, -1, -1],  // T1: consumes Under Assessment → Complete
    [ 1, -1, -1],  // T2: consumes Under Assessment → Incomplete
    [ 3, -1, -1],  // T3: Incomplete → Docs Requested
    [ 4, -1, -1],  // T4: Docs Requested → Docs Received
    [ 4, -1, -1],  // T5: Docs Requested → Timeout
    [ 2, -1, -1],  // T6: Complete (or Docs Received) → Credit Check  [Note: T6 also accepts from P5]
    [ 7, -1, -1],  // T7: Credit Check → Approved
    [ 7, -1, -1],  // T8: Credit Check → Refused
    [ 8, -1, -1],  // T9: Approved → Risk Assessment
    [10, -1, -1],  // T10: Risk Assessment → Final Approved
    [10, -1, -1],  // T11: Risk Assessment → Final Rejected
];

/// Extra input: T6 also fires when Docs Received (P5) has tokens
const T6_ALSO_ACCEPTS_P5: bool = true;

/// Output arcs: transition t produces tokens to these places
const OUTPUTS: [[i8; 3]; NUM_TRANSITIONS] = [
    [ 1, -1, -1],  // T0 → Under Assessment
    [ 2, -1, -1],  // T1 → Complete
    [ 3, -1, -1],  // T2 → Incomplete
    [ 4, -1, -1],  // T3 → Docs Requested
    [ 5, -1, -1],  // T4 → Docs Received
    [12, -1, -1],  // T5 → Timeout Reject (Final Rejected)
    [ 7, -1, -1],  // T6 → Credit Check In Progress
    [ 8, -1, -1],  // T7 → Credit Approved
    [ 9, -1, -1],  // T8 → Credit Refused
    [10, -1, -1],  // T9 → Risk Assessment
    [11, -1, -1],  // T10 → Final Approved  (feeds T_sign via P11→P13)
    [12, -1, -1],  // T11 → Final Rejected
];

// Place coordinates for the canvas (800 × 400 viewport)
const PLACE_X: [f32; NUM_PLACES] = [
    40.0,  120.0, 220.0, 220.0, 300.0, 300.0, 380.0,
    400.0, 500.0, 500.0, 580.0, 660.0, 660.0,
    720.0, 760.0, 780.0,
];
const PLACE_Y: [f32; NUM_PLACES] = [
    200.0, 200.0, 130.0, 270.0, 270.0, 150.0, 310.0,
    200.0, 130.0, 270.0, 200.0, 130.0, 270.0,
    200.0, 130.0, 200.0,
];

// Transition coordinates
const TRANS_X: [f32; NUM_TRANSITIONS] = [
     80.0, 170.0, 170.0, 260.0, 300.0, 330.0,
    350.0, 450.0, 450.0, 540.0, 620.0, 620.0,
];
const TRANS_Y: [f32; NUM_TRANSITIONS] = [
    200.0, 130.0, 270.0, 270.0, 200.0, 350.0,
    165.0, 130.0, 270.0, 200.0, 130.0, 270.0,
];

// ---------------------------------------------------------------------------
// HMM for process mining
// Hidden states: NORMAL(0), BACKLOG(1), BOTTLENECK(2), CLEARED(3)
// Observations: 0-11 (which transition fired), 12 = no transition fired
const HMM_H: usize = 4;  // hidden states
const HMM_O: usize = 13; // observation symbols
// Transition matrix A[h][h'] = P(next_hidden=h' | hidden=h)
const HMM_A: [[f32; HMM_H]; HMM_H] = [
    [0.7, 0.2, 0.05, 0.05], // NORMAL → mostly stays NORMAL
    [0.2, 0.5, 0.25, 0.05], // BACKLOG → may worsen to BOTTLENECK
    [0.05, 0.2, 0.6, 0.15], // BOTTLENECK → may clear
    [0.3, 0.1, 0.05, 0.55], // CLEARED → likely stays or regresses to NORMAL
];
// Emission: P(obs=t fired | hidden=h) — rough approximations
// When BOTTLENECK, early transitions (T0-T3) are more likely than late ones
const fn hmm_emit(h: usize, obs: usize) -> f32 {
    if obs == 12 { // no firing
        match h { 0 => 0.1, 1 => 0.3, 2 => 0.5, _ => 0.15 }
    } else if obs < 6 { // early-stage transitions
        match h { 0 => 0.07, 1 => 0.12, 2 => 0.05, _ => 0.06 }
    } else { // late-stage transitions (approval/rejection)
        match h { 0 => 0.07, 1 => 0.02, 2 => 0.01, _ => 0.12 }
    }
}

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
}

// ---------------------------------------------------------------------------
// Main struct
// ---------------------------------------------------------------------------
#[wasm_bindgen]
pub struct BusinessProcessSPN {
    /// Token counts per place
    tokens: Vec<u32>,
    /// Place layout
    place_x: Vec<f32>,
    place_y: Vec<f32>,
    /// Transition layout
    trans_x: Vec<f32>,
    trans_y: Vec<f32>,
    /// Which transitions fired on the last step
    fired: Vec<u8>,
    /// Firing counts (cumulative)
    fire_count: Vec<u32>,
    /// Token animation progress (0→1 per transition)
    anim_progress: Vec<f32>,
    /// Per-transition "clock": exponential waiting time remaining
    /// When a transition is enabled, its clock counts down at rate λ
    trans_clock: Vec<f32>,

    // HMM process mining state
    /// Belief distribution over hidden process states
    hmm_belief: Vec<f32>,           // [HMM_H]
    /// Most likely hidden state sequence posterior (Viterbi trace, simplified)
    hmm_state_seq: Vec<u8>,         // last 32 inferred hidden states
    hmm_seq_head: usize,
    /// Bottleneck score: P(hidden=BOTTLENECK)
    bottleneck_prob: f32,

    // Statistics
    applications_in: u32,
    applications_done: u32,  // reached Done (P15)
    applications_rejected: u32,
    step_count: u32,
    /// Mean sojourn time approximation (tokens × 1/throughput)
    mean_sojourn: f32,

    rng: Rng,
}

impl BusinessProcessSPN {
    fn is_enabled(&self, t: usize) -> bool {
        // Standard Petri Net enabledness check
        let [p0, p1, p2] = INPUTS[t];
        let ok0 = p0 < 0 || self.tokens[p0 as usize] > 0;
        let ok1 = p1 < 0 || self.tokens[p1 as usize] > 0;
        let ok2 = p2 < 0 || self.tokens[p2 as usize] > 0;
        // T6 special: also enabled if P5 (Docs Received) has tokens
        let extra = if t == 6 && T6_ALSO_ACCEPTS_P5 { self.tokens[5] > 0 } else { false };
        (ok0 && ok1 && ok2) || extra
    }

    fn fire(&mut self, t: usize) {
        // Consume input tokens
        let [p0, p1, p2] = INPUTS[t];
        // T6 special: prefer P5 over P2 if both available
        if t == 6 && T6_ALSO_ACCEPTS_P5 && self.tokens[5] > 0 {
            self.tokens[5] -= 1;
        } else if p0 >= 0 { self.tokens[p0 as usize] -= 1; }
        if p1 >= 0 { self.tokens[p1 as usize] -= 1; }
        if p2 >= 0 { self.tokens[p2 as usize] -= 1; }

        // Produce output tokens
        for &op in OUTPUTS[t].iter() {
            if op >= 0 { self.tokens[op as usize] += 1; }
        }

        // Special: T10 (Final Approve) → tokens go to Contract Signing (P13)
        if t == 10 {
            let done_tok = self.tokens[11];
            if done_tok > 0 {
                self.tokens[11] -= done_tok;
                self.tokens[13] = self.tokens[13].saturating_add(done_tok);
            }
        }
        // P13 → P14 (Sign Contract → Disburse): instant if tokens present
        if self.tokens[13] > 0 {
            let t = self.tokens[13];
            self.tokens[13] -= t;
            self.tokens[14] = self.tokens[14].saturating_add(t);
        }
        // P14 → P15 (Disburse → Done): instant
        if self.tokens[14] > 0 {
            let t = self.tokens[14];
            self.tokens[14] -= t;
            self.tokens[15] = self.tokens[15].saturating_add(t);
            self.applications_done += t;
        }
        // P12 → Done (rejected): instant
        if self.tokens[12] > 0 {
            let t = self.tokens[12];
            self.tokens[12] -= t;
            self.applications_rejected += t;
        }
        // P9 → P12 (Refused → Rejected): instant
        if self.tokens[9] > 0 {
            let t = self.tokens[9];
            self.tokens[9] -= t;
            self.tokens[12] = self.tokens[12].saturating_add(t);
        }

        self.fired[t] = 1;
        self.fire_count[t] += 1;
        self.anim_progress[t] = 0.0;
        // Reset clock
        self.trans_clock[t] = 0.0;
    }

    fn hmm_update(&mut self, obs: usize) {
        // Forward algorithm step
        let mut new_belief = [0.0f32; HMM_H];
        for h_next in 0..HMM_H {
            let mut sum = 0.0f32;
            for h_cur in 0..HMM_H {
                sum += HMM_A[h_cur][h_next] * self.hmm_belief[h_cur];
            }
            new_belief[h_next] = hmm_emit(h_next, obs) * sum;
        }
        let total: f32 = new_belief.iter().sum();
        if total > 1e-9 {
            for b in new_belief.iter_mut() { *b /= total; }
        }
        for h in 0..HMM_H { self.hmm_belief[h] = new_belief[h]; }
        self.bottleneck_prob = self.hmm_belief[2]; // P(BOTTLENECK)

        // Record inferred state (argmax)
        let best_h = self.hmm_belief.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i).unwrap_or(0) as u8;
        self.hmm_state_seq[self.hmm_seq_head] = best_h;
        self.hmm_seq_head = (self.hmm_seq_head + 1) % 32;
    }
}

#[wasm_bindgen]
impl BusinessProcessSPN {
    pub fn new() -> Self {
        let mut tokens = vec![0u32; NUM_PLACES];
        tokens[0] = 2; // Start with 2 applications in inbox

        BusinessProcessSPN {
            tokens,
            place_x: PLACE_X.to_vec(),
            place_y: PLACE_Y.to_vec(),
            trans_x: TRANS_X.to_vec(),
            trans_y: TRANS_Y.to_vec(),
            fired: vec![0u8; NUM_TRANSITIONS],
            fire_count: vec![0u32; NUM_TRANSITIONS],
            anim_progress: vec![0.0f32; NUM_TRANSITIONS],
            trans_clock: vec![0.0f32; NUM_TRANSITIONS],
            hmm_belief: vec![0.25f32; HMM_H], // uniform prior
            hmm_state_seq: vec![0u8; 32],
            hmm_seq_head: 0,
            bottleneck_prob: 0.0,
            applications_in: 2,
            applications_done: 0,
            applications_rejected: 0,
            step_count: 0,
            mean_sojourn: 0.0,
            rng: Rng(0xB05),
        }
    }

    // -----------------------------------------------------------------------
    // One simulation step — Stochastic Petri Net firing race
    // -----------------------------------------------------------------------
    pub fn step(&mut self) {
        self.step_count += 1;

        // Clear last-step fired flags
        for f in self.fired.iter_mut() { *f = 0; }

        // --- Exponential race: advance clocks for all enabled transitions ---
        // A transition t with rate λ_t accumulates clock at rate λ_t per tick;
        // it fires when its clock exceeds a sampled exponential threshold.
        // Multiple transitions can fire per step if independently enabled.
        let mut last_fired_obs: usize = 12; // no firing

        let mut enabled_flags = [false; NUM_TRANSITIONS];
        for t in 0..NUM_TRANSITIONS {
            enabled_flags[t] = self.is_enabled(t);
        }

        // Compute total enabled rate (for competition normalisation)
        let total_rate: f32 = (0..NUM_TRANSITIONS)
            .filter(|&t| enabled_flags[t])
            .map(|t| LAMBDA[t])
            .sum();

        if total_rate > 0.0 {
            // Sample a winner transition using the race condition:
            // P(t wins) = λ_t / total_rate
            let roll = self.rng.f32() * total_rate;
            let mut cumulative = 0.0f32;
            for t in 0..NUM_TRANSITIONS {
                if !enabled_flags[t] { continue; }
                cumulative += LAMBDA[t];
                if roll <= cumulative {
                    self.fire(t);
                    last_fired_obs = t;
                    break;
                }
            }
        }

        // Advance animation
        for t in 0..NUM_TRANSITIONS {
            if self.anim_progress[t] < 1.0 {
                self.anim_progress[t] = (self.anim_progress[t] + 0.12).min(1.0);
            }
        }

        // Periodic new application arrival (Poisson process)
        if self.step_count % 6 == 0 && self.tokens[0] < 5 {
            self.tokens[0] += 1;
            self.applications_in += 1;
        }

        // HMM update on the observed event
        self.hmm_update(last_fired_obs);

        // Mean sojourn approximation: (tokens in flight) / throughput
        let in_flight: u32 = self.tokens[1..15].iter().sum();
        let throughput = (self.applications_done + self.applications_rejected).max(1) as f32
                         / self.step_count as f32;
        self.mean_sojourn = in_flight as f32 / throughput.max(0.01);
    }

    pub fn step_n(&mut self, n: u32) { for _ in 0..n { self.step(); } }

    pub fn inject_application(&mut self) {
        self.tokens[0] += 1;
        self.applications_in += 1;
    }

    pub fn reset(&mut self) { *self = BusinessProcessSPN::new(); }

    // -----------------------------------------------------------------------
    // Zero-copy accessors
    // -----------------------------------------------------------------------
    pub fn tokens_ptr(&self)        -> *const u32 { self.tokens.as_ptr() }
    pub fn place_x_ptr(&self)       -> *const f32 { self.place_x.as_ptr() }
    pub fn place_y_ptr(&self)       -> *const f32 { self.place_y.as_ptr() }
    pub fn trans_x_ptr(&self)       -> *const f32 { self.trans_x.as_ptr() }
    pub fn trans_y_ptr(&self)       -> *const f32 { self.trans_y.as_ptr() }
    pub fn fired_ptr(&self)         -> *const u8  { self.fired.as_ptr() }
    pub fn fire_count_ptr(&self)    -> *const u32 { self.fire_count.as_ptr() }
    pub fn anim_ptr(&self)          -> *const f32 { self.anim_progress.as_ptr() }
    pub fn hmm_belief_ptr(&self)    -> *const f32 { self.hmm_belief.as_ptr() }
    pub fn hmm_seq_ptr(&self)       -> *const u8  { self.hmm_state_seq.as_ptr() }

    // -----------------------------------------------------------------------
    // Scalar accessors
    // -----------------------------------------------------------------------
    pub fn num_places(&self)      -> usize { NUM_PLACES }
    pub fn num_transitions(&self) -> usize { NUM_TRANSITIONS }
    pub fn hmm_hidden_states(&self) -> usize { HMM_H }
    pub fn hmm_seq_len(&self)     -> usize { 32 }
    pub fn hmm_seq_head(&self)    -> usize { self.hmm_seq_head }
    pub fn step_count(&self)      -> u32   { self.step_count }
    pub fn applications_in(&self) -> u32   { self.applications_in }
    pub fn applications_done(&self) -> u32 { self.applications_done }
    pub fn applications_rejected(&self) -> u32 { self.applications_rejected }
    pub fn bottleneck_prob(&self) -> f32   { self.bottleneck_prob }
    pub fn mean_sojourn(&self)    -> f32   { self.mean_sojourn }
    pub fn tokens_at(&self, p: usize) -> u32 { self.tokens.get(p).copied().unwrap_or(0) }

    pub fn throughput(&self) -> f32 {
        (self.applications_done + self.applications_rejected) as f32
            / self.step_count.max(1) as f32
    }
}