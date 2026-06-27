![[Pasted image 20260623083429.png]]

QFT maps computational basis states to their frequency-domain representation, exactly like the classical DFT but coherently across superpositions. The circuit structure is: for each qubit, apply H then a ladder of controlled-phase rotations from lower qubits, with angles decreasing as 2π/2^k. Finish with bit-reversal SWAPs. The gate count is O(n²) vs classical FFT's O(n log n), but QFT operates on exponentially many amplitudes simultaneously.

QFT is not useful by itself — you can't read out the frequency amplitudes without destroying the superposition. Its power is as a subroutine inside QPE and Shor's.