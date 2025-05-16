# Signals Breakout Programs - Solana Market Prediction Protocol

Signals Breakout Programs is a prediction market protocol operating on the Solana blockchain that implements a $(q+t)/(T+t)$ integral price formula using a Uniswap V3-style tick-based bin system. This protocol can operate multiple prediction markets simultaneously with a single program and provides unique pricing mechanisms through special betting cost calculations.

## Key Features

- Operate multiple prediction markets with a single manager contract
- Price range settings using Uniswap V3 tick structure (ranges/bins)
- Sophisticated betting cost calculation through the $(q+t)/(T+t)$ integral formula
- Betting across various ranges possible
- Winning range setting and reward distribution system

## Mathematical Background

The betting cost is calculated based on the following integral:

$$\int_{t=0}^{x} \frac{q+t}{T+t} \, dt = x + (q-T) \ln\left(\frac{T+x}{T}\right)$$

Where:

- $q$: Current amount of tokens in the bin
- $T$: Total supply of tokens in the entire market
- $x$: Amount of tokens to purchase

This formula means that the betting cost adjusts according to the market's liquidity. The more popular an interval is, the higher the cost to bet on it.

### Math Core

The mathematical functions are implemented in a separate `math-core` crate that you can find in the `programs/range-bet-program/math-core` directory. This crate can be compiled both for on-chain use (BPF) and as a WASM module for client-side applications. For more details, see the [Math Core README](programs/range-bet-program/math-core/README.md).

## Architecture

### Main Components

1. **Program State (ProgramState)**:

   - Management of program owner and market ID sequence
   - Tracking of recently closed markets

2. **Market (Market)**:

   - Management of market active status and closing status
   - Setting of tick spacing, min/max tick ranges
   - Tracking of token quantities by bin and total token supply
   - Management of collateral balance
   - Setting of market opening and closing times

3. **User Position (UserMarketPosition)**:

   - Tracking of market positions by user
   - Management of token balances by bin

4. **Math Core Library**:
   - Separate crate for mathematical calculations
   - Implements the core price formula and its derivatives
   - Supports both on-chain (BPF) and WASM compilation targets

### Key Functions

1. **Market Creation (createMarket)**:

   - Admin creates a new prediction market
   - Sets tick spacing, min/max tick range, closing time

2. **Token Purchase (buyTokens)**:

   - Users bet on various bins
   - Betting cost calculation through the $(q+t)/(T+t)$ integral formula

3. **Market Closing (closeMarket)**:

   - Admin closes the market and sets the winning bin

4. **Reward Claiming (claimReward)**:

   - Token holders of the winning bin claim rewards

5. **Collateral Withdrawal (withdrawCollateral)**:
   - Admin withdraws collateral after market closure

## Getting Started

### Requirements

- Node.js v16 or higher
- Solana CLI
- Yarn package manager
- Anchor framework

### Installation

```bash
# Clone repository
git clone https://github.com/signals-protocol/signals-breakout-programs.git
cd signals-breakout-programs

# Install dependencies
yarn install
```

### Compilation

Compile the contracts:

```bash
yarn build
```

### Testing

Run all tests:

```bash
yarn test:local
```

### Local Deployment

Deploy contracts to local development node:

```bash
# Start a local validator
solana-test-validator

# Deploy
yarn build
anchor deploy
```

### Devnet Deployment

Deploy to Solana Devnet:

```bash
yarn deploy:dev
```

### Program Upgrades

Upgrade programs on devnet:

```bash
# Range Bet Program
yarn upgrade:range-bet-program:dev

# Collateral Token Faucet
yarn upgrade:collateral-token-faucet:dev
```

## System Operation

### Creating Prediction Markets

The administrator (contract owner) can create a new prediction market by calling the `createMarket()` function:

