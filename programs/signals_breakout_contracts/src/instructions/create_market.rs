use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    token::{Mint, TokenAccount, Token},
    associated_token::AssociatedToken,
};
use crate::state::{ProgramState, Market, MarketCreated};
use crate::errors::RangeBetError;

#[derive(Accounts)]
#[instruction(tick_spacing: u32, min_tick: i64, max_tick: i64, close_ts: i64)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"range-bet-state"],
        bump,
        constraint = program_state.owner == owner.key() @ RangeBetError::OwnerOnly
    )]
    pub program_state: Account<'info, ProgramState>,
    
    /// 생성할 마켓 정보를 저장할 계정
    #[account(
        init,
        payer = owner,
        seeds = [b"market", program_state.market_count.to_le_bytes().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<Market>() // 최소 크기만 할당 (bins 없이)
    )]
    pub market: Account<'info, Market>,
    
    /// 담보 토큰의 Mint
    pub collateral_mint: Account<'info, Mint>,
    
    /// 마켓의 Vault 계정 (담보를 보관할 ATA)
    #[account(
        init,
        payer = owner,
        associated_token::mint = collateral_mint,
        associated_token::authority = vault_authority
    )]
    pub vault: Account<'info, TokenAccount>,
    
    /// Vault 권한 PDA (프로그램이 서명하는 PDA)
    #[account(
        seeds = [b"vault", program_state.market_count.to_le_bytes().as_ref()],
        bump
    )]
    /// CHECK: 실제 계정이 아니라 PDA로 사용됨
    pub vault_authority: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_market(
    ctx: Context<CreateMarket>,
    tick_spacing: u32,
    min_tick: i64,
    max_tick: i64,
    close_ts: i64,
) -> Result<()> {
    // 1. 파라미터 검증
    require!(tick_spacing > 0, RangeBetError::InvalidTickSpacing);
    require!(min_tick % tick_spacing as i64 == 0, RangeBetError::MinTickNotMultiple);
    require!(max_tick % tick_spacing as i64 == 0, RangeBetError::MaxTickNotMultiple);
    require!(min_tick < max_tick, RangeBetError::MinTickGreaterThanMax);

    // 2. bins 길이 계산
    let bin_count = ((max_tick - min_tick) / tick_spacing as i64 + 1) as usize;
    
    // 3. 계정 크기 증가 및 rent 계산
    let additional_space = 16 * bin_count;  // Vec 메타데이터 + u64 데이터
    let new_size = 8 + std::mem::size_of::<Market>() + additional_space;
    
    // 필요한 lamports 계산
    let rent = Rent::get()?;
    let needed_lamports = rent.minimum_balance(new_size);
    let market_ai = ctx.accounts.market.to_account_info();
    
    // 현재 계정의 lamports가 충분하지 않으면 추가 전송
    if needed_lamports > market_ai.lamports() {
        let diff = needed_lamports - market_ai.lamports();
        
        // owner → market 로 lamports 전송
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.owner.to_account_info(),
                    to: market_ai.clone(),
                },
            ),
            diff,
        )?;
    }
    
    // realloc 수행
    market_ai.realloc(new_size, false)?;

    let market_id = ctx.accounts.program_state.market_count;
    
    // 4. 마켓 초기화
    let market = &mut ctx.accounts.market;
    market.active = true;
    market.closed = false;
    market.tick_spacing = tick_spacing;
    market.min_tick = min_tick;
    market.max_tick = max_tick;
    market.t_total = 0;
    market.collateral_balance = 0;
    market.winning_bin = None; // 아직 결정되지 않음
    market.open_ts = Clock::get()?.unix_timestamp;
    market.close_ts = close_ts;
    
    // bins 배열을 생성하고 0으로 초기화
    market.bins = vec![0; bin_count];
    
    // 5. 프로그램 상태 업데이트
    ctx.accounts.program_state.market_count += 1;
    
    // 이벤트 발생
    emit!(MarketCreated {
        market_id,
        tick_spacing,
        min_tick,
        max_tick,
    });
    
    msg!("마켓 생성 완료: ID = {}", market_id);
    
    Ok(())
} 