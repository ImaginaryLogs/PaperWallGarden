The Electronic [[Hamiltonian]] must be compiled to qubit operators before any quantum algorithms can be used.

| Encoding         | Qubit Cost   | Single-Operator Weight | Update Cost    | Best For                        |
| ---------------- | ------------ | ---------------------- | -------------- | ------------------------------- |
| Jordan-Wigner    | N qubits     | O(N) Pauli strings     | O(1) bit flips | 1D chains                       |
| Bravyi-Kitaev    | N qubits     | O(log N) Pauli strings | O(log N)       | General molecules               |
| Symmetry-adapted | N - k qubits | Varies                 | Varies         | Exploiting conserved quantities |

> **Key CS Insight:** Jordan-Wigner is like a linked list — local fermionic operators become nonlocal in qubit space (long-range dependencies). Bravyi-Kitaev is like a balanced binary tree — it trades local updates for O(log N) operator weight, reducing circuit depth for two-qubit gates.