```typescript
await program.methods
  .createMarket(
    20, // tickSpacing: tick interval
    new BN(-240), // minTick: minimum tick
    new BN(960), // maxTick: maximum tick
    new BN(closeTime) // expected market closing time
  )
  .accounts({
    owner: wallet.publicKey,
    collateralMint: COLLATERAL_MINT,
  })
  .signers([wallet])
  .rpc();
```

### Token Purchase (Betting)

Users can bet on various ranges (bins) by calling the `buyTokens()` function:

```typescript
await program.methods
  .buyTokens(
    marketId, // market ID
    [0, 3], // bin indices to bet on
    [100000000, 50000000], // amount to bet on each bin
    200000000 // maximum willing to pay
  )
  .accounts({
    user: wallet.publicKey,
    userTokenAccount: userTokenAccount,
    vault: marketVault,
  })
  .signers([wallet])
  .rpc();
```

### Market Closing and Setting Winning Bin

The administrator calls the `closeMarket()` function to close the market and set the winning bin:

```typescript
await program.methods
  .closeMarket(
    marketId, // market ID
    winningBin // winning bin
  )
  .accounts({
    authority: wallet.publicKey,
  })
  .signers([wallet])
  .rpc();
```

### Claiming Rewards

Token holders of the winning bin can claim rewards by calling the `claimReward()` function:

```typescript
await program.methods
  .claimReward()
  .accounts({
    user: wallet.publicKey,
    userTokenAccount: userTokenAccount,
    vault: marketVault,
    vaultAuthority: vaultAuthPDA,
  })
  .signers([wallet])
  .rpc();
```

### Using the Collateral Token Faucet

For development and testing purposes, the protocol includes a collateral token faucet program that allows users to mint test tokens:

```typescript
await faucetProgram.methods
  .mintCollateralTokens(
    new BN(1000000000) // Amount of tokens to mint (e.g., 1000 tokens with 6 decimals)
  )
  .accounts({
    mint: COLLATERAL_MINT,
    receiver: userTokenAccount,
    user: wallet.publicKey,
  })
  .signers([wallet])
  .rpc();
```

This faucet program is intended for development environments only and simplifies testing by providing collateral tokens to participants. For detailed documentation on the faucet program, see the [Collateral Token Faucet Documentation](docs/collateral-token-faucet.md).

### WASM Package for Frontend

The Math Core is also available as an npm package named `range-bet-math-core` that can be used in frontend applications. The package provides the same mathematical functions as the on-chain program, allowing cost calculations client-side before submitting transactions. For detailed documentation, see the [WASM Package README](programs/range-bet-program/pkg-wasm/README.md).

To build the WASM package:

```bash
yarn build:wasm
```

To publish the WASM package:

```bash
yarn publish:wasm
```

Using the package in your project:

```typescript
import {
  calculateBinBuyCost,
  calculateMultiBinsBuyCost,
} from "range-bet-math-core";

// Calculate purchase cost for a single bin
const cost = calculateBinBuyCost(100n, 500n, 1000n);

// Calculate purchase cost for multiple bins
const bins = new BigUint64Array([300n, 400n, 500n]);
const multiCost = calculateMultiBinsBuyCost(100n, bins, 1000n);
```

## Project Documentation

This repository includes several documentation files organized by purpose:

### Core Concepts

- [Architecture](docs/architecture.md) - System architecture, components, and their interactions
- [Mathematical Model](docs/math.md) - Theoretical foundation and formulas of the pricing model

### Developer Resources

- [API Reference](docs/api-reference.md) - Complete instruction and account reference for both programs
- [Usage Guide](docs/usage.md) - Code examples and integration patterns for developers
- [Collateral Token Faucet](docs/collateral-token-faucet.md) - Test token utility for development environments

### Math Core Documentation

- [Math Core README](programs/range-bet-program/math-core/README.md) - Rust implementation details
- [TypeScript Guide](programs/range-bet-program/math-core/GUIDE.md) - Client-side TypeScript integration
- [WASM Package README](programs/range-bet-program/pkg-wasm/README.md) - npm package usage and examples

## License

Licensed under the ISC license. See the LICENSE file for details.
