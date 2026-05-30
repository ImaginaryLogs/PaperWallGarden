use wasm_bindgen::prelude::*;

/// # Quantum Communication — Network-Graph Petri Net + HMM Channel Estimation
const NUM_PLACES: usize = 12;
const NUM_TRANSITIONS: usize = 10;

/// Firing rates (base, may be modulated by HMM-estimated channel state)
const LAMBDA_BASE: [f32; NUM_TRANSITIONS] = [
    1.5,  // T0: EPR_Gen_A
    5.5,  // T1: Distribute_A
    1.5,  // T2: EPR_Gen_B
    5.5,  // T3: Distribute_B
    8.0,  // T4: Bell_Measure
    4.5,  // T5: Classical_Send
    6.0,  // T6: Entanglement_Done
    0.04, // T7: Decoherence
    2.5,  // T8: Error_Correct
    4.0,  // T9: Purification
];

/// Input arcs (place → transition) - FIXED TYPE TO i32 TO MATCH JS BUFFER CASTS
const PETRI_INPUTS: [[i32; 3]; NUM_TRANSITIONS] = [
    [-1, -1, -1], // T0: EPR_Gen_A      -> Pure generation (No inputs)
    [ 0, -1, -1], // T1: Distribute_A   -> Consumes from EPR_A_Attempts (0)
    [-1, -1, -1], // T2: EPR_Gen_B      -> Pure generation
    [ 3, -1, -1], // T3: Distribute_B   -> Consumes from EPR_B_Attempts (3)
    [ 2,  4, -1], // T4: Bell_Measure   -> Consumes from Repeater_L (2) AND Repeater_R (4)
    [ 6, -1, -1], // T5: Classical_Send -> Consumes from Bell_Ready (6)
    [ 1,  5,  7], // T6: Entangle_Done  -> Consumes Alice_Mem (1), Bob_Mem (5), Classical_Msg (7)
    [ 2,  4, -1], // T7: Decoherence    -> Purges vulnerable repeater memories
    [ 9, -1, -1], // T8: Error_Correct  -> Consumes Error_Detected (9)
    [11, 11, -1], // T9: Purification   -> Consumes from Fidelity_Pool pairs (11)
];

/// Output arcs (transition → place) - FIXED TYPE TO i32 TO MATCH JS BUFFER CASTS
const PETRI_OUTPUTS: [[i32; 4]; NUM_TRANSITIONS] = [
    [ 0, -1, -1, -1], // T0: Produces to EPR_A_Attempts (0)
    [ 1,  2, -1, -1], // T1: Produces to Alice_Memory (1) and Repeater_L (2)
    [ 3, -1, -1, -1], // T2: Produces to EPR_B_Attempts (3)
    [ 4,  5, -1, -1], // T3: Produces to Repeater_R (4) and Bob_Memory (5)
    [ 6, -1, -1, -1], // T4: Produces to Bell_Ready (6)
    [ 7, -1, -1, -1], // T5: Produces to Classical_Msg (7)
    [ 8, 11, -1, -1], // T6: Produces Entangled_Alice_Bob (8) and Fidelity_Pool (11)
    [-1, -1, -1, -1], // T7: Empties resources (No outputs)
    [10, -1, -1, -1], // T8: Produces QEC_Recovery (10)
    [ 8, -1, -1, -1], // T9: Returns purified token to Entangled pool (8)
];

// --- SPACED COORDINATE MATRICES ---
const PLACE_X: [f32; NUM_PLACES] = [120.0, 320.0, 520.0, 1280.0, 880.0, 1080.0, 700.0, 700.0, 700.0, 350.0, 700.0, 1050.0];
const PLACE_Y: [f32; NUM_PLACES] = [150.0, 280.0, 280.0,  150.0, 280.0,  280.0, 280.0, 120.0, 440.0, 600.0, 600.0,  600.0];

const TRANS_X: [f32; NUM_TRANSITIONS] = [220.0, 420.0, 1180.0, 980.0, 610.0, 700.0, 790.0, 700.0, 520.0, 880.0];
const TRANS_Y: [f32; NUM_TRANSITIONS] = [150.0, 280.0,  150.0, 280.0, 280.0, 200.0, 360.0, 280.0, 600.0, 600.0];

