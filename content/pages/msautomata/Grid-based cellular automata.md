
These are the simplest extension from your FSA background. Think of a 2D grid where every cell _is_ an FSA — it reads its own state and its neighbors' states, then transitions. Conway's Game of Life is the classic example. Each cell has states (dead/alive), and a local transition rule fires simultaneously across the whole grid. It's a massively parallel FSA. 

You get emergent global behavior (gliders, oscillators) from purely local rules — something a single FSA can't express.