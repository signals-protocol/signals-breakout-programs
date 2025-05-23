# Signals Breakout Contracts API Reference

This document provides a detailed reference for all instructions and account structures of the Signals Breakout Contracts Solana program.

## Program ID

```
97i8BgDJG6yZggN2Di5UnERs6X5PqYqnkSvkMdvw1d5J
```

## Account Structures

### ProgramState

An account that stores overall program settings and state.

```rust
pub struct ProgramState {
    pub owner: Pubkey,           // Program owner (administrator)
    pub market_count: u64,       // Market ID sequence (auto-incrementing)
    pub last_closed_market: Option<u64>, // Most recently closed market ID
}
```

### Market

An account that stores the state and configuration of individual prediction markets.

```rust
pub struct Market {
    pub active: bool,           // Active status
    pub closed: bool,           // Closed status
    pub tick_spacing: u32,      // Tick spacing
    pub min_tick: i64,          // Minimum tick
    pub max_tick: i64,          // Maximum tick
    pub t_total: u64,           // Total sum of tokens across all bins (T)
    pub collateral_balance: u64, // Collateral balance
    pub winning_bin: Option<u16>, // Winning bin index
    pub open_ts: i64,           // Time when market was opened
    pub close_ts: i64,          // Time when market is scheduled to close
    pub bins: Vec<u64>,         // Token quantities by bin
}
```

### UserMarketPosition

An account that tracks a user's position in a specific market.

```rust
pub struct UserMarketPosition {
    pub owner: Pubkey,      // Position owner
    pub market_id: u64,     // Market ID
    pub bins: Vec<BinBal>,  // Balances by bin
}

pub struct BinBal {
    pub index: u16,  // Bin index
    pub amount: u64, // Token quantity
}
```

## Instructions

### initialize_program

Initializes and sets up the program for first use.

**Parameters**: None

**Accounts**:

- `initializer`: User performing the initialization (signature required)
- `program_state`: Program state account (PDA)
- `system_program`: System program
- `rent`: Rent Sysvar

**Example**:

```typescript
await program.methods
  .initializeProgram()
  .accounts({
    initializer: wallet.publicKey,
  })
  .signers([wallet])
  .rpc();
```

### create_market

Creates a new prediction market.

**Parameters**:

- `tick_spacing`: u32 - Tick spacing
- `min_tick`: i64 - Minimum tick value
- `max_tick`: i64 - Maximum tick value
- `close_ts`: i64 - Scheduled market closing time (Unix timestamp)

**Accounts**:

- `owner`: Market creator (program owner)
- `program_state`: Program state account
- `market`: New market account to create
- `collateral_mint`: Collateral token Mint
- `vault`: Market's collateral token storage account
- `vault_authority`: Vault authority PDA
- `token_program`: Token program
- `associated_token_program`: Associated token program
- `system_program`: System program
- `rent`: Rent Sysvar

**Example**:

```typescript
await program.methods
  .createMarket(
    20, // tick_spacing
    new BN(-240), // min_tick
    new BN(960), // max_tick
    new BN(closeTime) // close_ts
  )
  .accounts({
    owner: wallet.publicKey,
    collateralMint: COLLATERAL_MINT,
  })
  .signers([wallet])
  .rpc();
```

### buy_tokens

Purchases tokens in multiple bins of a specific market.

**Parameters**:

- `market_id`: u64 - Market ID
- `bin_indices`: Vec<u16> - Array of bin indices to purchase
- `amounts`: Vec<u64> - Array of token quantities to purchase for each bin
- `max_collateral`: u64 - Maximum collateral willing to pay

**Accounts**:

- `user`: Token purchaser (signature required)
- `market`: Market account
- `user_position`: User's market position account
- `user_token_account`: User's collateral token account
- `vault`: Market's collateral token account
- `token_program`: Token program
- `system_program`: System program
- `rent`: Rent Sysvar

**Example**:

```typescript
await program.methods
  .buyTokens(marketId, [0, 3], [100000000, 50000000], 200000000)
  .accounts({
    user: wallet.publicKey,
    userTokenAccount: userTokenAccount,
    vault: marketVault,
  })
  .signers([wallet])
  .rpc();
```

### close_market

Closes a market and sets the winning bin.

**Parameters**:

- `market_id`: u64 - Market ID
- `winning_bin`: u16 - Winning bin index

