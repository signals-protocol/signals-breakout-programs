#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("9FTUsmsohGA7FUXiDbJEbP1BV9ifzukhEH3dAfiZxfpe");

mod math;
mod state;
mod errors;
mod instructions;

use instructions::*;

#[program]
pub mod range_bet_program {
    use super::*;

    pub fn initialize_program(ctx: Context<InitializeProgram>) -> Result<()> {
        msg!("Initialize Program");
        instructions::initialize_program::initialize_program(ctx)
    }

    pub fn create_market(
        ctx: Context<CreateMarket>,
        tick_spacing: u32,
        min_tick: i64,
        max_tick: i64,
        close_ts: i64,
    ) -> Result<()> {
        msg!("Create Market");
        instructions::create_market::create_market(ctx, tick_spacing, min_tick, max_tick, close_ts)
    }

    pub fn buy_tokens(
        ctx: Context<BuyTokens>,
        _market_id: u64,
        bin_indices: Vec<u16>,
        amounts: Vec<u64>,
        max_collateral: u64,
    ) -> Result<()> {
        msg!("Buy Tokens");
        instructions::buy_tokens::buy_tokens(ctx, _market_id, bin_indices, amounts, max_collateral)
    }

    pub fn close_market(
        ctx: Context<CloseMarket>,
        market_id: u64,
        winning_bin: u16,
    ) -> Result<()> {
        msg!("Close Market");
        instructions::close_market::close_market(ctx, market_id, winning_bin)
    }

    pub fn claim_reward(
        ctx: Context<ClaimReward>,
    ) -> Result<()> {
        msg!("Claim Reward");
        instructions::claim_reward::claim_reward(ctx)
    }

    pub fn activate_market(
        ctx: Context<ToggleMarketStatus>,
        _market_id: u64,
        active: bool,
    ) -> Result<()> {
        msg!("Toggle Market Status");
        instructions::toggle_market_status::toggle_market_status(ctx, _market_id, active)
    }

    pub fn withdraw_collateral(
        ctx: Context<WithdrawCollateral>,
        market_id: u64,
    ) -> Result<()> {
        msg!("Withdraw Collateral");
        instructions::withdraw_collateral::withdraw_collateral(ctx, market_id)
    }

    pub fn transfer_position(
        ctx: Context<TransferPosition>,
        bin_indices: Vec<u16>,
        amounts: Vec<u64>,
    ) -> Result<()> {
        msg!("Transfer Position");
        instructions::transfer_position::transfer_position(ctx, bin_indices, amounts)
    }

    // View 함수 (시뮬레이션용 인스트럭션)
    pub fn calculate_bin_cost(
        ctx: Context<CalculateBinCost>,
        _market_id: u64,
        index: u16,
        amount: u64,
    ) -> Result<u64> {
        msg!("Calculate Bin Cost");
        instructions::view_functions::calculate_bin_cost(ctx, index, amount)
    }

    pub fn calculate_x_for_bin(
        ctx: Context<CalculateXForBin>,
        _market_id: u64,
        index: u16,
        cost: u64,
    ) -> Result<u64> {
        msg!("Calculate X for Bin");
        instructions::view_functions::calculate_x_for_bin(ctx, index, cost)
    }

    pub fn calculate_bin_sell_cost(
        ctx: Context<CalculateBinSellCost>,
        _market_id: u64,
        index: u16,
        amount: u64,
    ) -> Result<u64> {
        msg!("Calculate Bin Sell Cost");
        instructions::view_functions::calculate_bin_sell_cost(ctx, index, amount)
    }
}
