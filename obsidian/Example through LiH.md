# Context
One of the biggest hurdles in green energy is storing hydrogen fuel. Compressing hydrogen gas requires heavy, high-pressure tanks. A safer alternative is **solid-state storage**: binding hydrogen to a light metal to create a solid crystal, like Lithium Hydride ($LiH$). $LiH$ holds a massive amount of hydrogen by weight.

**The Engineering Challenge:** To get the hydrogen gas _out_ of the solid $LiH$ battery so it can be burned or used in a fuel cell, you have to apply heat to break the $Li-H$ bond. If the bond is too strong, you waste more energy heating the battery than you get out of the fuel.

# The Computational Chemistry Problem

To find the BDE, we must use the Variational Principle ($E_0 = \min \bra{\psi}\hat H\ket{\psi}$) to calculate the energy of $LiH$ at two different physical states:
1. **The Ground State:** Lithium and Hydrogen happily bonded together.
2. **The Excited/Broken State:** The electrons have been forced into higher-energy antibonding orbitals, snapping the bond.
    

**The Active Space Setup:**
Lithium has 3 electrons, and Hydrogen has 1. To make this computationally perfectly efficient ($N=4$), we use the **Frozen Core Approximation**:
- We freeze the two core electrons of Lithium ($1s^2$) because they sit close to the nucleus and don't participate in the bond.
- We only simulate the **2 valence electrons** interacting between the Lithium $2s$ orbital and the Hydrogen $1s$ orbital.
- Accounting for spin (up/down), we have exactly **4 spin-orbitals** and **2 electrons**.


# Transition Mapping

To calculate the energy required to break the bond, the quantum computer must simulate a chemical excitation ($a^\dagger_{LUMO} a_{HOMO}$), where an electron is moved from the stable bonding orbital to the unstable antibonding orbital.

AKA, we start with the chemical action: a single electron hops from spin-orbital 1 to spin-orbital 2. In chemistry math, this is the excitation operator:

$$E = a_2^\dagger a_1$$

The quantum computer does not understand $a^\dagger$ or $a$ because qubits are not electrons; they don't naturally obey fermionic antisymmetry (the Pauli exclusion principle). We must translate this into the only language the hardware speaks: **Pauli Spin Matrices ($X, Y, Z$) and the Identity matrix ($I$).**

# Encoding to Qubits
# JW
JW translates creation/annihilation into $X$ and $Y$ operators (to flip the qubit state) and strings of $Z$ operators (to track parity across the chain).

- $a_1 = \frac{1}{2}(X_1 + iY_1)$
- $a_2^\dagger = \frac{1}{2}(X_2 - iY_2) \otimes Z_1$
When we multiply them together ($a_2^\dagger a_1$), we get a polynomial. The hardware must execute every term in that polynomial. One of the dominant terms that emerges is the Pauli string:
$$P_{JW} = X_2 Z_1 X_1$$

# BK
BK translates using Update ($U$), Parity ($P$), and Flip ($F$) sets based on a binary tree structure. Depending on the exact tree indexing, $a_2^\dagger a_1$ often resolves into a simpler string because the parity is pre-stored in tree nodes rather than stretched across the whole chain.

```
       Q3 (Global Parity of 0, 1, 2, 3)
        /
       Q1 (Block Parity of 0, 1)
      /  \
     Q0  Q2 (Local Occupations)
```

In this layout, the 4 qubits store the following data combinations:
- $Q_0$ stores the local occupation of orbital 0 ($n_0$)
- $Q_1$ stores the collective block parity of orbitals 0 and 1 ($n_0 + n_1$)
- $Q_2$ stores the local occupation of orbital 2 ($n_2$)
- $Q_3$ stores the global parity of all orbitals ($n_0 + n_1 + n_2 + n_3$)

A corresponding dominant term for this hop in BK might look like:

$$P_{BK} = X_2 X_1$$

# Pauli Gadget Compiling
**The Hardware Problem:** You cannot just "run" a Pauli string. To simulate chemistry on hardware, we use the Variational Quantum Eigensolver (VQE), which requires us to turn these Pauli strings into **Unitary Time-Evolution Operators** using an exponential function: $U = e^{-i \theta P}$.

To physically wire this exponential onto a quantum chip, compilers use a standard architecture called a **Pauli Gadget**.

### Building the JW Circuit
In this sequence, we represent the qubits as **Q1**, **Q2**, and **Q3**. The string $X_2 Z_1 X_1$ requires **Q1** to be an $X$ operator, **Q2** to be a $Z$ operator, and **Q3** (representing our second orbital) to be an $X$ operator.

```
Q1 (Basis Change/Parity):  --H---●------------------●------H--
                                 |                  |
Q2 (Z-operator/Parity):    ------X-----●------------X---------
                                       |
Q3 (Basis Change/Target):  --H---------X--[Rz]--X----------H--
```
### Breakdown of the Steps:
1. **The Basis Change ($H$ gates):** * **Q1** and **Q3** receive a Hadamard (**H**) gate. This rotates their state from the native $Z$-basis into the $X$-basis, which is required because our operator is $X_2 X_1$.
    - **Q2** needs no $H$ gate because it is a $Z$-operator, and the hardware natively computes in $Z$.
2. **The Parity Staircase (The CNOTs):**
    - This is the "Jordan-Wigner Tax." Even though Q2 is technically just a "spacer" (the $Z_1$ in the middle), the algorithm must entangle it to keep the fermionic parity consistent.
    - The **CNOTs** (represented by the **●** and **X**) act as a funnel. They take the information from Q1 and Q3 and pass it through Q2. This ensures that if the state of the intermediate qubit changes, the phase of the target qubit updates correctly.
3. **The Phase Rotation ($Rz$):**
    - The **[Rz]** gate is the "chemistry" step. This applies the rotation angle $\theta$ (derived from our variational optimization). This physically imprints the energy difference between the current state and the desired chemical state.
4. **The Uncompute (The Reverse):**
    - The gates after the **[Rz]** are a mirror image of the gates before it. This is crucial: it "cleans up" the entanglement and the basis change so that the qubits return to a state that can be measured accurately by the hardware.

### Building the BK Circuit on Hardware ($P_{BK} = X_2 X_1$)
Because the Bravyi-Kitaev math bypassed the $Z_1$ parity check, the Pauli string is just $X_2 X_1$. The hardware circuit shrinks drastically:

```.txt
Qubit 1: --H----●----------●----H--
                |          |
Qubit 2: --H----X--[Rz 2θ]-X----H--
```

There is no middle parity qubit to check. The hardware executes this in less than half the time, meaning the qubits are less likely to lose their quantum coherence (decohere) to background thermal noise.