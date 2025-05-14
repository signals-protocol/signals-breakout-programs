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
    
    // Set owner
    program_state.owner = ctx.accounts.initializer.key();
    // Initialize market count
    program_state.market_count = 0;
    // Initialize last closed market (None = no closed markets yet)
    program_state.last_closed_market = None;
    
    msg!("Program initialized: owner = {}", program_state.owner);
    
    Ok(())
} 