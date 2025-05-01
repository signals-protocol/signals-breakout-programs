use anchor_lang::prelude::*;
use crate::state::ProgramState;

#[derive(Accounts)]
pub struct InitializeProgram<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    
    #[account(
        init,
        payer = initializer,
        space = 8 + std::mem::size_of::<ProgramState>(),
        seeds = [b"range-bet-state"],
        bump
    )]
    pub program_state: Account<'info, ProgramState>,
    
    pub system_program: Program<'info, System>,
}

pub fn initialize_program(ctx: Context<InitializeProgram>) -> Result<()> {
    let program_state = &mut ctx.accounts.program_state;
    
    // 소유자 설정
    program_state.owner = ctx.accounts.initializer.key();
    // 마켓 카운트 초기화
    program_state.market_count = 0;
    // 마지막으로 닫힌 마켓 초기화 (-1 = 아직 없음)
    program_state.last_closed_market = -1;
    
    msg!("프로그램 초기화 완료: 소유자 = {}", program_state.owner);
    
    Ok(())
} 