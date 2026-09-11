---
date: 2026-07-17 02:24
tags:
aliases:
---

Here, we view how each asset moves in the economies they are in.

## Structure

### Log Returns

Here, we are using Fama's (1970) Efficient Market Hypothesis (EMH) which states that prices fully reflect all available information [2]. Because information arrives as unpredictable, exogenous shocks, price updates are independent and identically distributed ($i.i.d.$). This reduces the asset's structural state to a trivial, memoryless coordinate point on a flat Euclidean space.

We work in log returns, $r_t = \ln(P_t / P_{t-1})$, rather than simple returns, and this choice is doing more work than it looks like. EMH treats each day's price update as one independent "packet" of new information, so the total return over any horizon should just be the sum of the daily packets that arrived along the way. Log returns are exactly the quantity for which that's literally true: $\ln(P_T/P_0) = \sum_{t=1}^{T} r_t$, so a $T$-day cumulative return decomposes cleanly into $T$ additive daily innovations. Simple returns don't have this property - they compound multiplicatively, so "sum of daily simple returns" isn't the same object as "total simple return over the period." That additivity is not a cosmetic convenience: it is the specific mathematical property that later lets us test the square-root-of-time variance-scaling rule in the Dynamics subsection ($\sigma_T = \sigma_1\sqrt{T}$ only follows cleanly from i.i.d. returns when returns are additive across horizons). Log returns also stay well-defined and symmetric for large moves, which matters given the daily magnitude `BTC-USD` alone can realize on its worst and best days in this sample, whereas simple returns are bounded below at $-100\%$ but unbounded above, an asymmetry log returns remove [a1] (Sewell, 2012 [9], surveys this and related EMH empirics in detail).

A rolling 21-day window is what actually lets the EMH assumption of *constant* mean/variance be tested rather than assumed. If we instead computed a single mean, variance, skewness, and kurtosis over the full 2020-2025 sample, we would have no way to later ask "was the return process actually stable over time," because a single global estimate is stable *by construction* - it can't fail to be constant. The rolling window produces a local, time-varying trajectory for each moment, and the Inference subsection's decision rule (Jarque-Bera normality, Lo-MacKinlay variance ratios) is really asking whether that trajectory looks flat (consistent with EMH) or clustered into visibly different regimes seen in CAS. Twenty-one trading days is chosen because it corresponds to roughly one calendar month of trading ($\approx 252$ trading days / 12), which is short enough to catch a genuine regime shift (e.g., a market-wide shock playing out over a few weeks) while still leaving enough observations per window for the higher moments - skewness and especially kurtosis are noisy estimators on very short samples, so going much shorter than 21 days would make those two moments unreliable before they've even been used as evidence.



**Objective**: Establish, per asset, whether the local (21-day) trajectory of return mean, variance, skewness, and kurtosis is consistent with a single, time-invariant Gaussian generating process, as EMH implies - or whether it visibly clusters into distinct regimes before any formal test is run.

In short, we can compress it down these:
 1. Raw Input: Adjusted Closing Price on `UAL`, `PAL`, `BTC-USD`, and `SPY`.
 2. Enriched State: Log Return.
 3. Compression: Rolling Window of mean, variance, skewness, and kurtosis.

**Code**:
```{r price-structure-df-nce}

rolling_window_size <- 21
asset_list <- c("UAL", "PAL", "BTC", "SPY")

# 1. Dynamic Rolling Moment Generation
price_structure_df <- master_df %>%
  # Rename up front to simplify downstream dynamic column naming
  select(
    Date, 
    UAL = UAL.LogAdj, 
    PAL = PAL.LogAdj, 
    BTC = `BTC-USD.LogAdj`, 
    SPY = SPY.LogAdj
  ) %>%
  # Apply all 4 functions to all 4 assets dynamically using R 4.1+ lambdas
  mutate(
    across(
      all_of(asset_list),
      list(
        roll_mean = \(x) rollapplyr(x, rolling_window_size, mean, fill = NA),
        roll_var  = \(x) rollapplyr(x, rolling_window_size, var, fill = NA),
        roll_skew = \(x) rollapplyr(x, rolling_window_size, skewness, fill = NA),
        roll_kurt = \(x) rollapplyr(x, rolling_window_size, kurtosis, fill = NA)
      ),
      .names = "{.col}_{.fn}" # Dynamically names them e.g., UAL_roll_mean
    )
  ) %>%
  # Remove incomplete windows
  filter(!is.na(UAL_roll_mean))

print(price_structure_df)


# 2. Dynamic Plotting Loop (Bug Fixed)
price_structure_plot_list <- list()

for (asset in asset_list) {
  
  # Select Date and ONLY the columns matching the current loop's asset
  plot_df <- price_structure_df %>%
    select(Date, starts_with(paste0(asset, "_"))) %>%
    pivot_longer(
      cols = -Date, 
      names_to = "Metric", 
      values_to = "Value"
    )
  
  p_structure <- ggplot(plot_df, aes(x = Date, y = Value, color = Metric)) +
    geom_line(alpha = 0.7) +
    facet_wrap(~Metric, scales = "free_y", ncol = 1) +
    theme_minimal() +
    labs(
      title = paste(asset, "Structural State: Rolling Moments (21-Day)"),
      y = "Moment Value"
    )
  
  price_structure_plot_list[[asset]] <- ggplotly(p_structure, height=1000)
}

# Render the list of interactive plots
tagList(price_structure_plot_list)
```

