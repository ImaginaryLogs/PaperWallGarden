Here are applications of solving for eigenproblems in computation:

# Eigenvalues
These solves for the stability and reactivity of drugs.

## Maps the Energy Landscape

![[Pasted image 20260526074930.png]]

The **eigenvalues** ($E_1, E_2, ... E_n$) tell you the energy of the molecule at various states. In drug design, this is the most critical metric for stability and reactivity.

- **Ground State Energy ($E_0$):** Tells you the **thermodynamic stability** of the drug. If the ground state is not low enough compared to a competing reaction, the drug might break down before it ever reaches its target in the body.
- **Excited States ($E_n - E_0$):** Used to predict **spectroscopy and light-absorption**. This is essential for designing photodynamic therapies or understanding how a drug might be degraded by UV light.
- **Binding Affinity ($\Delta E$):** By calculating the energy difference between "Drug in solution" vs. "Drug bound to a protein target," you determine the binding affinity. Lower energy = tighter, more effective binding.

# Eigenstates
These solves for the shape of the electron cloud that is useful for docking problems and see whether an enzyme binds or not.

## Shows the Shape of the Cloud
An **eigenstate** ($\Psi$) is the full quantum mechanical "description" of the molecule. It encodes the 3D distribution of the electron density, which dictates how the molecule physically "fits" into a target protein.

- **Molecular Docking:** A drug is essentially a "key" trying to fit into a protein "lock." The eigenstate tells us exactly where the electron clouds are dense (negative) or sparse (positive). This allows us to map **electrostatic potential surfaces**.
- **Hydrogen Bonding:** By identifying areas of high electron density in the eigenstate, we can predict exactly where a drug will form hydrogen bonds with a protein, which is the primary driver of drug-target specificity.

![[Pasted image 20260526075444.png]]


# Eigenbasis
These solve the range or domain of stability and reactivity.

## Measures the Range of Reactivity / Stability 
The **eigenbasis** (the set of configurations we use to describe the molecule) reveals how electrons are allowed to shift and rearrange - the domain of **chemical reactivity**.

- **HOMO-LUMO Gap:** This is derived directly from the basis set and eigenvalues.
    - **Small Gap:** The drug is "soft" or highly reactive; it may be prone to unwanted chemical side reactions (toxicity).
    - **Large Gap:** The drug is "hard" or chemically stable; it will likely travel through the bloodstream intact.
- **Predicting Metabolites:** If a drug has a "weak" point in its eigenbasis where an electron can be easily stripped (oxidation), chemists can predict that the liver will metabolize the drug there. They can then "patch" that spot by adding a different functional group (e.g., a Fluorine atom) to make the drug last longer in the body.

>[!example]
> ![[Pasted image 20260526075539.png]]