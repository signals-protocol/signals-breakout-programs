use anchor_lang::prelude::*;

#[error_code]
pub enum RangeBetError {
    #[msg("Market is not active")]
    MarketNotActive,
    
    #[msg("Market is closed")]
    MarketClosed,
    
    #[msg("Tick spacing must be positive")]
    InvalidTickSpacing,
    
    #[msg("Min tick must be a multiple of tick spacing")]
    MinTickNotMultiple,
    
    #[msg("Max tick must be a multiple of tick spacing")]
    MaxTickNotMultiple,
    
    #[msg("Min tick must be less than max tick")]
    MinTickGreaterThanMax,
    
    #[msg("Bin index out of range")]
    BinIndexOutOfRange,
    
    #[msg("Array length mismatch")]
    ArrayLengthMismatch,
    
    #[msg("Must bet on at least one bin")]
    NoTokensToBuy,
    
    #[msg("Cost exceeds maximum collateral")]
    CostExceedsMaxCollateral,
    
    #[msg("Market is not closed")]
    MarketIsNotClosed,
    
    #[msg("Not a winning bin")]
    NotWinningBin,
    
    #[msg("No tokens to claim")]
    NoTokensToClaim,
    
    #[msg("No collateral to withdraw")]
    NoCollateralToWithdraw,
    
    #[msg("Insufficient balance")]
    InsufficientTokensToTransfer,
    
    #[msg("Math overflow occurred")]
    MathOverflow,
    
    #[msg("Math underflow occurred")]
    MathUnderflow,
    
    #[msg("Cannot sell more tokens than available in bin")]
    CannotSellMoreThanBin,
    
    #[msg("Cannot sell more tokens than total supply")]
    CannotSellMoreThanSupply,
    
    #[msg("Cannot sell entire market supply")]
    CannotSellEntireSupply,
    
    #[msg("Sell calculation underflow")]
    SellCalculationUnderflow,
    
    #[msg("Owner only function")]
    OwnerOnly,
    
    #[msg("Markets must be closed in sequential order")]
    IncorrectMarketOrderForClosing,
    
    #[msg("Cannot transfer to self")]
    CannotTransferToSelf,
    
    #[msg("Cannot sell tokens from empty bin")]
    CannotSellFromEmptyBin,
    
    #[msg("Bin token quantity cannot be greater than total token quantity")]
    InvalidBinState,
} 