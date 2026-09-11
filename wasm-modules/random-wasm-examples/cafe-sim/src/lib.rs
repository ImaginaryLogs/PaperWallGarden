use wasm_bindgen::prelude::*;

const NUM_PLACES: usize = 4;
const NUM_TRANSITIONS: usize = 4;

// --- PETRI NET TOPOLOGY ---
// Places: 
// 0: Order_Queue, 1: Beans_Hopper, 2: Completed_Drinks, 3: Frustrated_Walkouts
// Transitions: 
// T0: Customer_Arrival, T1: Grind_Beans, T2: Brew_Espresso, T3: Patience_Expire

const PETRI_INPUTS: [[i32; 2]; NUM_TRANSITIONS] = [
    [-1, -1], // T0: Pure generator (Arrivals)
    [-1, -1], // T1: Pure generator (Grinder)
    [ 0,  1], // T2: Brew consumes Order (0) AND Beans (1)
    [ 0, -1], // T3: Patience Expire consumes Order (0)
];

const PETRI_OUTPUTS: [[i32; 2]; NUM_TRANSITIONS] = [
    [ 0, -1], // T0: Deposits customer into Order_Queue (0)
    [ 1, -1], // T1: Deposits espresso into Beans_Hopper (1)
    [ 2, -1], // T2: Outputs a Completed_Drink (2)
    [ 3, -1], // T3: Outputs a Frustrated_Customer (3)
];

// --- CANVAS GEOMETRY ---
const PLACE_X: [f32; NUM_PLACES] = [200.0, 200.0, 650.0, 650.0];
const PLACE_Y: [f32; NUM_PLACES] = [150.0, 350.0, 250.0, 80.0];

const TRANS_X: [f32; NUM_TRANSITIONS] = [80.0, 80.0, 425.0, 425.0];
const TRANS_Y: [f32; NUM_TRANSITIONS] = [150.0, 350.0, 250.0, 80.0];

// --- LCG RNG ---
struct Rng(u64);
impl Rng {
    fn u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
    fn f32(&mut self) -> f32 { ((self.u64() >> 33) as f32) / (u32::MAX as f32) }
}

#[wasm_bindgen]
pub struct CafePetri {
    tokens: Vec<u32>,
    lambda_effective: Vec<f32>,
    step_count: u32,
    base_arrival_rate: f32,
    rng: Rng,
}

#[wasm_bindgen]
impl CafePetri {
    pub fn new() -> Self {
        Self {
            tokens: vec![0; NUM_PLACES],
            lambda_effective: vec![0.0; NUM_TRANSITIONS],
            step_count: 0,
            base_arrival_rate: 2.5,
            rng: Rng(0xCAFE_BABE), // Seeded RNG
        }
    }

    pub fn step(&mut self) {
        self.step_count += 1;
        
        let line_length = self.tokens[0];

        // 1. Dynamic Rule Feedback Loop
        // If the line has > 5 people, arrival rate drops (balking) and walkout rate skyrockets
        self.lambda_effective[0] = if line_length > 5 { self.base_arrival_rate * 0.1 } else { self.base_arrival_rate }; // T0: Arrivals
        self.lambda_effective[1] = 3.0; // T1: Grind beans (steady rate)
        self.lambda_effective[2] = 5.0; // T2: Brew speed
        self.lambda_effective[3] = if line_length > 5 { 2.5 } else { 0.1 }; // T3: Walkout (impatience spikes)

        // 2. Identify structural readiness
        let mut transition_ready = vec![true; NUM_TRANSITIONS];
        for t in 0..NUM_TRANSITIONS {
            for i in 0..2 {
                let p = PETRI_INPUTS[t][i];
                if p >= 0 && self.tokens[p as usize] == 0 {
                    transition_ready[t] = false;
                }
            }
        }

        // 3. Stochastically execute ready transitions
        for t in 0..NUM_TRANSITIONS {
            if transition_ready[t] {
                // Determine if transition fires this tick based on its lambda weight
                let fire_chance = self.lambda_effective[t] * 0.1; 
                if self.rng.f32() < fire_chance {
                    // Consume
                    for i in 0..2 {
                        let p = PETRI_INPUTS[t][i];
                        if p >= 0 { self.tokens[p as usize] -= 1; }
                    }
                    // Produce
                    for o in 0..2 {
                        let p = PETRI_OUTPUTS[t][o];
                        if p >= 0 { self.tokens[p as usize] += 1; }
                    }
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.tokens = vec![0; NUM_PLACES];
        self.step_count = 0;
    }

    pub fn set_arrival_rate(&mut self, rate: f32) {
        self.base_arrival_rate = rate;
    }

    // Pointers for zero-copy JS access
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
}