**Output**:

**Financial Interpretation**:


**What we expect to see, historically.** The four moments we roll aren't arbitrary descriptive statistics - they map directly onto the EMH null hypothesis tested later: mean and variance are the two parameters a Gaussian random walk (and Black-Scholes/GBM) needs, while skewness $=0$ and excess kurtosis $=0$ are the Gaussian benchmarks the Jarque-Bera test checks against. Given the 2020-2025 window spans several distinct macro regimes, the rolling-variance panel should *not* look like a flat band if EMH's constant-variance assumption is already in trouble at the descriptive-statistics stage - we'd expect visible, regime-like plateaus rather than noise fluctuating around one level: a sharp spike across all four assets in Q1 2020 coincident with the COVID-19 shock (`SPY` fell roughly a third in about five weeks, and volatility spiked simultaneously across essentially every asset class); a `BTC-USD`-specific elevated-variance regime through 2021 tied to that period's retail-driven speculative flows; a second broad cluster through 2022 as the Fed's rate-hiking cycle repriced risk assets generally, with `BTC-USD`'s 2022 drawdown standing out as unusually severe even against that backdrop; and likely asset-idiosyncratic spikes for `UAL` and `PAL` (pandemic-era travel restrictions and reopening, fuel-cost shocks, or Philippine market-specific events) that wouldn't necessarily show up in `SPY`'s or `BTC-USD`'s windows at all. 

### Fat Tails and Finite-Difference Embeddings
 
 
This follows from Mandelbrot's work [5], which states that financial returns are non-Gaussian and follow a stable Paretian distribution where variance is infinite or non-convergent over local windows. Rather than treating the log return alone as the state, we lift it into a **finite-difference phase embedding** - the point $(x, \dot{x}, \ddot{x})$ formed by the log-price, its first difference (velocity/momentum), and its second difference (acceleration). This is, formally, a discrete 2-jet of the price path: two assets sharing an identical log return $\dot{x}$ at time $t$ can still occupy entirely different points in this embedding if their momentum is trending up versus down, i.e. if their acceleration differs.

Additionally, we use **acceleration** rather than geometric curvature for this second coordinate. True curvature ($\kappa = |\dot{x}\ddot{x} - \dot{y}\ddot{y}|/(\dot{x}^2+\dot{y}^2)^{3/2}$ in the general case) normalizes bending by the trajectory's speed, which is attractive in principle but has a practical problem here: speed in this embedding *is* the return itself, so normalizing it away divides by something that legitimately goes to zero during quiet, low-volatility periods - exactly when we still want a well-defined reading. Acceleration has no such singularity, stays interpretable as "the return on the return" (a discrete analogue of gamma/convexity in options language), and is the more standard choice in the applied nonlinear-dynamics literature (e.g., Takens-style delay embeddings, see the Dynamics footnote below). We therefore use acceleration consistently in place of curvature throughout this section.

 
**Objective**:
 1. Raw Input: Adjusted Closing Price on `UAL`, `PAL`, `BTC-USD`, and `SPY`.
 2. Enriched State: Finite-Difference Phase Embedding of Log Return, with its Velocity, and Acceleration.
 3. Compression: Rolling Window, of 21 days (Days in a month for financial trading), of its mean, variance, skewness, and kurtosis.

**Code**:

