A Trotter step is one of those tiny slices of time. Instead of trying to simulate the entire complex system for the whole duration $t$ all at once, you divide the total time into $r$ small steps (where $\Delta t = t/r$).

For a single Trotter step, the approximation looks like this:

$$e^{-i(H_1 + H_2)\Delta t} \approx e^{-iH_1\Delta t} \cdot e^{-iH_2\Delta t}$$

To simulate the whole time $t$, you repeat this single Trotter step $r$ times in a row.