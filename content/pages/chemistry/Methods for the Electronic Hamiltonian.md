These are methods used to solve an equation of the Electronic [[Hamiltonian]], i.e. the Born-Oppenheimer Approximation, of a multi-electron or molecule. [[Solving Eigenstates]] here tells us how the electrons are positioned.

There are two widely-known methods:
- **Hartree-Fock Approximation** - get orbitals available for electrons to occupy.
- **Full Configuration Interaction**- get the preferred orbital arrangements.

### The One-Electron Layer (The Hartree-Fock / Mean-Field Level)
When we look for the eigenstates of a simplified, one-electron Hamiltonian (where each electron feels the average repulsion of all others), the resulting eigenvectors **are the molecular orbitals themselves**.

- **The Eigenvalues** correspond to the **orbital energies** (e.g., $1s, 2s, 2p$, or HOMO and LUMO).    
- **The Eigenvectors** give you the **coefficients** that mix your starting atomic basis functions together to shape those molecular orbitals.

By sorting the eigenvalues from lowest to highest energy, you get the exact arrangement and layout of the orbital "buckets" available for electrons to occupy.

### The Many-Electron Layer (Full Configuration Interaction)
When you try to solve the _true_ electronic Hamiltonian (including exact electron-electron repulsions), a single orbital configuration is no longer enough.

- The **Eigenbasis** becomes a massive set of **Slater Determinants** (every possible way to distribute your electrons into the available orbitals).
- The **True Eigenstate** of the molecule is a linear combination of these configurations.
    
By looking at the coefficients of this true eigenstate, you can see which orbital arrangements the molecule actually prefers. For instance, a stable molecule's ground-state eigenvector will be heavily dominated by the configuration where electrons smoothly occupy the lowest-energy orbitals.