```{r price-structure-df-cas}
rolling_window_size <- 21
logadj_cols <- names(master_df)[str_detect(names(master_df), "\\.LogAdj$")]

# --- 1. FINITE-DIFFERENCE PHASE EMBEDDING GENERATION (variable names kept as jet_bundle_* for continuity with downstream chunks) ---
jet_bundle_raw <- master_df %>%
  select(Date, all_of(logadj_cols)) %>%
  rename_with(~ str_replace(.x, "\\.LogAdj$", ".Position"), all_of(logadj_cols)) %>%
  mutate(
    across(ends_with(".Position"), ~ .x - lag(.x), 
           .names = "{str_remove(.col, '\\\\.Position')}.Velocity"),
    across(ends_with(".Position"), ~ .x - 2 * lag(.x) + lag(.x, 2), 
           .names = "{str_remove(.col, '\\\\.Position')}.Acceleration")
  )


# --- 2. MULTI-DIMENSIONAL ROLLING MOMENT COMPUTATION ---
rolled_phase_space <- jet_bundle_raw %>%
  pivot_longer(
    cols = -Date,
    names_to = c("Asset", "Dimension"),
    names_sep = "\\."
  ) %>%
  group_by(Asset, Dimension) %>%
  mutate(
    mean = rollapplyr(value, rolling_window_size, mean, fill = NA),
    var  = rollapplyr(value, rolling_window_size, var, fill = NA),
    skew = rollapplyr(value, rolling_window_size, skewness, fill = NA),
    kurt = rollapplyr(value, rolling_window_size, kurtosis, fill = NA)
  ) %>%
  ungroup() %>%
  filter(!is.na(mean)) %>%
  select(-value)


# --- 3. RESHAPE & CALCULATE LEAD (NEXT POINT) ACCELERATION ---
phase_space_4d <- rolled_phase_space %>%
  pivot_wider(
    names_from = Dimension,
    values_from = c(mean, var, skew, kurt),
    names_glue = "{Dimension}_{.value}"
  ) %>%
  filter(Date >= as.Date("2019-01-01") & Date <= as.Date("2025-12-31")) %>%
  mutate(Asset = str_remove(Asset, "^`|`$")) %>%
  # Group by Asset so we don't lead values across different assets
  group_by(Asset) %>%
  mutate(
    # Shifting the future acceleration variance back to the current row
    Next_Acceleration_Var = lead(Acceleration_var, 1)
  ) %>%
  ungroup()


# --- 4. GENERATE 3D PLOTS WITH LEAD-COLORED SEGMENTS ---
all_assets_4d_plots <- list()

for (current_asset in unique(phase_space_4d$Asset)) {
  
  asset_data <- phase_space_4d %>% filter(Asset == current_asset)
  
  p_4d <- plot_ly(asset_data, 
          x = ~Date, 
          y = ~Position_mean,  
          z = ~Velocity_mean,  
          
          # Map color to the next point's acceleration variance
          color = ~Next_Acceleration_Var, 
          colors = viridis(256, option = "magma"),
          type = 'scatter3d', 
          mode = 'lines+markers',
          
          # Markers setup
          marker = list(
            size = 3, 
            opacity = 0.85
          ),
          
          # Line setup: No 'color = black' override so it maps to color scale
          line = list(
            width = 3.5, # Slightly thicker lines to show off the segment colors
            opacity = 0.9
          )
  ) %>%
    layout(
      title = paste(current_asset, "System Phase Trajectory (Colored by Next Step Accel)"),
      scene = list(
        xaxis = list(
          title = 'Date (1D)',
          type = 'date',
          range = c("2019-01-01", "2025-12-31")
        ),
        yaxis = list(title = '21d Mean Position'),
        zaxis = list(title = '21d Mean Velocity'),
        
        aspectmode = "manual",
        aspectratio = list(x = 3, y = 1, z = 1)
      ),
      margin = list(l = 0, r = 0, b = 0, t = 40)
    )
  
  all_assets_4d_plots[[current_asset]] <- p_4d
}

htmltools::tagList(all_assets_4d_plots)
```

**Output**:

**Financial Interpretation**:

## Dynamics

## Asset Evolution with Black-Scholes

Under the Neoclassical Economics paradigm, the temporal evolution of asset prices is governed by continuous, memoryless diffusion. Concretely, this framework models state transitions through Black-Scholes diffusion, where price updates are driven entirely by an exogenous, standard Brownian motion ($dW_t$). Because each price change represents a rational reaction to unpredictable information shocks, the system possesses no memory of its historical path. Consequently, the trajectory represents a purely conservative, random walk in which future price states are independent of past states.

Transitioning from individual steps to a grouped temporal context, Daníelsson, J. and Zigard, J. (2006) states that risk scales predictably over time. Specifically, portfolio risk aggregates across multiple time horizons ($T$) according to the strict square-root-of-time scaling rule: $\sigma_T = \sigma_1 \times \sqrt{T}$. This linear scaling of variance assumes a flat, non-complex system with zero feedback loops or structural memory. Under this hypothesis, daily volatility remains a sufficient and robust metric to price longer-horizon risks, as the underlying transition dynamics remain invariant across time scales.
 
