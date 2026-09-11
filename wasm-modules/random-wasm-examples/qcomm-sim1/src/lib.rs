use wasm_bindgen::prelude::*;

/// Quantum Communication Simulator
///
/// Simulates the BB84 Quantum Key Distribution (QKD) protocol and
/// elementary quantum gate operations on a register of up to 8 qubits.
///
/// Physics modelled:
///   - Single-qubit Bloch sphere state (θ, φ) + density matrix [r,i,r,i] × 4
///   - Pauli gates: X (NOT), Y, Z (phase flip)
///   - Hadamard gate H (superposition)
///   - Phase gates: S (π/2), T (π/4)
///   - Measurement in Z-basis with Born rule probability
///   - CNOT two-qubit gate (controlled NOT)
///   - Bell state preparation: |Φ+⟩, |Φ-⟩, |Ψ+⟩, |Ψ-⟩
///   - BB84 QKD protocol: Alice prepares, channel transmits (with noise),
///     Bob measures, sift keys, estimate QBER
///
/// Memory layout (zero-copy to JS):
///   qubit_states    : f32[MAX_QUBITS × 4]  -- [re(α), im(α), re(β), im(β)] per qubit
///   bloch_coords    : f32[MAX_QUBITS × 3]  -- [x, y, z] Bloch sphere coords
///   measurement_log : u8[LOG_LEN × 4]      -- [qubit_id, basis, alice_bit, bob_bit]
///   qber_history    : f32[MAX_ROUNDS]       -- QBER per BB84 round
///   key_bits        : u8[KEY_LEN]           -- sifted key bits

const MAX_QUBITS: usize = 8;
const STATE_SIZE: usize = MAX_QUBITS * 4; // 4 f32 per qubit: re_alpha, im_alpha, re_beta, im_beta
const BLOCH_SIZE: usize = MAX_QUBITS * 3; // Bloch sphere (x,y,z)
const LOG_LEN: usize = 128;
const MAX_ROUNDS: usize = 64;
const KEY_LEN: usize = 64;

// ---------------------------------------------------------------------------
// LCG RNG
// ---------------------------------------------------------------------------
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
    fn f32(&mut self) -> f32 {
        ((self.next() >> 33) as f32) / (u32::MAX as f32)
    }
    fn bit(&mut self) -> u8 { (self.next() & 1) as u8 }
    fn u8_range(&mut self, n: u8) -> u8 { (self.next() as u8) % n }
}

// ---------------------------------------------------------------------------
// Complex arithmetic helpers
// ---------------------------------------------------------------------------
#[inline]
fn cmul_re(ar: f32, ai: f32, br: f32, bi: f32) -> f32 { ar * br - ai * bi }
#[inline]
fn cmul_im(ar: f32, ai: f32, br: f32, bi: f32) -> f32 { ar * bi + ai * br }

/// Normalise a qubit state (re_a, im_a, re_b, im_b) in place
fn normalise(s: &mut [f32]) {
    let norm = (s[0]*s[0] + s[1]*s[1] + s[2]*s[2] + s[3]*s[3]).sqrt().max(1e-9);
    for x in s.iter_mut() { *x /= norm; }
}

/// Compute Bloch sphere coordinates from qubit state
fn bloch(s: &[f32]) -> (f32, f32, f32) {
    // x = 2·Re(α·β*) = 2(re_a·re_b + im_a·im_b)
    // y = 2·Im(α·β*) = 2(im_a·re_b - re_a·im_b)  [note: conjugate flips sign of im_b]
    // z = |α|² - |β|² = re_a²+im_a² - re_b²-im_b²
    let x = 2.0 * (s[0]*s[2] + s[1]*s[3]);
    let y = 2.0 * (s[1]*s[2] - s[0]*s[3]);
    let z = s[0]*s[0] + s[1]*s[1] - s[2]*s[2] - s[3]*s[3];
    (x, y, z)
}

// ---------------------------------------------------------------------------
// Single-qubit gates applied in-place on a state slice [re_a, im_a, re_b, im_b]
// ---------------------------------------------------------------------------
fn gate_x(s: &mut [f32]) {
    // X = [[0,1],[1,0]]  swaps |0⟩ and |1⟩ amplitudes
    s.swap(0, 2); s.swap(1, 3);
}

