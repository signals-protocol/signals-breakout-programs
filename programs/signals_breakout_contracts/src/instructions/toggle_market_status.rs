use anchor_lang::prelude::*;
use crate::state::{ProgramState, Market};
use crate::errors::RangeBetError;

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct ToggleMarketStatus<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        seeds = [b"range-bet-state"],
        bump,
        constraint = program_state.owner == owner.key() @ RangeBetError::OwnerOnly
    )]
    pub program_state: Account<'info, ProgramState>,
    
    #[account(
        mut,
        seeds = [b"market", &market_id.to_le_bytes()],
        bump,
        constraint = !market.closed @ RangeBetError::MarketClosed
    )]
    pub market: Account<'info, Market>,
}

pub fn toggle_market_status(ctx: Context<ToggleMarketStatus>, market_id: u64, active: bool) -> Result<()> {
    // 마켓 활성/비활성화 상태 토글
    let market = &mut ctx.accounts.market;
    market.active = active;

    require!(
        market.closed,
        RangeBetError::MarketClosed
    );

    if active {
        msg!("마켓 활성화: ID = {}", market_id);
    } else {
        msg!("마켓 비활성화: ID = {}", market_id);
    }
    
    Ok(())
} 