use wasm_bindgen::prelude::*;

/// Federated Learning Simulator
///
/// Simulates a synchronous FedAvg (McMahan et al. 2017) round over N client nodes.
/// Each client trains a toy linear model (y = Wx + b) on a private mini-dataset
/// drawn from a distinct distribution to model non-IID data heterogeneity.
///
/// Architecture:
///   - 1 central aggregator
///   - CLIENT_COUNT client nodes
///   - Model: single weight matrix W (FEATURES × CLASSES) + bias b (CLASSES)
///   - Each round: clients receive global model → local SGD → send deltas →
///     aggregator performs weighted FedAvg → broadcast updated global model
///
/// Memory layout exposed via raw pointers (zero-copy to JS):
///   global_weights : f32[FEATURES * CLASSES]  -- global model weights
///   client_weights : f32[CLIENT_COUNT * FEATURES * CLASSES] -- per-client local models
///   loss_history   : f32[MAX_ROUNDS]           -- global validation loss per round
///   client_losses  : f32[CLIENT_COUNT]         -- per-client loss at last round
///   participation  : u8[CLIENT_COUNT]          -- 1 if client participated this round

const FEATURES: usize = 4;      // Input feature dimensions (e.g. 4 health indicators)
const CLASSES: usize = 2;       // Output classes (binary classification)
const MODEL_SIZE: usize = FEATURES * CLASSES + CLASSES; // weights + bias
const CLIENT_COUNT: usize = 6;  // Number of federated nodes / hospitals / devices
const MAX_ROUNDS: usize = 100;  // Rounds to store in history buffer
const LOCAL_EPOCHS: usize = 3;  // SGD steps per round per client
const LOCAL_SAMPLES: usize = 16; // Synthetic samples per client per round

// ---------------------------------------------------------------------------
// LCG RNG (deterministic, no-std)
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
    /// Gaussian sample via Box-Muller
    fn gaussian(&mut self, mean: f32, std: f32) -> f32 {
        let u = self.f32().max(1e-7);
        let v = self.f32();
        let z = (-2.0 * u.ln()).sqrt() * (2.0 * core::f32::consts::PI * v).cos();
        mean + std * z
    }
}

// ---------------------------------------------------------------------------
// Tiny linear model helpers
// ---------------------------------------------------------------------------

/// Softmax of a 2-element slice in-place
fn softmax2(x: &mut [f32; CLASSES]) {
    let m = x[0].max(x[1]);
    x[0] = (x[0] - m).exp();
    x[1] = (x[1] - m).exp();
    let s = x[0] + x[1];
    x[0] /= s;
    x[1] /= s;
}

/// Forward pass: logits = W·input + b
fn forward(weights: &[f32], input: &[f32; FEATURES]) -> [f32; CLASSES] {
    let mut logits = [0.0f32; CLASSES];
    for c in 0..CLASSES {
        logits[c] = weights[FEATURES * CLASSES + c]; // bias
        for f in 0..FEATURES {
            logits[c] += weights[c * FEATURES + f] * input[f];
        }
    }
    logits
}

/// Cross-entropy loss for a single sample
fn cross_entropy(logits: &[f32; CLASSES], label: usize) -> f32 {
    let mut probs = *logits;
    softmax2(&mut probs);
    -(probs[label].max(1e-7).ln())
}

/// SGD gradient step on a batch of samples, returns updated weights slice
fn sgd_step(
    weights: &mut [f32],
    samples: &[[f32; FEATURES]],
    labels: &[usize],
    lr: f32,
) -> f32 {
    let n = samples.len() as f32;
    let mut total_loss = 0.0f32;
    let mut grad = vec![0.0f32; MODEL_SIZE];

    for (x, &y) in samples.iter().zip(labels.iter()) {
        let mut logits = forward(weights, x);
        total_loss += cross_entropy(&logits, y);

        // Softmax probs
        softmax2(&mut logits);

        // Gradient of cross-entropy w.r.t. logits: p - one_hot(y)
        let mut dlogits = logits;
        dlogits[y] -= 1.0;

        // Backprop into weights (W) and biases (b)
        for c in 0..CLASSES {
            for f in 0..FEATURES {
                grad[c * FEATURES + f] += dlogits[c] * x[f] / n;
            }
            grad[FEATURES * CLASSES + c] += dlogits[c] / n;
        }
    }

    // Apply gradient with L2 regularisation (lambda=0.001)
    for i in 0..MODEL_SIZE {
        weights[i] -= lr * (grad[i] + 0.001 * weights[i]);
    }

    total_loss / n
}