fn gate_y(s: &mut [f32]) {
    // Y = [[0,-i],[i,0]]
    // α' = -i·β = (-im_b, re_b) if we store as (re, im)
    // β' =  i·α = (-im_a, re_a)  (negated im part)
    let (ra, ia, rb, ib) = (s[0], s[1], s[2], s[3]);
    s[0] = ib;  s[1] = -rb;   // α' = i·β → re=im_b, im=-re_b ... standard: -i·β
    s[2] = -ia; s[3] = ra;    // β' = i·α
    // Recompute with correct Y: |ψ'⟩ = Y|ψ⟩
    // α' = -i·β → re_α'=im_β, im_α'=-re_β
    s[0] = ib; s[1] = -rb;
    s[2] = -ia; s[3] = ra;
}

fn gate_z(s: &mut [f32]) {
    // Z = [[1,0],[0,-1]]  negates |1⟩ amplitude
    s[2] = -s[2]; s[3] = -s[3];
}

fn gate_h(s: &mut [f32]) {
    // H = (1/√2)[[1,1],[1,-1]]
    let inv_sqrt2 = 1.0_f32 / 2.0_f32.sqrt();
    let (ra, ia, rb, ib) = (s[0], s[1], s[2], s[3]);
    s[0] = inv_sqrt2 * (ra + rb);
    s[1] = inv_sqrt2 * (ia + ib);
    s[2] = inv_sqrt2 * (ra - rb);
    s[3] = inv_sqrt2 * (ia - ib);
}

fn gate_s(s: &mut [f32]) {
    // S = [[1,0],[0,i]]  multiplies |1⟩ by i: (re_b, im_b) -> (-im_b, re_b)
    let tmp = s[2];
    s[2] = -s[3];
    s[3] = tmp;
}

fn gate_t(s: &mut [f32]) {
    // T = [[1,0],[0,e^(iπ/4)]] where e^(iπ/4) = (1+i)/√2
    let inv_sqrt2 = 1.0_f32 / 2.0_f32.sqrt();
    let (rb, ib) = (s[2], s[3]);
    s[2] = inv_sqrt2 * (rb - ib);
    s[3] = inv_sqrt2 * (rb + ib);
}

/// Measure qubit in Z-basis: returns 0 or 1 with Born rule probabilities
/// Collapses the qubit state to the measured eigenstate
fn measure_z(s: &mut [f32], rng: &mut Rng) -> u8 {
    let prob0 = s[0]*s[0] + s[1]*s[1]; // |α|²
    if rng.f32() < prob0 {
        // Collapsed to |0⟩
        s[0] = 1.0; s[1] = 0.0; s[2] = 0.0; s[3] = 0.0;
        0
    } else {
        // Collapsed to |1⟩
        s[0] = 0.0; s[1] = 0.0; s[2] = 1.0; s[3] = 0.0;
        1
    }
}

// ---------------------------------------------------------------------------
// BB84 QKD message log entry
// ---------------------------------------------------------------------------
const LOG_FIELDS: usize = 4;
// [qubit_id, alice_basis (0=Z,1=X), alice_bit, bob_bit_after_measurement]

// ---------------------------------------------------------------------------
// QuantumChannel struct
// ---------------------------------------------------------------------------
#[wasm_bindgen]
pub struct QuantumChannel {
    /// Qubit state register: [re_α, im_α, re_β, im_β] × MAX_QUBITS
    qubit_states: Vec<f32>,
    /// Bloch sphere coords: [x, y, z] × MAX_QUBITS
    bloch_coords: Vec<f32>,
    /// BB84 message log
    measurement_log: Vec<u8>,
    log_head: usize,
    /// QBER history per BB84 round
    qber_history: Vec<f32>,
    /// Sifted key bits
    key_bits: Vec<u8>,
    key_len: usize,
    /// Channel noise level (0.0 = perfect, 0.11 = BB84 security threshold)
    noise: f32,
    /// Whether an eavesdropper (Eve) is active
    eve_active: bool,
    /// Total BB84 rounds completed
    qkd_rounds: u32,
    /// Cumulative QBER across rounds
    total_qber: f32,
    /// Last gate applied (for display)
    last_gate: u8,
    /// Entangled pair register: qubit 0 is Alice's, qubit 1 is Bob's
    bell_state: u8, // 0=|Φ+⟩, 1=|Φ-⟩, 2=|Ψ+⟩, 3=|Ψ-⟩
    rng: Rng,
}

