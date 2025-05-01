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

pub fn toggle_market_status(ctx: Context<ToggleMarketStatus>, active: bool) -> Result<()> {
    // Market의 Pubkey를 사용하여 market_id 추출
    let market_pubkey = ctx.accounts.market.key();
    let market_id_bytes = market_pubkey.to_bytes();
    let market_id = u64::from_le_bytes(market_id_bytes[0..8].try_into().unwrap_or([0; 8]));
    
    // 마켓 활성/비활성화 상태 토글
    let market = &mut ctx.accounts.market;
    market.active = active;
    
    if active {
        msg!("마켓 활성화: ID = {}", market_id);
    } else {
        msg!("마켓 비활성화: ID = {}", market_id);
    }
    
    Ok(())
} 