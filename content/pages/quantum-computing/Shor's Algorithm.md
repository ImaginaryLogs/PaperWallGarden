![[Pasted image 20260623083624.png]]

Shor's is essentially QPE applied to the modular exponentiation unitary. The top register encodes a superposition over all possible exponents simultaneously; the modular exponentiation entangles it with the bottom register; the inverse QFT then extracts the period from the resulting interference pattern. The classical post-processing (continued fractions algorithm) takes the measured bitstring and recovers the period r, from which you get the factors via GCD.

The **modular exponentiation** circuit is the bottleneck — it requires O((log N)³) gates and is why Shor's is fault-tolerant territory, not NISQ.