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
    // 다음 닫을 마켓 ID 계산
    let expected_next_id = match ctx.accounts.program_state.last_closed_market {
        Some(last_id) => last_id + 1,
        None => 0 // 아직 닫힌 마켓이 없으면 0번 마켓부터 시작
    };
    
    // 전달된 ID가 다음 닫아야 할 마켓 ID와 일치하는지 검증
    require!(
        market_id == expected_next_id, 
        RangeBetError::IncorrectMarketOrderForClosing
    );
    
    // 가변 참조 획득
    let market = &mut ctx.accounts.market;
    let program_state = &mut ctx.accounts.program_state;
        
    // winning_bin 인덱스가 bins 배열 범위 내에 있는지 확인
    require!(
        (winning_bin as usize) < market.bins.len(),
        RangeBetError::BinIndexOutOfRange
    );
    
    // 마켓 상태 업데이트
    market.closed = true;
    market.winning_bin = Some(winning_bin);
    
    // 프로그램 상태 업데이트 - 마지막으로 닫힌 마켓 ID 갱신
    program_state.last_closed_market = Some(market_id);
    
    // 이벤트 발생
    emit!(MarketClosed {
        market_id,
        winning_bin,
    });
    
    msg!("마켓 마감 완료: ID = {}, 승리 Bin 인덱스 = {}", market_id, winning_bin);
    
    Ok(())
} 