```{r rolling-dynamic-plot}
price_dynamics_df <- master_df %>%
  select(Date, UAL.LogAdj, PAL.LogAdj, `BTC-USD.LogAdj`, SPY.LogAdj) %>%
  
  # Mutate to build longer-horizon compound paths (Rolling sums of daily log returns)
  mutate(
    # UAL Multi-horizon paths
    UAL_5d_return  = rollapplyr(UAL.LogAdj, 5, sum, fill = NA),
    UAL_21d_return = rollapplyr(UAL.LogAdj, 21, sum, fill = NA),
    
    # PAL Multi-horizon paths
    PAL_5d_return  = rollapplyr(PAL.LogAdj, 5, sum, fill = NA),
    PAL_21d_return = rollapplyr(PAL.LogAdj, 21, sum, fill = NA),
    
    # BTC Multi-horizon paths
    BTC_5d_return  = rollapplyr(`BTC-USD.LogAdj`, 5, sum, fill = NA),
    BTC_21d_return = rollapplyr(`BTC-USD.LogAdj`, 21, sum, fill = NA),
    
    # SPY Multi-horizon paths
    SPY_5d_return  = rollapplyr(SPY.LogAdj, 5, sum, fill = NA),
    SPY_21d_return = rollapplyr(SPY.LogAdj, 21, sum, fill = NA)
  )
# --- DYNAMICS STEP ---
# 1. Compute empirical variance for each horizon (uncorrected by T)
empirical_summary <- price_dynamics_df %>%
  summarise(
    `1-Day`  = var(PAL.LogAdj, na.rm = TRUE),
    `5-Day`  = var(PAL_5d_return, na.rm = TRUE),
    `21-Day` = var(PAL_21d_return, na.rm = TRUE)
  ) %>%
  pivot_longer(everything(), names_to = "Horizon", values_to = "Empirical_Variance")

# 2. Extract the base 1-day baseline variance (our variance unit scale sigma^2)
base_var <- empirical_summary %>% 
  filter(Horizon == "1-Day") %>% 
  pull(Empirical_Variance)

# 3. Inject the NCE Black-Scholes Linear Grouped Progression Scale
scaling_df <- empirical_summary %>%
  mutate(
    # Map Horizon strings to their integer T parameters
    T_days = case_when(
      Horizon == "1-Day"  ~ 1,
      Horizon == "5-Day"  ~ 5,
      Horizon == "21-Day" ~ 21
    ),
    # NCE Definition: Variance scales linearly with time (sigma_1^2 * T)
    NCE_Theoretical_Variance = base_var * T_days
  ) %>%
  # Pivot to long format so ggplot can create separate color layers
  pivot_longer(
    cols = c(Empirical_Variance, NCE_Theoretical_Variance), 
    names_to = "Type", 
    values_to = "VarianceValue"
  ) %>%
  mutate(
    Type = case_when(
      Type == "Empirical_Variance"         ~ "Empirical (Actual Data)",
      Type == "NCE_Theoretical_Variance"   ~ "NCE: Black-Scholes Theoretical Line"
    )
  )

# 4. Generate the Visualization highlighting the grouping context deviation
p_dynamics <- ggplot(scaling_df, aes(x = factor(T_days), y = VarianceValue, fill = Type)) +
  geom_col(position = "dodge", alpha = 0.8) +
  scale_fill_manual(values = c("steelblue", "darkred")) +
  theme_minimal() +
  labs(
    title = "PAL Dynamics: Empirical vs. Black-Scholes Linear Risk Scaling",
    subtitle = "NCE requires empirical columns to exactly match theoretical anchors",
    x = "Time Horizon (Days)",
    y = "Total Return Variance",
    fill = "Framework Layer"
  )

ggplotly(p_dynamics)
```
 
## Asset Evolution with Hurst's Exponent

Conversely, the Complex Adaptive Systems paradigm rejects the assumption of memoryless diffusion, viewing price dynamics instead as a non-linear flow through a multi-dimensional phase space. Within this geometric framework, the system's trajectories are evaluated for two distinct behaviors: a conservative regime and a dissipative regime. While a conservative regime preserves phase-space volume, mimicking standard diffusion, a dissipative regime contracts this volume onto lower-dimensional structures. This contraction serves as a geometric signature of path dependency, where feedback loops, market-maker risk-premium bleed, and sudden liquidation cascades drag the system toward endogenous attractors rather than allowing it to diffuse freely.

Furthermore, because these feedback mechanisms prevent the system from settling into a simple equilibrium, variance scaling across horizons inevitably deviates from a linear path. Rather than adhering to the square-root rule, CAS transitions follow a multi-scale power law defined by the relationship $\text{Var}(r_{t+T}) \propto T^{2H}$, where the Hurst Exponent ($H \neq 0.5$) serves as an explicit measure of temporal memory. Consequently, these transitions group into persistent regimes where high-volatility events cluster endogenously (Mandelbrot 1963). This scaling behavior proves that the system's dynamics are fundamentally multi-scale, meaning that local, short-term volatility metrics systematically fail to capture long-term, systemic risk profiles. 

**Objective**:


**Code**:
```{r price-movement-cas-dynamics}
cas_dynamics_df <- jet_bundle_raw %>%
  select(Date, PAL.Position, PAL.Velocity, PAL.Acceleration) %>%
  mutate(
    # Position multi-horizon paths
    PAL_pos_5d  = rollapplyr(PAL.Position, 5, sum, fill = NA),
    PAL_pos_21d = rollapplyr(PAL.Position, 21, sum, fill = NA),
    
    # Velocity multi-horizon paths
    PAL_vel_5d  = rollapplyr(PAL.Velocity, 5, sum, fill = NA),
    PAL_vel_21d = rollapplyr(PAL.Velocity, 21, sum, fill = NA),
    
    # Acceleration multi-horizon paths
    PAL_cur_5d  = rollapplyr(PAL.Acceleration, 5, sum, fill = NA),
    PAL_cur_21d = rollapplyr(PAL.Acceleration, 21, sum, fill = NA)
  )

# --- CONSOLIDATE AND GENERATE SCALING MATRIX ---
# Gather empirical variances across every coordinate space tier
cas_empirical <- cas_dynamics_df %>%
  summarise(
    # Position Horizons
    pos_1d  = var(PAL.Position, na.rm = TRUE),
    pos_5d  = var(PAL_pos_5d, na.rm = TRUE),
    pos_21d = var(PAL_pos_21d, na.rm = TRUE),
    
    # Velocity Horizons
    vel_1d  = var(PAL.Velocity, na.rm = TRUE),
    vel_5d  = var(PAL_vel_5d, na.rm = TRUE),
    vel_21d = var(PAL_vel_21d, na.rm = TRUE),
    
    # Acceleration Horizons
    cur_1d  = var(PAL.Acceleration, na.rm = TRUE),
    cur_5d  = var(PAL_cur_5d, na.rm = TRUE),
    cur_21d = var(PAL_cur_21d, na.rm = TRUE)
  ) %>%
  pivot_longer(everything(), names_to = "Key", values_to = "Empirical_Variance") %>%
  separate(Key, into = c("Space", "Horizon"), sep = "_")

# Map integer T scales and baseline parameters to compute NCE theoretical projections
cas_scaling_df <- cas_empirical %>%
  mutate(
    T_days = case_when(
      Horizon == "1d"  ~ 1,
      Horizon == "5d"  ~ 5,
      Horizon == "21d" ~ 21
    ),
    Space_Label = case_when(
      Space == "pos" ~ "Position Space (Returns)",
      Space == "vel" ~ "Velocity Space (Momentum)",
      Space == "cur" ~ "Acceleration Space"
    )
  ) %>%
  # Dynamically fetch the 1-Day baseline variance for each respective space tier
  group_by(Space) %>%
  mutate(
    NCE_Theoretical_Variance = Empirical_Variance[Horizon == "1d"] * T_days
  ) %>%
  ungroup() %>%
  # Reshape for multi-layered side-by-side column mapping
  pivot_longer(
    cols = c(Empirical_Variance, NCE_Theoretical_Variance),
    names_to = "Framework_Type",
    values_to = "VarianceValue"
  ) %>%
  mutate(
    Framework_Type = case_when(
      Framework_Type == "Empirical_Variance"       ~ "Empirical (Actual CAS Dynamics)",
      Framework_Type == "NCE_Theoretical_Variance" ~ "NCE Benchmark (Linear Scaling)"
    )
  )
plotly_plots <- list()

# --- PLOT CAS DYNAMICS SEPARATELY BY SPACE TIER ---
for (current_space in unique(cas_scaling_df$Space_Label)) {
  
  space_dynamics_data <- cas_scaling_df %>% filter(Space_Label == current_space)
  
  p_cas_dynamics <- ggplot(space_dynamics_data, aes(x = factor(T_days), y = VarianceValue, fill = Framework_Type)) +
    geom_col(position = "dodge", alpha = 0.8) +
    scale_fill_manual(values = c("darkblue", "darkgrey")) +
    theme_minimal() +
    labs(
      title = paste("CAS Dynamics Scaling:", current_space),
      subtitle = "Significant differences between actual and benchmark indicate memory structures",
      x = "Time Horizon (Days)",
      y = "Total Coordinate Variance",
      fill = "Model Hypothesis"
    )
  
  plotly_plots[[current_space]] <- ggplotly(p_cas_dynamics)
}
tagList(plotly_plots)
```

```{r}

cas_true_dynamics_df <- cas_scaling_df %>%
  # Filter to look at unique empirical horizons to calculate H
  filter(Framework_Type == "Empirical (Actual CAS Dynamics)") %>%
  group_by(Space_Label) %>%
  mutate(
    # Estimate 2H via a local log-linear regression slope: ln(Var) vs ln(T)
    # Since Var = C * T^(2H) -> log(Var) = log(C) + 2H*log(T)
    log_T = log(T_days),
    log_Var = log(VarianceValue),
    
    # Extract the slope (2H) using basic linear algebra syntax
    two_H = cov(log_Var, log_T) / var(log_T),
    Hurst_Est = two_H / 2
  ) %>%
  ungroup()

# 1. Isolate the base 1-Day empirical variance for each Space tier to prevent recycling errors
base_variance_map <- cas_scaling_df %>%
  filter(Framework_Type == "Empirical (Actual CAS Dynamics)", Horizon == "1d") %>%
  select(Space, Base_Var = VarianceValue)

# 2. Join the base variance map and the estimated Hurst exponents back to the scaling dataset
plotting_matrix_df <- cas_scaling_df %>%
  left_join(base_variance_map, by = "Space") %>%
  left_join(
    cas_true_dynamics_df %>% select(Space, Hurst_Est) %>% unique(),
    by = "Space"
  )

# 3. Create the distinct CAS Fractional scaling curves using your exact formula: Var_1d * T^(2H)
cas_lines_df <- plotting_matrix_df %>%
  filter(Framework_Type == "NCE Benchmark (Linear Scaling)") %>%
  mutate(
    Framework_Type = "CAS Fractional Target (T^{2H})",
    # Override NCE linear scaling with Mandelbrot's power law profile
    VarianceValue = Base_Var * (T_days^(2 * Hurst_Est))
  )

# 4. Bind the new fractional curve back into your master visual comparison matrix
plotting_matrix_df <- bind_rows(cas_scaling_df, cas_lines_df) %>%
  left_join(
    unique(cas_true_dynamics_df[, c("Space_Label", "Hurst_Est")]), 
    by = "Space_Label", 
    suffix = c("", ".y")
  ) %>%
  # Fill missing Hurst_Est text descriptors generated by the bind_rows step
  group_by(Space) %>%
  fill(Hurst_Est, Space_Label, .direction = "updown") %>%
  ungroup()

plotly_plots_2 <- list()

# --- PLOT THE THREE-WAY PARADIGM COMPARISON ---
for (current_space in unique(plotting_matrix_df$Space_Label)) {
  
  space_data <- plotting_matrix_df %>% filter(Space_Label == current_space)
  h_val <- round(unique(space_data$Hurst_Est), 3)
  
  p_cas_true <- ggplot(space_data, aes(x = factor(T_days), y = VarianceValue, fill = Framework_Type)) +
    geom_col(position = "dodge", alpha = 0.85) +
    scale_fill_manual(values = c("darkblue", "darkorange", "darkgrey")) +
    theme_minimal() +
    labs(
      title = paste("True CAS Dynamics Scaling:", current_space),
      subtitle = paste0("Empirical Memory Profile vs. Linear Benchmarks (Estimated Hurst H = ", h_val, ")"),
      x = "Time Horizon (Days)",
      y = "Total Coordinate Variance",
      fill = "Model Class"
    )
  
  plotly_plots_2[[current_space]] <- ggplotly(p_cas_true)
}

tagList(plotly_plots_2)
```