// ---------------------------------------------------------------------------
// HMM — Hidden channel quality state
// ---------------------------------------------------------------------------
const HMM_H: usize = 3; 
const HMM_A: [[f32; HMM_H]; HMM_H] = [
    [0.8, 0.15, 0.05], 
    [0.2, 0.6,  0.2 ], 
    [0.1, 0.3,  0.6 ], 
];
const EMIT_SUCCESS: [f32; HMM_H] = [0.8, 0.4, 0.1];
const EMIT_DECOHERE: [f32; HMM_H] = [0.1, 0.4, 0.8];

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

#[wasm_bindgen]
pub struct QCommPetri {
    tokens: Vec<u32>,
    lambda_effective: Vec<f32>,
    step_count: u32,
    entanglements_established: u32,
    decoherence_events: u32,
    purifications: u32,
    fidelity_score: f32,
}

#[wasm_bindgen]
impl QCommPetri {
    pub fn new() -> Self {
        let mut tokens = vec![0; NUM_PLACES];
        tokens[0] = 2; 
        tokens[3] = 2;

        Self {
            tokens,
            lambda_effective: vec![1.0; NUM_TRANSITIONS],
            step_count: 0,
            entanglements_established: 0,
            decoherence_events: 0,
            purifications: 0,
            fidelity_score: 0.85,
        }
    }

    pub fn step(&mut self) {
        self.step_count += 1;
        
        // Match logic checks and evaluate rule steps
        let mut transition_ready = vec![true; NUM_TRANSITIONS];
        for t in 0..NUM_TRANSITIONS {
            for i in 0..3 {
                let p = PETRI_INPUTS[t][i];
                if p >= 0 && self.tokens[p as usize] == 0 {
                    transition_ready[t] = false;
                }
            }
        }

        // Execute ready transitions
        for t in 0..NUM_TRANSITIONS {
            if transition_ready[t] {
                // Prevent decoherence from firing every tick to allow synchronization
                if t == 7 && self.step_count % 3 != 0 { continue; }

                // Consume
                for i in 0..3 {
                    let p = PETRI_INPUTS[t][i];
                    if p >= 0 { self.tokens[p as usize] -= 1; }
                }
                // Produce
                for o in 0..4 {
                    let p = PETRI_OUTPUTS[t][o];
                    if p >= 0 { self.tokens[p as usize] += 1; }
                }

                // Increment dashboard metrics
                if t == 6 { self.entanglements_established += 1; }
                if t == 7 { self.decoherence_events += 1; }
                if t == 9 { self.purifications += 1; }
            }
        }

        // Automatic generation refresh rules
        if self.tokens[0] < 2 { self.tokens[0] += 1; }
        if self.tokens[3] < 2 { self.tokens[3] += 1; }
    }

    pub fn reset(&mut self) {
        self.tokens = vec![0; NUM_PLACES];
        self.tokens[0] = 2;
        self.tokens[3] = 2;
        self.step_count = 0;
        self.entanglements_established = 0;
        self.decoherence_events = 0;
        self.purifications = 0;
    }

    // Pointers for JS shared memory mapping
    pub fn tokens_ptr(&self) -> *const u32 { self.tokens.as_ptr() }
    pub fn lambda_effective_ptr(&self) -> *const f32 { self.lambda_effective.as_ptr() }
    pub fn inputs_ptr(&self) -> *const i32 { PETRI_INPUTS.as_ptr() as *const i32 }
    pub fn outputs_ptr(&self) -> *const i32 { PETRI_OUTPUTS.as_ptr() as *const i32 }
    pub fn place_x_ptr(&self) -> *const f32 { PLACE_X.as_ptr() }
    pub fn place_y_ptr(&self) -> *const f32 { PLACE_Y.as_ptr() }
    pub fn trans_x_ptr(&self) -> *const f32 { TRANS_X.as_ptr() }
    pub fn trans_y_ptr(&self) -> *const f32 { TRANS_Y.as_ptr() }

    pub fn num_places(&self) -> usize { NUM_PLACES }
    pub fn num_transitions(&self) -> usize { NUM_TRANSITIONS }
    pub fn step_count(&self) -> u32 { self.step_count }
    pub fn entanglements_established(&self) -> u32 { self.entanglements_established }
    pub fn decoherence_events(&self) -> u32 { self.decoherence_events }
    pub fn purifications(&self) -> u32 { self.purifications }
    pub fn fidelity_score(&self) -> f32 { self.fidelity_score }
    pub fn set_pump_power(&mut self, _val: f32) {}
}