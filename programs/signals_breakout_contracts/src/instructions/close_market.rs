use anchor_lang::prelude::*;
use crate::state::{ProgramState, Market, MarketClosed};
use crate::errors::RangeBetError;

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct CloseMarket<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        mut,
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

pub fn close_market(
    ctx: Context<CloseMarket>, 
    market_id: u64,
    winning_bin: u16
) -> Result<()> {
    // Calculate next market ID to close
    let expected_next_id = match ctx.accounts.program_state.last_closed_market {
        Some(last_id) => last_id + 1,
        None => 0 // If no closed markets exist, start from market 0
    };
    
    // Verify if the passed ID matches the next market ID to close
    require!(
        market_id == expected_next_id, 
        RangeBetError::IncorrectMarketOrderForClosing
    );
    
    // Get mutable references
    let market = &mut ctx.accounts.market;
    let program_state = &mut ctx.accounts.program_state;
        
    // Check if the winning bin index is within the bins array range
    require!(
        (winning_bin as usize) < market.bins.len(),
        RangeBetError::BinIndexOutOfRange
    );
    
    // Update market state
    market.closed = true;
    market.winning_bin = Some(winning_bin);
    
    // Update program state - last closed market ID
    program_state.last_closed_market = Some(market_id);
    
    // Emit event
    emit!(MarketClosed {
        market_id,
        winning_bin,
    });
    
    msg!("Market closed: ID = {}, Winning bin index = {}", market_id, winning_bin);
    
    Ok(())
} 