use anchor_lang::prelude::*;
use crate::state::Market; 
use crate::errors::RangeBetError;
use crate::math::RangeBetMath;

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct CalculateBinCost<'info> {
    #[account(
        seeds = [b"market", &market_id.to_le_bytes()],
        bump,
        constraint = market.active @ RangeBetError::MarketNotActive,
        constraint = !market.closed @ RangeBetError::MarketClosed
    )]
    pub market: Account<'info, Market>,
}

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct CalculateXForBin<'info> {
    #[account(
        seeds = [b"market", &market_id.to_le_bytes()],
        bump,
        constraint = market.active @ RangeBetError::MarketNotActive,
        constraint = !market.closed @ RangeBetError::MarketClosed
    )]
    pub market: Account<'info, Market>,
}

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct CalculateBinSellCost<'info> {
    #[account(
        seeds = [b"market", &market_id.to_le_bytes()],
        bump
    )]
    pub market: Account<'info, Market>,
}

/// 특정 Bin에 토큰을 구매하는 비용을 계산하는 Instruction(view)
pub fn calculate_bin_cost(
    ctx: Context<CalculateBinCost>,
    index: u16,
    amount: u64,
) -> Result<u64> {
    let market = &ctx.accounts.market;
    
    // 배열 인덱스 범위 확인
    require!(
        (index as usize) < market.bins.len(), 
        RangeBetError::BinIndexOutOfRange
    );
    
    // 선택한 Bin의 현재 q 값 가져오기
    let bin_q = market.bins[index as usize];
    
    // 코스트 계산
    let cost = RangeBetMath::calculate_cost(amount, bin_q, market.t_total)?;
    
    Ok(cost)
}

/// 특정 비용으로 구매 가능한 토큰 수량을 계산하는 Instruction(view)
pub fn calculate_x_for_bin(
    ctx: Context<CalculateXForBin>,
    index: u16,
    cost: u64,
) -> Result<u64> {
    let market = &ctx.accounts.market;
    
    // 배열 인덱스 범위 확인
    require!(
        (index as usize) < market.bins.len(), 
        RangeBetError::BinIndexOutOfRange
    );
    
    // 선택한 Bin의 현재 q 값 가져오기
    let bin_q = market.bins[index as usize];
    
    // 수량 계산
    let amount = RangeBetMath::calculate_x_for_cost(cost, bin_q, market.t_total)?;
    
    Ok(amount)
}

/// 특정 Bin에서 토큰을 판매했을 때 얻는 수익을 계산하는 Instruction(view)
pub fn calculate_bin_sell_cost(
    ctx: Context<CalculateBinSellCost>,
    index: u16,
    amount: u64,
) -> Result<u64> {
    let market = &ctx.accounts.market;
    
    // 배열 인덱스 범위 확인
    require!(
        (index as usize) < market.bins.len(), 
        RangeBetError::BinIndexOutOfRange
    );
    
    // 선택한 Bin의 현재 q 값 가져오기
    let bin_q = market.bins[index as usize];
    
    // 빈 마켓이거나 빈 bin인 경우 판매 불가
    require!(bin_q > 0, RangeBetError::CannotSellFromEmptyBin);
    
    // 판매 수량이 bin의 토큰 수량보다 많으면 에러
    require!(amount <= bin_q, RangeBetError::CannotSellMoreThanBin);
    
    // 판매 수익 계산
    let sell_revenue = RangeBetMath::calculate_sell_cost(amount, bin_q, market.t_total)?;
    
    Ok(sell_revenue)
} 