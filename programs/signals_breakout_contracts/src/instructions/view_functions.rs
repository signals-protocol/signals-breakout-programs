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
    bin_index: i64,
    amount: u64,
) -> Result<u64> {
    let market = &ctx.accounts.market;
    
    // Bin 유효성 검사
    require!(
        bin_index % market.tick_spacing as i64 == 0, 
        RangeBetError::BinIndexNotMultiple
    );
    require!(
        bin_index >= market.min_tick && bin_index <= market.max_tick, 
        RangeBetError::BinIndexOutOfRange
    );
    
    // 선택한 Bin의 현재 q 값 찾기
    let mut bin_q = 0;
    for bin in &market.bins {
        if bin.index == bin_index {
            bin_q = bin.q;
            break;
        }
    }
    
    // 코스트 계산
    let cost = RangeBetMath::calculate_cost(amount, bin_q, market.t_total)?;
    
    Ok(cost)
}

/// 특정 비용으로 구매 가능한 토큰 수량을 계산하는 Instruction(view)
pub fn calculate_x_for_bin(
    ctx: Context<CalculateXForBin>,
    bin_index: i64,
    cost: u64,
) -> Result<u64> {
    let market = &ctx.accounts.market;
    
    // Bin 유효성 검사
    require!(
        bin_index % market.tick_spacing as i64 == 0, 
        RangeBetError::BinIndexNotMultiple
    );
    require!(
        bin_index >= market.min_tick && bin_index <= market.max_tick, 
        RangeBetError::BinIndexOutOfRange
    );
    
    // 선택한 Bin의 현재 q 값 찾기
    let mut bin_q = 0;
    for bin in &market.bins {
        if bin.index == bin_index {
            bin_q = bin.q;
            break;
        }
    }
    
    // 수량 계산
    let amount = RangeBetMath::calculate_x_for_cost(cost, bin_q, market.t_total)?;
    
    Ok(amount)
}

/// 특정 Bin에서 토큰을 판매했을 때 얻는 수익을 계산하는 Instruction(view)
pub fn calculate_bin_sell_cost(
    ctx: Context<CalculateBinSellCost>,
    bin_index: i64,
    amount: u64,
) -> Result<u64> {
    let market = &ctx.accounts.market;
    
    // Bin 유효성 검사
    require!(
        bin_index % market.tick_spacing as i64 == 0, 
        RangeBetError::BinIndexNotMultiple
    );
    require!(
        bin_index >= market.min_tick && bin_index <= market.max_tick, 
        RangeBetError::BinIndexOutOfRange
    );
    
    // 선택한 Bin의 현재 q 값 찾기
    let mut bin_q = 0;
    for bin in &market.bins {
        if bin.index == bin_index {
            bin_q = bin.q;
            break;
        }
    }
    
    // 판매 수익 계산
    let sell_revenue = RangeBetMath::calculate_sell_cost(amount, bin_q, market.t_total)?;
    
    Ok(sell_revenue)
} 