// ---------------------------------------------------------------------------
// FederatedSim struct
// ---------------------------------------------------------------------------
#[wasm_bindgen]
pub struct FederatedSim {
    /// Global model (flat: W rows-major + b)
    global_weights: Vec<f32>,
    /// Per-client model snapshots (CLIENT_COUNT × MODEL_SIZE)
    client_weights: Vec<f32>,
    /// Per-client data distribution offsets (models non-IID data)
    client_biases: Vec<f32>,
    /// Loss history over rounds [MAX_ROUNDS]
    loss_history: Vec<f32>,
    /// Per-client loss at last round [CLIENT_COUNT]
    client_losses: Vec<f32>,
    /// Participation mask: 1 if participated, 0 if dropped out [CLIENT_COUNT]
    participation: Vec<u8>,
    /// Learning rate
    lr: f32,
    /// Client dropout probability per round
    dropout: f32,
    /// Current completed round
    round: u32,
    /// Aggregation method: 0 = FedAvg, 1 = FedProx, 2 = Simple mean
    agg_method: u8,
    rng: Rng,
}

#[wasm_bindgen]
impl FederatedSim {
    // -----------------------------------------------------------------------
    // Constructor
    // -----------------------------------------------------------------------
    pub fn new() -> Self {
        let mut rng = Rng(1337);

        // Initialise global model with small random weights
        let global_weights: Vec<f32> = (0..MODEL_SIZE)
            .map(|_| rng.gaussian(0.0, 0.1))
            .collect();

        // Each client starts with the same global model
        let mut client_weights = vec![0.0f32; CLIENT_COUNT * MODEL_SIZE];
        for c in 0..CLIENT_COUNT {
            let offset = c * MODEL_SIZE;
            client_weights[offset..offset + MODEL_SIZE]
                .copy_from_slice(&global_weights);
        }

        // Non-IID bias: each client has a different class prior
        // Client 0-1: majority class 0 (shift = -1.0)
        // Client 2-3: balanced (shift =  0.0)
        // Client 4-5: majority class 1 (shift = +1.0)
        let client_biases: Vec<f32> = (0..CLIENT_COUNT)
            .map(|c| match c {
                0 | 1 => -1.0,
                2 | 3 =>  0.0,
                _     =>  1.0,
            })
            .collect();

        FederatedSim {
            global_weights,
            client_weights,
            client_biases,
            loss_history: vec![f32::NAN; MAX_ROUNDS],
            client_losses: vec![1.0f32; CLIENT_COUNT],
            participation: vec![1u8; CLIENT_COUNT],
            lr: 0.05,
            dropout: 0.1,
            round: 0,
            agg_method: 0,
            rng,
        }
    }

    // -----------------------------------------------------------------------
    // Execute one complete federated learning round
    // -----------------------------------------------------------------------
    pub fn run_round(&mut self) {
        if self.round as usize >= MAX_ROUNDS { return; }

        // 1. Broadcast global model to all clients
        for c in 0..CLIENT_COUNT {
            let offset = c * MODEL_SIZE;
            self.client_weights[offset..offset + MODEL_SIZE]
                .copy_from_slice(&self.global_weights);
        }

        // 2. Each client does local training on private data
        let mut participating = Vec::new();
        for c in 0..CLIENT_COUNT {
            // Stochastic dropout simulation
            if self.rng.f32() < self.dropout {
                self.participation[c] = 0;
                continue;
            }
            self.participation[c] = 1;
            participating.push(c);

            // Generate synthetic mini-batch with client-specific distribution
            let bias = self.client_biases[c];
            let samples: Vec<[f32; FEATURES]> = (0..LOCAL_SAMPLES)
                .map(|_| {
                    let mut s = [0.0f32; FEATURES];
                    for f in 0..FEATURES {
                        s[f] = self.rng.gaussian(bias * 0.5, 1.0);
                    }
                    s
                })
                .collect();

            let labels: Vec<usize> = samples.iter().map(|s| {
                let score: f32 = s.iter().sum::<f32>() + bias;
                if score > 0.0 { 1 } else { 0 }
            }).collect();

            // Local SGD
            let offset = c * MODEL_SIZE;
            let mut local_w = self.client_weights[offset..offset + MODEL_SIZE].to_vec();
            let mut loss = 0.0f32;
            for _ in 0..LOCAL_EPOCHS {
                loss = sgd_step(&mut local_w, &samples, &labels, self.lr);
            }
            self.client_weights[offset..offset + MODEL_SIZE].copy_from_slice(&local_w);
            self.client_losses[c] = loss;
        }

        if participating.is_empty() { return; }

        // 3. Aggregation (FedAvg: weighted mean proportional to sample count)
        let n = participating.len() as f32;
        for i in 0..MODEL_SIZE {
            let agg = match self.agg_method {
                0 | 1 => {
                    // FedAvg / FedProx: simple mean of participating clients
                    participating.iter()
                        .map(|&c| self.client_weights[c * MODEL_SIZE + i])
                        .sum::<f32>() / n
                }
                _ => {
                    // Median aggregation (byzantine-robust sketch)
                    let mut vals: Vec<f32> = participating.iter()
                        .map(|&c| self.client_weights[c * MODEL_SIZE + i])
                        .collect();
                    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    vals[vals.len() / 2]
                }
            };

            // FedProx: proximal correction towards previous global
            if self.agg_method == 1 {
                let mu = 0.01;
                self.global_weights[i] = agg - mu * (agg - self.global_weights[i]);
            } else {
                self.global_weights[i] = agg;
            }
        }

        // 4. Record global validation loss (average of client losses)
        let avg_loss = self.client_losses.iter().sum::<f32>() / CLIENT_COUNT as f32;
        let idx = self.round as usize;
        self.loss_history[idx] = avg_loss;
        self.round += 1;
    }

