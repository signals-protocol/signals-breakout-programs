use anchor_lang::prelude::*;
use crate::state::{Market, UserMarketPosition, BinBal};
use crate::errors::RangeBetError;

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct TransferPosition<'info> {
    #[account(mut)]
    pub from_user: Signer<'info>,
    
    /// Recipient of the position transfer
    /// CHECK: Actual user account or PDA
    pub to_user: UncheckedAccount<'info>,
    
    /// Market information
    #[account(
        seeds = [b"market", &market_id.to_le_bytes()],
        bump
    )]
    pub market: Account<'info, Market>,
    
    /// Sender's position
    #[account(
        mut,
        seeds = [b"pos", from_user.key().as_ref(), &market_id.to_le_bytes()],
        bump,
        constraint = from_position.owner == from_user.key() @ RangeBetError::OwnerOnly
    )]
    pub from_position: Account<'info, UserMarketPosition>,
    
    /// Recipient position (created if doesn't exist)
    #[account(
        init_if_needed,
        payer = from_user,
        space = 8 + std::mem::size_of::<UserMarketPosition>() + 16 * 100, // Reserve space for 100 bins
        seeds = [b"pos", to_user.key().as_ref(), &market_id.to_le_bytes()],
        bump,
    )]
    pub to_position: Account<'info, UserMarketPosition>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn transfer_position(ctx: Context<TransferPosition>, market_id: u64, bin_indices: Vec<u16>, amounts: Vec<u64>) -> Result<()> {
    // Check if transferring to self
    require!(
        ctx.accounts.from_user.key() != ctx.accounts.to_user.key(),
        RangeBetError::CannotTransferToSelf
    );

    // Verify bin and amount array lengths
    require!(bin_indices.len() == amounts.len(), RangeBetError::ArrayLengthMismatch);
    
    // Check if transferring at least one token
    let mut total_amount = 0;
    for amount in &amounts {
        total_amount += amount;
    }
    require!(total_amount > 0, RangeBetError::NoTokensToBuy);
    
    // Initialize if needed
    if ctx.accounts.to_position.owner == Pubkey::default() {
        ctx.accounts.to_position.owner = ctx.accounts.to_user.key();
        ctx.accounts.to_position.market_id = market_id;
        ctx.accounts.to_position.bins = Vec::new();
    }
    
    // Process each bin
    for i in 0..bin_indices.len() {
        let index = bin_indices[i];
        let amount = amounts[i];
        
        // Skip if amount is 0
        if amount == 0 {
            continue;
        }
        
        // Find bin in sender position and validate amount
        let mut from_bin_found = false;
        
        for bin_bal in &mut ctx.accounts.from_position.bins {
            if bin_bal.index == index {
                require!(bin_bal.amount >= amount, RangeBetError::InsufficientTokensToTransfer);
                bin_bal.amount -= amount;
                from_bin_found = true;
                break;
            }
        }
        
        require!(from_bin_found, RangeBetError::BinIndexOutOfRange);
        
        // Add to recipient position
        let mut to_bin_found = false;
        
        for bin_bal in &mut ctx.accounts.to_position.bins {
            if bin_bal.index == index {
                bin_bal.amount += amount;
                to_bin_found = true;
                break;
            }
        }
        
        // Create new bin if not found in recipient position
        if !to_bin_found {
            ctx.accounts.to_position.bins.push(BinBal {
                index,
                amount,
            });
        }
    }
    
    msg!("Position transfer complete: {} -> {}", 
        ctx.accounts.from_user.key(), 
        ctx.accounts.to_user.key()
    );
    
    Ok(())
} 