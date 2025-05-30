# Signals Breakout Contracts Usage Guide

This document provides a guide to using Signals Breakout Contracts and its key features.

## Getting Started

### Deployment Preparation

Steps to set up the project and deploy the contract:

1. Install dependencies:

   ```bash
   yarn install
   ```

2. Build the contract:

   ```bash
   yarn build
   ```

3. Run local test validator:

   ```bash
   solana-test-validator
   ```

4. Deploy the contract:
   ```bash
   yarn build
   anchor deploy
   ```

### Devnet Deployment

To deploy the contract to Solana Devnet:

```bash
yarn deploy:dev
```

After deployment, the program IDs are:

- Range Bet Program: `97i8BgDJG6yZggN2Di5UnERs6X5PqYqnkSvkMdvw1d5J`
- Collateral Token Faucet: `DDFXv1hETR8pQSpNbzCxTX7jm1Hr57V4oihDGosXQfgC`

### Getting Test Tokens

For development and testing, you can use the Collateral Token Faucet to mint test tokens:

```typescript
// Initialize the faucet program
const faucetProgramId = new PublicKey(
  "DDFXv1hETR8pQSpNbzCxTX7jm1Hr57V4oihDGosXQfgC"
);
const faucetProgram = new Program(faucetIdl, faucetProgramId, provider);

// Mint test tokens (1000 tokens with 6 decimals = 1,000,000,000 raw units)
await faucetProgram.methods
  .mintCollateralTokens(new BN(1_000_000_000))
  .accounts({
    mint: COLLATERAL_MINT,
    receiver: userTokenAccount,
    user: wallet.publicKey,
  })
  .signers([wallet])
  .rpc();

console.log("Test tokens minted successfully");
```

For more details about the faucet program, see the [Collateral Token Faucet Documentation](./collateral-token-faucet.md).

## Program Initialization

Initialize the program before first use:

```typescript
await program.methods
  .initializeProgram()
  .accounts({
    initializer: wallet.publicKey,
  })
  .signers([wallet])
  .rpc();
```

## Market Management

### Market Creation

Create a new prediction market:

```typescript
// Set market parameters
const TICK_SPACING = 20;
const MIN_TICK = -240;
const MAX_TICK = 960;
const closeTime = getUTCMidnightTimestamp(1); // Tomorrow at UTC midnight

// Create market
await program.methods
  .createMarket(
    TICK_SPACING,
    new BN(MIN_TICK),
    new BN(MAX_TICK),
    new BN(closeTime)
  )
  .accounts({
    owner: wallet.publicKey,
    collateralMint: COLLATERAL_MINT,
  })
  .signers([wallet])
  .rpc();
```

### Activate/Deactivate Market

Change a market's active status:

```typescript
await program.methods
  .activateMarket(
    marketId, // Market ID
    true // Activation status (true: activate, false: deactivate)
  )
  .accounts({
    owner: wallet.publicKey,
  })
  .signers([wallet])
  .rpc();
```

### Close Market and Set Winning Bin

When a market ends, you can set the winning bin:

```typescript
await program.methods
  .closeMarket(
    marketId, // Market ID
    winningBin // Winning bin index
  )
  .accounts({
    authority: wallet.publicKey,
  })
  .signers([wallet])
  .rpc();
```

### Withdraw Collateral

Withdraw remaining collateral after the market has closed:

```typescript
await program.methods
  .withdrawCollateral(marketId)
  .accounts({
    owner: wallet.publicKey,
    ownerTokenAccount: ownerTokenAccount,
    vault: marketVault,
  })
  .signers([wallet])
  .rpc();
```

## User Features

### Buy Tokens (Betting)

Bet on various bins:

```typescript
// Set bin indices and amounts to purchase
const binIndices = [0, 3]; // Bet on two bins
const amounts = [100000000, 50000000]; // Token amounts for each bin
const maxCollateral = 200000000; // Maximum willing to pay collateral

// Buy tokens
await program.methods
  .buyTokens(marketId, binIndices, amounts, maxCollateral)
  .accounts({
    user: wallet.publicKey,
    userTokenAccount: userTokenAccount,
    vault: marketVault,
  })
  .signers([wallet])
  .rpc();
```

### Claim Reward

Claim rewards after the market has closed and the winning bin is determined:

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

### Transfer Position

Transfer your position to another address:

```typescript
await program.methods
  .transferPosition(
    marketId,
    binIndices, // Array of bin indices to transfer
    amounts // Array of amounts to transfer for each bin
  )
  .accounts({
    fromUser: wallet.publicKey,
    toUser: recipientPubkey,
  })
  .signers([wallet])
  .rpc();
```