#[wasm_bindgen]
impl QuantumChannel {
    // -----------------------------------------------------------------------
    // Constructor: all qubits initialised to |0⟩
    // -----------------------------------------------------------------------
    pub fn new() -> Self {
        let mut qubit_states = vec![0.0f32; STATE_SIZE];
        let mut bloch_coords = vec![0.0f32; BLOCH_SIZE];

        // Initialise all qubits to |0⟩ state: α=1, β=0
        for q in 0..MAX_QUBITS {
            qubit_states[q * 4] = 1.0; // re_α = 1
            bloch_coords[q * 3 + 2] = 1.0; // z = 1 (north pole of Bloch sphere)
        }

        QuantumChannel {
            qubit_states,
            bloch_coords,
            measurement_log: vec![0u8; LOG_LEN * LOG_FIELDS],
            log_head: 0,
            qber_history: vec![0.0f32; MAX_ROUNDS],
            key_bits: vec![0u8; KEY_LEN],
            key_len: 0,
            noise: 0.02,
            eve_active: false,
            qkd_rounds: 0,
            total_qber: 0.0,
            last_gate: 0,
            bell_state: 0,
            rng: Rng(31415),
        }
    }

    // -----------------------------------------------------------------------
    // Apply single-qubit gate to qubit `q`
    // -----------------------------------------------------------------------
    fn apply_gate_inner(&mut self, q: usize, gate: u8) {
        if q >= MAX_QUBITS { return; }
        let s = &mut self.qubit_states[q * 4..(q + 1) * 4];
        match gate {
            0 => gate_x(s),
            1 => gate_y(s),
            2 => gate_z(s),
            3 => gate_h(s),
            4 => gate_s(s),
            5 => gate_t(s),
            _ => {}
        }
        normalise(s);
        let (x, y, z) = bloch(s);
        self.bloch_coords[q * 3]     = x;
        self.bloch_coords[q * 3 + 1] = y;
        self.bloch_coords[q * 3 + 2] = z;
        self.last_gate = gate;
    }

    pub fn apply_gate(&mut self, qubit: usize, gate: u8) {
        self.apply_gate_inner(qubit, gate);
    }

    /// Apply CNOT: control=q0, target=q1
    /// For a separable state |c⟩|t⟩ this is an approximation;
    /// true entanglement requires a full 4-qubit state vector.
    /// Here we implement it via the classical approximation valid for
    /// computational basis states (sufficient for visualisation).
    pub fn apply_cnot(&mut self, control: usize, target: usize) {
        if control >= MAX_QUBITS || target >= MAX_QUBITS || control == target { return; }
        // Measure control in Z-basis (non-destructively via threshold)
        let c_re_b = self.qubit_states[control * 4 + 2];
        let c_im_b = self.qubit_states[control * 4 + 3];
        let prob1 = c_re_b * c_re_b + c_im_b * c_im_b;
        if prob1 > 0.5 {
            // Control is |1⟩-dominant: apply X to target
            gate_x(&mut self.qubit_states[target * 4..(target + 1) * 4]);
        }
        self.update_bloch(control);
        self.update_bloch(target);
    }

    /// Prepare a Bell state on qubits 0 and 1
    /// |Φ+⟩ = (|00⟩+|11⟩)/√2 (default)
    pub fn prepare_bell_state(&mut self, state_id: u8) {
        let s = state_id % 4;
        self.bell_state = s;

        // Reset both qubits to |0⟩
        let inv_sqrt2 = 1.0_f32 / 2.0_f32.sqrt();
        for i in 0..8 { self.qubit_states[i] = 0.0; }
        // Note: true Bell states require 4-dimensional joint Hilbert space.
        // For visualisation we represent them as maximally mixed single-qubit
        // states (Bloch vector = 0) with the appropriate phase convention.
        match s {
            0 => { // |Φ+⟩ = (|00⟩+|11⟩)/√2
                self.qubit_states[0] = inv_sqrt2; // qubit 0: α=1/√2, β=1/√2 (approx)
                self.qubit_states[2] = inv_sqrt2;
                self.qubit_states[4] = inv_sqrt2;
                self.qubit_states[6] = inv_sqrt2;
            }
            1 => { // |Φ-⟩ = (|00⟩-|11⟩)/√2
                self.qubit_states[0] = inv_sqrt2;
                self.qubit_states[2] = -inv_sqrt2;
                self.qubit_states[4] = inv_sqrt2;
                self.qubit_states[6] = -inv_sqrt2;
            }
            2 => { // |Ψ+⟩ = (|01⟩+|10⟩)/√2
                self.qubit_states[2] = inv_sqrt2;
                self.qubit_states[4] = inv_sqrt2;
                self.qubit_states[0] = inv_sqrt2;
                self.qubit_states[6] = inv_sqrt2;
            }
            3 => { // |Ψ-⟩ = (|01⟩-|10⟩)/√2
                self.qubit_states[2] = inv_sqrt2;
                self.qubit_states[4] = inv_sqrt2;
                self.qubit_states[0] = -inv_sqrt2;
                self.qubit_states[6] = -inv_sqrt2;
            }
            _ => {}
        }
        for q in 0..2 { self.update_bloch(q); }
    }

