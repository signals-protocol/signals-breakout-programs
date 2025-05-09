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
