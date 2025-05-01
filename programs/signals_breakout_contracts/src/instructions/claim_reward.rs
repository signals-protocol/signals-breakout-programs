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
    
    /// Vault 권한 PDA (프로그램이 서명하는 PDA)
    #[account(
        seeds = [b"vault", &user_position.market_id.to_le_bytes()],
        bump
    )]
    /// CHECK: 실제 계정이 아니라 PDA로 사용됨
    pub vault_authority: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let user_position = &mut ctx.accounts.user_position;
    let winning_bin = market.winning_bin;
    
    // 승리 Bin에 베팅한 토큰을 찾음
    let mut user_winning_amount = 0;
    let mut user_bin_index = None;
    
    for (i, bin_bal) in user_position.bins.iter().enumerate() {
        if bin_bal.index == winning_bin && bin_bal.amount > 0 {
            user_winning_amount = bin_bal.amount;
            user_bin_index = Some(i);
            break;
        }
    }
    
    // 청구할 토큰이 있는지 확인
    require!(user_winning_amount > 0, RangeBetError::NoTokensToClaim);
    
    // 시장에서 승리 Bin의 총 토큰 수량 찾기
    let mut total_winning_tokens = 0;
    
    for bin in &market.bins {
        if bin.index == winning_bin {
            total_winning_tokens = bin.q;
            break;
        }
    }
    
    // 보상 계산: (유저 토큰 수량 / 총 승리 토큰 수량) * 전체 담보 잔액
    let reward_amount = (user_winning_amount as u128 * market.collateral_balance as u128) 
        / total_winning_tokens as u128;
    
    // u64 범위 체크
    let reward_amount = if reward_amount > u64::MAX as u128 {
        u64::MAX
    } else {
        reward_amount as u64
    };
    
    // 토큰 전송 (vault -> 유저)
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
    
    // 유저 포지션에서 소각 (0으로 설정)
    if let Some(index) = user_bin_index {
        user_position.bins[index].amount = 0;
    }
    
    // 마켓 상태 업데이트
    market.collateral_balance = market.collateral_balance.checked_sub(reward_amount)
        .ok_or(error!(RangeBetError::MathUnderflow))?;
    
    // 이벤트 발생
    emit!(RewardClaimed {
        market_id: user_position.market_id,
        claimer: ctx.accounts.user.key(),
        amount: reward_amount,
    });
    
    msg!("보상 청구 완료: 시장 ID = {}, 청구자 = {}, 금액 = {}", 
        user_position.market_id, 
        ctx.accounts.user.key(), 
        reward_amount
    );
    
    Ok(())
} 