**Accounts**:

- `authority`: Market administrator (signature required)
- `market`: Market account
- `program_state`: Program state account

**Example**:

```typescript
await program.methods
  .closeMarket(marketId, winningBin)
  .accounts({
    authority: wallet.publicKey,
  })
  .signers([wallet])
  .rpc();
```

### claim_reward

Allows users who bet on the winning bin to claim their rewards.

**Parameters**: None

**Accounts**:

- `user`: Reward claimer (signature required)
- `market`: Market account
- `user_position`: User's market position account
- `user_token_account`: User's collateral token account
- `vault`: Market's collateral token account
- `vault_authority`: Vault authority PDA
- `token_program`: Token program

**Example**:

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

### activate_market

Changes a market's active status.

**Parameters**:

- `market_id`: u64 - Market ID
- `active`: bool - Activation status (true: activate, false: deactivate)

**Accounts**:

- `owner`: Market administrator (signature required)
- `market`: Market account

**Example**:

```typescript
await program.methods
  .activateMarket(marketId, true)
  .accounts({
    owner: wallet.publicKey,
  })
  .signers([wallet])
  .rpc();
```

### withdraw_collateral

Withdraws remaining collateral from a closed market.

**Parameters**:

- `market_id`: u64 - Market ID

**Accounts**:

- `owner`: Market administrator (signature required)
- `market`: Market account
- `vault`: Market's collateral token account
- `vault_authority`: Vault authority PDA
- `owner_token_account`: Administrator's collateral token account
- `token_program`: Token program

**Example**:

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

### transfer_position

Transfers part or all of a market position to another user.

**Parameters**:

- `market_id`: u64 - Market ID
- `bin_indices`: Vec<u16> - Array of bin indices to transfer
- `amounts`: Vec<u64> - Array of token quantities to transfer for each bin

**Accounts**:

- `user`: Position owner (signature required)
- `recipient`: Recipient address
- `market`: Market account
- `user_position`: User's market position account
- `recipient_position`: Recipient's market position account
- `system_program`: System program
- `rent`: Rent Sysvar

**Example**:

```typescript
await program.methods
  .transferPosition(marketId, binIndices, amounts)
  .accounts({
    fromUser: wallet.publicKey,
    toUser: recipientPubkey,
  })
  .signers([wallet])
  .rpc();
```

## Simulation Functions

### calculate_bin_cost

Calculates the cost to purchase tokens in a specific bin.

**Parameters**:

- `market_id`: u64 - Market ID
- `index`: u16 - Bin index
- `amount`: u64 - Token quantity to purchase

**Accounts**:

- `market`: Market account

**Return Value**: u64 - Purchase cost

**Example**:

```typescript
const cost = await program.methods
  .calculateBinCost(marketId, binIndex, amount)
  .accounts({})
  .view();
```

### calculate_x_for_bin

Calculates the token quantity that can be purchased with a specific cost.

**Parameters**:

- `market_id`: u64 - Market ID
- `index`: u16 - Bin index
- `cost`: u64 - Cost to pay

**Accounts**:

- `market`: Market account

**Return Value**: u64 - Purchasable token quantity

**Example**:

```typescript
const tokens = await program.methods
  .calculateXForBin(marketId, binIndex, cost)
  .accounts({})
  .view();
```

### calculate_bin_sell_cost

Calculates the collateral amount that can be obtained by selling tokens.

**Parameters**:

- `market_id`: u64 - Market ID
- `index`: u16 - Bin index
- `amount`: u64 - Token quantity to sell

**Accounts**:

- `market`: Market account

**Return Value**: u64 - Sale proceeds

**Example**:

```typescript
const sellCost = await program.methods
  .calculateBinSellCost(marketId, binIndex, amount)
  .accounts({})
  .view();
```

## Math Core API

The mathematical functions are implemented in a separate `math-core` crate to allow reuse across different contexts. This crate provides functions for both on-chain and WASM (client-side) use.

The WASM build is published as an NPM package named `range-bet-math-core`. For detailed documentation and usage examples of the WASM package, see the [WASM Package README](../programs/range-bet-program/pkg-wasm/README.md).

### Core Functions

#### calculate_bin_buy_cost

Calculates the cost to purchase tokens in a single bin.

```rust
pub fn calculate_bin_buy_cost(x: u64, q: u64, t: u64) -> Result<u64>
```

