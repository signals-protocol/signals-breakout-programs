use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Transfer};
use crate::state::{Market, UserMarketPosition, RewardClaimed};
use crate::errors::RangeBetError;

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"market", &user_position.market_id.to_le_bytes()],
        bump,
        constraint = market.closed @ RangeBetError::MarketNotActive
    )]
    pub market: Account<'info, Market>,
    
    #[account(
        mut,
        seeds = [b"pos", user.key().as_ref(), &user_position.market_id.to_le_bytes()],
        bump,
        constraint = user_position.owner == user.key() @ RangeBetError::OwnerOnly
    )]
    pub user_position: Account<'info, UserMarketPosition>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = vault.mint == user_token_account.mint
    )]
    pub vault: Account<'info, TokenAccount>,
    
    /// Vault authority PDA (program signing PDA)
    #[account(
        seeds = [b"vault", &user_position.market_id.to_le_bytes()],
        bump
    )]
    /// CHECK: This is not an actual account but used as a PDA
    pub vault_authority: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let user_position = &mut ctx.accounts.user_position;

    // Check if market is closed
    require!(market.closed, RangeBetError::MarketIsNotClosed);
    
    // Check if winning bin is set
    let winning_bin = market.winning_bin.ok_or(error!(RangeBetError::BinIndexOutOfRange))?;
    
    // Find tokens bet on the winning bin
    let mut user_winning_amount = 0;
    let mut user_bin_index = None;
    
    for (i, bin_bal) in user_position.bins.iter().enumerate() {
        if bin_bal.index == winning_bin && bin_bal.amount > 0 {
            user_winning_amount = bin_bal.amount;
            user_bin_index = Some(i);
            break;
        }
    }
    
    // Check if there are tokens to claim from the winning bin
    require!(user_winning_amount > 0, RangeBetError::NotWinningBin);
    
    // Get total token amount for the winning bin
    let total_winning_tokens = market.bins[winning_bin as usize];
    
    // Calculate reward: (user token amount / total winning tokens) * total collateral balance
    let reward_amount = (user_winning_amount as u128 * market.collateral_balance as u128) 
        / total_winning_tokens as u128;
    
    // Check u64 range
    let reward_amount = if reward_amount > u64::MAX as u128 {
        u64::MAX
    } else {
        reward_amount as u64
    };
    
    // Transfer tokens (vault -> user)
    let vault_authority_bump = ctx.bumps.vault_authority;
    let seeds = &[
        b"vault" as &[u8], 
        &user_position.market_id.to_le_bytes()[..] as &[u8], 
        &[vault_authority_bump]
    ];
    let signer_seeds = &[&seeds[..]];
    
    let cpi_accounts = Transfer {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    
    token::transfer(cpi_ctx, reward_amount)?;
    
    // Burn from user position (set to 0)
    if let Some(index) = user_bin_index {
        user_position.bins[index].amount = 0;
    }
    
    // Update market state
    market.collateral_balance = market.collateral_balance.checked_sub(reward_amount)
        .ok_or(error!(RangeBetError::MathUnderflow))?;
    
    // Emit event
    emit!(RewardClaimed {
        market_id: user_position.market_id,
        claimer: ctx.accounts.user.key(),
        amount: reward_amount,
    });
    
    msg!("Reward claimed: Market ID = {}, Claimer = {}, Amount = {}", 
        user_position.market_id, 
        ctx.accounts.user.key(), 
        reward_amount
    );
    
    Ok(())
} 