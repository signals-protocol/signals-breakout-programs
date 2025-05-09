# Signals Breakout Contracts - Solana Market Prediction Protocol

Signals Breakout Contracts is a prediction market protocol operating on the Solana blockchain that implements a (q+t)/(T+t) integral price formula using a Uniswap V3-style tick-based bin system. This protocol can operate multiple prediction markets simultaneously with a single contract and provides unique pricing mechanisms through special betting cost calculations.

## Key Features

- Operate multiple prediction markets with a single manager contract
- Price range settings using Uniswap V3 tick structure (ranges/bins)
- Sophisticated betting cost calculation through the (q+t)/(T+t) integral formula
- Betting across various ranges possible
- Winning range setting and reward distribution system

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

### Key Functions

1. **Market Creation (createMarket)**:

   - Admin creates a new prediction market
   - Sets tick spacing, min/max tick range, closing time

2. **Token Purchase (buyTokens)**:

   - Users bet on various bins
   - Betting cost calculation through the (q+t)/(T+t) integral formula

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
git clone https://github.com/signals-protocol/signals-breakout-contracts.git
cd signals-breakout-contracts

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
yarn test
```

Run specific tests:

```bash
yarn test:local
```

### Local Deployment

Deploy contracts to local development node:

```bash
# Run local validator in a new terminal
yarn local-validator

# Run deployment in another terminal
yarn deploy
```

### Devnet Deployment

Deploy to Solana Devnet:

```bash
yarn deploy:devnet
```

### Scripts

You can interact with the contracts using various scripts:

```bash
# Create markets
yarn create-markets

# Execute bets
yarn place-bets
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

## Mathematical Background

The betting cost is calculated based on the following integral:

![Integral Formula](https://latex.codecogs.com/png.latex?%5Cint_%7Bt%3D0%7D%5E%7Bx%7D%20%5Cfrac%7Bq%20%2B%20t%7D%7BT%20%2B%20t%7D%20%5C%2C%5Cmathrm%7Bd%7Dt%20%5C%3B%3D%5C%3B%20x%20%2B%20%28q%20-%20T%29%5C%2C%5Cln%5C%21%5CBigl%28%5Cfrac%7BT%20%2B%20x%7D%7BT%7D%5CBigr%29)

- `q`: Current amount of tokens in the bin
- `T`: Total supply of tokens in the entire market
- `x`: Amount of tokens to purchase

This formula means that the betting cost adjusts according to the market's liquidity. The more popular an interval is, the higher the cost to bet on it.

## License

Licensed under the ISC license. See the LICENSE file for details.
