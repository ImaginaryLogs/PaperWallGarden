// wasm-modules/quantum-sim/src/lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct QuantumState {
    alpha_re: f64,
    alpha_im: f64,
    beta_re: f64,
    beta_im: f64,
}

#[wasm_bindgen]
impl QuantumState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Initialize state to pure |0⟩ ground state: 1.0 + 0.0i
        QuantumState { 
            alpha_re: 1.0, 
            alpha_im: 0.0, 
            beta_re: 0.0, 
            beta_im: 0.0 
        }
    }

    /// Apply Hadamard gate: |0⟩ → (|0⟩+|1⟩)/√2, |1⟩ → (|0⟩-|1⟩)/√2
    #[wasm_bindgen]
    pub fn hadamard(&mut self) {
        let sqrt2 = 2.0f64.sqrt();
        
        let a_re = self.alpha_re;
        let a_im = self.alpha_im;
        let b_re = self.beta_re;
        let b_im = self.beta_im;

        // New Alpha = (Alpha + Beta) / sqrt(2)
        self.alpha_re = (a_re + b_re) / sqrt2;
        self.alpha_im = (a_im + b_im) / sqrt2;

        // New Beta = (Alpha - Beta) / sqrt(2)
        self.beta_re = (a_re - b_re) / sqrt2;
        self.beta_im = (a_im - b_im) / sqrt2;
    }

    /// Apply Pauli-X (NOT) gate
    #[wasm_bindgen]
    pub fn pauli_x(&mut self) {
        std::mem::swap(&mut self.alpha_re, &mut self.beta_re);
        std::mem::swap(&mut self.alpha_im, &mut self.beta_im);
    }

    /// Apply Pauli-Z (Phase Flip) gate
    #[wasm_bindgen]
    pub fn pauli_z(&mut self) {
        self.beta_re = -self.beta_re;
        self.beta_im = -self.beta_im;
    }

    /// Apply arbitrary Rotation around the X-axis by angle theta (in radians)
    #[wasm_bindgen]
    pub fn rotate_x(&mut self, theta: f64) {
        let c = (theta / 2.0).cos();
        let s = (theta / 2.0).sin();

        let a_re = self.alpha_re;
        let a_im = self.alpha_im;
        let b_re = self.beta_re;
        let b_im = self.beta_im;

        // Matrix transform multiplication: New Alpha = cos(θ/2)*α - i*sin(θ/2)*β
        self.alpha_re = c * a_re + s * b_im;
        self.alpha_im = c * a_im - s * b_re;

        // Matrix transform multiplication: New Beta = -i*sin(θ/2)*α + cos(θ/2)*β
        self.beta_re = c * b_re + s * a_im;
        self.beta_im = c * b_im - s * a_re;
    }

    /// Returns [theta, phi] for Bloch sphere coordinates from complex amplitudes
    #[wasm_bindgen]
    pub fn bloch_coords(&self) -> Vec<f64> {
        // Probability of reading |0⟩ state is |alpha|^2
        let prob_zero = (self.alpha_re * self.alpha_re) + (self.alpha_im * self.alpha_im);
        
        // Clamp value between 0.0 and 1.0 to shield against microscopic float rounding errors
        let prob_zero_clamped = prob_zero.max(0.0).min(1.0);
        
        // Theta = 2 * acos(sqrt(P(0)))
        let theta = 2.0 * prob_zero_clamped.sqrt().acos();

        // Compute phase angle phi = arg(beta) - arg(alpha)
        let alpha_phase = self.alpha_im.atan2(self.alpha_re);
        let beta_phase = self.beta_im.atan2(self.beta_re);
        let mut phi = beta_phase - alpha_phase;
        
        if phi < 0.0 {
            phi += 2.0 * std::f64::consts::PI;
        }

        vec![theta, phi]
    }
}