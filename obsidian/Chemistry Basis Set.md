In [[Quantum Chemistry]], a **basis set** is a collection of mathematical functions used to construct a molecule’s orbitals.

Think of a basis set as a **box of digital legos** or a **palette of shapes** for algorithms to use to solve quantum chemistry problems.

A computer cannot naturally guess the infinitely complex shape of an electron cloud, so  chemists provide it with a set of pre-defined atomic shapes approximation (usually Gaussian functions centered on each nucleus). The computer's job is then to adjust the size and mixing coefficients of these shapes to recreate the true molecular orbitals.

Why care about what basis sets to use?
- If you use too few shapes (a small basis set), your description of the electron cloud will be rigid and inaccurate. 
- If you use too many shapes (a large basis set), your calculation will be incredibly accurate, but the computational cost will skyrocket.

# Basis Set Hierarchy 
Basis sets are categorized by how many functions they assign to each electron shell:
1. [[Minimal Basis Sets]] - bare minimum for neural atoms, used for massive molecules
2. [[Split-Valence Basis Sets]] - lets valence electrons expand and contract in size
3. [[Polarization and Diffuse Functions Sets]] - allows interatom distortion and intramolecular charge to change its electrons
4. [[Correlation-Consistent Sets]] - advanced and used for electron correlation