Parameters:

- `x`: Amount of tokens to purchase
- `q`: Current token quantity in the bin
- `t`: Total token quantity in the market

Returns:

- `Result<u64>`: Cost of the purchase or an error

#### calculate_bin_sell_cost

Calculates the revenue from selling tokens in a single bin.

```rust
pub fn calculate_bin_sell_cost(x: u64, q: u64, t: u64) -> Result<u64>
```

Parameters:

- `x`: Amount of tokens to sell
- `q`: Current token quantity in the bin
- `t`: Total token quantity in the market

Returns:

- `Result<u64>`: Revenue from the sale or an error

#### calculate_multi_bins_buy_cost

Calculates the total cost to purchase tokens across multiple bins.

```rust
pub fn calculate_multi_bins_buy_cost(x: u64, qs: &[u64], t: u64) -> Result<u64>
```

Parameters:

- `x`: Amount of tokens to purchase for each bin
- `qs`: Array of current token quantities in each bin
- `t`: Total token quantity in the market

Returns:

- `Result<u64>`: Total cost of the purchase or an error

#### calculate_multi_bins_sell_cost

Calculates the revenue from selling tokens across multiple bins.

```rust
pub fn calculate_multi_bins_sell_cost(x: u64, qs: &[u64], t: u64) -> Result<u64>
```

Parameters:

- `x`: Amount of tokens to sell for each bin
- `qs`: Array of current token quantities in each bin
- `t`: Total token quantity in the market

Returns:

- `Result<u64>`: Total revenue from the sale or an error

#### calculate_x_for_multi_bins

Calculates the maximum token quantity that can be purchased with a given budget across multiple bins.

```rust
pub fn calculate_x_for_multi_bins(budget: u64, qs: &[u64], t: u64) -> Result<u64>
```

Parameters:

- `budget`: Maximum cost willing to pay
- `qs`: Array of current token quantities in each bin
- `t`: Total token quantity in the market

Returns:

- `Result<u64>`: Maximum purchasable token quantity or an error

## Events

### MarketCreated

Event emitted when a market is created.

```rust
pub struct MarketCreated {
    pub market_id: u64,
    pub tick_spacing: u32,
    pub min_tick: i64,
    pub max_tick: i64,
}
```

### TokensBought

Event emitted when tokens are purchased.

```rust
pub struct TokensBought {
    pub market_id: u64,
    pub buyer: Pubkey,
    pub total_cost: u64,
}
```

### MarketClosed

Event emitted when a market is closed.

```rust
pub struct MarketClosed {
    pub market_id: u64,
    pub winning_bin: u16,
}
```

### RewardClaimed

Event emitted when rewards are claimed.

```rust
pub struct RewardClaimed {
    pub market_id: u64,
    pub claimer: Pubkey,
    pub amount: u64,
}
```

### CollateralOut

Event emitted when collateral is withdrawn.

```rust
pub struct CollateralOut {
    pub to: Pubkey,
    pub amount: u64,
}
```

## Collateral Token Faucet API

### Program ID

```
DDFXv1hETR8pQSpNbzCxTX7jm1Hr57V4oihDGosXQfgC
```

### Account Structures

#### FaucetPDA

A PDA derived from the seed "collateral_faucet" that serves as the mint authority for the collateral token.

```rust
// This is a PDA without data structure - UncheckedAccount is used in the account validation
```

### Instructions

#### initialize

Initializes the faucet program.

**Parameters**: None

**Accounts**:

- No specific accounts required

**Example**:

```typescript
await faucetProgram.methods.initialize().rpc();
```

#### mintCollateralTokens

Mints collateral tokens to a specified receiver.

**Parameters**:

- `amount`: u64 - The amount of tokens to mint (in raw units, accounting for decimals)

**Accounts**:

- `mint`: Collateral token mint account (must have Faucet PDA as authority)
- `faucet_pda`: The PDA that acts as mint authority
- `receiver`: Token account to receive the minted tokens
- `user`: The transaction signer (pays fees)
- `token_program`: Token program
- `system_program`: System program
- `associated_token_program`: Associated token program
- `rent`: Rent Sysvar

**Example**:

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

For detailed information about the Collateral Token Faucet, see the [Collateral Token Faucet Documentation](./collateral-token-faucet.md).