```{r}
# 1. Isolate clean continuous log returns for your target asset (e.g., PAL)
returns_vector <- master_df %>%
  filter(!is.na(PAL.LogAdj)) %>%
  pull(PAL.LogAdj)

# 2. Compute a complete continuous spectrum of empirical variances across horizons 1 to 21
horizons <- 1:21
continuous_empirical_var <- sapply(horizons, function(k) {
  k_day_returns <- rollapplyr(returns_vector, k, sum, fill = NA)
  var(k_day_returns, na.rm = TRUE)
})

# 3. Extract base 1-day variance baseline unit
base_var <- continuous_empirical_var[1]

# 4. Programmatically estimate the exact Hurst Exponent (H) via log-linear fit
log_T <- log(horizons)
log_Var <- log(continuous_empirical_var)
fit_hurst <- lm(log_Var ~ log_T)
estimated_H <- as.numeric(coef(fit_hurst)[2] / 2)

# 5. Build the complete geometric coordinate dataset mapping the three trajectories
dynamics_manifold_df <- tibble(
  Horizon_T = horizons,
  Empirical_Path = continuous_empirical_var,
  NCE_Linear_Path = base_var * horizons,                     # Var(T) = sigma^2 * T
  CAS_Fractal_Path = base_var * (horizons^(2 * estimated_H)) # Var(T) = sigma^2 * T^(2H)
) %>%
  # Pivot into long format for line geom mapping
  pivot_longer(
    cols = -Horizon_T,
    names_to = "Paradigm_Layer",
    values_to = "Total_Variance"
  ) %>%
  mutate(
    Paradigm_Layer = case_when(
      Paradigm_Layer == "Empirical_Path"   ~ "Actual Empirical Path (Market Data)",
      Paradigm_Layer == "NCE_Linear_Path"  ~ "NCE / Black-Scholes Path (Linear)",
      Paradigm_Layer == "CAS_Fractal_Path" ~ paste0("CAS Fractal Curve (Hurst H = ", round(estimated_H, 3), ")")
    )
  )

# 6. Generate the Geometric Line Plot
p_dynamics_curves <- ggplot(dynamics_manifold_df, 
                            aes(x = Horizon_T, y = Total_Variance, color = Paradigm_Layer, linetype = Paradigm_Layer)) +
  geom_line(size = 1.2, alpha = 0.9) +
  geom_point(size = 2, alpha = 0.6) +
  scale_color_manual(values = c("darkblue", "darkorange", "red")) +
  scale_linetype_manual(values = c("solid", "dashed", "dotted")) +
  theme_minimal() +
  labs(
    title = "PAL Dynamics Manifold: Multi-Scale Variance Scaling Curves",
    subtitle = "Visualizing the geometric path difference between memoryless diffusion and complex systems",
    x = "Time Horizon Scaling (Days, T)",
    y = "Total Aggregated Return Variance",
    color = "Theoretical Framework",
    linetype = "Theoretical Framework"
  ) +
  theme(legend.position = "bottom")

# Render interactively for your integrated HTML report
ggplotly(p_dynamics_curves)
```

## Inference
### Testing for Misprice

To validate its core assumptions, the Neoclassical Economics framework relies on a highly structured empirical process that checks if short-horizon volatility is sufficient to price longer-horizon risk. This evidence-gathering phase utilizes two primary statistical metrics: the Jarque-Bera goodness-of-fit test to evaluate return normality, and the Lo-MacKinlay Variance Ratio Test ($VR(k)$) to examine variance scaling across multiple time horizons ($k$). Together, these indicators isolate whether the asset behaves as a memoryless, flat Euclidean random walk or if it exhibits underlying structural anomalies.

The final programmatic verdict is then determined by a strict, binary decision rule. If the Jarque-Bera test fails to reject the null hypothesis of normality ($p > 0.05$) and the variance ratio remains approximately equal to $1$ across all designated horizons ($VR(k) \approx 1$), the NCE framework is formally accepted. This statistical outcome confirms that daily volatility parameters are entirely sufficient to hedge long-horizon positions. Conversely, if the p-value drops below the significance threshold ($p \le 0.05$) or if the variance ratio significantly drifts away from unity, the NCE assumptions are rejected, proving that traditional equilibrium models systematically misprice and understate long-term structural risk.


