![[Pasted image 20260623083508.png]]

**The oracle** is the problem-specific part. It implements a phase kickback: if the input state is the marked element |ω⟩, it flips the phase to −|ω⟩; all others are untouched. At the circuit level this is a multi-controlled-Z or a multi-controlled phase gate, with the ancilla in state H|1⟩ = |−⟩ absorbing the phase. Building the oracle requires you to know what you're searching for — for database search it's a comparator circuit, for SAT problems it's a reversible circuit evaluating the formula.

**The diffuser** is always the same regardless of the problem: H⊗ⁿ, then a phase flip on all states except |0...0⟩ (implemented as X⊗ⁿ, multi-controlled-Z, X⊗ⁿ), then H⊗ⁿ again. This reflects the state around the uniform superposition |s⟩.

**Quadratic speedup**: classical search over N items is O(N). Grover's is O(√N). This is provably optimal for unstructured search on a quantum computer.