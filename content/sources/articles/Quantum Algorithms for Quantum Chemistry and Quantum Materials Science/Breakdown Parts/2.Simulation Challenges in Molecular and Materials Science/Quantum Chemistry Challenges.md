---
date: 2026-07-11 03:28
tags:
aliases:
---
## Quantum Chemistry
Quantum Chemistry concerns with Eigenanalysis of the electronic Hamiltonian to determine low-lying eigenstates (ground state and unique state). It uses the [[Born-Oppenheimer Approximation]]. 

> Determining the main features of the resulting potential energy surface, i.e., the electronic energy as a function of nuclear positions, its minima and saddle points, is key to understanding chemical reactivity, product distributions, and reaction rates.

Most methods in quantum chemistry are most accurate for problem where there is a dominant electronic configuration aka the [[single reference problem]]. These are found in many simple boring molecules like hydrocarbons.

However, most molecules are in excited states, in stretched bond geometries, and in transition metal chemistry have multiple valid states aka [[Multireference Chemistry|multireference quantum chemistry]]. 

> ![[Pasted image 20260704021644.png|center]]

### Quantum Chemistry Examples
Most of these involve multi-reference and strongly-correlated problems commonly seen because of the unique nature of metal. Additionally, there is also the challenge of simulating the environment.

#### Chemistry of Enzyme Active Sites
Such active sites can involve multiple coupled transition metals, famous examples being the four manganese ions in the oxygen evolving complex or the eight transition metals in the iron−sulfur clusters of nitrogenase.

They present the hardest multi-reference quantum chemistry problems in the biological world; thus, combined theoretical and experimental studies in the level of density functional theory, have proven successful at unravelling structural and electronic features of these enzyme active sites.

> However, more detailed understanding in its innerworkings, such as spin-coupling and delocalization, between metals is still at a insufficient level. 

#### Transition metal nanocatalysts and surface catalysts
Synthetic heterogenous catalysts remain a major challenge. 

> While density functional theory has been widely employed, predictions of even basic quantities such as the adsorption energy of small molecules are unreliable. Even the single-reference modeling of such chemistry, at a level significantly beyond density functional theory, is currently challenging or impossible

Multireference effects are expected to play a role in certain catalysts, such as transition metal oxides, or at intermediate geometries in reaction pathways. 

#### Light harvesting and the vision process
Although, it seems straightforward to investigate how light interacts with light-harvesting complex; 

>The quantum chemical questions revolve around the potential energy surfaces of the ground and excited states, and the influence of the environment on the spectrum

These questions are currently challenging due to the size of the systems involved as well as the varying degree of single- and multireference character in many of the conjugated excited states.

### Handling scales
There are many aspects of the chemical problems beyond the modeling of the electronic wave functions, for example, to treat environmental, solvent, and dynamical effects. 
We can interface less-relevant factors to classical computers through the use of QM/MM (quantum mechanics/molecular mechanics) models.