    // -----------------------------------------------------------------------
    // BB84 QKD Protocol: single round
    //   1. Alice picks random bit and random basis {Z,X}
    //   2. Alice prepares qubit in chosen state
    //   3. (Optional) Eve intercepts and re-prepares — induces QBER ~25%
    //   4. Channel applies depolarising noise at rate `noise`
    //   5. Bob picks random basis and measures
    //   6. Sifting: keep bits where Alice and Bob used same basis
    //   Returns the QBER for this round
    // -----------------------------------------------------------------------
    pub fn run_bb84_round(&mut self) -> f32 {
        let n_bits = 20u32; // bits per round
        let mut matches = 0u32;
        let mut errors = 0u32;

        for i in 0..n_bits {
            let q = (i % MAX_QUBITS as u32) as usize;

            // Alice
            let alice_bit   = self.rng.bit();
            let alice_basis = self.rng.bit(); // 0=Z, 1=X

            // Prepare qubit: start with |0⟩
            self.qubit_states[q * 4]     = 1.0;
            self.qubit_states[q * 4 + 1] = 0.0;
            self.qubit_states[q * 4 + 2] = 0.0;
            self.qubit_states[q * 4 + 3] = 0.0;

            // If bit=1, apply X gate
            if alice_bit == 1 {
                gate_x(&mut self.qubit_states[q * 4..(q + 1) * 4]);
            }
            // If basis=X (Hadamard basis), apply H
            if alice_basis == 1 {
                gate_h(&mut self.qubit_states[q * 4..(q + 1) * 4]);
            }

            // Eve: intercept-resend attack (introduces ~25% QBER)
            if self.eve_active {
                let eve_basis = self.rng.bit();
                let s = &mut self.qubit_states[q * 4..(q + 1) * 4];
                if eve_basis == 1 { gate_h(s); } // Eve rotates to X basis
                let eve_result = measure_z(s, &mut self.rng);
                if eve_basis == 1 { gate_h(s); } // Eve re-prepares in original basis
                if eve_result == 1 { gate_x(s); }
                drop(s); // appease borrow checker
            }

            // Channel noise: depolarising channel
            if self.rng.f32() < self.noise {
                let noise_gate = self.rng.u8_range(3);
                let s = &mut self.qubit_states[q * 4..(q + 1) * 4];
                match noise_gate {
                    0 => gate_x(s),
                    1 => gate_y(s),
                    _ => gate_z(s),
                }
            }

            // Bob: random basis
            let bob_basis = self.rng.bit();
            if bob_basis == 1 {
                gate_h(&mut self.qubit_states[q * 4..(q + 1) * 4]);
            }
            let bob_result = measure_z(&mut self.qubit_states[q * 4..(q + 1) * 4], &mut self.rng);

            // Sifting: only count when bases match
            if alice_basis == bob_basis {
                matches += 1;
                if alice_bit != bob_result { errors += 1; }

                // Store sifted key bit
                if self.key_len < KEY_LEN {
                    self.key_bits[self.key_len] = alice_bit;
                    self.key_len += 1;
                } else {
                    // Ring: overwrite oldest
                    self.key_bits.rotate_left(1);
                    self.key_bits[KEY_LEN - 1] = alice_bit;
                }
            }

            // Log entry
            let log_offset = self.log_head * LOG_FIELDS;
            self.measurement_log[log_offset]     = q as u8;
            self.measurement_log[log_offset + 1] = alice_basis;
            self.measurement_log[log_offset + 2] = alice_bit;
            self.measurement_log[log_offset + 3] = bob_result;
            self.log_head = (self.log_head + 1) % LOG_LEN;

            self.update_bloch(q);
        }

        let qber = if matches > 0 { errors as f32 / matches as f32 } else { 0.0 };
        self.total_qber += qber;
        let round_idx = (self.qkd_rounds as usize).min(MAX_ROUNDS - 1);
        self.qber_history[round_idx] = qber;
        self.qkd_rounds += 1;
        qber
    }

