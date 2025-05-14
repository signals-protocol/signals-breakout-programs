use anchor_lang::prelude::*;

/// Global configuration and metadata storage
#[account]
pub struct ProgramState {
    pub owner: Pubkey,           // Program owner (admin)
    pub market_count: u64,       // Market ID sequence (auto-increment)
    pub last_closed_market: Option<u64>, // Most recently closed market ID (None = not yet closed)
}

/// Market state structure
#[account]
pub struct Market {
    pub active: bool,
    pub closed: bool,
    pub tick_spacing: u32,
    pub min_tick: i64,
    pub max_tick: i64,
    pub t_total: u64,           // Total sum of all bin tokens (T)
    pub collateral_balance: u64,
    pub winning_bin: Option<u16>,  // Winning bin index (determined at close, None=undetermined)
    pub open_ts: i64,           // When the market was opened
    pub close_ts: i64,          // When the market is scheduled to close
    
    // Fixed offset array for bin storage
    // Index is calculated as (bin_index - min_tick) / tick_spacing
    // Values represent token quantity (q) in each bin
    pub bins: Vec<u64>,
}

/// User position structure for a specific market
#[account]
pub struct UserMarketPosition {
    pub owner: Pubkey,       // Position owner
    pub market_id: u64,
    
    // Internal ledger
    pub bins: Vec<BinBal>,
}

/// BinBal structure (stored within user position)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BinBal {
    pub index: u16, // Array index value (0-based index)
    pub amount: u64,
}

/// Event definitions
#[event]
pub struct MarketCreated {
    pub market_id: u64,
    pub tick_spacing: u32,
    pub min_tick: i64,
    pub max_tick: i64,
}

#[event]
pub struct TokensBought {
    pub market_id: u64,
    pub buyer: Pubkey,
    pub total_cost: u64,
}

#[event]
pub struct MarketClosed {
    pub market_id: u64,
    pub winning_bin: u16,
}

#[event]
pub struct RewardClaimed {
    pub market_id: u64,
    pub claimer: Pubkey,
    pub amount: u64,
}

#[event]
pub struct CollateralOut {
    pub to: Pubkey,
    pub amount: u64,
} 