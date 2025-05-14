use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Transfer};
use crate::state::{Market, UserMarketPosition, BinBal, TokensBought};
use crate::errors::RangeBetError;
pub use range_bet_math_core::RangeBetMath;

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// Market account
    #[account(
        mut,
        seeds = [b"market", &market_id.to_le_bytes()],
        bump,
        constraint = market.active @ RangeBetError::MarketNotActive,
        constraint = !market.closed @ RangeBetError::MarketClosed
    )]
    pub market: Account<'info, Market>,
    
    /// User position
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserMarketPosition>() + 16 * 100, // Reserve space for 100 bins
        seeds = [b"pos", user.key().as_ref(), &market_id.to_le_bytes()],
        bump
    )]
    pub user_position: Account<'info, UserMarketPosition>,
    
    /// User token account
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    /// Market Vault account
    #[account(
        mut,
        constraint = vault.mint == user_token_account.mint
    )]
    pub vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn buy_tokens(
    ctx: Context<BuyTokens>,
    market_id: u64,
    bin_indices: Vec<u16>,
    amounts: Vec<u64>,
    max_collateral: u64,
) -> Result<()> {
    // Validation
    require!(bin_indices.len() == amounts.len(), RangeBetError::ArrayLengthMismatch);
    require!(bin_indices.len() > 0, RangeBetError::NoTokensToBuy);
    
    let market = &mut ctx.accounts.market;
    let user_position = &mut ctx.accounts.user_position;
    
    // Initialize if needed
    if user_position.owner == Pubkey::default() {
        user_position.owner = ctx.accounts.user.key();
        // Use market_id directly from parameters
        user_position.market_id = market_id;
        user_position.bins = Vec::new();
    }
    
    let mut t_current = market.t_total;
    let mut total_cost: u64 = 0;
    
    // Process each bin
    for i in 0..bin_indices.len() {
        let index = bin_indices[i];
        let amount = amounts[i];
        
        // Skip if amount is 0
        if amount == 0 {
            continue;
        }
        
        // Check array index range
        require!(index < market.bins.len() as u16, RangeBetError::BinIndexOutOfRange);
        
        // Get quantity from market bin
        let bin_q = market.bins[index as usize];
        
        // Update market bin quantity
        market.bins[index as usize] += amount;
        
        // Calculate cost
        let cost = RangeBetMath::calculate_bin_buy_cost(amount, bin_q, t_current)?;
        total_cost = total_cost.checked_add(cost).ok_or(error!(RangeBetError::MathOverflow))?;
        
        // Add to user position
        let mut user_bin_found = false;
        
        for bin_bal in &mut user_position.bins {
            if bin_bal.index == index {
                bin_bal.amount += amount;
                user_bin_found = true;
                break;
            }
        }
        
        // Create new bin if not found in user position
        if !user_bin_found {
            user_position.bins.push(BinBal {
                index,
                amount,
            });
        }
        
        // Update T
        t_current += amount;
    }
    
    // Check if cost exceeds maximum collateral
    require!(total_cost <= max_collateral, RangeBetError::CostExceedsMaxCollateral);
    
    // Transfer tokens
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
    );
    
    token::transfer(cpi_ctx, total_cost)?;
    
    // Update market state
    market.t_total = t_current;
    market.collateral_balance = market.collateral_balance.checked_add(total_cost)
        .ok_or(error!(RangeBetError::MathOverflow))?;
    
    // Emit event
    emit!(TokensBought {
        market_id: user_position.market_id,
        buyer: ctx.accounts.user.key(),
        total_cost,
    });
    
    msg!("Token purchase complete: Market ID = {}, Buyer = {}, Cost = {}", 
        user_position.market_id, 
        ctx.accounts.user.key(), 
        total_cost
    );
    
    Ok(())
} 