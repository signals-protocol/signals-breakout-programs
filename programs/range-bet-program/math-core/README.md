# Range Bet Math Core

This crate implements the mathematical core for the Signals Protocol. It provides functions for calculating betting costs using the $(q+t)/(T+t)$ integral formula.

## Implementation Features

- **Dual compilation targets**: Both on-chain BPF (Solana) and WASM (browser)
- **Overflow protection**: Robust error handling for numerical edge cases
- **Comprehensive testing**: Unit tests, benchmarks, and property-based tests

## Core Functions

The crate exposes these primary functions:

### `calculate_bin_buy_cost`

```rust
pub fn calculate_bin_buy_cost(x: u64, q: u64, t: u64) -> Result<u64>
```

Calculates the cost to buy tokens in a single bin, where:

- `x`: Amount of tokens to purchase
- `q`: Current token quantity in the bin
- `t`: Total token quantity in the market

### `calculate_bin_sell_cost`

```rust
pub fn calculate_bin_sell_cost(x: u64, q: u64, t: u64) -> Result<u64>
```

Calculates the revenue from selling tokens in a bin.

### `calculate_multi_bins_buy_cost`

```rust
pub fn calculate_multi_bins_buy_cost(x: u64, qs: &[u64], t: u64) -> Result<u64>
```

Calculates the total cost to buy tokens across multiple bins.

### `calculate_multi_bins_sell_cost`

```rust
pub fn calculate_multi_bins_sell_cost(x: u64, qs: &[u64], t: u64) -> Result<u64>
```

Calculates the revenue from selling tokens across multiple bins.

### `calculate_x_for_multi_bins`

```rust
pub fn calculate_x_for_multi_bins(budget: u64, qs: &[u64], t: u64) -> Result<u64>
```

Finds the maximum token quantity purchasable within a budget.

## Technical Implementation Details

### Numerical Implementation

The library uses floating-point arithmetic (f64) internally for precision in logarithm calculations, while exposing an integer API:

```rust
// Example from the implementation
let q_f64 = q as f64;
let t_f64 = t as f64;
let x_f64 = x as f64;

// Calculate ratio: (t+x)/t = 1 + x/t
let ratio = (t_f64 + x_f64) / t_f64;
// Calculate natural logarithm
let ln_ratio = ratio.ln();
```

This approach:

- Allows for accurate logarithmic calculations
- Prevents overflows when calculating with large numbers
- Maintains consistency between on-chain and client-side calculations
- Returns integer values after rounding for deterministic results

### Error Handling

The implementation handles these error cases:

- Invalid bin state (q > t)
- Selling more than available (x > q)
- Selling more than supply (x > t)
- Mathematical overflows
- Calculation underflows

### Algorithm Optimizations

- Binary search for token quantity calculation
- Sequential processing for multi-bin operations
- Minimum value guarantees (always returning at least 1)

## Building

### As an on-chain program

```bash
cargo build-bpf
```

### As a WASM module

```bash
cargo build --features wasm --target wasm32-unknown-unknown
```

### As an npm package

```bash
# Build WASM with wasm-pack
wasm-pack build --target bundler --out-dir ../pkg-wasm --features wasm

# Or use the npm script
npm run build:wasm

# Publish to npm
npm run publish:wasm
```

## Testing

```bash
# Run all tests
cargo test

# Run benchmarks
cargo test bench -- --nocapture

# Run property-based tests
cargo test property_tests
```

## Integration with the Protocol

This library is used in two ways:

1. **On-chain calculations**: Used by Solana programs to calculate costs and revenues
2. **Client-side simulation**: Used via WASM in frontend applications

This architecture ensures that calculations remain consistent across on-chain and off-chain environments, providing a reliable user experience.

## Mathematical Background

The core price formula is based on the following integral:

$$\int_{t=0}^{x} \frac{q+t}{T+t} \, dt = x + (q-T) \ln\left(\frac{T+x}{T}\right)$$

Where:

- $q$: Current token quantity in the bin
- $T$: Total token supply in the market
- $x$: Token quantity to purchase

For a detailed explanation of the mathematical model and its properties:

- [Mathematical Model Documentation](../../../docs/math.md)

## Client Usage

- [TypeScript/JavaScript Guide](./GUIDE.md) - How to use the WASM build
- [WASM Package Documentation](../pkg-wasm/README.md) - npm package documentation