    // -----------------------------------------------------------------------
    // Run N rounds at once (for fast forward)
    // -----------------------------------------------------------------------
    pub fn run_rounds(&mut self, n: u32) {
        for _ in 0..n { self.run_round(); }
    }

    // -----------------------------------------------------------------------
    // Configuration setters
    // -----------------------------------------------------------------------
    pub fn set_lr(&mut self, lr: f32) { self.lr = lr; }
    pub fn set_dropout(&mut self, p: f32) { self.dropout = p.clamp(0.0, 0.9); }
    pub fn set_agg_method(&mut self, m: u8) { self.agg_method = m % 3; }

    pub fn reset(&mut self) {
        let mut rng = Rng(1337);
        self.global_weights = (0..MODEL_SIZE).map(|_| rng.gaussian(0.0, 0.1)).collect();
        for c in 0..CLIENT_COUNT {
            let offset = c * MODEL_SIZE;
            let gw = self.global_weights.clone();
            self.client_weights[offset..offset + MODEL_SIZE].copy_from_slice(&gw);
        }
        self.loss_history = vec![f32::NAN; MAX_ROUNDS];
        self.client_losses = vec![1.0f32; CLIENT_COUNT];
        self.participation = vec![1u8; CLIENT_COUNT];
        self.round = 0;
        self.rng = rng;
    }

    // -----------------------------------------------------------------------
    // Zero-copy memory accessors
    // -----------------------------------------------------------------------
    pub fn global_weights_ptr(&self) -> *const f32 { self.global_weights.as_ptr() }
    pub fn client_weights_ptr(&self) -> *const f32 { self.client_weights.as_ptr() }
    pub fn loss_history_ptr(&self) -> *const f32 { self.loss_history.as_ptr() }
    pub fn client_losses_ptr(&self) -> *const f32 { self.client_losses.as_ptr() }
    pub fn participation_ptr(&self) -> *const u8 { self.participation.as_ptr() }

    // -----------------------------------------------------------------------
    // Scalar accessors
    // -----------------------------------------------------------------------
    pub fn model_size(&self) -> usize { MODEL_SIZE }
    pub fn client_count(&self) -> usize { CLIENT_COUNT }
    pub fn max_rounds(&self) -> usize { MAX_ROUNDS }
    pub fn current_round(&self) -> u32 { self.round }
    pub fn features(&self) -> usize { FEATURES }
    pub fn classes(&self) -> usize { CLASSES }

    /// L2 distance between global model and client c's model
    pub fn client_divergence(&self, c: usize) -> f32 {
        if c >= CLIENT_COUNT { return 0.0; }
        let offset = c * MODEL_SIZE;
        self.global_weights.iter()
            .zip(self.client_weights[offset..offset + MODEL_SIZE].iter())
            .map(|(g, l)| (g - l).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// Mean absolute weight of the global model (proxy for model magnitude)
    pub fn global_model_norm(&self) -> f32 {
        self.global_weights.iter().map(|w| w.abs()).sum::<f32>() / MODEL_SIZE as f32
    }
}