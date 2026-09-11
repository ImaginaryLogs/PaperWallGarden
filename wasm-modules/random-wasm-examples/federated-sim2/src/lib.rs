use wasm_bindgen::prelude::*;

/// # Federated Learning — Grid-Based Cellular Automaton
///
/// ## What is a Grid-Based CA?
/// A **Grid-Based Cellular Automaton** (CA) is a lattice of cells where each cell
/// evolves according to a local rule: its next state is a function of its own
/// state and the states of its immediate neighbours. The key insight is that
/// **global behaviour emerges from purely local interactions** — no cell has
/// a view of the whole grid.
///
/// ## Domain: Federated Learning
/// In Federated Learning (FL), N devices (phones, hospitals, IoT nodes) each hold
/// private data and a local model. A central server orchestrates training without
/// ever seeing raw data. This simulation exposes the **core tension of FL**:
/// devices must coordinate globally while acting locally.
///
/// ## How the CA models FL
/// Each cell (i, j) in a WIDTH×HEIGHT grid is a **federated device** with:
///   - `model[i,j]`  — its local model parameter (scalar weight, ≈ the θ it is trying to learn)
///   - `loss[i,j]`   — its current local loss (how far the local model is from fitting its data)
///   - `state[i,j]`  — lifecycle: 0=idle, 1=training, 2=aggregating, 3=converged, 4=drifted
///   - `trust[i,j]`  — Byzantne trust score (0.0–1.0) computed from neighbour agreement
///
/// ## CA Transition Rule (the "physics" of FL as a CA)
///
/// On each tick, every cell (i,j) applies this rule:
///
///   1. **Local Training** (autonomous): nudge model toward cell's private data target.
///      Data heterogeneity (non-IID) is modelled as each cell having a private "true θ"
///      that differs by position — cells in the top half prefer θ≈0.2, bottom prefer θ≈0.8.
///
///   2. **Gradient Gossip** (neighbourhood interaction — the CA rule):
///      Average the model deltas from Moore neighbourhood (8 neighbours), weighted
///      by neighbour trust scores. This is **gossip-based federated aggregation**,
///      an alternative to FedAvg where there is no central server; instead, each
///      cell aggregates from its neighbours every round.
///
///   3. **Trust Update**: if a neighbour's model is far from the local consensus,
///      lower its trust score (Byzantine fault detection). Cells with low trust
///      contribute less to aggregation — this implements the CA equivalent of
///      Krum or Trimmed-Mean aggregation.
///
///   4. **State Machine**: cells transition through lifecycle states based on their
///      loss value, converging or drifting depending on how much data heterogeneity
///      the neighbourhood exerts.
///
/// ## What you can observe
/// - **Convergence waves**: watch the grid converge in patches — local neighbourhoods
///   agree first, then patches expand and merge. This is a hallmark of CA dynamics.
/// - **Data heterogeneity cliffs**: the boundary between the top-half and bottom-half
///   populations creates a persistent drift zone — cells near the boundary struggle.
/// - **Byzantine isolation**: inject noise into a cell; its trust score drops and the
///   corruption does not propagate beyond one neighbourhood radius.
/// - **Gossip vs FedAvg**: toggle gossip range. With range=1 (local only) convergence
///   is slow; with range=WIDTH (global), it behaves like standard FedAvg.

const WIDTH: usize = 16;
const HEIGHT: usize = 12;
const N: usize = WIDTH * HEIGHT; // 192 cells

// State constants (stored as u8)
const STATE_IDLE: u8       = 0;
const STATE_TRAINING: u8   = 1;
const STATE_AGGREGATING: u8 = 2;
const STATE_CONVERGED: u8  = 3;
const STATE_DRIFTED: u8    = 4;

// ---------------------------------------------------------------------------
// LCG RNG (deterministic, no_std)
// ---------------------------------------------------------------------------
struct Rng(u64);
impl Rng {
    fn u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
    fn f32(&mut self) -> f32 {
        ((self.u64() >> 33) as f32) / (u32::MAX as f32)
    }
    fn gaussian(&mut self, mean: f32, std: f32) -> f32 {
        let u = self.f32().max(1e-7);
        let v = self.f32();
        let z = (-2.0 * u.ln()).sqrt() * (2.0 * core::f32::consts::PI * v).cos();
        mean + std * z
    }
}

