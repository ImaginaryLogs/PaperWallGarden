![[Pasted image 20260623090045.png]]

This is the foundation everything else rests on, so it comes first.

Your quantum circuit prepares a state |ψ(θ)⟩. You want to know the energy ⟨ψ(θ)|H|ψ(θ)⟩. But H is a sum of Pauli strings from the Jordan-Wigner mapping — something like:

**H = 0.5·ZZ + 0.3·XI − 0.1·YY + ...** (potentially hundreds or thousands of terms)

You cannot measure H directly. Instead, for each Pauli string P_i, you rotate the circuit into the right basis and measure. Each shot gives a bitstring. Each qubit in the bitstring is either +1 or −1 after the measurement. The product of those values gives you a single sample of ⟨P_i⟩. You average across many shots to estimate the expectation value, then accumulate c_i·⟨P_i⟩ across all terms to get ⟨H⟩.