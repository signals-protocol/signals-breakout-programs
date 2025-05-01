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
        space = 8 + std::mem::size_of::<UserMarketPosition>() + 16 * 10, // 기본 10개 Bin에 대한 공간 예약
        seeds = [b"pos", to_user.key().as_ref(), &from_position.market_id.to_le_bytes()],
        bump
    )]
    pub to_position: Account<'info, UserMarketPosition>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn transfer_position(
    ctx: Context<TransferPosition>,
    bin_index: i64,
    amount: u64,
) -> Result<()> {
    let from_position = &mut ctx.accounts.from_position;
    let to_position = &mut ctx.accounts.to_position;
    let market = &ctx.accounts.market;
    
    // from_position에서 해당 bin 찾기
    let mut from_bin_amount = 0;
    let mut from_bin_index = None;
    
    for (i, bin_bal) in from_position.bins.iter().enumerate() {
        if bin_bal.index == bin_index {
            from_bin_amount = bin_bal.amount;
            from_bin_index = Some(i);
            break;
        }
    }
    
    // 양도할 충분한 양이 있는지 확인
    require!(from_bin_amount >= amount, RangeBetError::InsufficientTokensToTransfer);
    
    // to_position 초기화 (필요한 경우)
    if to_position.owner == Pubkey::default() {
        to_position.owner = ctx.accounts.to_user.key();
        to_position.market_id = from_position.market_id;
        to_position.bins = Vec::new();
    }
    
    // from_position에서 차감
    if let Some(i) = from_bin_index {
        from_position.bins[i].amount -= amount;
    }
    
    // to_position에 추가
    let mut to_bin_found = false;
    
    for bin_bal in &mut to_position.bins {
        if bin_bal.index == bin_index {
            bin_bal.amount += amount;
            to_bin_found = true;
            break;
        }
    }
    
    // 필요한 경우 새로운 bin 생성
    if !to_bin_found {
        to_position.bins.push(BinBal {
            index: bin_index,
            amount,
        });
    }
    
    msg!(
        "포지션 전송 완료: Bin = {}, 금액 = {}, 보낸 이 = {}, 받는 이 = {}", 
        bin_index, amount, ctx.accounts.from_user.key(), ctx.accounts.to_user.key()
    );
    
    Ok(())
} 