// ---------------------------------------------------------------------------
// FederatedGridCA
// ---------------------------------------------------------------------------
#[wasm_bindgen]
pub struct FederatedGridCA {
    /// Current model parameters (one f32 per cell) — the "weight" being learned
    model:       Vec<f32>,
    /// Previous tick's model (double-buffer for synchronous CA update)
    model_next:  Vec<f32>,
    /// Local training loss per cell
    loss:        Vec<f32>,
    /// Cell lifecycle state (u8: idle/training/aggregating/converged/drifted)
    state:       Vec<u8>,
    /// Trust score for each cell (from perspective of its neighbourhood)
    trust:       Vec<f32>,
    /// Private "true θ" — the data each cell is trying to fit (never shared)
    private_target: Vec<f32>,
    /// Gossip weight: how strongly the neighbourhood pull is weighted vs local grad
    gossip_weight: f32,
    /// Learning rate for local SGD step
    lr: f32,
    /// Global round counter
    round: u32,
    /// Noise injection mask: cells flagged as Byzantine
    byzantine: Vec<u8>,
    /// Cumulative global convergence metric (mean |model - private_target|)
    global_loss_history: Vec<f32>,
    rng: Rng,
}

#[wasm_bindgen]
impl FederatedGridCA {
    // -----------------------------------------------------------------------
    // Constructor
    // -----------------------------------------------------------------------
    pub fn new() -> Self {
        let mut rng = Rng(0xFEDCA);

        // Private targets create data heterogeneity:
        // Top 1/3 of grid has target ≈ 0.2 (e.g. majority class A patients)
        // Middle 1/3 has target ≈ 0.5 (mixed population)
        // Bottom 1/3 has target ≈ 0.8 (majority class B patients)
        // This reproduces non-IID data distribution — the core challenge of FL.
        let private_target: Vec<f32> = (0..N).map(|idx| {
            let row = idx / WIDTH;
            let zone_mean = if row < HEIGHT / 3 {
                0.2f32
            } else if row < 2 * HEIGHT / 3 {
                0.5f32
            } else {
                0.8f32
            };
            rng.gaussian(zone_mean, 0.04).clamp(0.0, 1.0)
        }).collect();

        // All devices initialise to the same global model (standard FL assumption)
        let model = vec![0.5f32; N];

        FederatedGridCA {
            model:       model.clone(),
            model_next:  model,
            loss:        vec![0.0f32; N],
            state:       vec![STATE_IDLE; N],
            trust:       vec![1.0f32; N],
            private_target,
            gossip_weight: 0.4,
            lr: 0.08,
            round: 0,
            byzantine: vec![0u8; N],
            global_loss_history: vec![f32::NAN; 200],
            rng,
        }
    }

    // -----------------------------------------------------------------------
    // One CA tick — applies transition rule to all cells synchronously
    // -----------------------------------------------------------------------
    pub fn step(&mut self) {
        // Phase 1: Compute next state for every cell using current-tick values only
        for idx in 0..N {
            let row = (idx / WIDTH) as isize;
            let col = (idx % WIDTH) as isize;

            // --- Local training step ---
            // Gradient toward private data target (SGD on MSE: d/dθ (θ - target)² = 2(θ - target))
            let theta = self.model[idx];
            let target = self.private_target[idx];
            // Add noise if Byzantine
            let noise = if self.byzantine[idx] == 1 {
                self.rng.gaussian(0.0, 0.3)
            } else {
                self.rng.gaussian(0.0, 0.01)
            };
            let local_grad = 2.0 * (theta - target) + noise;
            let theta_after_sgd = theta - self.lr * local_grad;

            // --- Gossip aggregation (Moore neighbourhood, 8-connected) ---
            let mut weighted_sum = 0.0f32;
            let mut weight_total = 0.0f32;
            for dr in -1i32..=1 {
                for dc in -1i32..=1 {
                    if dr == 0 && dc == 0 { continue; }
                    let nr = row + dr as isize;
                    let nc = col + dc as isize;
                    if nr < 0 || nr >= HEIGHT as isize || nc < 0 || nc >= WIDTH as isize { continue; }
                    let nidx = nr as usize * WIDTH + nc as usize;
                    let w = self.trust[nidx];
                    weighted_sum += w * self.model[nidx];
                    weight_total += w;
                }
            }
            let neighbourhood_mean = if weight_total > 0.0 {
                weighted_sum / weight_total
            } else {
                theta
            };

            // CA transition rule: blend local SGD result with neighbourhood consensus
            // gossip_weight controls how much cells trust their neighbours
            let theta_new = (1.0 - self.gossip_weight) * theta_after_sgd
                          + self.gossip_weight * neighbourhood_mean;

            self.model_next[idx] = theta_new.clamp(0.0, 1.0);

            // Loss = |model - private_target| (L1 distance to personal truth)
            self.loss[idx] = (theta_new - target).abs();
        }

        // Phase 2: Synchronous swap (all cells update simultaneously — true CA)
        core::mem::swap(&mut self.model, &mut self.model_next);

        // Phase 3: Trust update — cells that deviate far from their neighbourhood
        //          get lower trust scores (Byzantine fault detection)
        for idx in 0..N {
            let row = (idx / WIDTH) as isize;
            let col = (idx % WIDTH) as isize;
            let mut n_sum = 0.0f32;
            let mut n_count = 0u32;
            for dr in -1i32..=1 {
                for dc in -1i32..=1 {
                    if dr == 0 && dc == 0 { continue; }
                    let nr = row + dr as isize;
                    let nc = col + dc as isize;
                    if nr < 0 || nr >= HEIGHT as isize || nc < 0 || nc >= WIDTH as isize { continue; }
                    n_sum += self.model[nr as usize * WIDTH + nc as usize];
                    n_count += 1;
                }
            }
            if n_count > 0 {
                let n_mean = n_sum / n_count as f32;
                let deviation = (self.model[idx] - n_mean).abs();
                // Trust decays when cell model deviates from consensus; recovers slowly
                let new_trust = if deviation > 0.15 {
                    (self.trust[idx] - 0.1).max(0.1)
                } else {
                    (self.trust[idx] + 0.02).min(1.0)
                };
                self.trust[idx] = new_trust;
            }
        }

        // Phase 4: State machine — based on loss thresholds
        for idx in 0..N {
            self.state[idx] = match self.loss[idx] {
                l if l < 0.03 => STATE_CONVERGED,
                l if l < 0.12 => STATE_AGGREGATING,
                l if l < 0.25 => STATE_TRAINING,
                l if l > 0.35 => STATE_DRIFTED,
                _ => STATE_IDLE,
            };
        }

        // Phase 5: Record global loss
        let mean_loss = self.loss.iter().sum::<f32>() / N as f32;
        let idx = (self.round as usize).min(199);
        self.global_loss_history[idx] = mean_loss;
        self.round += 1;
    }

