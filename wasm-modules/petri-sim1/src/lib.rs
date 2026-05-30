use wasm_bindgen::prelude::*;

/// A Petri Net models business processes as a directed bipartite graph.
/// Places hold tokens (resources/conditions); Transitions fire when
/// all input places are sufficiently marked, consuming and producing tokens.
///
/// This implementation encodes a canonical Order Fulfilment workflow:
///   [Order Received] -> (Validate) -> [Order Validated]
///                                   -> (Check Stock) -> [In Stock] -> (Pick & Pack) -> [Packed]
///                                                    -> [Out of Stock] -> (Reorder) -> [Restocked]
///   [Packed] -> (Ship) -> [Shipped] -> (Deliver) -> [Delivered]

// ---------------------------------------------------------------------------
// Data layout (all arrays are parallel, indexed by place/transition id)
// ---------------------------------------------------------------------------
// Places (8 total):
//  0  Order Received
//  1  Order Validated
//  2  In Stock
//  3  Out of Stock
//  4  Packed
//  5  Restocked
//  6  Shipped
//  7  Delivered

// Transitions (6 total):
//  0  Validate Order      (consumes: [0]        produces: [1])
//  1  Check Stock         (consumes: [1]        produces: [2] OR [3])  -- probabilistic
//  2  Pick and Pack       (consumes: [2]        produces: [4])
//  3  Reorder Stock       (consumes: [3]        produces: [5])
//  4  Restock -> Pack     (consumes: [5]        produces: [4])
//  5  Ship                (consumes: [4]        produces: [6])
//  6  Deliver             (consumes: [6]        produces: [7])

const NUM_PLACES: usize = 8;
const NUM_TRANSITIONS: usize = 7;

/// Incidence matrix rows = places, cols = transitions.
/// Negative = consume, Positive = produce.
/// Row 2 and 3 for transition 1 (Check Stock) are handled via
/// a probabilistic branch, so the matrix entry for that transition
/// is left 0 and resolved in `fire_transitions`.
#[rustfmt::skip]
const INCIDENCE: [[i32; NUM_TRANSITIONS]; NUM_PLACES] = [
    //  T0   T1   T2   T3   T4   T5   T6
    [  -1,   0,   0,   0,   0,   0,   0 ],  // P0 Order Received
    [   1,  -1,   0,   0,   0,   0,   0 ],  // P1 Order Validated
    [   0,   0,  -1,   0,   0,   0,   0 ],  // P2 In Stock     (T1 writes here probabilistically)
    [   0,   0,   0,  -1,   0,   0,   0 ],  // P3 Out of Stock (T1 writes here probabilistically)
    [   0,   0,   1,   0,   1,  -1,   0 ],  // P4 Packed
    [   0,   0,   0,   1,  -1,   0,   0 ],  // P5 Restocked
    [   0,   0,   0,   0,   0,   1,  -1 ],  // P6 Shipped
    [   0,   0,   0,   0,   0,   0,   1 ],  // P7 Delivered
];

/// Which place must be marked for a transition to be enabled.
/// (transition_id -> required input place ids)
const INPUT_PLACES: [[i32; 2]; NUM_TRANSITIONS] = [
    [0, -1],  // T0: needs P0
    [1, -1],  // T1: needs P1
    [2, -1],  // T2: needs P2
    [3, -1],  // T3: needs P3
    [5, -1],  // T4: needs P5
    [4, -1],  // T5: needs P4
    [6, -1],  // T6: needs P6
];

// ---------------------------------------------------------------------------
// Layout coordinates for the canvas renderer (480 x 320 viewport)
// ---------------------------------------------------------------------------
// Places: (x, y) centre of each place circle
const PLACE_X: [f32; NUM_PLACES] = [40.0, 120.0, 200.0, 200.0, 320.0, 260.0, 400.0, 480.0];
const PLACE_Y: [f32; NUM_PLACES] = [160.0, 160.0, 100.0, 220.0, 160.0, 220.0, 160.0, 160.0];

// Transitions: (x, y) centre of each transition rectangle
const TRANS_X: [f32; NUM_TRANSITIONS] = [80.0, 160.0, 260.0, 230.0, 290.0, 360.0, 440.0];
const TRANS_Y: [f32; NUM_TRANSITIONS] = [160.0, 160.0, 100.0, 220.0, 190.0, 160.0, 160.0];

// ---------------------------------------------------------------------------
// Simple LCG random number generator (no std dependency for wasm)
// ---------------------------------------------------------------------------
struct Rng(u64);
impl Rng {
    fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as f32) / (u32::MAX as f32)
    }
}

// ---------------------------------------------------------------------------
// PetriNet struct exposed to JavaScript
// ---------------------------------------------------------------------------
#[wasm_bindgen]
pub struct PetriNet {
    tokens: Vec<u32>,
    place_x: Vec<f32>,
    place_y: Vec<f32>,
    trans_x: Vec<f32>,
    trans_y: Vec<f32>,
    /// Which transitions fired on the last step (0 = dormant, 1 = fired)
    fired: Vec<u8>,
    /// Token flow animation: for each transition, fractional progress 0.0..1.0
    anim_progress: Vec<f32>,
    /// Cumulative count of how many times each transition has fired
    fire_count: Vec<u32>,
    rng: Rng,
    /// Orders currently injected and awaiting processing
    pending_orders: u32,
    /// Total successfully delivered orders
    delivered_total: u32,
    step_count: u32,
}

