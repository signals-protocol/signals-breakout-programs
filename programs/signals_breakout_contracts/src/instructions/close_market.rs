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
    winning_bin: i64
) -> Result<()> {
    // 먼저 market key로부터 market_id 추출
    let market_id = ctx.accounts.market.key().to_bytes()[0..8]
        .try_into()
        .map(|bytes: [u8; 8]| u64::from_le_bytes(bytes))
        .map_err(|_| error!(RangeBetError::ArrayLengthMismatch))?;

    // 그 후 market 및 program_state 가변 참조 획득
    let market = &mut ctx.accounts.market;
    let program_state = &mut ctx.accounts.program_state;
        
    // 승리 Bin 유효성 검사
    let tick_spacing = market.tick_spacing;
    let min_tick = market.min_tick;
    let max_tick = market.max_tick;
    
    require!(winning_bin % tick_spacing as i64 == 0, RangeBetError::WinningBinNotMultiple);
    require!(
        winning_bin >= min_tick && winning_bin <= max_tick, 
        RangeBetError::WinningBinOutOfRange
    );
    
    // 마켓 상태 업데이트
    market.closed = true;
    market.winning_bin = winning_bin;
    
    // 프로그램 상태 업데이트
    program_state.last_closed_market = market_id as i64;
    
    // 이벤트 발생
    emit!(MarketClosed {
        market_id,
        winning_bin,
    });
    
    msg!("마켓 마감 완료: ID = {}, 승리 Bin = {}", market_id, winning_bin);
    
    Ok(())
} 