use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Transfer};
use crate::state::{ProgramState, CollateralOut, Market};
use crate::errors::RangeBetError;

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct WithdrawCollateral<'info> {
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
        constraint = market.closed @ RangeBetError::MarketIsNotClosed,
    )]
    pub market: Account<'info, Market>,
    
    #[account(mut)]
    pub owner_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    
    /// Vault authority PDA (program-signing PDA)
    #[account(
        seeds = [b"vault", &market_id.to_le_bytes()],
        bump
    )]
    /// CHECK: Not an actual account, used as PDA
    pub vault_authority: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn withdraw_collateral(
    ctx: Context<WithdrawCollateral>,
    market_id: u64,
) -> Result<()> {
    // Amount to withdraw (market collateral balance)
    let amount = ctx.accounts.market.collateral_balance;
    
    // Check if there is an amount to withdraw
    require!(amount > 0, RangeBetError::NoCollateralToWithdraw);
    
    // Transfer tokens (vault -> owner)
    let vault_authority_bump = ctx.bumps.vault_authority;
    let market_id_bytes = market_id.to_le_bytes();
    
    let seeds = &[
        b"vault" as &[u8], 
        &market_id_bytes as &[u8], 
        &[vault_authority_bump]
    ];
    
    // Bind signer seeds to ensure stability
    let signer_seeds = &[&seeds[..]];
    
    let cpi_accounts = Transfer {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.owner_token_account.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    
    token::transfer(cpi_ctx, amount)?;
    
    // Set market collateral balance to 0
    ctx.accounts.market.collateral_balance = 0;
    
    // Emit event
    emit!(CollateralOut {
        to: ctx.accounts.owner.key(),
        amount,
    });
    
    msg!(
        "Collateral withdrawn: target = {}, amount = {}", 
        ctx.accounts.owner.key(),
        amount
    );
    
    Ok(())
} 