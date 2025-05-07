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
    
    #[msg("최소 하나의 bin에 베팅해야 합니다")]
    NoTokensToBuy,
    
    #[msg("비용이 최대 담보를 초과합니다")]
    CostExceedsMaxCollateral,
    
    #[msg("Market is not closed")]
    MarketIsNotClosed,
    
    #[msg("승리 bin이 아닙니다")]
    NotWinningBin,
    
    #[msg("청구할 토큰이 없습니다")]
    NoTokensToClaim,
    
    #[msg("No collateral to withdraw")]
    NoCollateralToWithdraw,
    
    #[msg("Insufficient balance")]
    InsufficientTokensToTransfer,
    
    #[msg("수학 연산에서 오버플로우 발생")]
    MathOverflow,
    
    #[msg("수학 연산에서 언더플로우 발생")]
    MathUnderflow,
    
    #[msg("Cannot sell more tokens than available in bin")]
    CannotSellMoreThanBin,
    
    #[msg("전체 공급량보다 더 많은 토큰을 판매할 수 없습니다")]
    CannotSellMoreThanSupply,
    
    #[msg("시장 전체 공급량과 같은 양을 판매할 수 없습니다")]
    CannotSellEntireSupply,
    
    #[msg("판매 계산 중 언더플로우 발생")]
    SellCalculationUnderflow,
    
    #[msg("Owner only function")]
    OwnerOnly,
    
    #[msg("올바른 순서로 마켓을 닫아야 합니다")]
    IncorrectMarketOrderForClosing,
    
    #[msg("Cannot transfer to self")]
    CannotTransferToSelf,
    
    #[msg("Cannot sell tokens from empty bin")]
    CannotSellFromEmptyBin,
} 