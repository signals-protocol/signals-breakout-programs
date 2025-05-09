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

/// Instruction (view) to calculate the cost of buying tokens in a specific bin
pub fn calculate_bin_cost(
    ctx: Context<CalculateBinCost>,
    index: u16,
    amount: u64,
) -> Result<u64> {
    let market = &ctx.accounts.market;
    
    // Check array index range
    require!(
        (index as usize) < market.bins.len(), 
        RangeBetError::BinIndexOutOfRange
    );
    
    // Get current q value for the selected bin
    let bin_q = market.bins[index as usize];
    
    // Calculate cost
    let cost = RangeBetMath::calculate_cost(amount, bin_q, market.t_total)?;
    
    Ok(cost)
}

/// Instruction (view) to calculate the amount of tokens purchasable for a specific cost
pub fn calculate_x_for_bin(
    ctx: Context<CalculateXForBin>,
    index: u16,
    cost: u64,
) -> Result<u64> {
    let market = &ctx.accounts.market;
    
    // Check array index range
    require!(
        (index as usize) < market.bins.len(), 
        RangeBetError::BinIndexOutOfRange
    );
    
    // Get current q value for the selected bin
    let bin_q = market.bins[index as usize];
    
    // Calculate amount
    let amount = RangeBetMath::calculate_x_for_cost(cost, bin_q, market.t_total)?;
    
    Ok(amount)
}

/// Instruction (view) to calculate the revenue from selling tokens in a specific bin
pub fn calculate_bin_sell_cost(
    ctx: Context<CalculateBinSellCost>,
    index: u16,
    amount: u64,
) -> Result<u64> {
    let market = &ctx.accounts.market;
    
    // Check array index range
    require!(
        (index as usize) < market.bins.len(), 
        RangeBetError::BinIndexOutOfRange
    );
    
    // Get current q value for the selected bin
    let bin_q = market.bins[index as usize];
    
    // Cannot sell if market is empty or bin is empty
    require!(bin_q > 0, RangeBetError::CannotSellFromEmptyBin);
    
    // Error if sell amount is greater than bin token amount
    require!(amount <= bin_q, RangeBetError::CannotSellMoreThanBin);
    
    // Calculate sell revenue
    let sell_revenue = RangeBetMath::calculate_sell_cost(amount, bin_q, market.t_total)?;
    
    Ok(sell_revenue)
} 