**Objective**: 
 1. Gather the Evidence: 
    - The Goodness of Fit test (Jarque-Bera) for normality, and 
    - The Lo-MacKinlay Variance Ratio Test ($VR(k)$) to evaluate variance scaling across multiple horizons ($k$).
 2. Decision Rule:  
    - **Null Hypothesis** ($H_0$): If Jarque-Bera fails to reject the null hypothesis of normality ($p > 0.05$); and $VR(k) \approx 1$ across all designated horizons ($k$). Then, we accept the NCE framework. Daily volatility is sufficient to price longer-horizon risk, confirming a flat, memoryless Euclidean price space.
    - **Alternative Hypothesis** ($H_1$): If Jarque-Bera rejects normality ($p \le 0.05$); or $VR(k)$ significantly drifts away from $1$ as $k$ increases. Then, the NCE assumption fails. Short-horizon volatility calculations systematically misprice or understate long-horizon structural risk.
 3. Give the Verdict

**Code**:
```{r inference-}
# --- INFERENCE STEP ---
# Pick an asset to test dynamically (or loop through them all)
target_asset <- "PAL"

# Gather vectors from your completed dynamics/structure frames
daily_returns  <- na.omit(price_dynamics_df[[paste0(target_asset, ".LogAdj")]])
ret_5d         <- na.omit(price_dynamics_df[[paste0(target_asset, "_5d_return")]])
ret_21d        <- na.omit(price_dynamics_df[[paste0(target_asset, "_21d_return")]])

# Execute NCE Significance Tests
jb_test   <- jarque.bera.test(daily_returns)
p_val_jb  <- jb_test$p.value

# Form the Variance Ratios to check against 1
vr_5d  <- var(ret_5d) / (5 * var(daily_returns))
vr_21d <- var(ret_21d) / (21 * var(daily_returns))

# Execute CAS Significance Tests
# Note:Hurst Exponents via Rescaled Range, and Hill Estimators for heavy tails.
hurst_val <- pracma::hurstexp(daily_returns, display = FALSE)$Hs
# tail_index <- custom_hill_estimator(daily_returns) 

# 4. Print Outputs to clear the TBD fields
cat("--- INFERENCE REPORT FOR:", target_asset, "---\n",
    "Jarque-Bera p-value: ", p_val_jb, "\n",
    "Variance Ratio (5-Day Target = 1): ", vr_5d, "\n",
    "Variance Ratio (21-Day Target = 1): ", vr_21d, "\n",
    "Hurst Exponent (Target = 0.5): ", hurst_val, "\n")
```

**Output**:


### 

In contrast, the Complex Adaptive Systems framework structures its inference engine around the premise that traditional Gaussian approximations underprice catastrophic tail risk. To capture these non-equilibrium traits, the CAS evidence-gathering layer bypasses basic moment calculations in favor of multi-scale fractal metrics. Specifically, it computes the Hurst Exponent ($H$) via Rescaled Range Analysis to detect long-range dependency and volatility clustering, while simultaneously utilizing Hill’s Estimator to determine the Tail Index ($\alpha$), which explicitly measures the power-law decay of extreme tail shocks.

**Objective**:
 1. Gather the Evidence:
   - Hurst Exponent ($H$) via Rescaled Range Analysis to evaluate long-range dependency and clustering ($H \neq 0.5$).
   - Hill’s Estimator for Tail Index ($\alpha$) to compute the power-law decay exponent of the return distribution's extreme tails.
 2. Decision Rule:
    - **Null Hypothesis ($H_0$)**: If the estimated Hurst exponent deviates significantly from equilibrium ($H > 0.5$ for persistence or $H < 0.5$ for anti-persistence) and Hill's tail index indicates heavy tails ($\alpha < 3$, where variance or kurtosis begins to fail standard convergence limits). Then, we validate the CAS paradigm. The asset demonstrates structural memory and fat tails, proving that return distributions cannot be compressed into a memoryless scalar without ignoring catastrophic systemic tail risk.
    - **Alternative Hypothesis ($H_1$)**: If $H \approx 0.5$ and $\alpha \ge 3$ (approaching Gaussian tail decay thresholds). Then, the asset behaves like a standard equilibrium system. Deviations are transient noise, and the traditional random walk approximation remains robust.
 3. Give the Verdict

