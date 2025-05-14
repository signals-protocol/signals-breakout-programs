use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    token::{Mint, TokenAccount, Token},
    associated_token::AssociatedToken,
};
use crate::state::{ProgramState, Market, MarketCreated};
use crate::errors::RangeBetError;

#[derive(Accounts)]
#[instruction(tick_spacing: u32, min_tick: i64, max_tick: i64, close_ts: i64)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"range-bet-state"],
        bump,
        constraint = program_state.owner == owner.key() @ RangeBetError::OwnerOnly
    )]
    pub program_state: Account<'info, ProgramState>,
    
    /// Account to store market information to be created
    #[account(
        init,
        payer = owner,
        seeds = [b"market", program_state.market_count.to_le_bytes().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<Market>() // Minimum size (no bins)
    )]
    pub market: Account<'info, Market>,
    
    /// Collateral token Mint
    pub collateral_mint: Account<'info, Mint>,
    
    /// Market's Vault account (stores collateral)
    #[account(
        init,
        payer = owner,
        associated_token::mint = collateral_mint,
        associated_token::authority = vault_authority
    )]
    pub vault: Account<'info, TokenAccount>,
    
    /// Vault authority PDA (program-signing PDA)
    #[account(
        seeds = [b"vault", program_state.market_count.to_le_bytes().as_ref()],
        bump
    )]
    /// CHECK: Not an actual account, used as PDA
    pub vault_authority: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_market(
    ctx: Context<CreateMarket>,
    tick_spacing: u32,
    min_tick: i64,
    max_tick: i64,
    close_ts: i64,
) -> Result<()> {
    // 1. Parameter validation
    require!(tick_spacing > 0, RangeBetError::InvalidTickSpacing);
    require!(min_tick % tick_spacing as i64 == 0, RangeBetError::MinTickNotMultiple);
    require!(max_tick % tick_spacing as i64 == 0, RangeBetError::MaxTickNotMultiple);
    require!(min_tick < max_tick, RangeBetError::MinTickGreaterThanMax);

    // 2. Calculate bins length
    let bin_count = ((max_tick - min_tick) / tick_spacing as i64 + 1) as usize;
    
    // 3. Increase account size and calculate rent
    let additional_space = 16 * bin_count;  // Vec metadata + u64 data
    let new_size = 8 + std::mem::size_of::<Market>() + additional_space;
    
    // Calculate needed lamports
    let rent = Rent::get()?;
    let needed_lamports = rent.minimum_balance(new_size);
    let market_ai = ctx.accounts.market.to_account_info();
    
    // If current account's lamports are insufficient, transfer additional lamports
    if needed_lamports > market_ai.lamports() {
        let diff = needed_lamports - market_ai.lamports();
        
        // Transfer lamports from owner to market
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.owner.to_account_info(),
                    to: market_ai.clone(),
                },
            ),
            diff,
        )?;
    }
    
    // Perform realloc
    market_ai.realloc(new_size, false)?;

    let market_id = ctx.accounts.program_state.market_count;
    
    // 4. Initialize market
    let market = &mut ctx.accounts.market;
    market.active = true;
    market.closed = false;
    market.tick_spacing = tick_spacing;
    market.min_tick = min_tick;
    market.max_tick = max_tick;
    market.t_total = 0;
    market.collateral_balance = 0;
    market.winning_bin = None; // Not determined yet
    market.open_ts = Clock::get()?.unix_timestamp;
    market.close_ts = close_ts;
    
    // Create and initialize bins array
    market.bins = vec![0; bin_count];
    
    // 5. Update program state
    ctx.accounts.program_state.market_count += 1;
    
    // Emit event
    emit!(MarketCreated {
        market_id,
        tick_spacing,
        min_tick,
        max_tick,
    });
    
    msg!("Market created: ID = {}", market_id);
    
    Ok(())
} 