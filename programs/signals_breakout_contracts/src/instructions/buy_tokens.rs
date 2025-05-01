use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Transfer};
use crate::state::{Market, UserMarketPosition, Bin, BinBal, TokensBought};
use crate::errors::RangeBetError;
use crate::math::RangeBetMath;

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// 마켓 계정
    #[account(
        mut,
        seeds = [b"market", &market_id.to_le_bytes()],
        bump,
        constraint = market.active @ RangeBetError::MarketNotActive,
        constraint = !market.closed @ RangeBetError::MarketClosed
    )]
    pub market: Account<'info, Market>,
    
    /// 유저 포지션
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserMarketPosition>() + 16 * 10, // 기본 10개 Bin에 대한 공간 예약
        seeds = [b"pos", user.key().as_ref(), &market_id.to_le_bytes()],
        bump
    )]
    pub user_position: Account<'info, UserMarketPosition>,
    
    /// 유저 토큰 계정
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    /// 마켓 Vault 계정
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
    bin_indices: Vec<i64>,
    amounts: Vec<u64>,
    max_collateral: u64,
) -> Result<()> {
    // 유효성 검사
    require!(bin_indices.len() == amounts.len(), RangeBetError::ArrayLengthMismatch);
    require!(bin_indices.len() > 0, RangeBetError::NoTokensToBuy);
    
    let market = &mut ctx.accounts.market;
    let user_position = &mut ctx.accounts.user_position;
    let tick_spacing = market.tick_spacing;
    let min_tick = market.min_tick;
    let max_tick = market.max_tick;
    
    // 초기화가 필요한 경우
    if user_position.owner == Pubkey::default() {
        user_position.owner = ctx.accounts.user.key();
        // market_id는 이제 함수 매개변수로 전달됨
        user_position.market_id = market.key().to_bytes()[0..8]
            .try_into()
            .map(u64::from_le_bytes)
            .unwrap_or(0);
        user_position.bins = Vec::new();
    }
    
    let mut t_current = market.t_total;
    let mut total_cost: u64 = 0;
    
    // 각 Bin에 대해 처리
    for i in 0..bin_indices.len() {
        let bin_index = bin_indices[i];
        let amount = amounts[i];
        
        // 양이 0이면 건너뜀
        if amount == 0 {
            continue;
        }
        
        // Bin 유효성 검사
        require!(bin_index % tick_spacing as i64 == 0, RangeBetError::BinIndexNotMultiple);
        require!(bin_index >= min_tick && bin_index <= max_tick, RangeBetError::BinIndexOutOfRange);
        
        // 마켓에서 해당 Bin 찾기 또는 생성
        let mut bin_found = false;
        let mut bin_q = 0;
        
        for bin in &mut market.bins {
            if bin.index == bin_index {
                bin_q = bin.q;
                bin.q += amount;
                bin_found = true;
                break;
            }
        }
        
        // Bin이 없으면 새로 생성
        if !bin_found {
            market.bins.push(Bin {
                index: bin_index,
                q: amount,
            });
        }
        
        // 비용 계산
        let cost = RangeBetMath::calculate_cost(amount, bin_q, t_current)?;
        total_cost = total_cost.checked_add(cost).ok_or(error!(RangeBetError::MathOverflow))?;
        
        // 사용자 포지션에 추가
        let mut user_bin_found = false;
        
        for bin_bal in &mut user_position.bins {
            if bin_bal.index == bin_index {
                bin_bal.amount += amount;
                user_bin_found = true;
                break;
            }
        }
        
        // 사용자 Bin이 없으면 새로 생성
        if !user_bin_found {
            user_position.bins.push(BinBal {
                index: bin_index,
                amount,
            });
        }
        
        // T 업데이트
        t_current += amount;
    }
    
    // 비용이 최대 담보를 초과하지 않는지 확인
    require!(total_cost <= max_collateral, RangeBetError::CostExceedsMaxCollateral);
    
    // 토큰 전송 
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
    
    // 마켓 상태 업데이트
    market.t_total = t_current;
    market.collateral_balance = market.collateral_balance.checked_add(total_cost)
        .ok_or(error!(RangeBetError::MathOverflow))?;
    
    // 이벤트 발생
    emit!(TokensBought {
        market_id: user_position.market_id,
        buyer: ctx.accounts.user.key(),
        total_cost,
    });
    
    msg!("토큰 구매 완료: 시장 ID = {}, 구매자 = {}, 비용 = {}", 
        user_position.market_id, 
        ctx.accounts.user.key(), 
        total_cost
    );
    
    Ok(())
} 