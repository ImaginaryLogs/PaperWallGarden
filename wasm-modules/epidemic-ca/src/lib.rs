use wasm_bindgen::prelude::*;

const NUM_NODES: usize = 30;
// FIXED: Adjusted to match the exact count of declared edges in the array below
const NUM_EDGES: usize = 32; 

// States for the SIR Cellular Automaton
const S: u8 = 0; // Susceptible (Blue)
const I: u8 = 1; // Infected (Red)
const R: u8 = 2; // Recovered/Immune (Green)

// --- GRAPH TOPOLOGY (The "Cellular" Neighborhoods) ---
const EDGES: [[usize; 2]; NUM_EDGES] = [
    // Hub 0 cluster
    [0,1], [0,2], [0,3], [0,4], [0,5], [1,2], [3,4],
    // Hub 10 cluster
    [10,11], [10,12], [10,13], [10,14], [10,15], [10,16], [12,13], [14,15],
    // Hub 20 cluster
    [20,21], [20,22], [20,23], [20,24], [20,25], [20,26], [20,27], [22,23],
    // Inter-cluster bridges (The weak links that spread the virus)
    [0,10], [10,20], [5,11], [16,21], [4,27], [2,13],
    // Outliers
    [28, 20], [29, 0], [28, 29]
];

// --- CANVAS GEOMETRY ---
const NODE_X: [f32; NUM_NODES] = [
    200., 150., 250., 140., 260., 200., 120., 280., 180., 220., // Cluster 0
    450., 400., 500., 400., 500., 450., 380., 520., 420., 480., // Cluster 10
    700., 650., 750., 640., 760., 700., 620., 780., 660., 740.  // Cluster 20
];
const NODE_Y: [f32; NUM_NODES] = [
    180., 130., 130., 230., 230., 100., 180., 180., 270., 270.,
    180., 130., 130., 230., 230., 280., 180., 180., 100., 100.,
    180., 130., 130., 230., 230., 100., 180., 180., 270., 270.
];

struct Rng(u64);
impl Rng {
    fn u64(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); self.0 }
    fn f32(&mut self) -> f32 { ((self.u64() >> 33) as f32) / (u32::MAX as f32) }
}

#[wasm_bindgen]
pub struct EpidemicCA {
    states: Vec<u8>,
    tick: u32,
    transmission_rate: f32, 
    recovery_rate: f32,     
    rng: Rng,
    
    history_s: Vec<u32>,
    history_i: Vec<u32>,
    history_r: Vec<u32>,
}

#[wasm_bindgen]
impl EpidemicCA {
    pub fn new() -> Self {
        let mut sim = Self {
            states: vec![S; NUM_NODES],
            tick: 0,
            transmission_rate: 0.15,
            recovery_rate: 0.05,
            // FIXED: Replaced invalid characters with valid hex sequence (0-9, A-F)
            rng: Rng(0x0E1A_CAFE), 
            history_s: Vec::new(), history_i: Vec::new(), history_r: Vec::new(),
        };
        sim.reset();
        sim
    }

    pub fn step(&mut self) {
        self.tick += 1;
        
        let mut next_states = self.states.clone();

        for i in 0..NUM_NODES {
            match self.states[i] {
                S => {
                    let mut infected_neighbors = 0;
                    for edge in EDGES.iter() {
                        if edge[0] == i && self.states[edge[1]] == I { infected_neighbors += 1; }
                        if edge[1] == i && self.states[edge[0]] == I { infected_neighbors += 1; }
                    }

                    if infected_neighbors > 0 {
                        let escape_prob = (1.0 - self.transmission_rate).powi(infected_neighbors as i32);
                        let infection_prob = 1.0 - escape_prob;
                        
                        if self.rng.f32() < infection_prob {
                            next_states[i] = I;
                        }
                    }
                },
                I => {
                    if self.rng.f32() < self.recovery_rate {
                        next_states[i] = R;
                    }
                },
                _ => {} 
            }
        }

        self.states = next_states; 
        self.record_history();
    }

    fn record_history(&mut self) {
        let mut count_s = 0; let mut count_i = 0; let mut count_r = 0;
        for &state in &self.states {
            match state { S => count_s+=1, I => count_i+=1, R => count_r+=1, _=>{} }
        }
        self.history_s.push(count_s);
        self.history_i.push(count_i);
        self.history_r.push(count_r);
        
        if self.history_s.len() > 100 {
            self.history_s.remove(0); self.history_i.remove(0); self.history_r.remove(0);
        }
    }

    pub fn reset(&mut self) {
        self.states = vec![S; NUM_NODES];
        self.tick = 0;
        self.history_s.clear(); self.history_i.clear(); self.history_r.clear();
        
        self.states[10] = I; 
        self.record_history();
    }

    pub fn set_transmission(&mut self, val: f32) { self.transmission_rate = val; }
    pub fn set_recovery(&mut self, val: f32) { self.recovery_rate = val; }

    pub fn states_ptr(&self) -> *const u8 { self.states.as_ptr() }
    pub fn node_x_ptr(&self) -> *const f32 { NODE_X.as_ptr() }
    pub fn node_y_ptr(&self) -> *const f32 { NODE_Y.as_ptr() }
    pub fn edges_ptr(&self) -> *const usize { EDGES.as_ptr() as *const usize }
    pub fn hist_s_ptr(&self) -> *const u32 { self.history_s.as_ptr() }
    pub fn hist_i_ptr(&self) -> *const u32 { self.history_i.as_ptr() }
    pub fn hist_r_ptr(&self) -> *const u32 { self.history_r.as_ptr() }

    pub fn num_nodes(&self) -> usize { NUM_NODES }
    pub fn num_edges(&self) -> usize { NUM_EDGES }
    pub fn current_tick(&self) -> u32 { self.tick }
    pub fn hist_len(&self) -> usize { self.history_s.len() }
}