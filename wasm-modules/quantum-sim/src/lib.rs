use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct MultiQubitState {
    // Amplitudes for |00>, |01>, |10>, |11>
    // Stored as separate real and imaginary components for WASM compatibility
    c00_re: f64, c00_im: f64,
    c01_re: f64, c01_im: f64,
    c10_re: f64, c10_im: f64,
    c11_re: f64, c11_im: f64,
}

#[wasm_bindgen]
impl MultiQubitState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Initialize to the pure ground state |00> (1.0 + 0.0i, all others 0)
        MultiQubitState {
            c00_re: 1.0, c00_im: 0.0,
            c01_re: 0.0, c01_im: 0.0,
            c10_re: 0.0, c10_im: 0.0,
            c11_re: 0.0, c11_im: 0.0,
        }
    }

    /// Apply Hadamard to Qubit 0 (Targeting the first index slot)
    #[wasm_bindgen]
    pub fn hadamard_q0(&mut self) {
        let s = 2.0f64.sqrt();
        
        // Pairwise combinations mapping state transformations:
        // |00> and |10> mix together; |01> and |11> mix together.
        let (r00, i00, r10, i10) = (self.c00_re, self.c00_im, self.c10_re, self.c10_im);
        self.c00_re = (r00 + r10) / s; self.c00_im = (i00 + i10) / s;
        self.c10_re = (r00 - r10) / s; self.c10_im = (i00 - i10) / s;

        let (r01, i01, r11, i11) = (self.c01_re, self.c01_im, self.c11_re, self.c11_im);
        self.c01_re = (r01 + r11) / s; self.c01_im = (i01 + i11) / s;
        self.c11_re = (r01 - r11) / s; self.c11_im = (i01 - i11) / s;
    }

    /// Apply Controlled-NOT (CNOT). Qubit 0 is Control, Qubit 1 is Target.
    /// If Qubit 0 is |1>, flip the state of Qubit 1 (|10> <-> |11>).
    #[wasm_bindgen]
    pub fn cnot(&mut self) {
        // Swap amplitudes of |10> and |11>
        std::mem::swap(&mut self.c10_re, &mut self.c11_re);
        std::mem::swap(&mut self.c10_im, &mut self.c11_im);
    }

    /// Returns probabilities for [|00>, |01>, |10>, |11>] to render on the UI chart
    #[wasm_bindgen]
    pub fn get_probabilities(&self) -> Vec<f64> {
        vec![
            self.c00_re.powi(2) + self.c00_im.powi(2),
            self.c01_re.powi(2) + self.c01_im.powi(2),
            self.c10_re.powi(2) + self.c10_im.powi(2),
            self.c11_re.powi(2) + self.c11_im.powi(2),
        ]
    }
}