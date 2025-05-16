# Signals Breakout Contracts Mathematical Model

This document provides the canonical explanation of the mathematical model used in the Signals Breakout Contracts project. For implementation details, see the `math-core` crate documentation.

## Core Price Formula

Signals uses a special integral formula to calculate betting costs that fairly reflects price impact based on bet size:

$$\int_{t=0}^{x} \frac{q+t}{T+t} \, dt$$

Where:

- $q$: Current token quantity in the bin
- $T$: Total token supply in the entire market
- $t$: Integration variable
- $x$: Token quantity to purchase

## Integral Solution

This integral evaluates to:

$$Cost = x + (q-T)\ln\left(\frac{T+x}{T}\right) = x + (q-T)\ln\left(1 + \frac{x}{T}\right)$$

## Key Properties

This formula has several important properties:

1. **Fair Pricing**: The ratio of tokens in a specific bin to the total token quantity represents the probability of that bin, following the "price = probability" principle.

2. **Dynamic Price Adjustment**: As the token quantity ($q$) in a specific bin increases, the cost of additional betting on that bin also increases. This means that as the market assigns a higher probability to a particular outcome, betting on it becomes more expensive.

3. **Liquidity-Based Pricing**: As the total token supply ($T$) increases, the relative impact of individual bets decreases. This ensures the market becomes more stable as it matures.

4. **Early Participant Incentive**: When betting on a new bin ($q=0$), the cost is less than the token amount, providing an incentive for early participants.

5. **Proportional Rewards**: When the market closes, users who bet on the winning bin distribute the entire collateral pool proportionally to their token holdings.

## Bitcoin Price Prediction Market

The primary application of this model is a Bitcoin price prediction market that addresses these key challenges:

1. **Continuous Price Forecasting**: Rather than binary YES/NO markets, it handles infinitely many continuous outcomes
2. **Unified Liquidity**: A single AMM curve handles all price ranges simultaneously
3. **Proportional Price Impact**: The cost function charges costs proportional to the impact a bet has on the distribution

This makes it ideal for predicting a future BTC price range, where users can visualize the market's confidence across the entire price spectrum.

## Unified Liquidity for n-Outcomes

Unlike traditional prediction markets that split liquidity across multiple binary YES/NO markets, Signals maintains unified liquidity while supporting n continuous outcomes:

1. A single AMM curve handles all price ranges simultaneously
2. The cost function charges costs proportional to the impact a bet has on the overall distribution
3. Larger bets pay for their proportional price impact

This approach solves the fundamental limitations of traditional prediction markets when applied to continuous price prediction.

## Heat-map Visualization

The model produces a probability distribution that is visualized as a heat-map in the frontend:

1. The x-axis represents different price ranges
2. Color intensity shows the market's confidence in each price range
3. The entire visualization represents a normalized probability distribution

This visualization transforms BTC price predictions into an intuitive visual format rather than a series of binary YES/NO bets.

## Implementation Note

> While the mathematical model is described using continuous calculus, the actual implementation uses floating-point calculations (f64) internally for precision, while maintaining an integer-based public API. This approach allows for accurate logarithmic calculations while ensuring deterministic results. See the implementation documentation for details.

## Further Implementation Details

For specific implementation details, see:

- [Math Core README](../programs/range-bet-program/math-core/README.md) - Implementation details in Rust
- [API Reference](./api-reference.md#math-core-api) - Complete API reference
- [Usage Guide](./usage.md#using-the-math-core-library) - Examples of using the math functions
