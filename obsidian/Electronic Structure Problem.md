The Hamiltonian of a molecule consisting of K nuclei and N electrons is the following:
$$
H = -\sum_i{\frac{\nabla^2_i}{2}} - \sum_I{\frac{\nabla^2_I}{2M'_I}-\sum_{i, I}{\frac{Z_i}{|r_i-R_I|}}}+\frac{1}{2}{\sum{\frac{1}{|r_i-r_j|}+\frac{1}{2}{\sum_{I\neq J}{\frac{Z_I Z_J}{|R_I-R_j|}}}}}
$$

where $M_I$, $R_I$, and $Z_I$ denote the mass, position, and atomic number of the $I^{th}$ nucleus, and $r_i$ is the position of the $i_{th}$ electron. The first two sums in $H$ are the kinetic terms of the electrons and nuclei, respectively. The final three sums represent the Coulomb repulsion

If we break it down qualitatively, we get:
$$
H=- \sum_i\text{KE}_\text{electrons}-\sum_I\text{KE}_\text{nuclei}-\sum_{i,I}\text{PE}_\text{electrons,nuclei}-\sum_{i\neq j}\text{PE}_\text{electrons}-\sum_{I\neq J}\text{PE}_\text{nuclei}
$$

$$H=KE+PE$$

The paper narrows its focus to one specific task: finding the ground state energy of a molecule's electrons. This is called the electronic structure problem.

Why just energy? Because in chemistry, energy is everything. If you know the energy landscape of a molecule across different configurations, you can predict reaction rates, stable structures, and optical properties. Energy is the master quantity.

The paper adopts the [[Born-Oppenheimer Approximation]] - treating atomic nuclei as fixed classical point charges, since they are over 1000x heavier than electrons and move much more slowly. This simplifies purely on the electrons, described by the electronic Hamiltonian $H_e$. 


$$
H_e = -\sum_i{\frac{\nabla^2_i}{2}} - \sum_{i, I}{\frac{Z_i}{|r_i-R_I|}} +\frac{1}{2}{\sum{\frac{1}{|r_i-r_j|}}}
$$

$$
H_e=- \sum_i\text{KE}_\text{electrons}-\sum_{i,I}\text{PE}_\text{electrons,nuclei}-\sum_{i\neq j}\text{PE}_\text{electrons}
$$

As a computer scientist, it is the main job to find the lowest eigenvalue (ground energy state) and the corresponding eigenvector (ground state wavefunction).

**Chemical accuracy**  ere is defined as error less than 1.6 × 10⁻³ Hartree (~0.04 eV). This is the threshold where predicted reaction rates are meaningful. Everything in this paper is oriented toward hitting that target.