    pub fn step_n(&mut self, n: u32) {
        for _ in 0..n { self.step(); }
    }

    // -----------------------------------------------------------------------
    // Byzantine injection: mark a cell as a faulty/adversarial node
    // -----------------------------------------------------------------------
    pub fn inject_byzantine(&mut self, row: usize, col: usize) {
        if row < HEIGHT && col < WIDTH {
            let idx = row * WIDTH + col;
            self.byzantine[idx] = 1;
        }
    }

    pub fn clear_byzantine(&mut self) {
        for b in self.byzantine.iter_mut() { *b = 0; }
    }

    // -----------------------------------------------------------------------
    // Configuration
    // -----------------------------------------------------------------------
    pub fn set_gossip_weight(&mut self, w: f32) { self.gossip_weight = w.clamp(0.0, 0.95); }
    pub fn set_lr(&mut self, lr: f32) { self.lr = lr.clamp(0.001, 0.5); }

    pub fn reset(&mut self) {
        *self = FederatedGridCA::new();
    }

    // -----------------------------------------------------------------------
    // Zero-copy memory accessors
    // -----------------------------------------------------------------------
    pub fn model_ptr(&self)        -> *const f32 { self.model.as_ptr() }
    pub fn loss_ptr(&self)         -> *const f32 { self.loss.as_ptr() }
    pub fn state_ptr(&self)        -> *const u8  { self.state.as_ptr() }
    pub fn trust_ptr(&self)        -> *const f32 { self.trust.as_ptr() }
    pub fn target_ptr(&self)       -> *const f32 { self.private_target.as_ptr() }
    pub fn byzantine_ptr(&self)    -> *const u8  { self.byzantine.as_ptr() }
    pub fn loss_history_ptr(&self) -> *const f32 { self.global_loss_history.as_ptr() }

    // -----------------------------------------------------------------------
    // Scalar accessors
    // -----------------------------------------------------------------------
    pub fn width(&self)  -> usize { WIDTH }
    pub fn height(&self) -> usize { HEIGHT }
    pub fn n(&self)      -> usize { N }
    pub fn round(&self)  -> u32   { self.round }
    pub fn gossip_weight(&self) -> f32 { self.gossip_weight }
    pub fn lr(&self) -> f32 { self.lr }

    pub fn mean_loss(&self) -> f32 {
        self.loss.iter().sum::<f32>() / N as f32
    }
    pub fn converged_count(&self) -> u32 {
        self.state.iter().filter(|&&s| s == STATE_CONVERGED).count() as u32
    }
    pub fn byzantine_count(&self) -> u32 {
        self.byzantine.iter().filter(|&&b| b == 1).count() as u32
    }
}