#[wasm_bindgen]
impl PetriNet {
    // -----------------------------------------------------------------------
    // Constructor
    // -----------------------------------------------------------------------
    pub fn new() -> Self {
        let mut tokens = vec![0u32; NUM_PLACES];
        // Bootstrap: one order already received
        tokens[0] = 1;

        PetriNet {
            tokens,
            place_x: PLACE_X.to_vec(),
            place_y: PLACE_Y.to_vec(),
            trans_x: TRANS_X.to_vec(),
            trans_y: TRANS_Y.to_vec(),
            fired: vec![0u8; NUM_TRANSITIONS],
            anim_progress: vec![0.0f32; NUM_TRANSITIONS],
            fire_count: vec![0u32; NUM_TRANSITIONS],
            rng: Rng(42),
            pending_orders: 0,
            delivered_total: 0,
            step_count: 0,
        }
    }

    // -----------------------------------------------------------------------
    // Inject a new order into the net
    // -----------------------------------------------------------------------
    pub fn inject_order(&mut self) {
        self.tokens[0] += 1;
        self.pending_orders += 1;
    }

    // -----------------------------------------------------------------------
    // Single simulation step: evaluate all enabled transitions and fire them
    // -----------------------------------------------------------------------
    pub fn step(&mut self) {
        self.step_count += 1;

        // Clear last-step firing flags
        for f in self.fired.iter_mut() { *f = 0; }

        // Determine which transitions are enabled
        let mut enabled = [false; NUM_TRANSITIONS];
        for t in 0..NUM_TRANSITIONS {
            let [p0, p1] = INPUT_PLACES[t];
            let ok0 = p0 < 0 || self.tokens[p0 as usize] > 0;
            let ok1 = p1 < 0 || self.tokens[p1 as usize] > 0;
            enabled[t] = ok0 && ok1;
        }

        // Fire enabled transitions
        for t in 0..NUM_TRANSITIONS {
            if !enabled[t] { continue; }

            // Consume input tokens
            let [p0, p1] = INPUT_PLACES[t];
            if p0 >= 0 { self.tokens[p0 as usize] -= 1; }
            if p1 >= 0 { self.tokens[p1 as usize] -= 1; }

            // Produce output tokens via incidence matrix
            // Special case: T1 (Check Stock) is probabilistic
            if t == 1 {
                // 70% chance stock is available
                if self.rng.next_f32() < 0.70 {
                    self.tokens[2] += 1; // In Stock
                } else {
                    self.tokens[3] += 1; // Out of Stock
                }
            } else {
                for p in 0..NUM_PLACES {
                    let delta = INCIDENCE[p][t];
                    if delta > 0 {
                        self.tokens[p] = self.tokens[p].saturating_add(delta as u32);
                    }
                }
            }

            self.fired[t] = 1;
            self.fire_count[t] += 1;
            self.anim_progress[t] = 0.0;

            // Track deliveries
            if t == 6 { // Deliver transition
                self.delivered_total += 1;
                if self.pending_orders > 0 { self.pending_orders -= 1; }
            }
        }

        // Advance animation progress for recently-fired transitions
        for t in 0..NUM_TRANSITIONS {
            if self.anim_progress[t] < 1.0 {
                self.anim_progress[t] = (self.anim_progress[t] + 0.15).min(1.0);
            }
        }

        // Periodically inject new orders to keep the net active (every 8 steps)
        if self.step_count % 8 == 0 && self.tokens[0] < 3 {
            self.inject_order();
        }
    }

    // -----------------------------------------------------------------------
    // Memory pointer accessors (zero-copy reads from JavaScript)
    // -----------------------------------------------------------------------
    pub fn tokens_ptr(&self) -> *const u32 { self.tokens.as_ptr() }
    pub fn place_x_ptr(&self) -> *const f32 { self.place_x.as_ptr() }
    pub fn place_y_ptr(&self) -> *const f32 { self.place_y.as_ptr() }
    pub fn trans_x_ptr(&self) -> *const f32 { self.trans_x.as_ptr() }
    pub fn trans_y_ptr(&self) -> *const f32 { self.trans_y.as_ptr() }
    pub fn fired_ptr(&self) -> *const u8 { self.fired.as_ptr() }
    pub fn anim_ptr(&self) -> *const f32 { self.anim_progress.as_ptr() }
    pub fn fire_count_ptr(&self) -> *const u32 { self.fire_count.as_ptr() }

    // -----------------------------------------------------------------------
    // Scalar accessors
    // -----------------------------------------------------------------------
    pub fn num_places(&self) -> usize { NUM_PLACES }
    pub fn num_transitions(&self) -> usize { NUM_TRANSITIONS }
    pub fn delivered_total(&self) -> u32 { self.delivered_total }
    pub fn step_count(&self) -> u32 { self.step_count }
    pub fn pending_orders(&self) -> u32 { self.pending_orders }

    /// Returns the token count for a specific place
    pub fn tokens_at(&self, place: usize) -> u32 {
        self.tokens.get(place).copied().unwrap_or(0)
    }

    /// Returns 1 if transition t fired on the last step
    pub fn transition_fired(&self, t: usize) -> u8 {
        self.fired.get(t).copied().unwrap_or(0)
    }

    /// Reset the net to initial state
    pub fn reset(&mut self) {
        for t in self.tokens.iter_mut() { *t = 0; }
        self.tokens[0] = 1;
        for f in self.fired.iter_mut() { *f = 0; }
        for a in self.anim_progress.iter_mut() { *a = 0.0; }
        for c in self.fire_count.iter_mut() { *c = 0; }
        self.delivered_total = 0;
        self.pending_orders = 0;
        self.step_count = 0;
    }
}