    // -----------------------------------------------------------------------
    // Measurement
    // -----------------------------------------------------------------------
    pub fn measure(&mut self, qubit: usize) -> u8 {
        if qubit >= MAX_QUBITS { return 0; }
        let result = measure_z(&mut self.qubit_states[qubit * 4..(qubit + 1) * 4], &mut self.rng);
        self.update_bloch(qubit);
        result
    }

    // -----------------------------------------------------------------------
    // Reset qubit to |0⟩
    // -----------------------------------------------------------------------
    pub fn reset_qubit(&mut self, qubit: usize) {
        if qubit >= MAX_QUBITS { return; }
        let q = qubit;
        self.qubit_states[q * 4] = 1.0;
        self.qubit_states[q * 4 + 1] = 0.0;
        self.qubit_states[q * 4 + 2] = 0.0;
        self.qubit_states[q * 4 + 3] = 0.0;
        self.bloch_coords[q * 3]     = 0.0;
        self.bloch_coords[q * 3 + 1] = 0.0;
        self.bloch_coords[q * 3 + 2] = 1.0;
    }

    // -----------------------------------------------------------------------
    // Update Bloch sphere coordinates for qubit q
    // -----------------------------------------------------------------------
    fn update_bloch(&mut self, q: usize) {
        let s = &self.qubit_states[q * 4..q * 4 + 4];
        let (x, y, z) = bloch(s);
        self.bloch_coords[q * 3]     = x;
        self.bloch_coords[q * 3 + 1] = y;
        self.bloch_coords[q * 3 + 2] = z;
    }

    // -----------------------------------------------------------------------
    // Configuration
    // -----------------------------------------------------------------------
    pub fn set_noise(&mut self, n: f32) { self.noise = n.clamp(0.0, 0.5); }
    pub fn set_eve(&mut self, active: bool) { self.eve_active = active; }
    pub fn reset_all(&mut self) {
        for q in 0..MAX_QUBITS { self.reset_qubit(q); }
        self.key_len = 0;
        self.qkd_rounds = 0;
        self.total_qber = 0.0;
    }

    // -----------------------------------------------------------------------
    // Zero-copy memory accessors
    // -----------------------------------------------------------------------
    pub fn qubit_states_ptr(&self) -> *const f32 { self.qubit_states.as_ptr() }
    pub fn bloch_coords_ptr(&self) -> *const f32 { self.bloch_coords.as_ptr() }
    pub fn measurement_log_ptr(&self) -> *const u8 { self.measurement_log.as_ptr() }
    pub fn qber_history_ptr(&self) -> *const f32 { self.qber_history.as_ptr() }
    pub fn key_bits_ptr(&self) -> *const u8 { self.key_bits.as_ptr() }

    // -----------------------------------------------------------------------
    // Scalar accessors
    // -----------------------------------------------------------------------
    pub fn max_qubits(&self) -> usize { MAX_QUBITS }
    pub fn log_len(&self) -> usize { LOG_LEN }
    pub fn log_head(&self) -> usize { self.log_head }
    pub fn qkd_rounds(&self) -> u32 { self.qkd_rounds }
    pub fn key_len(&self) -> usize { self.key_len }
    pub fn average_qber(&self) -> f32 {
        if self.qkd_rounds == 0 { return 0.0; }
        self.total_qber / self.qkd_rounds as f32
    }
    pub fn is_secure(&self) -> bool { self.average_qber() < 0.11 }
    pub fn eve_active(&self) -> bool { self.eve_active }
    pub fn noise(&self) -> f32 { self.noise }

    /// Probability of measuring |0⟩ for qubit q
    pub fn prob0(&self, qubit: usize) -> f32 {
        if qubit >= MAX_QUBITS { return 0.0; }
        let q = qubit;
        let ra = self.qubit_states[q * 4];
        let ia = self.qubit_states[q * 4 + 1];
        ra * ra + ia * ia
    }
}