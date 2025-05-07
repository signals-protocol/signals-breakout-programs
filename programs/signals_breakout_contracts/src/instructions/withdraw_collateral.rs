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
    
    /// Vault 권한 PDA (프로그램이 서명하는 PDA)
    #[account(
        seeds = [b"vault", &market_id.to_le_bytes()],
        bump
    )]
    /// CHECK: 실제 계정이 아니라 PDA로 사용됨
    pub vault_authority: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn withdraw_collateral(
    ctx: Context<WithdrawCollateral>,
    market_id: u64,
) -> Result<()> {
    // 인출할 수 있는 금액 (마켓의 담보 잔액)
    let amount = ctx.accounts.market.collateral_balance;
    
    // 인출할 금액이 있는지 확인
    require!(amount > 0, RangeBetError::NoCollateralToWithdraw);
    
    // 토큰 전송 (vault -> 소유자)
    let vault_authority_bump = ctx.bumps.vault_authority;
    let market_id_bytes = market_id.to_le_bytes();
    
    let seeds = &[
        b"vault" as &[u8], 
        &market_id_bytes as &[u8], 
        &[vault_authority_bump]
    ];
    
    // signer seeds를 안정적으로 유지하기 위한 바인딩
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
    
    // 마켓의 담보 잔액을 0으로 설정
    ctx.accounts.market.collateral_balance = 0;
    
    // 이벤트 발생
    emit!(CollateralOut {
        to: ctx.accounts.owner.key(),
        amount,
    });
    
    msg!(
        "담보 인출 완료: 대상 = {}, 금액 = {}", 
        ctx.accounts.owner.key(),
        amount
    );
    
    Ok(())
} 