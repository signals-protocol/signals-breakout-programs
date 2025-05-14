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
    // Toggle market active/inactive status
    let market = &mut ctx.accounts.market;
    market.active = active;

    if active {
        msg!("Market activated: ID = {}", market_id);
    } else {
        msg!("Market deactivated: ID = {}", market_id);
    }
    
    Ok(())
} 