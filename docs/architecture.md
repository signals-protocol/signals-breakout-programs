# Signals Breakout Contracts Architecture

This document describes the architecture and core concepts of the Signals Breakout Contracts project.

## Overview

Signals Breakout Contracts is a prediction market protocol operating on the Solana blockchain, enabling prediction betting on various markets. This protocol uses an approach similar to Uniswap V3's tick-based system to divide price ranges into segments (bins) and calculates betting costs using a unique price formula.

## Core Components

### Program State (ProgramState)

Program state is an account that stores global settings and metadata.

```rust
#[account]
pub struct ProgramState {
    pub owner: Pubkey,           // Program owner (administrator)
    pub market_count: u64,       // Market ID sequence (auto-incrementing)
    pub last_closed_market: Option<u64>, // Most recently closed market ID (None = not yet closed)
}
```

### Market (Market)

A market is the unit where prediction betting takes place, defining tick ranges, bin status, and closing conditions.

```rust
#[account]
pub struct Market {
    pub active: bool,           // Active status
    pub closed: bool,           // Closed status
    pub tick_spacing: u32,      // Tick spacing
    pub min_tick: i64,          // Minimum tick
    pub max_tick: i64,          // Maximum tick
    pub t_total: u64,           // Total sum of tokens across all bins (T)
    pub collateral_balance: u64, // Collateral balance
    pub winning_bin: Option<u16>, // Winning bin index (determined at closing, None=undetermined)
    pub open_ts: i64,           // Time when market was opened
    pub close_ts: i64,          // Time when market is scheduled to close

    // Fixed offset array for bin storage
    // Index is calculated as (bin_index - min_tick) / tick_spacing
    // Values represent token quantities (q) in each bin
    pub bins: Vec<u64>,
}
```

### User Market Position (UserMarketPosition)

User position tracks tokens held and betting status of a user in a specific market.

```rust
#[account]
pub struct UserMarketPosition {
    pub owner: Pubkey,      // Position owner
    pub market_id: u64,     // Market ID

    // Internal ledger
    pub bins: Vec<BinBal>,  // Balances by bin
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BinBal {
    pub index: u16,  // Array index value (0-based index)
    pub amount: u64, // Token quantity
}
```

### Math Core (External Crate)

The mathematical functions for price calculations are implemented in a separate crate called `math-core`. This allows for modular development and reuse of the mathematical model.

Key features of the Math Core:

- Separate compilation targets for on-chain (BPF) and WASM
- Implementation of the (q+t)/(T+t) price formula
- Functions for buying and selling in single and multiple bins
- Robust error handling and overflow protection

The WASM build is available as an NPM package named `range-bet-math-core` for use in client applications. For detailed documentation and examples, see the [WASM Package README](../programs/range-bet-program/pkg-wasm/README.md).

### Collateral Token Faucet (Support Program)

The Collateral Token Faucet is a utility program that provides test tokens for development and testing environments. It's designed to work alongside the main prediction market protocol.

```rust
#[program]
pub mod collateral_token_faucet {
    // Program instructions
    pub fn initialize(ctx: Context<Initialize>) -> Result<()>
    pub fn mint_collateral_tokens(ctx: Context<MintCollateralTokens>, amount: u64) -> Result<()>
}

#[derive(Accounts)]
pub struct MintCollateralTokens<'info> {
    // Account validation
    pub mint: Account<'info, Mint>,
    pub faucet_pda: UncheckedAccount<'info>,
    pub receiver: Account<'info, TokenAccount>,
    pub user: Signer<'info>,
    // System accounts
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}
```

The faucet program uses a PDA (Program Derived Address) as the mint authority for collateral tokens:

1. **Faucet PDA**:
   - Seeds: `["collateral_faucet"]`
   - Role: Acts as the mint authority for the collateral token mint

This utility program enables seamless testing of the prediction market protocol by allowing users to mint test tokens that can be used for betting.

## Main Instructions

### Program Initialization (initializeProgram)

Sets up and initializes the program for first use.

### Market Creation (createMarket)

Creates a new prediction market. Sets tick spacing, min/max ticks, and closing time.

```rust
pub fn create_market(
    ctx: Context<CreateMarket>,
    tick_spacing: u32,
    min_tick: i64,
    max_tick: i64,
    close_ts: i64,
) -> Result<()>
```

### Token Purchase (buyTokens)

Allows users to bet on various bins in a specific market.

```rust
pub fn buy_tokens(
    ctx: Context<BuyTokens>,
    _market_id: u64,
    bin_indices: Vec<u16>,
    amounts: Vec<u64>,
    max_collateral: u64,
) -> Result<()>
```

### Market Closing (closeMarket)

Administrator closes the market and sets the winning bin.

```rust
pub fn close_market(
    ctx: Context<CloseMarket>,
    market_id: u64,
    winning_bin: u16,
) -> Result<()>
```

### Reward Claiming (claimReward)

Token holders of the winning bin claim their rewards.

```rust
pub fn claim_reward(
    ctx: Context<ClaimReward>,
) -> Result<()>
```

### Position Transfer (transferPosition)

Users can transfer their position to another address.

```rust
pub fn transfer_position(
    ctx: Context<TransferPosition>,
    market_id: u64,
    bin_indices: Vec<u16>,
    amounts: Vec<u64>,
) -> Result<()>
```

## Account Structure

The program uses the following PDA (Program Derived Address) account structure:

1. **Program State**:

   - Seeds: `["range-bet-state"]`

2. **Market**:

   - Seeds: `["market", market_id.to_le_bytes()]`

3. **User Position**:

   - Seeds: `["pos", user_pubkey, market_id.to_le_bytes()]`

4. **Vault Authority**:
   - Seeds: `["vault", market_id.to_le_bytes()]`

## Mathematical Model

The betting system uses a special integral formula to calculate costs:

```
Cost = ∫(q+t)/(T+t) dt, from 0 to x
```

Where:

- `q`: Current token quantity in the bin
- `T`: Total token supply in the entire market
- `t`: Integration variable
- `x`: Token quantity to purchase

This formula allows costs to be dynamically adjusted according to liquidity. The more bets placed on a specific range, the higher the cost of betting on that range.

For the detailed implementation of these mathematical functions, refer to the `math-core` crate in the project.

## Events

The program emits the following events:

- **MarketCreated**: When a new market is created
- **TokensBought**: When tokens are purchased
- **MarketClosed**: When a market is closed
- **RewardClaimed**: When rewards are claimed
- **CollateralOut**: When collateral is withdrawn
