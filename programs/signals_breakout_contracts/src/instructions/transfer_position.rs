use anchor_lang::prelude::*;
use crate::state::{Market, UserMarketPosition, BinBal};
use crate::errors::RangeBetError;

#[derive(Accounts)]
pub struct TransferPosition<'info> {
    #[account(mut)]
    pub from_user: Signer<'info>,
    
    /// 포지션 양도할 대상
    /// CHECK: 실제 사용자 계정 또는 PDA
    pub to_user: UncheckedAccount<'info>,
    
    /// 마켓 정보
    #[account(
        seeds = [b"market", &from_position.market_id.to_le_bytes()],
        bump
    )]
    pub market: Account<'info, Market>,
    
    /// 양도자 포지션
    #[account(
        mut,
        seeds = [b"pos", from_user.key().as_ref(), &from_position.market_id.to_le_bytes()],
        bump,
        constraint = from_position.owner == from_user.key() @ RangeBetError::OwnerOnly
    )]
    pub from_position: Account<'info, UserMarketPosition>,
    
    /// 수신자 포지션 (존재하지 않으면 생성)
    #[account(
        init_if_needed,
        payer = from_user,
        space = 8 + std::mem::size_of::<UserMarketPosition>() + 16 * 100, // 기본 100개 Bin에 대한 공간 예약
        seeds = [b"pos", to_user.key().as_ref(), &from_position.market_id.to_le_bytes()],
        bump,
    )]
    pub to_position: Account<'info, UserMarketPosition>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn transfer_position(ctx: Context<TransferPosition>, bin_indices: Vec<u16>, amounts: Vec<u64>) -> Result<()> {
    // 자기 자신에게 전송하는지 확인
    require!(
        ctx.accounts.from_user.key() != ctx.accounts.to_user.key(),
        RangeBetError::CannotTransferToSelf
    );

    // Bin 및 금액 배열 길이 확인
    require!(bin_indices.len() == amounts.len(), RangeBetError::ArrayLengthMismatch);
    
    // 최소 한 개 이상의 토큰을 전송하는지 확인
    let mut total_amount = 0;
    for amount in &amounts {
        total_amount += amount;
    }
    require!(total_amount > 0, RangeBetError::NoTokensToBuy);
    
    // 초기화가 필요한 경우
    if ctx.accounts.to_position.owner == Pubkey::default() {
        ctx.accounts.to_position.owner = ctx.accounts.to_user.key();
        ctx.accounts.to_position.market_id = ctx.accounts.from_position.market_id;
        ctx.accounts.to_position.bins = Vec::new();
    }
    
    // 각 Bin에 대해 처리
    for i in 0..bin_indices.len() {
        let index = bin_indices[i];
        let amount = amounts[i];
        
        // 양이 0이면 건너뜀
        if amount == 0 {
            continue;
        }
        
        // 송신자 포지션에서 해당 bin 찾기 및 금액 검증
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
        
        // 수신자 포지션에 추가
        let mut to_bin_found = false;
        
        for bin_bal in &mut ctx.accounts.to_position.bins {
            if bin_bal.index == index {
                bin_bal.amount += amount;
                to_bin_found = true;
                break;
            }
        }
        
        // 수신자 Bin이 없으면 새로 생성
        if !to_bin_found {
            ctx.accounts.to_position.bins.push(BinBal {
                index,
                amount,
            });
        }
    }
    
    msg!("포지션 양도 완료: {} -> {}", 
        ctx.accounts.from_user.key(), 
        ctx.accounts.to_user.key()
    );
    
    Ok(())
} 