# Signals Breakout Programs - Solana Market Prediction Protocol

Signals Breakout Programs is a prediction market protocol operating on the Solana blockchain that implements a $(q+t)/(T+t)$ integral price formula using a Uniswap V3-style tick-based bin system. This protocol can operate multiple prediction markets simultaneously with a single program and provides unique pricing mechanisms through special betting cost calculations.

## Key Features

- Operate multiple prediction markets with a single manager contract
- Price range settings using Uniswap V3 tick structure (ranges/bins)
- Sophisticated betting cost calculation through the $(q+t)/(T+t)$ integral formula
- Betting across various ranges possible
- Winning range setting and reward distribution system

## Mathematical Background

The protocol uses an innovative pricing formula based on an integral:

$$\int_{t=0}^{x} \frac{q+t}{T+t} \, dt = x + (q-T) \ln\left(\frac{T+x}{T}\right)$$

Where $q$ is the current bin quantity, $T$ is the total supply, and $x$ is the purchase amount. This formula dynamically adjusts costs based on market liquidity, making popular outcomes more expensive.

For a detailed explanation of the mathematical model, see the [Mathematics Documentation](docs/math.md).

The implementation is available in the [Math Core](programs/range-bet-program/math-core/README.md) crate, which compiles for both on-chain and client-side use.

## Architecture

The protocol consists of several key components working together to enable prediction markets on Solana. For detailed architecture documentation, see the [Architecture Guide](docs/architecture.md).

### Main Components

1. **Program State**: Global program settings and market registry
2. **Market**: Individual prediction markets with bins and tick settings
3. **User Position**: User's tokens across different market bins
4. **Math Core**: Pricing formula implementation (available as Rust crate and WASM package)
5. **Collateral Token Faucet**: Utility program for minting test tokens (development only)

### Key Functions

1. **Market Creation**: Create new prediction markets with customizable parameters
2. **Token Purchase**: Bet on market outcomes using the $(q+t)/(T+t)$ integral pricing formula
3. **Market Closing**: Close markets and set winning bins
4. **Reward Claiming**: Distribute rewards to winning participants
5. **Position Management**: Transfer positions between users and withdraw collateral

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

The protocol provides a comprehensive API for interacting with prediction markets. Below are key operations with simplified examples. For detailed API documentation, see the [API Reference](docs/api-reference.md) and [Usage Guide](docs/usage.md).

### Creating Prediction Markets

```typescript
await program.methods
  .createMarket(
    20, // tickSpacing
    new BN(-240), // minTick
    new BN(960), // maxTick
    new BN(closeTime) // market closing time
  )
  .accounts({
    owner: wallet.publicKey,
    collateralMint: COLLATERAL_MINT,
  })
  .signers([wallet])
  .rpc();
```

### Betting on Outcomes

```typescript
await program.methods
  .buyTokens(
    marketId,
    [0, 3], // bin indices to bet on
    [100000000, 50000000], // amounts for each bin
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

### Claiming Rewards

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

### Using the Test Token Faucet

```typescript
await faucetProgram.methods
  .mintCollateralTokens(new BN(1_000_000_000))
  .accounts({
    mint: COLLATERAL_MINT,
    receiver: userTokenAccount,
    user: wallet.publicKey,
  })
  .signers([wallet])
  .rpc();
```

### WASM Package for Frontend

The Math Core is also available as an npm package for frontend applications, allowing client-side cost calculations:

```typescript
import { calculateBinBuyCost } from "range-bet-math-core";

// Calculate purchase cost
const cost = calculateBinBuyCost(100n, 500n, 1000n);
```

Installation and build:

```bash
# Install
npm install range-bet-math-core

# For development: build and publish
yarn build:wasm
yarn publish:wasm
```

For detailed documentation on the WASM package, see the [WASM Package README](programs/range-bet-program/pkg-wasm/README.md) and [TypeScript Guide](programs/range-bet-program/math-core/GUIDE.md).

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
