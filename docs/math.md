# Signals Breakout Contracts Mathematical Model

This document explains the core mathematical models and cost calculation algorithms used in the Signals Breakout Contracts project.

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
Cost = x + q * ln((T+x)/(T)) = x + q * ln(1 + x/T)
```

This formula is implemented in `rust` code as follows:

```rust
pub fn calculate_cost(amount: u64, bin_q: u64, t_total: u64) -> Result<u64> {
    if amount == 0 {
        return Ok(0);
    }

    // First term: x (amount)
    let mut cost = amount;

    // Calculate second term if q is not 0: q * ln((T+x)/T)
    if bin_q > 0 {
        // Calculate logarithm part: ln(1 + x/T)
        // Use fixed-point arithmetic to prevent overflow
        let ln_part = calculate_ln_part(amount, t_total)?;

        // q * ln_part
        let second_term = (bin_q as u128 * ln_part as u128) / PRECISION;

        // Add second term to total cost
        cost = cost.checked_add(second_term as u64)
            .ok_or(error!(RangeBetError::MathOverflow))?;
    }

    Ok(cost)
}
```

## Reverse Calculation: Quantity from Cost

Calculating token quantity corresponding to a specific cost is done through the inverse transformation of the integral. This is equivalent to finding the solution to the following equation:

```
cost = x + q * ln(1 + x/T)
```

The code implements this using binary search:

```rust
pub fn calculate_tokens_for_cost(cost: u64, bin_q: u64, t_total: u64) -> Result<u64> {
    if cost == 0 {
        return Ok(0);
    }

    // Return minimum 1 token if cost is too small
    if cost <= 1 {
        return Ok(1);
    }

    // Set initial range for binary search
    let mut low: u64 = 1;
    let mut high: u64 = u64::MAX - 1;

    while low <= high {
        let mid = low + (high - low) / 2;

        // Calculate cost for the given token quantity
        let calculated_cost = calculate_cost(mid, bin_q, t_total)?;

        // Compare costs
        match calculated_cost.cmp(&cost) {
            std::cmp::Ordering::Equal => return Ok(mid),
            std::cmp::Ordering::Less => low = mid + 1,
            std::cmp::Ordering::Greater => {
                if mid == 1 {
                    return Ok(1);
                }
                high = mid - 1;
            }
        }
    }

    // Return the closest estimate
    Ok(high)
}
```

## Logarithm Function Implementation

The natural logarithm function required for integral calculation is implemented using fixed-point operations. The logarithm is calculated using Taylor series expansion or other numerical approximations:

```rust
/// Logarithm calculation part: ln(1 + x/T)
pub fn calculate_ln_part(x: u64, t: u64) -> Result<u64> {
    if t == 0 {
        return Err(error!(RangeBetError::DivisionByZero));
    }

    // Calculate x/T ratio (using fixed-point precision)
    let ratio = (x as u128 * PRECISION as u128) / t as u128;

    // Calculate ln(1 + ratio)
    // Using Taylor series or other approximation
    let ln_result = natural_log(PRECISION + ratio as u64)?;

    Ok(ln_result)
}
```

## Market Mechanism and Balance

The key characteristics of this price formula are:

1. **Dynamic Price Adjustment**: As the token quantity (q) in a specific bin increases, the cost of additional betting on that bin also increases.

2. **Liquidity-Based Pricing**: As the total token supply in the market (T) increases, the relative impact of individual bets decreases.

3. **Initial Discount**: When betting on a new bin (q=0) for the first time, the cost is exactly equal to the bet amount (cost = x).

4. **Winning Reward Pool**: When the market closes and a winning bin is determined, users who bet on that bin distribute the entire collateral pool according to their investment ratio.

These mechanisms encourage market participants to bet more when they are confident in their predictions, while simultaneously increasing the cost of betting on popular predictions to maintain market balance.
