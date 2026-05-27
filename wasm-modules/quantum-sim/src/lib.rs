// wasm-modules/quantum-sim/src/lib.rs
use wasm_bindgen::prelude::*;
use std::f64::consts::PI;

#[wasm_bindgen]
pub struct QuantumState {
    alpha: f64,  // amplitude |0⟩
    beta: f64,   // amplitude |1⟩
    phase: f64,
}

#[wasm_bindgen]
impl QuantumState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        QuantumState { alpha: 1.0, beta: 0.0, phase: 0.0 }
    }

    /// Apply Hadamard gate: |0⟩ → (|0⟩+|1⟩)/√2
    pub fn hadamard(&mut self) {
        let new_alpha = (self.alpha + self.beta) / 2f64.sqrt();
        let new_beta  = (self.alpha - self.beta) / 2f64.sqrt();
        self.alpha = new_alpha;
        self.beta  = new_beta;
    }

    /// Apply Rz(θ) rotation
    pub fn rz(&mut self, theta: f64) {
        self.phase = (self.phase + theta) % (2.0 * PI);
    }

    pub fn prob_zero(&self) -> f64 { self.alpha * self.alpha }
    pub fn prob_one(&self)  -> f64 { self.beta  * self.beta  }
    pub fn get_phase(&self) -> f64 { self.phase }

    /// Returns [theta, phi] for Bloch sphere coordinates
    pub fn bloch_coords(&self) -> Vec<f64> {
        let theta = 2.0 * self.alpha.acos();
        let phi   = self.phase;
        vec![theta, phi]
    }
}