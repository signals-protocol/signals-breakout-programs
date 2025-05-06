use anchor_lang::prelude::*;

/// 전역 설정 및 메타데이터 저장
#[account]
pub struct ProgramState {
    pub owner: Pubkey,           // 프로그램 소유자 (관리자)
    pub market_count: u64,       // Market ID 시퀀스 (auto-increment)
    pub last_closed_market: Option<u64>, // 가장 최근에 close된 market ID (None = 아직 없음)
}

/// Market 상태 구조체
#[account]
pub struct Market {
    pub active: bool,
    pub closed: bool,
    pub tick_spacing: u32,
    pub min_tick: i64,
    pub max_tick: i64,
    pub t_total: u64,           // 전체 Bin 토큰 합(T)
    pub collateral_balance: u64,
    pub winning_bin: Option<u16>,  // 승리 Bin 인덱스 (close 시 결정, None=미결정)
    pub open_ts: i64,           // 시장이 열린 시간
    pub close_ts: i64,          // 시장이 닫힐(메타) 예정 시각
    
    // 고정 오프셋 배열로 Bin 저장
    // 인덱스는 (bin_index - min_tick) / tick_spacing으로 계산
    // 값은 각 bin의 토큰 수량 (q)
    pub bins: Vec<u64>,
}

/// 특정 유저가 특정 Market에서 보유한 포지션 구조체
#[account]
pub struct UserMarketPosition {
    pub owner: Pubkey,       // 포지션 소유자
    pub market_id: u64,
    
    // 내부 레저
    pub bins: Vec<BinBal>,
}

/// BinBal 구조체 (유저 포지션 내 저장)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BinBal {
    pub index: u16, // 배열 인덱스 값 (0부터 시작하는 인덱스)
    pub amount: u64,
}

/// 이벤트 정의
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