## Main Equation

For a hydrogen atom, the Hamiltonian ($H$) consists of the kinetic energy of the single electron and its electrostatic attraction to the single proton in the nucleus:

$$H = -\frac{\hbar^2}{2m_e}\nabla^2 - \frac{e^2}{4\pi\epsilon_0 r}$$

When you solve the eigenvalue problem $H\psi = E\psi$ using spherical coordinates $(r, \theta, \phi)$, the wave function splits into two parts: a **radial part** (how far the electron is from the nucleus) and an **angular part** (the 3D shape).

$$\psi_{n,\ell,m}(r, \theta, \phi) = R_{n,\ell}(r) Y_{\ell}^{m}(\theta, \phi)$$
# Problem with Analytical 
This analytical solution yields an _infinite_ number of valid discrete eigenstates (orbitals), characterized by three quantum numbers:
- $n$ (Principal: energy level) $\rightarrow 1, 2, 3, \dots$
- $\ell$ (Angular: shape, $s, p, d, f$) $\rightarrow 0, \dots, n-1$
- $m$ (Magnetic: orientation) $\rightarrow -\ell, \dots, +\ell$
    
Because the electron can also have a spin of up ($+1/2$) or down ($-1/2$), each spatial orbital splits into **2 spin-orbitals**.

# Setting up the Computational Problem

Even though hydrogen has an infinite number of theoretical orbitals, electrons in the real world mostly hang out in the lowest energy states. To solve this on a computer, we truncate the infinite choices down to a finite subset of $N$ spin-orbitals [[Chemistry Basis Set]].

Let's pick a standard, high-quality chemistry basis set called **cc-pVDZ** for Hydrogen.
- This basis set assigns **5 spatial orbitals** to a single Hydrogen atom.
- Since each spatial orbital holds 2 spins, this gives us exactly **$N = 10$ spin-orbitals**.

|             | H-He                          | Li-Ne                             | Na-Ar                             |
| ----------- | ----------------------------- | --------------------------------- | --------------------------------- |
| cc-pVDZ     | [2_s_1_p_] → 5 func.          | [3_s_2_p_1_d_] → 14 func.         | [4_s_3_p_1_d_] → 18 func.         |
| cc-pVTZ     | [3_s_2_p_1_d_] → 14 func.     | [4_s_3_p_2_d_1_f_] → 30 func.     | [5_s_4_p_2_d_1_f_] → 34 func.     |
| cc-pVQZ     | [4_s_3_p_2_d_1_f_] → 30 func. | [5_s_4_p_3_d_2_f_1_g_] → 55 func. | [6_s_5_p_3_d_2_f_1_g_] → 59 func. |
| aug-cc-pVDZ | [3_s_2_p_] → 9 func.          | [4_s_3_p_2_d_] → 23 func.         | [5_s_4_p_2_d_] → 27 func.         |
| aug-cc-pVTZ | [4_s_3_p_2_d_] → 23 func.     | [5_s_4_p_3_d_2_f_] → 46 func.     | [6_s_5_p_3_d_2_f_] → 50 func.     |
| aug-cc-pVQZ | [5_s_4_p_3_d_2_f_] → 46 func. | [6_s_5_p_4_d_3_f_2_g_] → 80 func. | [7_s_6_p_4_d_3_f_2_g_] → 84 func. |
Source: https://en.wikipedia.org/wiki/Basis_set_(chemistry)

## 3. How Many Bits (Classical Computer)?

To solve for the ground state on a classical computer exactly (Full Configuration Interaction), we need to map out every single way to arrange hydrogen's **1 electron** into these **10 available spin-orbitals**.

The number of configurations is given by the binomial coefficient

$$\binom{N_{orbitals}}{n_{electrons}}=\binom{10}{1} = 10 \text{ possible states}$$

Because 10 is an incredibly small number, the classical memory requirement is trivial.

- To store the quantum state vector, we just need to store 10 complex numbers.
- Assuming standard 64-bit complex numbers (Two Single Precision Floating Points), you need:

$$\text{Memory} = 10 \times 64 \text{ bits} = 640 \text{ bits} \quad (\sim 80 \text{ bytes})$$

_Even if you pushed the simulation to an incredibly massive basis set of $N = 100$ spin-orbitals, $\binom{100}{1} = 100$ states, requiring just a few kilobytes of classical memory._ Because there is only one electron, there is zero electron-electron correlation, meaning classical computers never face an exponential explosion for hydrogen.

## 4. How Many Qubits (Quantum Computer)?
On a quantum computer, we map the electronic structure problem differently using a standard mapping technique like **[[Jordan-Wigner]]**.

Under this mapping, **every single spin-orbital gets assigned its own dedicated qubit**.
- If a qubit is in state $|1\rangle$, that specific spin-orbital is occupied by an electron.
- If a qubit is in state $|0\rangle$, that spin-orbital is empty.

Using our exact same high-quality cc-pVDZ basis set ($N = 10$):
$$\text{Required Qubits} = N = 10 \text{ qubits}$$
The entire quantum state of the hydrogen atom would be represented by a 10-qubit wavefunction that looks like a superposition of states:

$$|\Psi\rangle = c_1|1000000000\rangle + c_2|0100000000\rangle + \dots + c_{10}|0000000001\rangle$$

For [[Bravyi-Kitaev]], it stores more in detail as its store approximately $log_2(N)=5$ in a binary tree