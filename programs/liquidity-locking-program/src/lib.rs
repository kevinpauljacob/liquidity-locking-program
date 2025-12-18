use anchor_lang::prelude::*;
use instructions::{
    user::{
        create_position_ix::*,
        add_liquidity_ix::*,
        lock_position_ix::*,
        lock_liquidity::*,
        unlock_liquidity::*,
    },
    admin::{
        initialize_config::*,
    }  
};

pub mod instructions;
pub mod context;
pub mod states;
pub mod errors;
pub mod constants;

declare_id!("DtnLiyCepzKfNiyFHBHEqabhrNe65tx8FPxLWQeh6JeC");

#[program]
pub mod liquidity_locking_program {
    use super::*;

    pub fn initialize_config(ctx: Context<InitializeConfig>, pool_id: Pubkey, fee_bps: u16, slf_mint: Pubkey) -> Result<()> {
        handle_initialize_config(ctx, pool_id, fee_bps, slf_mint)
    }

    pub fn create_position_ix(ctx: Context<DammV2CreatePosition>) -> Result<()> {
        handle_create_position(ctx)
    }

    pub fn add_liquidity_ix(ctx: Context<DammV2AddLiquidity>, liquidity_delta: u128) -> Result<()> {
        handle_add_liquidity(ctx, liquidity_delta)
    }

    pub fn lock_position_ix(
        ctx: Context<DammV2LockPosition>,
        params: crate::context::damm_v2::VestingParameters,
    ) -> Result<()> {
        handle_lock_position(ctx, params)
    }

    pub fn lock_liquidity(ctx: Context<LockLiquidity>, liquidity_delta: u128, duration_months: u8) -> Result<()> {
        handle_lock_liquidity(ctx, liquidity_delta, duration_months)
    }

    pub fn unlock_liquidity(ctx: Context<UnlockLiquidity>, liquidity_delta: u128) -> Result<()> {
    handle_unlock_liquidity(ctx, liquidity_delta)
}
}
