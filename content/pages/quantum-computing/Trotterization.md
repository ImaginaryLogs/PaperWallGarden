To understand Trotterization, we have to look at a major challenge in quantum physics: **simulating how a quantum system changes over time.**

According to the Schrödinger equation, a system evolves over time based on its total energy, described by an operator called the **Hamiltonian ($H$)**. If a system has multiple interacting parts, its Hamiltonian is a sum of different pieces:

$$H = H_1 + H_2 + H_3 + ...$$

To simulate this perfectly, we need to calculate $e^{-iHt}$. However, if the different pieces of the Hamiltonian ($H_1$ and $H_2$) do not _commute_ (meaning the order in which you apply them matters, $H_1 H_2 \neq H_2 H_1$), you cannot just split the exponential like normal math:

$$e^{-i(H_1 + H_2)t} \neq e^{-iH_1t} \cdot e^{-iH_2t}$$

**Trotterization** (named after mathematician Hale Trotter) is the clever workaround. It is a method used to approximate the evolution of a complex quantum system by breaking it down into a sequence of simpler, separate operations that a quantum computer _can_ actually execute.

By using the **Lie-Trotter product formula**, we can say that if we break the time $t$ into a massive number of incredibly tiny slices, the error from splitting them up becomes negligible.