## Finding PDA Addresses

How to find key PDAs (Program Derived Addresses) to interact with the program:

### Program State PDA

```typescript
const [programState] = PublicKey.findProgramAddressSync(
  [Buffer.from("range-bet-state")],
  PROGRAM_ID
);
```

### Market PDA

```typescript
const [market] = PublicKey.findProgramAddressSync(
  [Buffer.from("market"), marketId.toBuffer("le", 8)],
  PROGRAM_ID
);
```

### Vault Authority PDA

```typescript
const [vaultAuthority] = PublicKey.findProgramAddressSync(
  [Buffer.from("vault"), marketId.toBuffer("le", 8)],
  PROGRAM_ID
);
```

### User Position PDA

```typescript
const [userPosition] = PublicKey.findProgramAddressSync(
  [Buffer.from("pos"), userPubkey.toBuffer(), marketId.toBuffer("le", 8)],
  PROGRAM_ID
);
```

## Query Market State

Get program state and market information:

```typescript
// Get program state
const stateInfo = await program.account.programState.fetch(programState);
console.log("Current market count:", stateInfo.marketCount.toNumber());

// Get market information
const marketInfo = await program.account.market.fetch(marketPDA);
console.log("Market active status:", marketInfo.active);
console.log("Market closed status:", marketInfo.closed);
console.log("Total token quantity:", marketInfo.tTotal.toString());
console.log("Collateral balance:", marketInfo.collateralBalance.toString());
```

## Query User Position

Get information about a user's market position:

```typescript
const userPos = await program.account.userMarketPosition.fetch(userPositionPDA);
console.log("Owner:", userPos.owner.toString());
console.log("Market ID:", userPos.marketId.toString());

// Print bin balances
userPos.bins.forEach((bin) => {
  console.log(`Bin ${bin.index}: ${bin.amount.toString()} tokens`);
});
```

## Cost Calculation Simulation

Simulation functions to calculate token purchase costs in advance:

```typescript
// Calculate the cost of purchasing a specific quantity of tokens in a specific bin
const cost = await program.methods
  .calculateBinCost(
    marketId, // Market ID
    binIndex, // Bin index
    amount // Token amount to purchase
  )
  .accounts({
    market: marketPDA,
  })
  .view();

console.log(`Cost to purchase ${amount} tokens: ${cost.toString()}`);

// Calculate the token quantity that can be purchased with a specific cost
const tokens = await program.methods
  .calculateXForBin(
    marketId, // Market ID
    binIndex, // Bin index
    cost // Cost to pay
  )
  .accounts({
    market: marketPDA,
  })
  .view();

console.log(`Tokens purchasable with ${cost} cost: ${tokens.toString()}`);
```

## Using the Math Core Library

The mathematical functions for calculating betting costs are implemented in a separate crate called `math-core`. This library can be used in two ways:

### For On-chain Program

The on-chain program uses the BPF version of the math-core library, which is automatically included during the build process.

### For Client Applications (WASM)

For client-side applications, you can use the WASM version of the library:

1. Build the WASM module:

```bash
yarn build:wasm
```

2. Import and use in your JavaScript/TypeScript application:

```typescript
// Import the math-core WASM module
import * as mathCore from "range-bet-math-core";

// Calculate buying cost for a single bin
const cost = mathCore.calculateBinBuyCost(
  tokenAmount, // x: token amount to purchase
  binQuantity, // q: current bin quantity
  totalTokens // T: total token quantity
);

// Calculate selling cost for a single bin
const revenue = mathCore.calculateBinSellCost(
  tokenAmount, // x: token amount to sell
  binQuantity, // q: current bin quantity
  totalTokens // T: total token quantity
);

// Calculate buying cost across multiple bins
const multiBinCost = mathCore.calculateMultiBinsBuyCost(
  tokenAmount, // x: token amount to purchase for each bin
  binQuantities, // qs: array of bin quantities
  totalTokens // T: total token quantity
);

// Calculate selling cost across multiple bins
const multiBinRevenue = mathCore.calculateMultiBinsSellCost(
  tokenAmount, // x: token amount to sell for each bin
  binQuantities, // qs: array of bin quantities
  totalTokens // T: total token quantity
);

// Calculate maximum purchasable tokens for a given budget
const maxTokens = mathCore.calculateXForMultiBins(
  budget, // Maximum budget to spend
  binQuantities, // qs: array of bin quantities
  totalTokens // T: total token quantity
);
```

For detailed documentation, installation instructions, and usage examples of the WASM package, see the [WASM Package README](../programs/range-bet-program/pkg-wasm/README.md).

## Example Scripts
