---
date: 2026-07-13 21:41
tags:
aliases:
---
# Financial Instruments Asset
All Financial instruments have Open, High, Low, Close, Volume, and Adjusted 
**Assets**: "UAL",  "PAL", "BTC-USD",  "SPY",  "VIX", 
**Fred Tickers**: "DGS10", "FEDFUNDS", "CPIAUCSL"

## 

# Transformation
These are transformations Needed
## Log Adjusted Returns
It used to normalize returns and are calculated as the natural logarithm of the ratio of the current price to the previous price..
$$
r=ln(\frac{P_t}{P_{t-1}})
$$
## Simple Returns
It measure the gain or loss generated on an investment relative to the amount of money invested.
$$
r_i=\frac{P_t}{P_{t-1}}-1
$$
# Analysis
# Distractions / Interested to Implement:
- Modern Laws behind Finance: https://www.youtube.com/watch?v=AKIkhuR7HYY


## Black book of Financial Analysis 
Link: https://www.youtube.com/watch?v=cvL_uWtMTA

Any quant should answer three questions:
- **Structure**: What is the structure?
- **Dynamics**: How does evolve?
- **Inference**: What can be inferred from the evolution?

A geometric program for blackboard quants. This is a lecture on markets as geometry. The state lives on a manifold $M$; prices, books, and factors are charts. Dynamics is a flow on $M$ (or its jet bundle). Inference is a map out of the compressed geometry. Nothing in what follows a particular asset class: the same object describe rates, equities, and portfolios once the chart $\pi$ is fixed.

Most amateur quants or standard machine learning models start directly with prediction (Inference). The speaker argues this is backwards.  We must first mathematically define what the system is even _allowed_ to do without causing logical contradictions.
### Structure
You must first mathematically define what the system is even _allowed_ to do without causing logical contradictions.
- **The Problem with "Price" without Evolution**: Two assets might both be priced at $100, but one has massive buy-order momentum, and the other is suffering a liquidity crunch. If your program treats them as identical states, your system fails.
- **Enriched States**: You must "lift" the price into an enriched state. In differential geometry, this is called a _Jet Bundle_; in CS, its just state tracking. Your object must tracks the position (price), first derivative (velocity/momentum), and second derivative (acceleration/order-book depth).
- **Topological Compression**: Raw market data as an incredibly noisy, high-dimensional graph. Use **Persistent Homology** (which relies heavily on graph theory and simplicial complexes) to find the "holes" or invariants in the data. This acts like a lossless compression algorithm, mapping a chaotic input stream into a clean, lower-dimensional **Latent State Machine** ($Z$) where each state represents a distinct market "regime" (e.g., _High Volatility Panic_ or _Quiet Accumulation_).

### Dynamics
Once you have defined your compressed state space ($Z$), you need to map how states transition over time. Instead of looking at discrete "candlestick charts," time-series data is treated as a continuous **flow on the manifold**. _Symplectic_ and _Contact_ geometry

Transition Evolution: **Conservative vs. Dissipative Forces.** The physics-grade modeling splits market movement into two primary parts:
    1. **Symplectic/Hamiltonian systems (Conservative)**: The parts of the market that conserve "volume" or phase space (e.g., core supply/demand mechanics)
    2. **Contact Geometry (Dissipation)**: The parts where energy/information bleeds out due to market inefficiencies, liquidations, or entropy.

The market is full of noise. If your data has underlying symmetries, you can use **Orbit Theory** to group equivalent states together.

### Inference
Because the **Structure** step perfectly compressed the data and the **Dynamics** step mapped the allowable trajectories, making a prediction becomes the easiest part of the pipeline.

- **Admissible ML Architectures:** Instead of feeding raw, noisy data into a massive deep neural network and praying for orthogonality, you pass the highly compressed, geometrically sound latent state ($Z$)
- - **Singular Learning Theory:** In advanced ML, instead of just counting parameters or blindly adding dropout layers to prevent overfitting, you look at the **algebraic geometry of the loss landscape**. By analyzing the _Log Canonical Threshold_ ($\lambda$), you can evaluate a model's generalization capacity based entirely on the geometry of its state transitions. A geometrically sound state machine will always require fewer parameters and less compute than a brute-force deep learning model.
- **Prediction as Graph Traversal:** Making a forecast is no longer a guessing game. It is simply **integrating a path along a geodesic**—which, in CS terms, is equivalent to finding the _Shortest Path_ or _Optimal Trajectory_ across your curved latent state graph ($Z$) from your current node to a future node.



# Theoretical Lenses Comparison
### Neoclassical Paradigm
Classic financial concepts that treats the financial system as a linear, rational, mechanical engine that naturally tends toward a stable equilibrium.

### Complex Adaptive System Paradigm
It views markets not as sterile machines, but as biological ecosystems where feedback loops, friction, and shifting participant behavior constantly warp the environment out of equilibrium.