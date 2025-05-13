# Range Bet Math Core

This crate implements the mathematical core for the Signals Breakout Contracts protocol. It provides the essential mathematical functions for calculating betting costs using the (q+t)/(T+t) integral formula.

## Features

- Configurable build targets (on-chain BPF and WASM)
- Robust mathematical implementation with overflow protection
- Complete test suite including property-based tests

## Core Functions

The crate exposes the following core mathematical functions:

### `calculate_bin_buy_cost`

Calculates the cost to buy tokens in a single bin.

```rust
pub fn calculate_bin_buy_cost(x: u64, q: u64, t: u64) -> Result<u64>
```

Where:

- `x`: Amount of tokens to purchase
- `q`: Current token quantity in the bin
- `t`: Total token quantity in the market

### `calculate_bin_sell_cost`

Calculates the revenue from selling tokens in a single bin.

```rust
pub fn calculate_bin_sell_cost(x: u64, q: u64, t: u64) -> Result<u64>
```

### `calculate_multi_bins_buy_cost`

Calculates the total cost to buy tokens across multiple bins.

```rust
pub fn calculate_multi_bins_buy_cost(x: u64, qs: &[u64], t: u64) -> Result<u64>
```

### `calculate_multi_bins_sell_cost`

Calculates the revenue from selling tokens across multiple bins.

```rust
pub fn calculate_multi_bins_sell_cost(x: u64, qs: &[u64], t: u64) -> Result<u64>
```

### `calculate_x_for_multi_bins`

Inverse calculation - finds the maximum number of tokens that can be purchased within a budget.

```rust
pub fn calculate_x_for_multi_bins(budget: u64, qs: &[u64], t: u64) -> Result<u64>
```

## Mathematical Model

The core price formula is an integral:

```
Cost = ∫(q+t)/(T+t) dt, from t=0 to t=x
```

Which evaluates to:

```
Cost = x + (q-T)*ln((T+x)/T)
```

Where:

- `q`: Current token quantity in the bin
- `T`: Total token supply in the market
- `x`: Token quantity to purchase

## Building

### As an on-chain program

```bash
cargo build-bpf
```

### As a WASM module

```bash
cargo build --features wasm --target wasm32-unknown-unknown
```

## Testing

Run the comprehensive test suite:

```bash
cargo test
```

Run benchmarks:

```bash
cargo test bench -- --nocapture
```

Run property-based tests:

```bash
cargo test property_tests
```
