#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

declare_id!("DDFXv1hETR8pQSpNbzCxTX7jm1Hr57V4oihDGosXQfgC");

#[program]
pub mod collateral_token_faucet {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn mint_collateral_tokens(ctx: Context<MintCollateralTokens>, amount: u64) -> Result<()> {
        msg!("Minting {} tokens to receiver account.", amount);
        msg!("Mint: {}", ctx.accounts.mint.key());
        msg!("Receiver ATA: {}", ctx.accounts.receiver.key());
        msg!("Faucet PDA: {}", ctx.accounts.faucet_pda.key());
        msg!("User (payer): {}", ctx.accounts.user.key());

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.receiver.to_account_info(),
            authority: ctx.accounts.faucet_pda.to_account_info(),
        };
        
        let seeds = &[b"collateral_faucet".as_ref(), &[ctx.bumps.faucet_pda]];
        let signer_seeds = &[&seeds[..]];
        
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            ),
            amount,
        )?;

        msg!("Tokens minted successfully.");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct MintCollateralTokens<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"collateral_faucet".as_ref()],
        bump
    )]
    /// CHECK: PDA는 서명 역할만 수행하며, 데이터는 저장하지 않습니다.
    pub faucet_pda: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub receiver: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}
