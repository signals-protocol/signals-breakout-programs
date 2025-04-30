use anchor_lang::prelude::*;

declare_id!("9FTUsmsohGA7FUXiDbJEbP1BV9ifzukhEH3dAfiZxfpe");

#[program]
pub mod signals_breakout_contracts {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
