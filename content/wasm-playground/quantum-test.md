---
title: Quantum State Simulator Test
wasm-module: quantum_sim
---

# Testing My Quantum Workspace Component
This page is automatically intercepted by Quartz. The custom Preact component 
`WasmPlaygroundBody` handles compiling and running your code underneath this text!
## 2-Qubit Entanglement Sandbox (Bell State Creator)

<div style="padding: 1.5rem; background: #161618; border: 1px solid #393639; border-radius: 8px; color: #ebebec;">
  <h3>State Probabilities</h3>
  <div style="margin-bottom: 1rem;">
    <div style="display: flex; align-items: center; margin: 0.5rem 0;">
      <span style="width: 40px; font-family: monospace;">|00⟩:</span>
      <div style="flex-grow: 1; background: #2b2b2b; height: 20px; border-radius: 4px; overflow: hidden;">
        <div id="bar-00" style="background: #7b97aa; width: 100%; height: 100%; transition: width 0.3s ease;"></div>
      </div>
      <span id="val-00" style="width: 60px; text-align: right; font-family: monospace; marginLeft: 0.5rem;">100%</span>
    </div>
    <div style="display: flex; align-items: center; margin: 0.5rem 0;">
      <span style="width: 40px; font-family: monospace;">|01⟩:</span>
      <div style="flex-grow: 1; background: #2b2b2b; height: 20px; border-radius: 4px; overflow: hidden;">
        <div id="bar-01" style="background: #7b97aa; width: 0%; height: 100%; transition: width 0.3s ease;"></div>
      </div>
      <span id="val-01" style="width: 60px; text-align: right; font-family: monospace; marginLeft: 0.5rem;">0%</span>
    </div>
    <div style="display: flex; align-items: center; margin: 0.5rem 0;">
      <span style="width: 40px; font-family: monospace;">|10⟩:</span>
      <div style="flex-grow: 1; background: #2b2b2b; height: 20px; border-radius: 4px; overflow: hidden;">
        <div id="bar-10" style="background: #7b97aa; width: 0%; height: 100%; transition: width 0.3s ease;"></div>
      </div>
      <span id="val-10" style="width: 60px; text-align: right; font-family: monospace; marginLeft: 0.5rem;">0%</span>
    </div>
    <div style="display: flex; align-items: center; margin: 0.5rem 0;">
      <span style="width: 40px; font-family: monospace;">|11⟩:</span>
      <div style="flex-grow: 1; background: #2b2b2b; height: 20px; border-radius: 4px; overflow: hidden;">
        <div id="bar-11" style="background: #7b97aa; width: 0%; height: 100%; transition: width 0.3s ease;"></div>
      </div>
      <span id="val-11" style="width: 60px; text-align: right; font-family: monospace; marginLeft: 0.5rem;">0%</span>
    </div>
  </div>

  <div style="display: flex; gap: 0.5rem;">
    <button id="btn-h0" style="background: #284b63; color: white; border: none; padding: 0.5rem 1rem; border-radius: 4px; cursor: pointer;">
      Apply H on Q0
    </button>
    <button id="btn-cnot" style="background: #84a59d; color: white; border: none; padding: 0.5rem 1rem; border-radius: 4px; cursor: pointer;">
      Apply CNOT (Q0→Q1)
    </button>
    <button id="btn-clear" style="background: transparent; border: 1px solid #646464; color: #ebebec; padding: 0.5rem 1rem; border-radius: 4px; cursor: pointer;">
      Reset System
    </button>
  </div>
</div>

<script type="module">
  const WASM_JS = "/PaperWallGarden/wasm/quantum_sim.js";
  const WASM_BG = "/PaperWallGarden/wasm/quantum_sim_bg.wasm";

  async function run() {
    try {
        console.log("Loading WASM module from:", WASM_JS);
        
        // 2. Dynamic import() returns a Promise that resolves to the module
        const module = await import(WASM_JS);
        
        // 3. Extract the default export (the init function) and your class
        const init = module.default;
        const { MultiQubitState } = module;

        // 4. Initialize the WASM binary
        await init(WASM_BG);
        
        // 5. Initialize your system
        let system = new MultiQubitState();

        function updateUI() {
            const probs = system.get_probabilities()
            const states = ["00", "01", "10", "11"]
            
            states.forEach((state, idx) => {
                const percentage = (probs[idx] * 100).toFixed(1)
                document.getElementById(`bar-${state}`).style.width = `${percentage}%`
                document.getElementById(`val-${state}`).innerText = `${percentage}%`
            })
            console.log("Probabilities: ", probs)
        }

        document.getElementById("btn-h0").addEventListener("click", () => {
            system.hadamard_q0();
            updateUI();
        });

        document.getElementById("btn-cnot").addEventListener("click", () => {
            system.cnot();
            updateUI();
        });

        document.getElementById("btn-clear").addEventListener("click", () => {
            system.free();
            system = new MultiQubitState();
            updateUI();
        });

        updateUI();
        console.log("Quantum system initialized successfully.");
    } catch (e) {
        console.error("WASM Load Failed. Check network path:", e);
    }
  }

  run()
</script>