**Code**:
```{r cas-inference-engine, echo=TRUE, message=FALSE, warning=FALSE}
# ==============================================================================
# CAS INFERENCE ENGINE: EVIDENCE, DECISION RULE, & PROGRAMMATIC VERDICT
# ==============================================================================

# 1. Isolate target asset log returns (e.g., PAL) and strip NA values
returns_vector <- master_df %>%
  filter(!is.na(PAL.LogAdj)) %>%
  pull(PAL.LogAdj)

### A. Hurst Exponent (H) via Log-Variance Multi-Scale Regression
# We compute variance across horizons 1 to 21 to capture the exact memory decay profile
horizons <- 1:21
horizon_variances <- sapply(horizons, function(k) {
  # Generate k-day rolling compounded paths
  k_day_returns <- zoo::rollapplyr(returns_vector, k, sum, fill = NA)
  var(k_day_returns, na.rm = TRUE)
})

# Log-linear fit: ln(Var) = ln(C) + 2H*ln(T)
log_T <- log(horizons)
log_Var <- log(horizon_variances)
fit_hurst <- lm(log_Var ~ log_T)
estimated_H <- as.numeric(coef(fit_hurst)[2] / 2)


### B. Hill's Estimator for Tail Index (alpha)
# We isolate the upper extreme tail (top 5% positive tail shocks) to extract the power law decay
tail_threshold <- 0.05 
sorted_returns <- sort(returns_vector, decreasing = TRUE)
n_tail <- floor(length(sorted_returns) * tail_threshold)

# Extract extreme threshold anchor
x_threshold <- sorted_returns[n_tail + 1]
tail_shocks <- sorted_returns[1:n_tail]

# Hill Estimator Formula: 1 / [ (1/k) * Sum( ln(x_i / x_{k+1}) ) ]
hill_alpha <- 1 / (mean(log(tail_shocks / x_threshold)))

# Print Computed Evidence Summary Table
cat("===========================================\n",
    "      CAS EVIDENCE GATHERING MATRIX        \n",
    "===========================================\n",
    "Target Asset Component: PAL\n",
    "Estimated Hurst Exponent (H) : ", round(estimated_H, 4), "\n",
    "Hill's Tail Decay Index (α)  : ", round(hill_alpha, 4), "\n",
    "-------------------------------------------\n\n")

# Establish strict structural thresholds based on your framework rules
hurst_lower_bound <- 0.45
hurst_upper_bound <- 0.55
alpha_fat_tail_threshold <- 3.0

if ((estimated_H > hurst_upper_bound || estimated_H < hurst_lower_bound) && hill_alpha < alpha_fat_tail_threshold) {
  
  # --- BRANCH 1: VALIDATE COMPLEX ADAPTIVE SYSTEMS PARADIGM ---
  cas_verdict <- "VALIDATED (REJECT NEOCLASSICAL EQUILIBRIUM)"
  
  cas_interpretation <- paste0(
    "VERDICT ANALYSIS: The CAS paradigm is empirically confirmed.\n",
    "Reasoning: The asset demonstrates strong anomalous scaling (H = ", round(estimated_H, 3), 
    if_else(estimated_H < 0.5, " indicating anti-persistent mean-reversion", " indicating persistent trend-reinforcement"), 
    ") coupled with extreme fat-tailed decay (α = ", round(hill_alpha, 3), ").\n",
    "Systemic Impact: Because α < 3, the traditional Gaussian variance boundaries are violated.\n",
    "Compressing this asset into standard local variance metrics or flat i.i.d assumptions will systematically\n",
    "underprice options structures and risk profiles, rendering standard Black-Scholes risk models blind\n",
    "to endogenous, structural liquidation cascades."
  )
  
} else {
  
  # --- BRANCH 2: VALIDATE NEOCLASSICAL EQUILIBRIUM ---
  cas_verdict <- "REJECTED (NEOCLASSICAL RANDOM WALK ROBUST)"
  
  cas_interpretation <- paste0(
    "VERDICT ANALYSIS: The Neoclassical Equilibrium framework holds robustly.\n",
    "Reasoning: The asset's scaling footprint closely approximates memoryless Brownian motion (H = ", round(estimated_H, 3), 
    ") and the distribution's tail profile safely matches or exceeds normal exponential decay structures (α = ", round(hill_alpha, 3), ").\n",
    "Systemic Impact: Shocks are transient, exogenous, and uncorrelated. Traditional Value-at-Risk (VaR)\n",
    "and square-root-of-time scaling mechanics provide a dependable framework for structural risk budgeting."
  )
}

# Output programmatically derived Verdict and Detailed Summary
cat("======================================================================\n", 
    "FINAL PROGRAMMATIC INFERENCE VERDICT:", cas_verdict, "\n",
    "======================================================================\n",
    cas_interpretation, "\n")
```

**Financial Interpretation**: 


## Synthesis
The Price Movement analysis serves as our most granular lens, examining individual asset vectors in isolation. If the empirical verdicts yield deviations from normality and square-root scaling, it proves that "price" is not a simple, self-contained coordinate point. Instead, assets contain intrinsic path dependency and memory, prompting us to step outward into our next lens: how these unique asset states interact with one another under varying regimes.

## Footnote:
[9] Sewell, M. (2012). The efficient market hypothesis: Empirical evidence. *International Journal of Statistics and Probability*, 1(2), 164-178.

[a2] On "Symplectic/Contact geometry": true symplectic geometry requires an explicit closed, non-degenerate 2-form on an even-dimensional phase space, and contact geometry requires an explicit contact 1-form - neither is constructed anywhere in this code, so naming them without deriving them would be unearned. What *is* legitimate and directly testable with the tools already in this section is the underlying distinction those formalisms are gesturing at: whether the flow through phase space is volume-preserving (conservative, NCE-consistent) or volume-contracting onto a lower-dimensional attractor (dissipative, CAS-consistent). This is the same question chaos theory asks of a reconstructed phase space via delay embeddings (Takens, 1981) and answers with a correlation-dimension estimate (Grassberger & Procaccia, 1983); it has prior precedent in finance specifically (Hsieh, 1991; Peters, 1991). If we want the fuller geometric machinery later - e.g. an actual contact Hamiltonian formulation of dissipation, as in Bravetti, Cruz, & Tapias (2017) - it is a legitimate direction, but it's a research extension, not something implied for free by taking a second difference.
