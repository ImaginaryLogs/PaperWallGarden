As circuits get deeper and wider, gradients exponentially vanish. The landscape becomes flat, and optimization fails. This is a major open problem in variational QC.

![[Pasted image 20260623090305.png]]

**Problem-inspired ansätze** like UCCSD avoid barren plateaus by restricting the circuit to physically motivated unitaries — the landscape is not random, so it retains structure. This is why quantum chemists care deeply about ansatz design.

**Layer-by-layer training** initializes and trains one layer at a time rather than all parameters simultaneously. Each layer sees a lower-dimensional optimization that still has visible gradients.

**Identity initialization** starts all parameters at zero (or small random values near zero), which keeps the initial circuit close to the identity. Near-identity circuits don't suffer from barren plateaus because the state hasn't been randomized yet. This is counterintuitive but empirically effective