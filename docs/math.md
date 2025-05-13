# Signals Breakout Contracts Mathematical Model

This document explains the core mathematical models and cost calculation algorithms used in the Signals Breakout Contracts project. These functions are implemented in the separate `math-core` crate.

## Core Price Formula

Signals Breakout Contracts implements the (q+t)/(T+t) integral price formula to calculate betting costs. The basic concept of this formula is as follows:

```
Cost = ∫(q+t)/(T+t) dt, from 0 to x
```

Where:

- `q`: Current token quantity in the bin
- `T`: Total token supply in the entire market
- `t`: Integration variable
- `x`: Token quantity to purchase

## Integral Solution

This integral has the following mathematical solution:

```
Cost = x + (q-T)*ln((T+x)/T) = x + (q-T)*ln(1 + x/T)
```

## Implementation in Math Core

The formula is implemented in the `math-core` crate, which is available at `programs/signals_breakout_contracts/math-core/`. The crate provides a set of functions for different use cases:

### Single Bin Operations

- `calculate_bin_buy_cost`: Calculates the cost to purchase tokens in a single bin
- `calculate_bin_sell_cost`: Calculates the revenue from selling tokens in a single bin

### Multi-bin Operations

- `calculate_multi_bins_buy_cost`: Calculates the cost to purchase tokens across multiple bins
- `calculate_multi_bins_sell_cost`: Calculates the revenue from selling tokens across multiple bins
- `calculate_x_for_multi_bins`: Calculates the maximum token quantity that can be purchased with a given budget

## Logarithm Calculation

The crate includes a custom implementation of the natural logarithm function optimized for fixed-point arithmetic to prevent overflows and provide accurate results even with large numbers.

## Compilation Targets

The `math-core` crate can be compiled for two targets:

1. **On-chain BPF**: For use in the Solana program
2. **WASM**: For client-side applications to perform the same calculations

This dual compilation capability ensures that both the on-chain program and client applications use identical mathematical logic.

## Market Mechanism and Balance

The key characteristics of this price formula are:

1. **Dynamic Price Adjustment**: As the token quantity (q) in a specific bin increases, the cost of additional betting on that bin also increases.

2. **Liquidity-Based Pricing**: As the total token supply in the market (T) increases, the relative impact of individual bets decreases.

3. **Initial Discount**: When betting on a new bin (q=0) for the first time, the cost is exactly equal to the bet amount (cost = x).

4. **Winning Reward Pool**: When the market closes and a winning bin is determined, users who bet on that bin distribute the entire collateral pool according to their investment ratio.

These mechanisms encourage market participants to bet more when they are confident in their predictions, while simultaneously increasing the cost of betting on popular predictions to maintain market balance.
