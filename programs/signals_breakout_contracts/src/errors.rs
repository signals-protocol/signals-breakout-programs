use anchor_lang::prelude::*;

#[error_code]
pub enum RangeBetError {
    #[msg("시장이 활성화되지 않았습니다")]
    MarketNotActive,
    
    #[msg("시장이 이미 닫혔습니다")]
    MarketClosed,
    
    #[msg("Tick spacing은 0보다 커야 합니다")]
    InvalidTickSpacing,
    
    #[msg("Min tick은 tick spacing의 배수여야 합니다")]
    MinTickNotMultiple,
    
    #[msg("Max tick은 tick spacing의 배수여야 합니다")]
    MaxTickNotMultiple,
    
    #[msg("Min tick은 Max tick보다 작아야 합니다")]
    MinTickGreaterThanMax,
    
    #[msg("Bin 인덱스가 유효한 범위를 벗어났습니다")]
    BinIndexOutOfRange,
    
    #[msg("배열 길이가 일치해야 합니다")]
    ArrayLengthMismatch,
    
    #[msg("최소 하나의 bin에 베팅해야 합니다")]
    NoTokensToBuy,
    
    #[msg("비용이 최대 담보를 초과합니다")]
    CostExceedsMaxCollateral,
    
    #[msg("시장이 아직 닫히지 않았습니다")]
    MarketIsNotClosed,
    
    #[msg("승리 bin이 아닙니다")]
    NotWinningBin,
    
    #[msg("청구할 토큰이 없습니다")]
    NoTokensToClaim,
    
    #[msg("인출할 담보가 없습니다")]
    NoCollateralToWithdraw,
    
    #[msg("양도할 충분한 토큰이 없습니다")]
    InsufficientTokensToTransfer,
    
    #[msg("수학 연산에서 오버플로우 발생")]
    MathOverflow,
    
    #[msg("수학 연산에서 언더플로우 발생")]
    MathUnderflow,
    
    #[msg("Bin에 있는 것보다 더 많은 토큰을 판매할 수 없습니다")]
    CannotSellMoreThanBin,
    
    #[msg("전체 공급량보다 더 많은 토큰을 판매할 수 없습니다")]
    CannotSellMoreThanSupply,
    
    #[msg("시장 전체 공급량과 같은 양을 판매할 수 없습니다")]
    CannotSellEntireSupply,
    
    #[msg("판매 계산 중 언더플로우 발생")]
    SellCalculationUnderflow,
    
    #[msg("소유자만 이 작업을 수행할 수 있습니다")]
    OwnerOnly,
    
    #[msg("올바른 순서로 마켓을 닫아야 합니다")]
    IncorrectMarketOrderForClosing,
} 